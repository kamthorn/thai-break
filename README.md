# ThaiBreak: High-Throughput Thai Word Segmentation & Typographic Line Breaking Engine

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

An open-source, deterministic, high-accuracy Thai word segmentation and typographic line breaking engine implemented in Rust, designed for high-throughput production environments (search indexing, database query parsing, web rendering, and typesetting).

## Key Highlights

- **State-of-the-Art Accuracy for Non-Neural Segmenters:** Achieves **86.71% Word F1** and **93.56% Boundary F1** on the 5.05-million-word NECTEC BEST Corpus benchmark, outperforming ICU4C (81.73%), LibThai (81.71%), ICU4X Dictionary (79.96%), and ICU4X LSTM (84.81%).
- **Ultra-High Throughput:** Processes text at **16.25 - 22.99 MB/s per core** (~1.46M - 2.06M words/second) and scales to **243.45 MB/s** (~21.95M words/second) on a 28-core system in native Rust.
- **Extended TCC Deterministic DFA:** Extends the classic Thai Character Cluster ruleset to properly capture compound vowel onsets (e.g., เครื่อง, เพลิง, เปล่า), colloquial loanword vowel shorteners (`[ข-ฮ]็อck` like ล็อก, ช็อก), and multi-character silent endings (`K`), compiled into a single leftmost-first Regex DFA for strict $O(N)$ scanning.
- **Direct 2-Level Thai Block Jump Table:** Uses a compact Finite State Transducer (`fst::Map`) dictionary with a direct index jump table for the Unicode Thai block (U+0E00..U+0E7F) that eliminates 2 levels of Trie state transitions.
- **Context-Aware Disambiguation & OOV Isolation:** Employs single-pass Viterbi DP with Bigram collocation rescoring, a dedicated abbreviation validator (`abbreviation_is_words`), and 39 grammatical function words that prevent out-of-vocabulary (OOV) tokens from swallowing adjacent particles.
- **Unicode UAX #14 Line Breaking:** Implements full Unicode Standard Annex #14 line break opportunities with visual terminal column width calculation (`display_width`).

## Architecture Overview

```
Input Text
    │
    ▼
[Text Normalizer]           (Maps common typing variants with zero-copy offset preservation)
    │
    ▼
[Extended TCC DFA]         (Enforces hard orthographic cluster constraints in O(N))
    │
    ▼
[Candidate Graph Search]    (FST Trie with 2-char jump table + Abbr + OOV syllable chunking)
    │
    ▼
[Viterbi DP Optimizer]      (Unigram log-odds cost + Bigram collocation bonus + Tie-breaking)
    │
    ▼
[Traceback & Output]        (Produces word tokens or zero-allocation line break offsets)
```

## Repository Structure

```
├── rust/                 # Core engine in Rust (thaibreak)
│   ├── src/tcc.rs        # Extended TCC DFA specification and table compiler
│   ├── src/trie.rs       # Zero-copy FST dictionary and 2-level prefix table
│   ├── src/tokenizer.rs  # Single-pass Viterbi DP and abbreviation disambiguation
│   ├── src/bigram.rs     # Bigram collocation model
│   └── src/linebreaker.rs# Unicode UAX #14 line breaking implementation
├── data/                 # Standard dictionaries (words.fst, words.txt, word-weights.tsv, bigrams.tsv)
├── python/               # Python bindings (PyO3)
├── typescript/           # TypeScript / JavaScript port
├── go/                   # Go port
├── typst/                # WebAssembly plugin for Typst typesetting
└── tools/                # Benchmark and validation utilities
```

## Requirements

- **Rust:** 1.85+ (or latest stable)
- **Cargo**

## Building the Core Engine

To build the optimized release library and tools:

```bash
cd rust
cargo build --release
```

## Running Unit & Correctness Tests

Run the full suite of unit tests, TCC property-based tests, and regression tests:

```bash
cd rust
cargo test
```

## Benchmark & Evaluation

Evaluation against the 5.05-million-word NECTEC BEST Corpus:

```bash
# Build the native benchmark runner
cargo build --release --bin bench_native

# Execute the evaluation
./target/release/bench_native
```

### Benchmark Results Summary (NECTEC BEST Corpus, 5,049,010 words)

| Engine | Architecture | Word F1 | Boundary F1 | Single-Core Speed | Multi-Core (28T) |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **This Work (Rust Native)** | **Viterbi DP + FST + Bigram** | **86.71%** | **93.56%** | **16.25 MB/s** | **243.45 MB/s** |
| ICU4X (dict) | Trie Dictionary (Rust) | 79.96% | 89.02% | 35.20 MB/s | 746.48 MB/s |
| ICU4C (dict) | Dictionary (C/C++) | 81.73% | 91.13% | 3.30 MB/s | - |
| LibThai | Maximal Matching (C) | 81.71% | 91.10% | 5.00 MB/s | - |
| ICU4X (lstm) | Neural LSTM (Rust) | 84.81% | 92.28% | 0.79 MB/s | - |

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
