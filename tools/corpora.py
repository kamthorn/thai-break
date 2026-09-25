#!/usr/bin/env python3
"""
tools/corpora.py
Readers for word-segmented gold corpora used by tools/benchmark.py.

The corpora themselves are NOT part of this repository. Point the readers at
a local copy; their licenses (e.g. the LST20 usage agreement) forbid
redistribution, so never copy corpus files or derived data into data/ or dist/.

Every reader yields "chunks": lists of gold words with no whitespace inside,
split at spaces and at sentence boundaries. A segmenter is evaluated on the
concatenated text of each chunk.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable, Iterator

THAI_WORD = re.compile(r"^[฀-๿]+$")

# Tokens that stand for whitespace or annotation markers rather than text
LST20_SPACE = "_"
BLACKBOARD_SEPARATORS = {"_", "$$"}


def corpus_files(directory: Path) -> list[Path]:
    """Sorted *.txt / *.conll files, skipping macOS resource forks (._*)."""
    files = [p for p in directory.iterdir() if p.suffix in (".txt", ".conll") and not p.name.startswith("._")]
    return sorted(files)


def lst20_chunks(split_dir: Path) -> Iterator[list[str]]:
    """Read every file of an LST20 split (train/eval/test); see lst20_file_chunks()."""
    for path in corpus_files(Path(split_dir)):
        yield from lst20_file_chunks(path)


def lst20_file_chunks(path: Path) -> Iterator[list[str]]:
    """
    Read one LST20 file. Each line is WORD<TAB>POS<TAB>NE<TAB>CLAUSE;
    "_" is a space and a blank line ends a sentence. Both end a chunk.
    """
    chunk: list[str] = []
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            fields = line.rstrip("\n").split("\t")
            word = fields[0] if len(fields) >= 2 else ""
            if word == "" or word == LST20_SPACE or word.strip() == "":
                if chunk:
                    yield chunk
                    chunk = []
                continue
            chunk.append(word)
    if chunk:
        yield chunk


def blackboard_chunks(conll_dir: Path, subword: bool = False) -> Iterator[list[str]]:
    """
    Read the Blackboard Treebank CoNLL files (thai10_conll). Column 2 is the
    word; "|" inside a word marks its sub-words (e.g. "ยัน|ปฏิเสธ"). With
    subword=True the sub-words are the gold units, otherwise the whole word.
    """
    for path in corpus_files(Path(conll_dir)):
        chunk: list[str] = []
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                if line.startswith("#"):
                    continue
                fields = line.rstrip("\n").split("\t")
                if len(fields) < 2:
                    if chunk:
                        yield chunk
                        chunk = []
                    continue
                word = fields[1]
                units = word.split("|") if subword else [word.replace("|", "")]
                for unit in units:
                    if unit in BLACKBOARD_SEPARATORS or unit.strip() == "":
                        if chunk:
                            yield chunk
                            chunk = []
                    else:
                        chunk.append(unit)
        if chunk:
            yield chunk


def without_texts(chunks: Iterable[list[str]], texts: set[str]) -> Iterator[list[str]]:
    """Drop chunks whose text also occurs in `texts` (e.g. to avoid train/test overlap)."""
    for chunk in chunks:
        if "".join(chunk) not in texts:
            yield chunk
