use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PrefixMatch {
    pub word: String,
    pub end: usize,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct ThaiTrie {
    prefixes: HashMap<String, f64>,
    max_weight: f64,
}

impl Default for ThaiTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl ThaiTrie {
    pub fn new() -> Self {
        Self {
            prefixes: HashMap::new(),
            max_weight: 1.0,
        }
    }

    pub fn max_weight(&self) -> f64 {
        self.max_weight
    }

    pub fn len(&self) -> usize {
        self.prefixes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prefixes.is_empty()
    }

    pub fn add(&mut self, word: &str, weight: f64) {
        if word.is_empty() {
            return;
        }

        if weight > self.max_weight {
            self.max_weight = weight;
        }

        let chars: Vec<char> = word.chars().collect();
        let n = chars.len();

        let mut p = String::new();
        for ch in chars.iter().take(n - 1) {
            p.push(*ch);
            self.prefixes.entry(p.clone()).or_insert(0.0);
        }

        let entry = self.prefixes.entry(word.to_string()).or_insert(weight);
        if weight > *entry {
            *entry = weight;
        }
    }

    pub fn prefixes_from_chars(&self, chars: &[char], start: usize, max_len: usize) -> Vec<PrefixMatch> {
        let limit = (start + max_len).min(chars.len());
        let mut matches = Vec::new();
        let mut cur = String::new();

        for (idx, &ch) in chars[start..limit].iter().enumerate() {
            cur.push(ch);
            match self.prefixes.get(&cur) {
                None => break, // No dictionary words share this prefix
                Some(&w) => {
                    if w > 0.0 {
                        matches.push(PrefixMatch {
                            word: cur.clone(),
                            end: start + idx + 1,
                            weight: w,
                        });
                    }
                }
            }
        }

        matches
    }

    pub fn load_tsv<R: Read>(reader: R) -> io::Result<Self> {
        let mut trie = Self::new();
        let buf = BufReader::new(reader);

        for line in buf.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.split('\t');
            if let Some(word) = parts.next() {
                let weight = parts
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0);
                trie.add(word, weight);
            }
        }

        Ok(trie)
    }

    pub fn load_tsv_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::load_tsv(file)
    }
}
