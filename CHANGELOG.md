# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.2] - 2026-09-30

### Added
- **Rust crate embeds the base dictionary** (feature `embedded-dict`, on by
  default): `thaibreak::words()` and the C API (`thaibreak_init(NULL, NULL)`)
  work with no files on disk, including when installed from crates.io. Use
  `default-features = false` to leave the dictionary out. `ThaiTrie::embedded()`
  exposes it, and the WASM build gains `WasmThaiBreak.withDefault()`.
- `tools/sync_dict.sh` (`stage` / `check`) and a CI job that fails when the
  committed copy in `go/data` drifts from `data/`.

### Changed
- `data/` is the only place the dictionary is maintained. Rust stages
  `words.fst` into the crate at package time (`rust/data/`, gitignored; the
  build fails if it is missing), Python stages it into the wheel, and
  TypeScript copies `data/words.dawg` during `npm run build`. The duplicate
  `typescript/src/data/words.dawg` is removed.
- The `typst-plugin` feature uses the same embedded dictionary instead of an
  `include_bytes!` path outside the crate, so it builds from the published crate.
- `/typst` is excluded from the Composer archive.

## [1.1.1] - 2026-09-30

### Removed
- `data/bigrams.tsv` (bigram counts derived from a news corpus) is no longer
  shipped: the libraries use only the base dictionary (`words.dawg` / `words.fst`).
  A bigram file next to the dictionary is still loaded if the user provides one.

### Added
- **Platform wheels for Python**: `py3-none-<platform>` wheels bundle the Rust
  library and the base dictionary (`words.fst`), so `pip install thaibreak`
  works without Rust and `thaibreak.words()` needs no `init()`. Built for
  Linux x86_64/aarch64 (manylinux_2_28), macOS x86_64/arm64 and Windows x64 by
  `.github/workflows/python-wheels.yml` (`tools/build_python_wheel.sh`).
  The loader now looks for `.dylib`/`.dll` as well as `.so`.
- **Thai Text Normalizer (`rust/src/normalizer.rs`)**:
  - Canonicalizes mis-typed vowels (`เเ` → `แ`, `ํา` → `ำ`), reorders
    misplaced tones/vowels, collapses elongations (`มากกก` → `มาก`,
    dictionary-verified for 2-char repeats), strips zero-width characters,
    spaces before marks, and dangling non-base marks.
  - `NormalizeOptions` for per-rule control; `normalize_text_with_dict`
    consults the dictionary before de-elongation.
  - `ThaiTrie::contains` and `Tokenizer::trie` accessors; `set_default_dual`
    for separate words/lines engines.

## [1.1.0] - 2026-09-27

### Added
- **FST (Finite State Transducer) Engine in Rust**:
  - Pre-compiled binary dictionary `data/words.fst` (289.8 KB, 58.8% of plain text size).
  - High-performance `FstTrie` supporting zero-copy memory traversal (`&[u8]`).
  - Runtime dynamic word addition with zero-rebuild overhead via Dynamic Overlay.
  - Dedicated CLI compiler tool `thaibreak_compile_fst` to build FST dictionaries from any TSV wordlist.
- **Compact DAWG (Minimal DFA) Engine in Go**:
  - Ultra-compact binary dictionary `data/words.dawg` (167.4 KB, 34% of original size).
  - Native `CompactDawg` reader and unified `ThaiTrie` supporting both DAWG and flat hash maps.
  - Embedded binary dictionary via `//go:embed data/words.dawg` with zero runtime external dependencies.
  - Startup time reduced to **~48 µs** (300x faster) and RAM footprint reduced to **< 0.2 MB**.
- **Compact DAWG Engine in TypeScript / Node.js / Browser**:
  - Binary `CompactDawg` decoder using `DataView` and `Uint8Array` with zero external dependencies.
  - `ThaiTrie.fromBinary()` and `init({ dictBinary: ... })` options.
  - Startup time reduced to **~0.7 ms** and throughput reaching **~31.8 million lookups/sec**.
- **Compact DAWG & OPcache Preloading in PHP**:
  - Native `CompactDawg` class reading `words.dawg` (167 KB), dropping memory from ~8 MB to **< 0.5 MB** and load time to **~0.3 ms** (70x faster).
  - OPcache Preload array file `data/words.php` and generator tool `tools/build_php_dict.php` for **0.00 ms instant startup and 0 MB worker RAM**.
  - `DictionaryLoader::fromDawgFile()`, `fromPhpFile()`, and auto-detecting `fromFile()`.
- **Cross-Language Dictionary Compilers**:
  - `tools/build_dawg.py`: Generates `words.dawg` (167 KB) from any TSV/Text wordlist in under 1 second.
  - `tools/build_php_dict.php`: Generates OPcache-ready PHP associative arrays.
  - `rust/src/bin/thaibreak_compile_fst.rs`: Compiles FST binaries from wordlists.

### Changed
- Standardized Apache-2.0 license formatting across all subpackages (`go/LICENSE`, `rust/LICENSE`, `typescript/LICENSE`, `LICENSE`) for 100% automated SPDX license detection on `pkg.go.dev`.
- Updated default dictionary loading candidates across Go, Rust, TypeScript, and PHP to prioritize compact binary dictionaries (`words.dawg`, `words.fst`, `words.php`) over plain text.
- Bumped versions across all language packages: Rust `1.1.0`, TypeScript `1.1.0`, Python `1.1.0`.

---

## [1.0.3] - 2026-09-25

### Added
- Typographic line breaking conforming to Unicode 16.0 UAX #14.
- Zero-copy C-FFI exports and Python bindings.
- Multi-language packaging for npm (`thai-break`), crates.io (`thaibreak`), and Composer (`kamthorn/thai-break`).
