# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.5.0] - 2026-10-19

### Fixed
- **A Rust panic no longer aborts the host process.** `panic = "abort"` is gone from the release profile, so
  the `catch_unwind` in the C API and the PyO3 binding work: a panic returns an error (NULL / a Python
  exception) instead of killing the Python, PHP or C program that loaded the library. The library is 59 KB
  (3%) larger and about 2% slower; the Typst wasm is unchanged (wasm always aborts). `cargo test --release`
  works now.
- **A word added to the Rust dictionary no longer hides the dictionary word that is its prefix.** After
  `add("ไปตำ")`, `contains("ไป")` was false because the overlay holds `ไป` as a prefix marker, so the full stop of
  `ต่อไป.` was read as part of an abbreviation (`ต่อ|ไป.`). This hit custom words in Rust and Python; Go,
  TypeScript and PHP were not affected.
- **Go, TypeScript and PHP check a DAWG file when they load it.** A header with more states than the file has
  bytes, a state that runs past the end, or (Go, TypeScript) an edge to a state that does not exist is an error
  now. A forged header (`numStates = 0xFFFFFFFF`) used to allocate gigabytes. PHP load time is unchanged (0.4 ms).
- **The ports tokenize like the Rust core.** Go and TypeScript counted Thai digits as separate from ASCII digits
  (`1๐.๐๐` was five tokens, now one as in Rust and PHP); TypeScript kept U+0085 as a token and cut the -ีย vowel
  before a non-Thai character differently; PHP did that before a space. Go, TypeScript and PHP now give the
  same tokens as Rust on 2,299 short sentences, 150,000 random lines and 65,014 news paragraphs (before: Go
  differed on 4 news paragraphs and 428 random lines, TypeScript on 69 and 451, PHP on 2 random lines).
  `getTCCPattern()` in TypeScript now returns the general rules only; `getTCCLookaheadPattern()` returns the
  lookahead rules.

### Changed
- **The base dictionary has 393 more words** (25,411 to 25,804): the names, abbreviations and spellings that
  tripped the segmenter most often on news text, taken from the lists of `thai-break-dict-extra` that
  are our own or CC0 (PyThaiNLP): `เว็บไซต์`, `สหรัฐอเมริกา`, `เกาหลีเหนือ`, `เกาหลีใต้`, `กรุงเทพฯ`,
  `กรุงเทพมหานคร`, `พระนครศรีอยุธยา`, `เด็กชาย`, `เด็กหญิง`, `ส.ส.`, `อบต.`, `เสธ.`, `เรอะ`, country names, common abbreviations,
  frequent misspellings (`เฟสบุ๊ค`, `อนุญาติ`), and car and shop brands. Word F1 on the base dictionary goes up on every
  test set: LST20 test 86.9 to 87.1, LST20 eval 83.5 to 83.8, Blackboard 86.6 to 86.8, Wisesight-1000 84.5 to 84.7.
  On 65,014 news paragraphs 3,068 are segmented differently, always because of one of the new words; the
  base plus dict-extra scores were unchanged by these words because dict-extra already had them (it gained 19 compounds later, see below). **Tokens change,
  so reindex if you search on `words()` output.** A single dictionary word wider than the line is cut at cluster boundaries
  when `wrap(..., cutLongWords)` is used, so `กรุงเทพมหานคร` in a 4-column line is now `กรุง|เทพม|หานค|ร`
  instead of `กรุง|เทพ|มหา|นคร`.
- **thai-break-dict-extra has 19 more words** (14,381), mostly compounds kept whole (`ค้นคว้า`, `ค้นหา`, `สวยงาม`,
  `พูดคุย`, `เติบโต`, `ตัดสินใจ`, `ค่อนข้าง`, `แกนนำ`, `ยาบ้า`, ...); `ค้าขาย` has weight 1.0 so it does not
  take `ค้า` from `แม่ค้า`. With the base dictionary plus dict-extra, word F1 is LST20 test 84.0 (was 83.7),
  LST20 eval 80.9 (80.6), Blackboard 84.5 (84.2), Wisesight-1000 82.8 (82.7); the base dictionary alone
  is 87.1 / 83.9 / 86.8 / 84.8.
- **Faster segmentation again, same tokens.** Thai character clusters are matched with a DFA built from the
  same cluster rules, instead of a regular expression: in Rust at first use (157 states, 9 KB, 0.7 ms), in
  Go and TypeScript from tables that `cargo run --example thaibreak_gen_tcc` writes into `go/tcc_dfa.go` and
  `typescript/src/tcc_dfa.ts` (a Rust test fails when they are out of date). Rust `boundaries` no longer
  builds a string per word. On news text: Rust `thaibreak_boundaries` 1.34x faster (within 12% of libthai),
  Go 1.7x, TypeScript 1.16x. PHP keeps PCRE, which is already fast there. The tokens are identical on 65,014
  news paragraphs, short sentences and 150,000 random lines (`docs/performance.md`,
  `docs/performance-ports.md`).
- Typst plugin 0.2.0: `typst/thaibreak.wasm` is rebuilt from the 1.4.1 core, so it has the out-of-vocabulary
  words, word weights and bug fixes of 1.3.0-1.4.1 (it was built before 1.3.0). For example
  `ชวรัตน์` and `เอทีเอ็ม` are one word. The file is 21 KB larger (1.38 MB).
- `tools/sync_dict.sh rebuild` builds `words.dawg`, `words.fst` and `words.php` from `data/words.txt` and stages
  the copies; `words.php` no longer holds its build time, so a rebuild gives the same file.

## [1.4.1] - 2026-10-16

### Changed
- **Faster segmentation, same tokens.** Rust is about 11 times faster (13.2 s to 1.2 s for 8M characters
  of news): the `regex` crate is built with its DFA (`perf-dfa`; TCC took 70% of the time without it),
  the DP keeps no word string per edge, function words are found once per text as bit masks, and the
  dictionary walk reuses a buffer and caches the FST states after the first Thai characters. Go, TypeScript
  and PHP use the same techniques and are 2-4 times faster; PHP no longer revalidates the UTF-8 of the
  rest of the text at every match. The tokens are identical on 2.8M characters of news and random text
  (`docs/performance.md`, `docs/performance-ports.md`). The Rust crate grows by about 100 KB with `perf-dfa`.
- Example service: CORS is closed by default. It sent `Access-Control-Allow-Origin: *`, so any web
  page could call the API from a visitor's browser, which matters when the service runs inside a
  network. Set `CORS_ALLOWED_ORIGINS` (comma-separated, or `*`) to allow browser calls. `marker`
  and `delimiter` may be at most 16 characters (was 64), which limits how much larger than its
  input a response can get.

### Security
- CI: the actions are pinned to commit SHAs (Dependabot keeps them current) and the workflows
  have read-only `GITHUB_TOKEN` permissions.

### Fixed
- PHP: `DictionaryLoader::fromFile()` ran any file ending in `.php` with `require`, and
  `ThaiTokenizer::withDefaultDict()` also looked for `data/words.php` in the working directory, so
  a dictionary path from the environment or a writable working directory could run code. `fromFile()`
  now refuses `.php` (call `fromPhpFile()` for a trusted one), only the copies that ship with the
  package are used as the default `words.php`, and a `dict_path` or bigram path that does not
  exist is an error instead of an empty dictionary.
- Example service: a dictionary or bigram path given by option or environment that cannot be loaded
  stops the start-up with a message, instead of serving the embedded dictionary.
- Rust: a TBF1 header whose word count overflows `count * 8` on a 32-bit target (wasm32) is an
  error instead of a panic. The C header documents where the default dictionary is looked for.
- Example service: the `docker-compose.yml` healthcheck called `curl`, which the image does not
  have, so the container was always reported unhealthy. It uses `wget` now.
- Example service: `bahttext` with a text amount near the `i64` limit, such as
  `9223372036854775807.995`, overflowed when the rounded satang carried into the baht (a panic in
  debug builds, a wrong amount in release builds). It now returns "Baht amount out of range".
- C: `thaibreak_tokenize` handed out a `Vec` buffer whose capacity may exceed its length, and
  `thaibreak_free_tokens` rebuilt it assuming they are equal. The array is now a boxed slice, so
  the free matches the allocation.

### Security
- Laravel: `@thaibreak` and `@thailinebreak` printed their text unescaped, and HTML in it was kept
  as is, so `@thaibreak($post->content)` let user content inject markup and scripts (XSS). They now
  escape the text as `{{ }}` does. The second argument is the break marker; the old boolean
  `$isHtml` argument is ignored. `@thaibreakHtml($html, $marker)` keeps the tags of trusted HTML
  unescaped, like `{!! !!}`.
- HTML mode (Rust, Go, TypeScript, PHP and the example service) took quadratic time on markup
  without its terminator, such as many `<script>`, `<!--` or `<a` without `</script>`, `-->` or
  `>`: 320 KB of `<script>` took about 13 s. A linear scanner replaces the regex and finds the same
  tags, entities, comments and script and style blocks, so the output is unchanged. TypeScript
  also detects HTML in linear time now.

## [1.4.0] - 2026-10-12

### Documentation
- The accuracy tables report two dictionaries: the base dictionary alone, and the base plus
  thai-break-dict-extra (the set the OpenSearch plugin and the example service use). dict-extra keeps
  compounds and named entities whole on purpose, so word F1 on the gold corpora, which split them,
  is lower (LST20 test 86.9 → 83.7, boundary F1 93.8 → 92.2). The numbers do not measure retrieval.

### Added
- Rust: `BreakIterator`, a boundary iterator in the style of ICU `BreakIterator` and
  `Intl.Segmenter` (`first_boundary`, `last_boundary`, `next_boundary`, `previous`, `current`,
  `following`, `preceding`, `is_boundary`; byte offsets, `None` for ICU's `DONE`). It uses the
  Viterbi boundaries, so the segmentation is unchanged. Whitespace and punctuation are segments.
- `boundaries(text)` in every language: C `thaibreak_boundaries` / `thaibreak_free_boundaries`
  (byte offsets), Python (str indices, ctypes wheel and PyO3), WebAssembly and TypeScript (UTF-16
  offsets), Go (`Boundaries`, byte offsets) and PHP (`ThaiBreak::boundaries`, bytes, or code points
  with `$inCharacters`). Rust, Go, TypeScript and PHP give identical boundaries on 51,475 LST20
  texts.
- Word weights: `set_weight` / `apply_weights` / `apply_weights_file` on Rust `ThaiTrie`, C
  `thaibreak_load_weights`, Go `SetWeight` / `ApplyWeights`, TypeScript `setWeight` / `applyWeights`,
  PHP `setWeight` / `applyWeights`. They set the weight of words already in the dictionary and keep
  the unigram total exact. `tools/build_word_weights.py` builds `data/word-weights.tsv`, and
  `tools/benchmark.py --weights` measures a dictionary with them.

### Changed
- thai-break-dict-extra drops the loanwords harvested from PyThaiNLP `th_en_transliteration` (CC BY 4.0, not CC0 as
  labelled before). The extra dictionary has 14,331 words (was 17,819), and the lines dictionary 5,663 (was 9,190).
- thai-break-dict-extra no longer contains entries selected from LST20, which is used only to measure
  accuracy. The standard abbreviation lists (ranks, titles, addresses, units, months) are restored. The
  base plus extra dictionary has 38,483 words, and `examples/service/data/*.fst` is rebuilt from it.
- **An out-of-vocabulary word may not contain a frequent function word either**, only start or end
  with one was forbidden before, and `ไว้` joins the list (37 words). A name followed by a function
  word and a short word came out as one token (`ชวรัตน์ที่ลืมไว้`), because every word of the
  base dictionary has the same weight and extending an unknown word by one cluster costs less
  than a separate word. It is now `ชวรัตน์|ที่|ลืม|ไว้`. `มา` and `มี` may still occur inside an
  unknown word, since they are frequent syllables of transliterated names (`อาห์มาดิเนจัด`). On
  LST20 test, unknown words that swallow a function word drop from 60 to 12, names (NE spans)
  tokenized the same alone and in context rise from 97.2% to 97.5%, and word F1 goes from 86.23
  to 86.28 (eval 82.95 to 83.00, Blackboard sub-word 85.91 to 85.97, Wisesight-1000 83.42 to
  83.90). Raising the per-cluster cost instead stopped the merging but split real names
  (`ศุภชัย`, `กอร์ปศักดิ์`) and lost 0.7 points of word F1. Output changes for text with unknown
  words, so indices built from it should be reindexed. Rust, Go, TypeScript and PHP give
  identical tokens on LST20 eval and test.
- **An out-of-vocabulary word may not swallow a title before a name either:** `นาย นาง พี่ น้อง
  แพทย์ ตัว` join the function-word list (43 words), so `พี่เต๋อ` is `พี่|เต๋อ` and the name is
  tokenized the same with and without its title. On LST20 test, unknown words that swallow a
  dictionary word drop from 144 to 127, names tokenized the same alone and in context rise from
  97.5% to 97.7%, and word F1 goes from 86.28 to 86.30 (eval 83.00 to 83.02, Blackboard sub-word
  85.97 to 85.98, Wisesight-1000 83.90 to 83.97).
- **The default dictionary has word weights.** `data/word-weights.tsv` gives the 3,000 most
  frequent words of the base dictionary a weight of (count / count of the 3,000th word)^0.5 (up to
  40.6 for `ที่`), counted on the Prachathai-67k news corpus (Apache-2.0, see `NOTICE`); every other
  word keeps weight 1. Until now every word had the same weight, so the cost model could not tell
  `ที่` from a rare word, and an unknown name swallowed the frequent words next to it. Word F1 goes
  from 86.30 to 86.56 on LST20 test (eval 83.02 to 83.19, Blackboard sub-word 85.98 to 86.25,
  Wisesight-1000 83.97 to 84.25), and with dict-extra from 84.07 to 84.53; unknown words that
  swallow a dictionary word drop from 127 to 97 on LST20 test. Weighting every word instead loses
  0.6 points, because frequent short words then split names. The word list itself stays public
  domain: `words.txt`, `words.fst` and `words.dawg` are unchanged, and each language applies the
  weights file next to the default dictionary (or the embedded copy); `words.php` has the weights
  built in. A dictionary loaded from an explicit path gets no weights. Segmentation speed is
  unchanged in Rust and about 2.5% (Go) to 4% (TypeScript) slower; packages grow by 71 KB. Output
  changes for some text, so indices built from it should be reindexed. Rust, Go, TypeScript and PHP
  give identical tokens on LST20 eval and test.

- **A run of two or more spelled-out letters is one token** (`เอทีเอ็ม`, `ยูเอ็น`, `ดีเอสไอ`,
  `เอสเอ็มอี`): the names of the letters of the Latin alphabet in Thai (`เอ บี ซี ดี อี เอฟ จี เอช
  ไอ เจ เค แอล เอ็ม เอ็น โอ พี คิว อาร์ เอส ที ยู วี ดับเบิลยู เอ็กซ์ วาย แซด`) were cut into single
  letters, so a search for `ดีเอสไอ` with AND matched any text with `ดี`, `เอส` and `ไอ` in it.
  A single letter name stays a word, and a tone mark keeps `ที่` from being the letter T. Word F1
  goes from 86.58 to 86.93 on LST20 test (train 86.32 to 86.61, eval 83.20 to 83.47, Blackboard
  86.29 to 86.59, Wisesight-1000 84.26 to 84.50); 374 (eval) and 406 (test) LST20 lines change.
  It costs about 2% of the speed. Output changes for text with spelled-out letters, so indices
  built from it should be reindexed. Rust, Go, TypeScript, PHP and Java give identical tokens on
  LST20 eval and test.

### Fixed
- **A dictionary word can have up to 40 characters** (it was 25 in Rust, Go, TypeScript and PHP; the
  Java engine of the OpenSearch plugin takes the limit from the dictionary). A longer word of a
  user dictionary or of thai-break-dict-extra was never matched: the royal title
  `พระบาทสมเด็จพระเจ้าอยู่หัว` has 26 characters and came out as `พระ|บาท|สมเด็จ|…` however it was
  added. The base dictionary has no word over 19 characters, so its output does not change; the
  lookup walks the dictionary only as far as a word can continue, so the limit costs no time.
- **A word followed by a full stop is no longer read as an abbreviation.** The abbreviation pattern
  takes one to four consonant letters before a full stop, and อ ว ย count as consonants, so
  `ต่อไป.` came out as `ต่อ|ไป.`, `ได้.` as one token with its full stop, and a function word in
  front of an abbreviation was glued to it (`ของส.ส.`, `ของพ.ต.ท.`). It is not an abbreviation
  now when it starts with a function word followed by more letters (`ของ|ส.ส.`), or when it is
  one dictionary word with a vowel or tone mark and a final full stop that is not followed by a
  letter (`ต่อ|ไป|.`, `หรือ|ไม่|.`), unless the dictionary has the abbreviation itself. A string of
  consonants only (`มทส.`, `นพ.`) stays an abbreviation, and so does a run (`ก.พ.`, `ผบ.ตร.`) or an
  abbreviation followed by a name (`จ.|เชียงใหม่`). LST20 has 38 (eval) and 26 (test) lines with such
  text; word F1 does not fall on any corpus (LST20 test 86.56 to 86.58). Rust, Go, TypeScript, PHP
  and Java give identical tokens on LST20 eval and test.

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
