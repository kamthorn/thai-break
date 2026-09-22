use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BigramModel {
    bigrams: HashMap<String, f64>,
    alpha: f64,
}

impl Default for BigramModel {
    fn default() -> Self {
        Self::new(0.15)
    }
}

impl BigramModel {
    pub fn new(alpha: f64) -> Self {
        Self {
            bigrams: HashMap::new(),
            alpha,
        }
    }

    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha;
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    pub fn len(&self) -> usize {
        self.bigrams.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bigrams.is_empty()
    }

    pub fn add(&mut self, prev_word: &str, next_word: &str, count: f64) {
        let key = format!("{}\t{}", prev_word, next_word);
        self.bigrams.insert(key, count);
    }

    pub fn get_bonus(&self, prev_word: &str, next_word: &str, word_len: usize) -> f64 {
        if self.alpha <= 0.0 || prev_word.is_empty() || next_word.is_empty() {
            return 0.0;
        }

        let key = format!("{}\t{}", prev_word, next_word);
        match self.bigrams.get(&key) {
            None => 0.0,
            Some(&count) => {
                let denom = (word_len as f64).max(1.0);
                (self.alpha * (1.0 + count).ln()) / denom
            }
        }
    }

    pub fn load_tsv<R: Read>(reader: R, alpha: f64) -> io::Result<Self> {
        let mut model = Self::new(alpha);
        let buf = BufReader::new(reader);

        for line in buf.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.split('\t');
            if let (Some(w1), Some(w2), Some(count_str)) = (parts.next(), parts.next(), parts.next()) {
                if let Ok(count) = count_str.parse::<f64>() {
                    if count > 0.0 {
                        model.add(w1, w2, count);
                    }
                }
            }
        }

        Ok(model)
    }

    pub fn load_tsv_file<P: AsRef<Path>>(path: P, alpha: f64) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::load_tsv(file, alpha)
    }
}
