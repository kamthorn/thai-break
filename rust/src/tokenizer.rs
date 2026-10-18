use once_cell::sync::Lazy;
use regex::Regex;

use crate::bigram::BigramModel;
use crate::tcc::tcc_pos_array;
use crate::trie::ThaiTrie;

/// Cost of an abbreviation pattern, relative to the cost of the rarest word.
const ABBR_COST_FACTOR: f64 = 1.5;
/// Extra cost per letter of an abbreviation pattern, so "เขต|จ." beats "เข|ตจ."
/// (a pattern taking the last letter of the previous word).
const ABBR_LETTER_COST_FACTOR: f64 = 0.01;
/// Cost of an unknown-word fallback edge, relative to the cost of the rarest word.
const UNKNOWN_COST_FACTOR: f64 = 2.0;
/// An out-of-vocabulary word may span up to this many TCC clusters. Such an edge competes with the
/// dictionary words, so a long unknown word is kept whole instead of being cut into short words.
const OOV_MAX_CLUSTERS: usize = 6;
/// Cost of an out-of-vocabulary edge of one cluster, relative to the cost of the rarest word.
const OOV_COST_FACTOR: f64 = 3.0;
/// Extra cost per further cluster of an out-of-vocabulary edge, relative to the cost of the rarest
/// word. Below about 0.5 an unknown word beats real words and F1 drops sharply.
const OOV_CLUSTER_COST_FACTOR: f64 = 0.8;
/// Frequent function words. An out-of-vocabulary edge may not start or end with one of them, nor
/// contain one at a cluster boundary, so an unknown word does not swallow its neighbours
/// ("ฮิวจ์ส|ไม่|ได้", not "ฮิวจ์สไม่ได้"; "ชวรัตน์|ที่|ลืม|ไว้", not "ชวรัตน์ที่ลืมไว้"): the same name
/// would otherwise be tokenized differently on its own and next to a function word, and a search
/// for it would miss the document.
const OOV_BOUNDARY_WORDS: &[&str] = &[
    "ที่", "และ", "ของ", "ใน", "ได้", "ให้", "ไม่", "ว่า", "เป็น", "มี", "จะ", "ไป", "มา", "ก็", "กับ", "แต่", "หรือ", "จาก",
    "โดย", "เพื่อ", "แล้ว", "อยู่", "นี้", "นั้น", "ซึ่ง", "การ", "ความ", "ต่อ", "ถึง", "ยัง", "เมื่อ", "ทั้ง", "ตาม", "ด้วย",
    "อีก", "คือ", "ไว้",
    // words that often come right before a name
    "นาย", "นาง", "พี่", "น้อง", "แพทย์", "ตัว",
];
/// Function words that may still occur inside an out-of-vocabulary word: they are frequent
/// syllables of transliterated names (อาห์มาดิเนจัด, อะมีรุ้ลฮัจย์).
const OOV_INNER_ALLOWED: &[&str] = &["มา", "มี"];
/// The longest dictionary word that is looked up, in characters (a base word has at most 19, and the royal
/// title พระบาทสมเด็จพระเจ้าอยู่หัว has 26).
const MAX_WORD_LEN: usize = 40;
/// Names of the letters of the Latin alphabet in Thai. A name that starts another one comes first
/// (เอ็ม before เอ), so the longest name wins.
const LETTER_NAMES: &[&str] = &[
    "ดับเบิลยู", "เอ็กซ์", "แอล", "เอ็ม", "เอ็น", "เอฟ", "เอช", "เอส", "อาร์", "คิว", "แซด", "ซี", "ดี", "จี", "ไอ", "เจ", "เค",
    "โอ", "พี", "ยู", "วี", "วาย", "บี", "ที", "อี", "เอ",
];
/// Whether a letter name can start with `ch`; a cheap test before comparing every name.
fn starts_a_letter_name(ch: char) -> bool {
    matches!(ch, 'ด' | 'เ' | 'แ' | 'อ' | 'ค' | 'ซ' | 'จ' | 'ไ' | 'โ' | 'พ' | 'ย' | 'ว' | 'บ' | 'ท')
}
static LETTER_NAME_CHARS: Lazy<Vec<Vec<char>>> = Lazy::new(|| LETTER_NAMES.iter().map(|n| n.chars().collect()).collect());
/// Letter names by first character, as an offset from U+0E00, in `LETTER_NAMES` order.
static LETTER_NAMES_BY_FIRST: Lazy<Vec<Vec<usize>>> = Lazy::new(|| {
    let mut by_first = vec![Vec::new(); 0x80];
    for (k, name) in LETTER_NAME_CHARS.iter().enumerate() {
        by_first[name[0] as usize - 0x0E00].push(k);
    }
    by_first
});
/// Cost of a run of two or more spelled-out letters (เอทีเอ็ม, ยูเอ็น), relative to the cost of the
/// rarest word.
const LETTER_RUN_COST_FACTOR: f64 = 1.0;
/// Costs closer than this are a tie. Edges into a position are visited from
/// the longest word to the shortest, so on a tie the later, shorter last word
/// wins and the earlier words stay longer ("ผิด|ราย" rather than "ผิ|ดราย").
const TIE_EPSILON: f64 = 1e-9;

static PAT_NONTHAI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?:[a-zA-Z]+(?:[-_'][a-zA-Z0-9]+)*|\d+(?:,\d+)*(?:\.\d+)?%?|[ \t]+|\r?\n|[^\x{0e00}-\x{0e7f}a-zA-Z0-9 \t\r\n])",
    )
    .expect("Failed to compile non-Thai regex")
});

static PAT_ABBR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:(?:[เแโใไ]?[ก-ฮ][ัิีึืุู็่้๊๋]?|[ก-ฮ]{1,4})\.)+")
        .expect("Failed to compile abbreviation regex")
});

/// Normalize common Thai spelling variants for dictionary matching:
/// เ + เ → แ, ํ + า → ำ, ํ + tone + า → tone + ำ (e.g. "นํ้า" → "น้ำ"), and a
/// tone mark typed after Sara Am is moved before it (ำ + tone → tone + ำ, "นำ้" → "น้ำ").
/// Returns the normalized characters and, for each of them, the index of the
/// original character it starts at (plus a final entry for the end). Token
/// boundaries never fall inside a replaced pair, so tokens map back to exact
/// slices of the original text.
fn normalize_for_matching(chars: &[char]) -> (Vec<char>, Vec<usize>) {
    let n = chars.len();
    let mut norm = Vec::with_capacity(n);
    let mut orig = Vec::with_capacity(n + 1);
    let at = |i: usize| chars.get(i).copied().unwrap_or('\0');
    let mut i = 0;
    while i < n {
        let next = at(i + 1);
        if chars[i] == 'เ' && next == 'เ' {
            norm.push('แ');
            orig.push(i);
            i += 2;
        } else if chars[i] == 'ำ' && is_tone_mark(next) {
            norm.extend([next, 'ำ']);
            orig.extend([i, i]);
            i += 2;
        } else if chars[i] == '\u{0E4D}' && next == 'า' && is_tone_mark(at(i + 2)) {
            norm.extend([at(i + 2), 'ำ']);
            orig.extend([i, i]);
            i += 3;
        } else if chars[i] == '\u{0E4D}' && next == 'า' {
            norm.push('ำ');
            orig.push(i);
            i += 2;
        } else if chars[i] == '\u{0E4D}' && is_tone_mark(next) && at(i + 2) == 'า' {
            norm.extend([next, 'ำ']);
            orig.extend([i, i]);
            i += 3;
        } else {
            norm.push(chars[i]);
            orig.push(i);
            i += 1;
        }
    }
    orig.push(n);
    (norm, orig)
}

#[inline]
fn is_tone_mark(ch: char) -> bool {
    ('่'..='๋').contains(&ch)
}

/// Characters an out-of-vocabulary edge may cover: Thai letters, vowels and marks, but not ๆ, ฯ,
/// digits or other symbols, which are tokens of their own.
#[inline]
fn is_oov_char(ch: char) -> bool {
    matches!(ch as u32, 0x0E01..=0x0E2E | 0x0E30..=0x0E3A | 0x0E40..=0x0E45 | 0x0E47..=0x0E4E)
}

const _: () = assert!(OOV_BOUNDARY_WORDS.len() <= 64);

static FUNCTION_WORD_CHARS: Lazy<Vec<Vec<char>>> =
    Lazy::new(|| OOV_BOUNDARY_WORDS.iter().map(|f| f.chars().collect()).collect());
/// Function words that end an OOV edge only when the edge has no room for them inside.
static FUNCTION_WORD_INNER_MASK: Lazy<u64> = Lazy::new(|| {
    OOV_BOUNDARY_WORDS
        .iter()
        .enumerate()
        .filter(|(_, f)| !OOV_INNER_ALLOWED.contains(f))
        .fold(0, |m, (k, _)| m | (1 << k))
});
/// Indices into `OOV_BOUNDARY_WORDS` by first character, as an offset from U+0E00.
static FUNCTION_WORDS_BY_FIRST: Lazy<Vec<Vec<usize>>> = Lazy::new(|| {
    let mut by_first = vec![Vec::new(); 0x80];
    for (k, f) in FUNCTION_WORD_CHARS.iter().enumerate() {
        by_first[f[0] as usize - 0x0E00].push(k);
    }
    by_first
});

/// For each character index p, bit k of `starts[p]` is set when `OOV_BOUNDARY_WORDS[k]` begins
/// at p, and bit k of `ends[p]` when it ends at p.
fn function_word_positions(chars: &[char]) -> (Vec<u64>, Vec<u64>) {
    let n = chars.len();
    let mut starts = vec![0u64; n + 1];
    let mut ends = vec![0u64; n + 1];
    for p in 0..n {
        // Every function word starts with a Thai character
        let first = (chars[p] as u32).wrapping_sub(0x0E00) as usize;
        let Some(words) = FUNCTION_WORDS_BY_FIRST.get(first) else {
            continue;
        };
        for &k in words {
            let f = &FUNCTION_WORD_CHARS[k];
            if chars[p..].starts_with(f) {
                starts[p] |= 1 << k;
                ends[p + f.len()] |= 1 << k;
            }
        }
    }
    (starts, ends)
}

#[inline]
fn is_thai_rune(ch: char) -> bool {
    let cp = ch as u32;
    (0x0E00..=0x0E7F).contains(&cp)
}

#[derive(Clone, Debug)]
pub struct Tokenizer {
    trie: ThaiTrie,
    bigram_model: Option<BigramModel>,
}

impl Tokenizer {
    pub fn new(trie: ThaiTrie, bigram_model: Option<BigramModel>) -> Self {
        Self { trie, bigram_model }
    }

    pub fn set_bigram_model(&mut self, model: Option<BigramModel>) {
        self.bigram_model = model;
    }

    pub fn trie(&self) -> &ThaiTrie {
        &self.trie
    }

    pub fn trie_mut(&mut self) -> &mut ThaiTrie {
        &mut self.trie
    }

    /// Whether what the abbreviation pattern matched is really words and a full stop. The pattern
    /// accepts any one to four consonant letters before a full stop, and อ ว ย count as consonants,
    /// so it matches "ไป." in "ต่อไป." and "ของส." in "ของส.ส.". It is not an abbreviation when
    /// - it starts with a function word followed by more letters ("ของส.ส." is ของ + ส.ส.), or
    /// - it is one dictionary word with a vowel or tone mark and a final full stop that is not followed
    ///   by a letter ("ต่อไป."), unless the dictionary has the abbreviation itself. A string of consonants
    ///   only (มทส. นพ.) stays an abbreviation even when the dictionary has it without the full stop.
    ///
    /// `end` is the index of the character after the match.
    fn abbreviation_is_words(&self, chars: &[char], end: usize, abbr: &str) -> bool {
        let first = abbr.split('.').next().unwrap_or("");
        if OOV_BOUNDARY_WORDS.iter().any(|f| first.len() > f.len() && first.starts_with(f)) {
            return true;
        }
        let next_is_thai = chars.get(end).is_some_and(|c| ('\u{0e01}'..='\u{0e5b}').contains(c));
        abbr.matches('.').count() == 1
            && abbr.ends_with('.')
            && first.chars().count() >= 2
            && first.chars().any(|c| !('ก'..='ฮ').contains(&c))
            && !next_is_thai
            && self.trie.contains(first)
            && !self.trie.contains(abbr)
    }

    pub fn tokenize(&self, text: &str, keep_whitespace: bool) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let chars: Vec<char> = text.chars().collect();
        let mut tokens: Vec<String> = self
            .token_spans(text, &chars)
            .into_iter()
            .map(|(start, end)| chars[start..end].iter().collect())
            .collect();

        if !keep_whitespace {
            tokens.retain(|t| !t.trim().is_empty());
        }
        tokens
    }

    /// Byte offsets of the token boundaries of `text`, whitespace included, from 0 to the text
    /// length: the boundaries of `tokenize(text, true)`, without building the token strings.
    pub(crate) fn boundaries(&self, text: &str) -> Vec<usize> {
        let mut out = vec![0];
        if text.is_empty() {
            return out;
        }
        let chars: Vec<char> = text.chars().collect();
        let spans = self.token_spans(text, &chars);
        out.reserve(spans.len());
        let (mut byte, mut ci) = (0, 0);
        for (_, end) in spans {
            byte += chars[ci..end].iter().map(|c| c.len_utf8()).sum::<usize>();
            ci = end;
            out.push(byte);
        }
        out.dedup();
        out
    }

    /// Tokens as character ranges of `chars` (the characters of `text`), whitespace included.
    /// Matching runs on a normalized copy; token boundaries never fall inside a replaced pair,
    /// so the ranges map back to exact slices of the original characters.
    fn token_spans(&self, text: &str, chars: &[char]) -> Vec<(usize, usize)> {
        let (norm, orig_pos) = normalize_for_matching(chars);
        if norm == chars {
            return self.segment(&norm, Some(text));
        }
        let mut spans = self.segment(&norm, None);
        for span in spans.iter_mut() {
            *span = (orig_pos[span.0], orig_pos[span.1]);
        }
        spans
    }

    /// Run the Viterbi segmentation over `chars` and return all tokens as character ranges,
    /// including whitespace. Every edge covers `chars[i..j]`, so the DP keeps
    /// only the predecessor of each position and slices the words at the end.
    /// `source` is the text of `chars`, if the caller has it.
    fn segment(&self, chars: &[char], source: Option<&str>) -> Vec<(usize, usize)> {
        let owned: String;
        let text = match source {
            Some(text) => text,
            None => {
                owned = chars.iter().collect();
                owned.as_str()
            }
        };
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        let valid_pos = tcc_pos_array(chars);
        let (fw_starts, fw_ends) = function_word_positions(chars);
        let inner_mask = *FUNCTION_WORD_INNER_MASK;

        // Precompute character index to byte offset mapping
        let mut char_byte_offsets = Vec::with_capacity(n + 1);
        let mut b = 0;
        for &ch in chars {
            char_byte_offsets.push(b);
            b += ch.len_utf8();
        }
        char_byte_offsets.push(b);

        // Unigram costs: -log(weight / total); a word of weight 1 costs rare_cost.
        // Non-Thai tokens cost rare_cost, abbreviation patterns and unknown-word
        // fallbacks more, so a dictionary word followed by "." beats a pattern
        // such as "ว." that would cut the word.
        let normalizer = self.trie.total_weight() + 1.0;
        let rare_cost = normalizer.ln();

        // Single-pass Viterbi DP. Positions are visited in order. When position
        // i is reached, every edge into it has been relaxed, so dp[i] is final:
        // an unreachable i gets an unknown-word edge, then the edges starting at
        // i are relaxed right away. Edges are never stored, so memory stays O(n)
        // for any text length.
        let mut dp = vec![f64::INFINITY; n + 1];
        let mut from = vec![usize::MAX; n + 1];
        let mut is_unk = vec![false; n + 1];
        dp[0] = 0.0;

        let mut prefixes: Vec<(usize, f64)> = Vec::new();

        for i in 0..=n {
            if !valid_pos[i] {
                continue;
            }

            // Unknown-word fallback: connect to nearest reachable predecessor
            if dp[i].is_infinite() {
                if let Some(k) = (0..i).rev().find(|&k| !dp[k].is_infinite() && valid_pos[k]) {
                    dp[i] = dp[k] + UNKNOWN_COST_FACTOR * rare_cost;
                    from[i] = k;
                    is_unk[i] = true;
                }
            }

            if i == n {
                break;
            }

            // Edges starting at i are relaxed as they are found: they all end after i, and
            // finding them reads no dp state, so only the order of relaxation matters.
            let prev: String = match &self.bigram_model {
                Some(_) if from[i] != usize::MAX => chars[from[i]..i].iter().collect(),
                _ => String::new(),
            };
            let has_prev = from[i] != usize::MAX;
            let dp_i = dp[i];
            let mut relax = |j: usize, cost: f64, unknown: bool| {
                let edge_cost = match &self.bigram_model {
                    Some(bigrams) if has_prev => {
                        let word: String = chars[i..j].iter().collect();
                        (cost - bigrams.get_bonus(&prev, &word, j - i)).max(0.01)
                    }
                    _ => cost,
                };
                // Ties go to the later start, i.e. the shorter last word (see TIE_EPSILON)
                let new_cost = dp_i + edge_cost;
                if new_cost <= dp[j] + TIE_EPSILON {
                    dp[j] = new_cost;
                    from[j] = i;
                    is_unk[j] = unknown;
                }
            };
            let sub_text = &text[char_byte_offsets[i]..];

            if is_thai_rune(chars[i]) {
                // 1. Thai dictionary words starting at i
                prefixes.clear();
                self.trie.prefix_ends_from_chars(chars, i, MAX_WORD_LEN, &mut prefixes);
                for &(end, weight) in &prefixes {
                    if end <= n && valid_pos[end] {
                        relax(end, (normalizer / weight).ln(), false);
                    }
                }

                // 2. Thai abbreviation patterns. A match has a full stop within its first five
                // characters, so the regex only runs where there is one.
                let has_stop = chars[i + 1..n.min(i + 5)].contains(&'.');
                if let Some(m) = has_stop.then(|| PAT_ABBR.find(sub_text)).flatten() {
                    let m_str = m.as_str();
                    let abbr_len = m_str.chars().count();
                    let j = i + abbr_len;
                    if j <= n && valid_pos[j] && !self.abbreviation_is_words(chars, i + abbr_len, m_str) {
                        let letters = abbr_len - m_str.matches('.').count();
                        relax(j, (ABBR_COST_FACTOR + ABBR_LETTER_COST_FACTOR * letters as f64) * rare_cost, false);
                    }
                }

                // 3. Runs of two or more spelled-out letters (เอทีเอ็ม, ยูเอ็น, ดีเอสไอ)
                let mut end = i;
                let mut letters = 0;
                while let Some(len) = chars
                    .get(end)
                    .filter(|&&c| starts_a_letter_name(c))
                    .and_then(|&c| {
                        LETTER_NAMES_BY_FIRST[c as usize - 0x0E00]
                            .iter()
                            .map(|&k| &LETTER_NAME_CHARS[k])
                            .find(|name| chars[end..].starts_with(name) && valid_pos[end + name.len()])
                    })
                    .map(|name| name.len())
                {
                    end += len;
                    letters += 1;
                    if letters >= 2 {
                        relax(end, LETTER_RUN_COST_FACTOR * rare_cost, false);
                    }
                }

                // 4. Out-of-vocabulary words of 1..=OOV_MAX_CLUSTERS TCC clusters
                let mut clusters = 0;
                for j in i + 1..=n {
                    if !is_oov_char(chars[j - 1]) {
                        break;
                    }
                    if !valid_pos[j] {
                        continue;
                    }
                    clusters += 1;
                    if clusters > OOV_MAX_CLUSTERS {
                        break;
                    }
                    let edge_bytes = char_byte_offsets[j] - char_byte_offsets[i];
                    let mut hits = fw_starts[i] | fw_ends[j];
                    let mut borders = false;
                    while hits != 0 {
                        let k = hits.trailing_zeros() as usize;
                        hits &= hits - 1;
                        borders |= edge_bytes > OOV_BOUNDARY_WORDS[k].len();
                    }
                    if borders {
                        let mut inner = fw_ends[j] & inner_mask;
                        let mut inner_hit = false;
                        while inner != 0 {
                            let k = inner.trailing_zeros() as usize;
                            inner &= inner - 1;
                            let len = FUNCTION_WORD_CHARS[k].len();
                            inner_hit |= j - i > len && valid_pos[j - len];
                        }
                        if inner_hit {
                            break;
                        }
                        continue;
                    }
                    relax(j, (OOV_COST_FACTOR + OOV_CLUSTER_COST_FACTOR * (clusters - 1) as f64) * rare_cost, true);
                }
            } else if let Some(m) = PAT_NONTHAI.find(sub_text) {
                // 5. Non-Thai tokens
                let j = i + m.as_str().chars().count();
                if j <= n && valid_pos[j] {
                    relax(j, rare_cost, false);
                }
            }
        }

        // Traceback
        if dp[n].is_infinite() {
            return vec![(0, n)];
        }

        let mut spans = Vec::new();
        let mut pos = n;
        while pos > 0 {
            spans.push((from[pos], pos, is_unk[pos]));
            pos = from[pos];
            if pos == usize::MAX {
                break;
            }
        }
        spans.reverse();

        // Syllable-based OOV chunking: consecutive unknown Thai edges are one token
        let mut tokens = Vec::with_capacity(spans.len());
        let mut chunk: Option<(usize, usize)> = None;
        for (start, end, unk) in spans {
            if unk && chars[start..end].iter().all(|&c| is_thai_rune(c)) {
                chunk = Some((chunk.map_or(start, |c| c.0), end));
            } else {
                tokens.extend(chunk.take());
                tokens.push((start, end));
            }
        }
        tokens.extend(chunk);

        tokens
    }
}
