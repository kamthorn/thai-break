#!/usr/bin/env python3
"""
Compile merged Thai dictionaries (Base + Extra) into optimized binary FST files:
  - data/words.fst (for semantic word segmentation)
  - data/lines.fst (for layout typographic line breaking)
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
REPO_DIR = ROOT_DIR.parent.parent
DATA_DIR = ROOT_DIR / "data"

def find_engine_dir():
    """Locate the engine of this repository (two levels up), or THAIBREAK_DIR."""
    candidates = []
    if os.environ.get("THAIBREAK_DIR"):
        candidates.append(Path(os.environ["THAIBREAK_DIR"]))
    candidates += [REPO_DIR / "rust"]
    for c in candidates:
        if (c / "Cargo.toml").exists():
            return c
    return None

ENGINE_DIR = find_engine_dir()

def find_compiler():
    """Resolve thaibreak_compile_fst: env override, cargo bin, or sibling target."""
    if os.environ.get("THAIBREAK_COMPILE_FST"):
        p = Path(os.environ["THAIBREAK_COMPILE_FST"])
        if p.exists():
            return p
    home_bin = Path.home() / ".cargo" / "bin" / "thaibreak_compile_fst"
    if home_bin.exists():
        return home_bin
    if ENGINE_DIR is not None:
        p = ENGINE_DIR / "target" / "release" / "thaibreak_compile_fst"
        if p.exists():
            return p
    return None

COMPILER_BIN = find_compiler()

def find_file(candidates):
    for c in candidates:
        p = Path(c)
        if p.exists():
            return p
    return None

def load_wordlist(path):
    words = {}
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            word = parts[0].strip()
            if not word:
                continue
            weight = float(parts[1]) if len(parts) > 1 else 1.0
            words[word] = weight
    return words

def compile_tsv_to_fst(entries, output_fst_path):
    # Sort lexicographically by UTF-8 bytes (mandatory for FST)
    sorted_entries = sorted(entries.items(), key=lambda x: x[0].encode("utf-8"))
    
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".tsv", delete=False) as tmp:
        tmp_path = tmp.name
        for word, weight in sorted_entries:
            tmp.write(f"{word}\t{weight:.2f}\n")
    
    try:
        cmd = [str(COMPILER_BIN), tmp_path, str(output_fst_path)]
        print(f"Running: {' '.join(cmd)}")
        res = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(res.stdout)
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)

def main():
    global COMPILER_BIN
    if COMPILER_BIN is None or not COMPILER_BIN.exists():
        if ENGINE_DIR is None:
            sys.exit(
                "Error: thaibreak_compile_fst not found and no engine checkout "
                "located. Set THAIBREAK_COMPILE_FST or THAIBREAK_DIR, or place "
                "a thai-break checkout next to this repo."
            )
        print(f"Compiler binary not found. Building it from {ENGINE_DIR} ...")
        subprocess.run(
            ["cargo", "build", "--release", "--bin", "thaibreak_compile_fst",
             "--manifest-path", str(ENGINE_DIR / "Cargo.toml")],
            check=True,
        )
        COMPILER_BIN = ENGINE_DIR / "target" / "release" / "thaibreak_compile_fst"

    # 1. Locate Base dictionary
    base_file = find_file([
        DATA_DIR / "words.txt",
        REPO_DIR / "data" / "words.txt",
    ])
    if not base_file:
        sys.exit("Error: Could not locate base words.txt")
    print(f"Using base dictionary: {base_file}")
    base_words = load_wordlist(base_file)
    print(f"Loaded {len(base_words)} base words")

    # 2. Locate Extra Words dictionary
    extra_words_file = find_file([
        DATA_DIR / "words-extra-words.tsv",
        DATA_DIR / "words-extra.tsv",
        REPO_DIR.parent / "thai-break-dict-extra" / "dist" / "words-extra.tsv",
    ])
    if not extra_words_file:
        sys.exit("Error: Could not locate extra words dictionary")
    print(f"Using extra words dictionary: {extra_words_file}")
    extra_words = load_wordlist(extra_words_file)
    print(f"Loaded {len(extra_words)} extra words")

    # 3. Locate Extra Lines dictionary
    extra_lines_file = find_file([
        DATA_DIR / "words-extra-lines.tsv",
        REPO_DIR.parent / "thai-break-dict-extra" / "dist" / "words-extra-lines.tsv",
    ])
    if not extra_lines_file:
        sys.exit("Error: Could not locate extra lines dictionary")
    print(f"Using extra lines dictionary: {extra_lines_file}")
    extra_lines = load_wordlist(extra_lines_file)
    print(f"Loaded {len(extra_lines)} extra lines")

    DATA_DIR.mkdir(parents=True, exist_ok=True)

    # Compile words.fst
    words_merged = dict(base_words)
    words_merged.update(extra_words)
    words_fst_path = DATA_DIR / "words.fst"
    print(f"\nCompiling words.fst with {len(words_merged)} words...")
    compile_tsv_to_fst(words_merged, words_fst_path)

    # Compile lines.fst
    lines_merged = dict(base_words)
    lines_merged.update(extra_lines)
    lines_fst_path = DATA_DIR / "lines.fst"
    print(f"\nCompiling lines.fst with {len(lines_merged)} words...")
    compile_tsv_to_fst(lines_merged, lines_fst_path)

    print("\nSuccessfully compiled dictionaries to FST:")
    print(f"  - {words_fst_path} ({words_fst_path.stat().st_size:,} bytes)")
    print(f"  - {lines_fst_path} ({lines_fst_path.stat().st_size:,} bytes)")

if __name__ == "__main__":
    main()
