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

/// An edge from the current position to `to`.
#[derive(Clone, Debug)]
struct DagEdge {
    to: usize,
    word: String,
    cost: f64,
}

#[inline]
fn is_thai_rune(ch: char) -> bool {
    let cp = ch as u32;
    (0x0E00..=0x0E7F).contains(&cp)
}

#[inline]
fn is_thai_string(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_thai_rune)
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

    pub fn tokenize(&self, text: &str, keep_whitespace: bool) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        let valid_pos = tcc_pos_array(&chars);

        // Precompute character index to byte offset mapping
        let mut char_byte_offsets = Vec::with_capacity(n + 1);
        let mut b = 0;
        for &ch in &chars {
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
        let mut word = vec![String::new(); n + 1];
        let mut is_unk = vec![false; n + 1];
        dp[0] = 0.0;

        let mut edges: Vec<DagEdge> = Vec::new();

        for i in 0..=n {
            if !valid_pos[i] {
                continue;
            }

            // Unknown-word fallback: connect to nearest reachable predecessor
            if dp[i].is_infinite() {
                if let Some(k) = (0..i).rev().find(|&k| !dp[k].is_infinite() && valid_pos[k]) {
                    dp[i] = dp[k] + UNKNOWN_COST_FACTOR * rare_cost;
                    from[i] = k;
                    word[i] = chars[k..i].iter().collect();
                    is_unk[i] = true;
                }
            }

            if i == n {
                break;
            }

            // Edges starting at i
            edges.clear();
            let sub_text = &text[char_byte_offsets[i]..];

            if is_thai_rune(chars[i]) {
                // 1. Thai dictionary words starting at i
                for m in self.trie.prefixes_from_chars(&chars, i, 25) {
                    if m.end <= n && valid_pos[m.end] {
                        edges.push(DagEdge {
                            to: m.end,
                            word: m.word,
                            cost: (normalizer / m.weight).ln(),
                        });
                    }
                }

                // 2. Thai abbreviation patterns
                if let Some(m) = PAT_ABBR.find(sub_text) {
                    let m_str = m.as_str();
                    let abbr_len = m_str.chars().count();
                    let j = i + abbr_len;
                    if j <= n && valid_pos[j] {
                        let letters = abbr_len - m_str.matches('.').count();
                        edges.push(DagEdge {
                            to: j,
                            word: m_str.to_string(),
                            cost: (ABBR_COST_FACTOR + ABBR_LETTER_COST_FACTOR * letters as f64) * rare_cost,
                        });
                    }
                }
            } else if let Some(m) = PAT_NONTHAI.find(sub_text) {
                // 3. Non-Thai tokens
                let j = i + m.as_str().chars().count();
                if j <= n && valid_pos[j] {
                    edges.push(DagEdge {
                        to: j,
                        word: m.as_str().to_string(),
                        cost: rare_cost,
                    });
                }
            }

            // Relax the edges
            for edge in edges.drain(..) {
                let j = edge.to;
                let edge_cost = match &self.bigram_model {
                    Some(bigrams) if !word[i].is_empty() => {
                        (edge.cost - bigrams.get_bonus(&word[i], &edge.word, j - i)).max(0.01)
                    }
                    _ => edge.cost,
                };

                // Ties go to the later start, i.e. the shorter last word (see TIE_EPSILON)
                let new_cost = dp[i] + edge_cost;
                if new_cost <= dp[j] + TIE_EPSILON {
                    dp[j] = new_cost;
                    from[j] = i;
                    word[j] = edge.word;
                    is_unk[j] = false;
                }
            }
        }

        // Traceback
        if dp[n].is_infinite() {
            return vec![text.to_string()];
        }

        let mut raw_tokens = Vec::new();
        let mut raw_is_unk = Vec::new();
        let mut pos = n;
        while pos > 0 {
            raw_tokens.push(word[pos].clone());
            raw_is_unk.push(is_unk[pos]);
            pos = from[pos];
            if pos == usize::MAX {
                break;
            }
        }

        raw_tokens.reverse();
        raw_is_unk.reverse();

        // Syllable-based OOV chunking
        let mut tokens = Vec::new();
        let mut cur_chunk = String::new();

        for (idx, tok_str) in raw_tokens.into_iter().enumerate() {
            if raw_is_unk[idx] && is_thai_string(&tok_str) {
                cur_chunk.push_str(&tok_str);
            } else {
                if !cur_chunk.is_empty() {
                    tokens.push(cur_chunk.clone());
                    cur_chunk.clear();
                }
                tokens.push(tok_str);
            }
        }
        if !cur_chunk.is_empty() {
            tokens.push(cur_chunk);
        }

        if !keep_whitespace {
            tokens.into_iter().filter(|t| !t.trim().is_empty()).collect()
        } else {
            tokens
        }
    }
}
