# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Rust: `BreakIterator`, a boundary iterator in the style of ICU `BreakIterator` and
  `Intl.Segmenter` (`first_boundary`, `last_boundary`, `next_boundary`, `previous`, `current`,
  `following`, `preceding`, `is_boundary`; byte offsets, `None` for ICU's `DONE`). It uses the
  Viterbi boundaries, so the segmentation is unchanged. Whitespace and punctuation are segments.
- `boundaries(text)` in the bindings: C `thaibreak_boundaries` / `thaibreak_free_boundaries` (byte
  offsets), Python (str indices, ctypes wheel and PyO3) and WebAssembly (UTF-16 offsets). Go,
  TypeScript and PHP do not have it yet.

## [1.3.0] - 2026-10-04

> **Reindex recommended.** Segmentation output changes for text with out-of-vocabulary words and
> for some syllables with Mai Han-akat or a consonant cluster after Sara E, so terms indexed with
> an earlier version may not match queries tokenized with this one.

### Changed

- **A long out-of-vocabulary word is no longer cut into short dictionary words.** The Viterbi
  search now also considers an unknown word of 1 to 6 TCC clusters at every position, with a cost
  that grows with its length (3.0 + 0.8 per further cluster, relative to the cost of the rarest
  word), so a name or loanword such as `ชวรัตน์` or `ศุภชัย` stays whole instead of becoming
  `ชว|รัตน์` and `ศุภ|ชัย`. The unknown word covers Thai letters, vowels and tone marks only, so
  `ๆ`, `ฯ` and digits stay separate tokens (`อื่น|ๆ`). On the LST20 test split word F1 goes from
  0.8522 to 0.8599 and from 7.9% to 25.3% of out-of-vocabulary person names come out as one
  token. Gold words missing from the dictionary that are cut wrongly drop from 12,734 to 12,037,
  while gold dictionary words that are cut wrongly rise from 5,433 to 5,955. On Blackboard F1
  goes from 0.6724 to 0.6751 (word level) and from 0.8516 to 0.8572 (sub-word level).
  Segmentation is about 9% slower. The trade-off is that an unknown word next to a short
  dictionary word can merge with it, which is why the unknown cost is not lower and why the
  function-word rule below exists. Rust, Go, TypeScript and PHP give identical tokens.
- **An out-of-vocabulary word may not start or end with a frequent function word** (36 words such
  as `ที่ และ ไม่ ได้ มา ว่า`). Without this rule an unknown word swallowed the function words
  next to it when the text has no spaces (`ฮิวจ์สไม่ได้`, `จินตะและ`), so the same name was
  tokenized differently on its own and in a document, and a search for it missed that document.
  In a retrieval test on the 483 LST20 test articles with 450 name queries, recall for names
  went from 0.986 to 0.990 (0.993 before out-of-vocabulary words competed with the dictionary),
  and LST20 test word F1 goes from 0.8599 to 0.8613. `รึยัง` is `รึ|ยัง` again.

Accuracy of the default dictionary on the four test sets (word F1 / boundary F1, 1.2.0 to 1.3.0):
LST20 test 85.2 / 92.6 to 86.1 / 93.3, LST20 eval 81.6 / 90.0 to 82.6 / 90.7, Blackboard sub-word
85.1 / 93.1 to 85.8 / 93.5, Wisesight-1000 83.0 / 90.3 to 83.3 / 90.2 (boundary F1 essentially
unchanged on social media text).

### Fixed

- A syllable with Mai Han-akat (`ั`) was cut after the vowel when it had no tone mark, so `ยัง`
  became the clusters `ยั|ง`, `หัว` became `หั|ว`, and `ผัวะ` became `ผั|วะ`. The TCC rule for
  `ั` now requires its final (a consonant, or `วะ` for -ัวะ), with an optional tone mark, so the
  syllable is one cluster. On the LST20 test split, clusters average 1.82 characters instead of
  1.74, every gold word boundary is still a cluster boundary, and segmentation is unchanged
  (word F1 0.8521 to 0.8522).
- After Sara E, a true consonant cluster (กร กล กว ขร ขล ขว คร คล คว ตร ปร ปล พร พล ผล บร บล ดร ทร)
  or a ห-led onset (หง หญ หน หม หย หร หล หว) followed by `ิ`, `ือ` or `า` was cut between the
  two consonants (`เป|ล่า`, `เค|รือ`, `เพ|ลิง`). These syllables are now one cluster. Other
  consonant pairs are left alone because they can start the next word (`เท|ลง`, `ทะเล|ว่า`).
  On LST20 (train, eval and test) clusters drop by 0.22% and no gold word boundary is lost.

## [1.2.0] - 2026-10-03

### Fixed

- A tone mark typed after Sara Am (`นำ้` instead of `น้ำ`), or after a decomposed Sara Am
  (`นํา้`), did not match the dictionary, so `นำ้ตาลทราย` was segmented as `นำ้|ตาล|ทราย`. The
  tokenizers in all languages now match these as tone mark + `ำ` (tokens keep the original
  text), and `normalize_text` in Rust rewrites them to `น้ำ`.

### Changed

- Python: `requires-python` is now `>=3.9`, matching the versions the wheels are built and
  tested for (3.9-3.13).

### Removed

- 505 dictionary entries spelled with a decomposed Sara Am (`ํ` + `า`, optionally with a tone
  mark between, e.g. `กระทํา`, `กระป่ํา`). Each one already had a `ำ` twin (`กระทำ`, `กระป่ำ`),
  and the tokenizers match text with Sara Am recomposed, so these entries could never match.
  `data/words.txt` now has 25,402 words; `words.fst`, `words.dawg` and `words.php` are rebuilt.
  Segmentation output is unchanged (LST20 test: word F1 0.8521, boundary F1 0.9263 before and
  after).

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
