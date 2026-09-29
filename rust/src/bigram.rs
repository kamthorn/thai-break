use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

/// Bigram scoring formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BigramFormula {
    /// Current: alpha * ln(1+count) / word_len
    Count,
    /// PMI-like: alpha * ln(1+count) / ln(1+count(w1))
    /// Normalizes by unigram count of previous word, so bigrams involving
    /// rare words get higher bonuses.
    PMI,
    /// Probability: alpha * ln(1 + count(w1,w2)/count(w1))
    /// Uses conditional probability P(w2|w1) directly.
    Probability,
}

#[derive(Debug, Clone)]
pub struct BigramModel {
    bigrams: HashMap<String, f64>,
    unigram: HashMap<String, f64>,
    alpha: f64,
    formula: BigramFormula,
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
            unigram: HashMap::new(),
            alpha,
            formula: BigramFormula::Count,
        }
    }

    pub fn with_formula(alpha: f64, formula: BigramFormula) -> Self {
        Self {
            bigrams: HashMap::new(),
            unigram: HashMap::new(),
            alpha,
            formula,
        }
    }

    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha;
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    pub fn formula(&self) -> BigramFormula {
        self.formula
    }

    pub fn set_formula(&mut self, formula: BigramFormula) {
        self.formula = formula;
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
        *self.unigram.entry(prev_word.to_string()).or_insert(0.0) += count;
    }

    pub fn get_bonus(&self, prev_word: &str, next_word: &str, word_len: usize) -> f64 {
        if self.alpha <= 0.0 || prev_word.is_empty() || next_word.is_empty() {
            return 0.0;
        }

        let key = format!("{}\t{}", prev_word, next_word);
        let count = match self.bigrams.get(&key) {
            Some(&c) => c,
            None => return 0.0,
        };

        let denom = (word_len as f64).max(1.0);
        match self.formula {
            BigramFormula::Count => (self.alpha * (1.0 + count).ln()) / denom,
            BigramFormula::PMI => {
                let uw1 = self.unigram.get(prev_word).copied().unwrap_or(1.0);
                (self.alpha * (1.0 + count).ln()) / (1.0 + uw1).ln()
            }
            BigramFormula::Probability => {
                let uw1 = self.unigram.get(prev_word).copied().unwrap_or(1.0);
                let prob = count / uw1;
                (self.alpha * (1.0 + prob).ln()) / denom
            }
        }
    }

    pub fn load_tsv<R: Read>(reader: R, alpha: f64) -> io::Result<Self> {
        Self::load_tsv_with_formula(reader, alpha, BigramFormula::Count)
    }

    pub fn load_tsv_with_formula<R: Read>(
        reader: R,
        alpha: f64,
        formula: BigramFormula,
    ) -> io::Result<Self> {
        let mut model = Self::with_formula(alpha, formula);
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

    pub fn load_tsv_file_with_formula<P: AsRef<Path>>(
        path: P,
        alpha: f64,
        formula: BigramFormula,
    ) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::load_tsv_with_formula(file, alpha, formula)
    }
}
