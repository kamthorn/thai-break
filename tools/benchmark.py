#!/usr/bin/env python3
"""
tools/benchmark.py
Measure word segmentation accuracy against a gold corpus.

Examples:
  # ThaiBreak (the Python binding in python/) on the LST20 eval split
  python3 tools/benchmark.py --corpus lst20 --corpus-dir ../LST20_Corpus/eval \\
      --dict data/words.txt

  # Any other segmenter that reads lines on stdin and prints tokens joined by
  # "|" ({dict} is replaced by the dictionary path) -- e.g. the PHP port
  python3 tools/benchmark.py --corpus lst20 --corpus-dir ../LST20_Corpus/eval \\
      --dict data/words.txt --segmenter-cmd "php tools/segment.php {dict}"

  # Merge in extra word lists, e.g. from a sibling thai-break-dict-extra checkout
  python3 tools/benchmark.py --corpus lst20 --corpus-dir ../LST20_Corpus/eval \\
      --dict data/words.txt --dict ../thai-break-dict-extra/dist/words-extra.txt

  # Blackboard Treebank, sub-word level, without sentences that occur in LST20 train
  python3 tools/benchmark.py --corpus blackboard --corpus-dir ../Corpus-BlackboardTreebank/thai10_conll \\
      --subword --exclude-overlap ../LST20_Corpus/train --dict data/words.txt

Tune on train/eval data and report the test split only for final results.
The corpora are not part of this repository; see the README for their licenses.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import shlex
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Callable

sys.path.insert(0, str(Path(__file__).resolve().parent))
from corpora import THAI_WORD, blackboard_chunks, lst20_chunks, without_texts  # noqa: E402

Segmenter = Callable[[list[str]], list[list[str]]]

WHITESPACE = re.compile(r"\s+")


def spans(tokens: list[str]) -> set[tuple[int, int]]:
    result = set()
    pos = 0
    for tok in tokens:
        result.add((pos, pos + len(tok)))
        pos += len(tok)
    return result


def prf(correct: int, predicted: int, gold: int) -> tuple[float, float, float]:
    p = correct / predicted if predicted else 0.0
    r = correct / gold if gold else 0.0
    f = 2 * p * r / (p + r) if p + r else 0.0
    return p, r, f


def evaluate(gold_chunks: list[list[str]], predictions: list[list[str]], dict_words: set[str] | None = None) -> dict:
    """
    Word-level scores count a gold word as correct when the prediction has a
    token with exactly the same span. Boundary scores compare the positions
    between tokens inside each chunk. Whitespace inside tokens is ignored.
    """
    words_gold = words_pred = words_correct = 0
    b_gold = b_pred = b_correct = 0
    skipped = 0
    errors: collections.Counter[str] = collections.Counter()
    oov: collections.Counter[str] = collections.Counter()

    for gold, pred in zip(gold_chunks, predictions):
        # Whitespace is not scored: some gold words contain spaces or NBSP
        # (multi-word names), while segmenters may drop whitespace tokens.
        gold = [t for t in (WHITESPACE.sub("", w) for w in gold) if t]
        pred = [t for t in (WHITESPACE.sub("", w) for w in pred) if t]
        text = "".join(gold)
        if "".join(pred) != text:
            skipped += 1
            continue
        gs, ps = spans(gold), spans(pred)
        words_gold += len(gs)
        words_pred += len(ps)
        words_correct += len(gs & ps)
        gb = {end for _, end in gs} - {len(text)}
        pb = {end for _, end in ps} - {len(text)}
        b_gold += len(gb)
        b_pred += len(pb)
        b_correct += len(gb & pb)

        if dict_words is not None:
            pos = 0
            for word in gold:
                span = (pos, pos + len(word))
                pos += len(word)
                if span in ps:
                    continue
                if not THAI_WORD.match(word):
                    errors["non-Thai or mixed token"] += 1
                elif word in dict_words:
                    errors["in dictionary but mis-segmented"] += 1
                else:
                    errors["not in dictionary"] += 1
                    oov[word] += 1

    wp, wr, wf = prf(words_correct, words_pred, words_gold)
    bp, br, bf = prf(b_correct, b_pred, b_gold)
    return {
        "word": {"precision": wp, "recall": wr, "f1": wf},
        "boundary": {"precision": bp, "recall": br, "f1": bf},
        "gold_words": words_gold,
        "predicted_words": words_pred,
        "correct_words": words_correct,
        "chunks": len(gold_chunks),
        "skipped_chunks": skipped,
        "errors": dict(errors.most_common()),
        "top_oov": oov.most_common(30),
    }


def thaibreak_segmenter(binding_dir: Path, dict_path: Path) -> Segmenter:
    """Use the thaibreak Python binding (Rust core)."""
    sys.path.insert(0, str(binding_dir))
    import thaibreak  # type: ignore

    if not thaibreak.init(str(dict_path)):
        sys.exit(f"thaibreak.init() failed; build the Rust library in {binding_dir.parent / 'rust'}")
    return lambda texts: [thaibreak.words(t) for t in texts]


def command_segmenter(command: str, dict_path: Path) -> Segmenter:
    """Run a command once: one text per stdin line, tokens joined by "|" on stdout."""

    def run(texts: list[str]) -> list[list[str]]:
        argv = [arg.replace("{dict}", str(dict_path)) for arg in shlex.split(command)]
        out = subprocess.run(argv, input="\n".join(texts) + "\n", capture_output=True, text=True, check=True).stdout
        lines = out.split("\n")
        if len(lines) < len(texts):
            sys.exit(f"segmenter returned {len(lines)} lines for {len(texts)} inputs")
        return [line.split("|") for line in lines[: len(texts)]]

    return run


def merged_dictionary(paths: list[Path]) -> tuple[Path, set[str]]:
    """Merge word lists (first column of TSV lines) into a temporary file."""
    if len(paths) == 1:
        words = {line.split("\t")[0].strip() for line in paths[0].read_text(encoding="utf-8").splitlines()}
        return paths[0], {w for w in words if w and not w.startswith("#")}
    lines: dict[str, str] = {}
    for path in paths:
        for line in path.read_text(encoding="utf-8").splitlines():
            word = line.split("\t")[0].strip()
            if word and not word.startswith("#"):
                lines.setdefault(word, line.strip())
    tmp = tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False, encoding="utf-8")
    tmp.write("\n".join(lines.values()) + "\n")
    tmp.close()
    return Path(tmp.name), set(lines)


def load_chunks(args: argparse.Namespace) -> list[list[str]]:
    if args.corpus == "lst20":
        chunks = lst20_chunks(args.corpus_dir)
    else:
        chunks = blackboard_chunks(args.corpus_dir, subword=args.subword)
    if args.exclude_overlap:
        seen = {"".join(c) for c in lst20_chunks(args.exclude_overlap)}
        chunks = without_texts(chunks, seen)
    chunks = list(chunks)
    return chunks[: args.limit] if args.limit else chunks


def main() -> None:
    parser = argparse.ArgumentParser(description="Benchmark Thai word segmentation against a gold corpus.")
    parser.add_argument("--corpus", choices=["lst20", "blackboard"], required=True)
    parser.add_argument("--corpus-dir", type=Path, required=True, help="LST20 split directory or thai10_conll directory")
    parser.add_argument("--subword", action="store_true", help="Blackboard: evaluate against sub-words (split at '|')")
    parser.add_argument("--exclude-overlap", type=Path, help="Drop chunks whose text occurs in this LST20 split")
    parser.add_argument("--dict", type=Path, action="append", required=True, help="Word list (repeat to merge)")
    parser.add_argument("--segmenter-cmd", help='Command reading stdin lines; "{dict}" is replaced by the dictionary path')
    parser.add_argument("--binding-dir", type=Path, default=Path(__file__).resolve().parents[1] / "python")
    parser.add_argument("--limit", type=int, help="Evaluate only the first N chunks")
    parser.add_argument("--json", action="store_true", help="Print results as JSON")
    args = parser.parse_args()

    dict_path, dict_words = merged_dictionary(args.dict)
    chunks = load_chunks(args)
    texts = ["".join(c) for c in chunks]
    segment = (command_segmenter(args.segmenter_cmd, dict_path) if args.segmenter_cmd
               else thaibreak_segmenter(args.binding_dir, dict_path))

    start = time.time()
    predictions = segment(texts)
    elapsed = time.time() - start
    result = evaluate(chunks, predictions, dict_words)
    result["seconds"] = round(elapsed, 2)

    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return

    w, b = result["word"], result["boundary"]
    print(f"Corpus   : {args.corpus} {args.corpus_dir}{' (sub-words)' if args.subword else ''}")
    print(f"Dictionary: {', '.join(str(p) for p in args.dict)} ({len(dict_words)} words)")
    print(f"Chunks   : {result['chunks']} ({result['skipped_chunks']} skipped), gold words: {result['gold_words']}, {elapsed:.2f}s")
    print(f"Word     : P={w['precision']:.4f} R={w['recall']:.4f} F1={w['f1']:.4f}")
    print(f"Boundary : P={b['precision']:.4f} R={b['recall']:.4f} F1={b['f1']:.4f}")
    print("Missed gold words by cause:")
    for cause, count in result["errors"].items():
        print(f"  {cause:34s} {count}")
    print("Most frequent missed words not in the dictionary:")
    print("  " + ", ".join(f"{word} ({n})" for word, n in result["top_oov"][:20]))


if __name__ == "__main__":
    main()
