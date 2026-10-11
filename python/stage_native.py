#!/usr/bin/env python3
"""Copy the native library, dictionary and license into the package before building a wheel.

Run `cargo build --release` in ../rust first. The staged files are git-ignored.
"""
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
LIB_NAME = {"win32": "thaibreak.dll", "darwin": "libthaibreak.dylib"}.get(sys.platform, "libthaibreak.so")

lib = ROOT / "rust" / "target" / "release" / LIB_NAME
if not lib.exists():
    sys.exit(f"{lib} not found: run `cargo build --release` in rust/ first")

(HERE / "thaibreak" / "_lib").mkdir(exist_ok=True)
(HERE / "thaibreak" / "data").mkdir(exist_ok=True)
shutil.copy2(lib, HERE / "thaibreak" / "_lib" / LIB_NAME)
for name in ("words.fst", "word-weights.tsv"):
    shutil.copy2(ROOT / "data" / name, HERE / "thaibreak" / "data" / name)
for name in ("LICENSE", "NOTICE"):
    shutil.copy2(ROOT / name, HERE / name)
print(f"staged {LIB_NAME}, words.fst, word-weights.tsv, LICENSE and NOTICE")
