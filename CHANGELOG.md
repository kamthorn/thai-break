# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **10,133 Thai given names merged into `data/words.txt`** (36,040 words total, up from
  25,907): fixes given names composed of two dictionary morphemes wrongly splitting
  (`สมศักดิ์` → `สม|ศักดิ์`, `ณัฐพล` → `ณัฐ|พล`, `สมชัย` → `สม|ชัย`, ...). Sourced from
  PyThaiNLP's `person_names_female_th.txt`/`person_names_male_th.txt` (Apache-2.0), selected
  with `tools/select_names.py`: a name is only added if it does *not* also occur as an
  ordinary two-word sequence outside a person-name span in the LST20 gold-segmented corpus
  (e.g. `โชคดี` "lucky" is excluded so it keeps splitting normally in ordinary sentences).
  LST20 is used only to validate/select which words to add — no LST20 text is included in
  the dictionary. Validated on the LST20 **test** split (held out from selection): word
  boundary F1 94.03% → 94.21% (net +814 correct boundaries: 883 fixed vs. 69 newly wrong,
  across 483 documents). Regenerated `data/words.dawg`, `data/words.fst`, `data/words.php`,
  and the Go/TypeScript embedded copies from the updated `data/words.txt`.
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
