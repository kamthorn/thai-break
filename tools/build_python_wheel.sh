#!/usr/bin/env bash
# Build one platform wheel for the Python package: Rust library + bundled dictionary.
# Usage: PYTHON=python3 OUT=python/dist tools/build_python_wheel.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PY="${PYTHON:-python3}"
OUT="${OUT:-$ROOT/python/dist}"

(cd "$ROOT/rust" && cargo build --release)
cd "$ROOT/python"
"$PY" stage_native.py
"$PY" -m pip install --quiet build wheel setuptools
"$PY" -m build --wheel --outdir "$OUT"
