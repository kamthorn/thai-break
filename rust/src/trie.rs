use fst::raw::Output;
use fst::{Map, MapBuilder};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixMatch {
    pub word: String,
    pub end: usize,
    pub weight: f64,
}

/// Zero-copy byte buffer wrapper supporting sub-slicing for `fst::Map`.
#[derive(Clone)]
pub struct FstData(Arc<[u8]>, usize, usize);

impl AsRef<[u8]> for FstData {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0[self.1..self.2]
    }
}

impl std::fmt::Debug for FstData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FstData(len={})", self.2 - self.1)
    }
}

/// Flat Prefix Hash Map implementation (standard fallback).
#[derive(Debug, Clone)]
pub struct FlatTrie {
    prefixes: HashMap<String, f64>,
    max_weight: f64,
    total_weight: f64,
}

impl Default for FlatTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl FlatTrie {
    pub fn new() -> Self {
        Self {
            prefixes: HashMap::new(),
            max_weight: 1.0,
            total_weight: 0.0,
        }
    }

    pub fn max_weight(&self) -> f64 {
        self.max_weight
    }

    pub fn total_weight(&self) -> f64 {
        self.total_weight
    }

    pub fn len(&self) -> usize {
        self.prefixes.values().filter(|&&w| w > 0.0).count()
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

        let entry = self.prefixes.entry(word.to_string()).or_insert(0.0);
        if weight > *entry {
            self.total_weight += weight - *entry;
            *entry = weight;
        }
    }

    /// Set the weight of a word that is already in the dictionary. Returns false for other words.
    pub fn set_weight(&mut self, word: &str, weight: f64) -> bool {
        match self.prefixes.get_mut(word) {
            Some(entry) if *entry > 0.0 => {
                self.total_weight += weight - *entry;
                *entry = weight;
                self.max_weight = self.max_weight.max(weight);
                true
            }
            _ => false,
        }
    }

    pub fn prefixes_from_chars(&self, chars: &[char], start: usize, max_len: usize) -> Vec<PrefixMatch> {
        let limit = (start + max_len).min(chars.len());
        let mut matches = Vec::new();
        let mut cur = String::new();

        for (idx, &ch) in chars[start..limit].iter().enumerate() {
            cur.push(ch);
            match self.prefixes.get(&cur) {
                None => break,
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

    #[inline]
    pub fn contains(&self, word: &str) -> bool {
        self.prefixes.get(word).copied().unwrap_or(0.0) > 0.0
    }
}

/// High-performance Finite State Transducer (FST) Trie with optional dynamic overlay.
#[derive(Debug, Clone)]
pub struct FstTrie {
    map: Map<FstData>,
    weights: Vec<f64>,
    total_weight: f64,
    max_weight: f64,
    overlay: HashMap<String, f64>,
    /// Weights set with `set_weight` for words of the FST, by word id
    weight_overrides: HashMap<u64, f64>,
}

impl FstTrie {
    pub fn from_bytes(bytes: Arc<[u8]>) -> io::Result<Self> {
        let len = bytes.len();
        if len >= 28 && &bytes[0..4] == b"TBF1" {
            let flags = bytes[4];
            let total_weight = f64::from_le_bytes(bytes[8..16].try_into().unwrap());
            let max_weight = f64::from_le_bytes(bytes[16..24].try_into().unwrap());
            let count = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) as usize;
            let mut offset = 28;

            let mut weights = Vec::new();
            if flags & 1 != 0 {
                let weights_len = count * 8;
                if len < offset + weights_len {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid FST header weights size"));
                }
                weights.reserve(count);
                for i in 0..count {
                    let w_bytes: [u8; 8] = bytes[offset + i * 8..offset + (i + 1) * 8].try_into().unwrap();
                    weights.push(f64::from_le_bytes(w_bytes));
                }
                offset += weights_len;
            }

            let data = FstData(bytes.clone(), offset, len);
            let map = Map::new(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            Ok(Self {
                map,
                weights,
                total_weight,
                max_weight,
                overlay: HashMap::new(),
                weight_overrides: HashMap::new(),
            })
        } else {
            // Raw FST binary
            let data = FstData(bytes.clone(), 0, len);
            let map = Map::new(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let count = map.len();
            Ok(Self {
                map,
                weights: Vec::new(),
                total_weight: count as f64,
                max_weight: 1.0,
                overlay: HashMap::new(),
                weight_overrides: HashMap::new(),
            })
        }
    }

    pub fn max_weight(&self) -> f64 {
        self.max_weight
    }

    pub fn total_weight(&self) -> f64 {
        self.total_weight
    }

    pub fn len(&self) -> usize {
        self.map.len() + self.overlay.values().filter(|&&w| w > 0.0).count()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty() && self.overlay.is_empty()
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
            self.overlay.entry(p.clone()).or_insert(0.0);
        }

        let entry = self.overlay.entry(word.to_string()).or_insert(0.0);
        if weight > *entry {
            self.total_weight += weight - *entry;
            *entry = weight;
        }
    }

    /// Set the weight of a word that is already in the dictionary. Returns false for other words.
    pub fn set_weight(&mut self, word: &str, weight: f64) -> bool {
        let old = if let Some(entry) = self.overlay.get_mut(word).filter(|w| **w > 0.0) {
            std::mem::replace(entry, weight)
        } else if let Some(id) = self.map.get(word) {
            let fst_weight = self.weights.get(id as usize).copied().unwrap_or(1.0);
            self.weight_overrides.insert(id, weight).unwrap_or(fst_weight)
        } else {
            return false;
        };
        self.total_weight += weight - old;
        self.max_weight = self.max_weight.max(weight);
        true
    }

    pub fn prefixes_from_chars(&self, chars: &[char], start: usize, max_len: usize) -> Vec<PrefixMatch> {
        let limit = (start + max_len).min(chars.len());
        let mut matches = Vec::new();
        let raw = self.map.as_fst();
        let mut node = raw.root();
        let mut out = Output::zero();
        let mut buf = [0u8; 4];

        for idx in start..limit {
            let ch = chars[idx];
            let bytes = ch.encode_utf8(&mut buf).as_bytes();
            let mut failed = false;
            for &b in bytes {
                if let Some(trans_idx) = node.find_input(b) {
                    let trans = node.transition(trans_idx);
                    node = raw.node(trans.addr);
                    out = out.cat(trans.out);
                } else {
                    failed = true;
                    break;
                }
            }
            if failed {
                break;
            }
            if node.is_final() {
                let total_out = out.cat(node.final_output());
                let word_id = total_out.value() as usize;
                let weight = match self.weight_overrides.get(&(word_id as u64)) {
                    Some(&w) => w,
                    None => self.weights.get(word_id).copied().unwrap_or(1.0),
                };
                matches.push(PrefixMatch {
                    word: chars[start..=idx].iter().collect(),
                    end: idx + 1,
                    weight,
                });
            }
        }

        if !self.overlay.is_empty() {
            let mut cur = String::new();
            for (idx, &ch) in chars[start..limit].iter().enumerate() {
                cur.push(ch);
                match self.overlay.get(&cur) {
                    None => break,
                    Some(&w) => {
                        if w > 0.0 {
                            if let Some(existing) = matches.iter_mut().find(|m| m.word == cur) {
                                existing.weight = w;
                            } else {
                                matches.push(PrefixMatch {
                                    word: cur.clone(),
                                    end: start + idx + 1,
                                    weight: w,
                                });
                            }
                        }
                    }
                }
            }
        }

        matches
    }

    #[inline]
    pub fn contains(&self, word: &str) -> bool {
        if let Some(&w) = self.overlay.get(word) {
            return w > 0.0;
        }
        self.map.contains_key(word)
    }
}

/// Unified ThaiTrie supporting both Flat HashMap and FST engines.
#[derive(Debug, Clone)]
pub enum ThaiTrie {
    Flat(FlatTrie),
    Fst(FstTrie),
}

impl Default for ThaiTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl ThaiTrie {
    pub fn new() -> Self {
        Self::Flat(FlatTrie::new())
    }

    pub fn from_fst_bytes(bytes: Arc<[u8]>) -> io::Result<Self> {
        Ok(Self::Fst(FstTrie::from_bytes(bytes)?))
    }

    /// The base dictionary compiled into this build, with its word weights (feature `embedded-dict`).
    #[cfg(feature = "embedded-dict")]
    pub fn embedded() -> io::Result<Self> {
        static WORDS_FST: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/words.fst"));
        static WORD_WEIGHTS: &str = include_str!(concat!(env!("OUT_DIR"), "/word-weights.tsv"));
        let mut trie = Self::from_fst_bytes(WORDS_FST.into())?;
        trie.apply_weights(WORD_WEIGHTS.as_bytes())?;
        Ok(trie)
    }

    /// Set the weight of a word that is already in the dictionary. Returns false for other words.
    pub fn set_weight(&mut self, word: &str, weight: f64) -> bool {
        match self {
            Self::Flat(t) => t.set_weight(word, weight),
            Self::Fst(t) => t.set_weight(word, weight),
        }
    }

    /// Apply a word-weights file (`word<TAB>weight` lines, `#` comments) such as
    /// `data/word-weights.tsv`. Words that are not in the dictionary are skipped.
    /// Returns the number of words whose weight was set.
    pub fn apply_weights<R: Read>(&mut self, reader: R) -> io::Result<usize> {
        let mut applied = 0;
        for line in BufReader::new(reader).lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (word, weight) = trimmed
                .split_once('\t')
                .and_then(|(w, x)| x.trim().parse::<f64>().ok().filter(|x| *x > 0.0).map(|x| (w, x)))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("bad word-weights line: {trimmed}")))?;
            if self.set_weight(word, weight) {
                applied += 1;
            }
        }
        Ok(applied)
    }

    pub fn apply_weights_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<usize> {
        self.apply_weights(File::open(path)?)
    }

    pub fn load_fst_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let bytes: Arc<[u8]> = std::fs::read(path)?.into();
        Self::from_fst_bytes(bytes)
    }

    pub fn max_weight(&self) -> f64 {
        match self {
            Self::Flat(t) => t.max_weight(),
            Self::Fst(t) => t.max_weight(),
        }
    }

    pub fn total_weight(&self) -> f64 {
        match self {
            Self::Flat(t) => t.total_weight(),
            Self::Fst(t) => t.total_weight(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Flat(t) => t.len(),
            Self::Fst(t) => t.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Flat(t) => t.is_empty(),
            Self::Fst(t) => t.is_empty(),
        }
    }

    #[inline]
    pub fn contains(&self, word: &str) -> bool {
        match self {
            Self::Flat(t) => t.contains(word),
            Self::Fst(t) => t.contains(word),
        }
    }

    pub fn add(&mut self, word: &str, weight: f64) {
        match self {
            Self::Flat(t) => t.add(word, weight),
            Self::Fst(t) => t.add(word, weight),
        }
    }

    pub fn prefixes_from_chars(&self, chars: &[char], start: usize, max_len: usize) -> Vec<PrefixMatch> {
        match self {
            Self::Flat(t) => t.prefixes_from_chars(chars, start, max_len),
            Self::Fst(t) => t.prefixes_from_chars(chars, start, max_len),
        }
    }

    pub fn load_tsv<R: Read>(reader: R) -> io::Result<Self> {
        let mut trie = FlatTrie::new();
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

        Ok(Self::Flat(trie))
    }

    pub fn load_tsv_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::load_tsv(file)
    }

    /// Load dictionary file, automatically detecting if it is FST or TSV/text.
    pub fn load_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref();
        if p.extension().is_some_and(|ext| ext == "fst") {
            return Self::load_fst_file(p);
        }

        // Check if header starts with TBF1
        let mut file = File::open(p)?;
        let mut magic = [0u8; 4];
        if let Ok(n) = file.read(&mut magic) {
            if n == 4 && &magic == b"TBF1" {
                return Self::load_fst_file(p);
            }
        }

        Self::load_tsv_file(p)
    }

    /// Compile a TSV wordlist into an optimized FST binary.
    pub fn compile_tsv_to_fst<R: Read, W: Write>(reader: R, mut writer: W) -> io::Result<()> {
        let buf = BufReader::new(reader);
        let mut entries: Vec<(String, f64)> = Vec::new();

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
                entries.push((word.to_string(), weight));
            }
        }

        // Sort lexicographically by UTF-8 bytes (required by FST)
        entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        entries.dedup_by(|a, b| a.0 == b.0);

        let has_custom_weights = entries.iter().any(|(_, w)| (*w - 1.0).abs() > 1e-9);
        let mut total_weight = 0.0;
        let mut max_weight = 1.0f64;
        for (_, w) in &entries {
            total_weight += *w;
            if *w > max_weight {
                max_weight = *w;
            }
        }

        // Header: "TBF1" (4) + flags (1) + reserved (3) + total_weight (8) + max_weight (8) + count (4)
        writer.write_all(b"TBF1")?;
        let flags = if has_custom_weights { 1u8 } else { 0u8 };
        writer.write_all(&[flags, 0, 0, 0])?;
        writer.write_all(&total_weight.to_le_bytes())?;
        writer.write_all(&max_weight.to_le_bytes())?;
        writer.write_all(&(entries.len() as u32).to_le_bytes())?;

        if has_custom_weights {
            for (_, w) in &entries {
                writer.write_all(&w.to_le_bytes())?;
            }
        }

        let mut build = MapBuilder::new(writer).map_err(io::Error::other)?;
        for (i, (word, _)) in entries.iter().enumerate() {
            build.insert(word.as_bytes(), i as u64).map_err(io::Error::other)?;
        }
        build.finish().map_err(io::Error::other)?;

        Ok(())
    }

    /// Compile a TSV file directly to an FST file.
    pub fn compile_tsv_file_to_fst<P1: AsRef<Path>, P2: AsRef<Path>>(tsv_path: P1, fst_path: P2) -> io::Result<()> {
        let file_in = File::open(tsv_path)?;
        let file_out = File::create(fst_path)?;
        Self::compile_tsv_to_fst(file_in, std::io::BufWriter::new(file_out))
    }
}
