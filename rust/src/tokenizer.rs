use once_cell::sync::Lazy;
use regex::Regex;

use crate::bigram::BigramModel;
use crate::tcc::tcc_pos_array;
use crate::trie::ThaiTrie;

const UNKNOWN_WORD_COST: f64 = 10.0;
const ABBR_WEIGHT: f64 = 60000.0;
const MAX_EDGES: usize = 50000;

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

#[derive(Clone, Debug)]
struct DagEdge {
    from: usize,
    word: String,
    weight: f64,
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

        // Pass 1: Collect edges_to[j]
        let mut edges_to: Vec<Vec<DagEdge>> = vec![Vec::new(); n + 1];
        let mut max_weight = 1.0;
        let mut edge_count = 0;

        'collect_loop: for i in 0..n {
            if !valid_pos[i] {
                continue;
            }

            let byte_pos = char_byte_offsets[i];
            let sub_text = &text[byte_pos..];

            if is_thai_rune(chars[i]) {
                // 1. Thai dictionary words starting at i
                let matches = self.trie.prefixes_from_chars(&chars, i, 25);
                for m in matches {
                    let j = m.end;
                    if j > n || !valid_pos[j] {
                        continue;
                    }
                    if m.weight > max_weight {
                        max_weight = m.weight;
                    }
                    edges_to[j].push(DagEdge {
                        from: i,
                        word: m.word,
                        weight: m.weight,
                    });
                    edge_count += 1;
                    if edge_count >= MAX_EDGES {
                        break 'collect_loop;
                    }
                }

                // 2. Thai abbreviation patterns
                if let Some(m) = PAT_ABBR.find(sub_text) {
                    let m_str = m.as_str();
                    let abbr_len = m_str.chars().count();
                    let j = i + abbr_len;
                    if j <= n && valid_pos[j] {
                        if ABBR_WEIGHT > max_weight {
                            max_weight = ABBR_WEIGHT;
                        }
                        edges_to[j].push(DagEdge {
                            from: i,
                            word: m_str.to_string(),
                            weight: ABBR_WEIGHT,
                        });
                    }
                }
            } else {
                // 3. Non-Thai tokens
                if let Some(m) = PAT_NONTHAI.find(sub_text) {
                    let m_str = m.as_str();
                    let word_len = m_str.chars().count();
                    let j = i + word_len;
                    if j <= n {
                        edges_to[j].push(DagEdge {
                            from: i,
                            word: m_str.to_string(),
                            weight: 1.0,
                        });
                    }
                }
            }
        }

        // Pass 2: Viterbi forward DP
        let mut dp = vec![f64::INFINITY; n + 1];
        let mut from = vec![usize::MAX; n + 1];
        let mut word = vec![String::new(); n + 1];
        let mut is_unk = vec![false; n + 1];

        dp[0] = 0.0;

        for j in 1..=n {
            if !valid_pos[j] {
                continue;
            }

            // Try all dictionary/pattern edges ending at j
            for edge in &edges_to[j] {
                let i = edge.from;
                if dp[i].is_infinite() {
                    continue;
                }
                let w_len = (j - i).max(1);
                let normalized = edge.weight / max_weight;
                let base_cost = if normalized > 0.0 {
                    -normalized.ln() / (w_len as f64)
                } else {
                    UNKNOWN_WORD_COST / (w_len as f64)
                };

                let edge_cost = if let Some(ref bigrams) = self.bigram_model {
                    if !word[i].is_empty() {
                        let bonus = bigrams.get_bonus(&word[i], &edge.word, w_len);
                        (base_cost - bonus).max(0.01)
                    } else {
                        base_cost
                    }
                } else {
                    base_cost
                };

                let new_cost = dp[i] + edge_cost;
                if new_cost < dp[j] {
                    dp[j] = new_cost;
                    from[j] = i;
                    word[j] = edge.word.clone();
                    is_unk[j] = false;
                }
            }

            // Unknown-word fallback: connect to nearest reachable predecessor
            if dp[j].is_infinite() {
                for i in (0..j).rev() {
                    if !dp[i].is_infinite() && valid_pos[i] {
                        let unknown_word: String = chars[i..j].iter().collect();
                        let new_cost = dp[i] + UNKNOWN_WORD_COST;
                        dp[j] = new_cost;
                        from[j] = i;
                        word[j] = unknown_word;
                        is_unk[j] = true;
                        break;
                    }
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
