use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use regex::Regex;
use thaibreak::{BigramModel, LineBreaker, ThaiTrie, Tokenizer, DEFAULT_BREAK_MARKER};
use tracing::info;

use crate::config::Config;

const EMBEDDED_WORDS_FST: &[u8] = include_bytes!("../data/words.fst");
const EMBEDDED_LINES_FST: &[u8] = include_bytes!("../data/lines.fst");

static PAT_HTML_DETECT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<[a-zA-Z/!?][^>]*>").expect("Failed to compile PAT_HTML_DETECT")
});

static PAT_HTML_TAGS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?si:(<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;))")
        .expect("Failed to compile PAT_HTML_TAGS")
});

#[derive(Clone)]
pub struct ThaiBreakEngine {
    words_tokenizer: Tokenizer,
    linebreaker: LineBreaker,
    base_word_count: usize,
    words_dict_count: usize,
    lines_dict_count: usize,
}

impl ThaiBreakEngine {
    pub fn new(config: &Config) -> Self {
        let env_words_path = std::env::var("THAIBREAK_DICT_PATH").ok();
        let custom_words_path = config
            .words_fst_path
            .as_deref()
            .or(env_words_path.as_deref());

        // 1. Prepare Words Trie (Base + Extra Words)
        let (words_trie, words_dict_count) = Self::load_fst(
            custom_words_path,
            &[
                "data/words.fst",
                "/app/data/words.fst",
                "../data/words.fst",
            ],
            EMBEDDED_WORDS_FST,
            "words",
        );

        // 2. Prepare Lines Trie (Base + Extra Lines)
        let (lines_trie, lines_dict_count) = Self::load_fst(
            config.lines_fst_path.as_deref(),
            &[
                "data/lines.fst",
                "/app/data/lines.fst",
                "../data/lines.fst",
            ],
            EMBEDDED_LINES_FST,
            "lines",
        );

        let bigram_model = Self::load_bigrams(config.bigram_path.as_deref());

        let words_tokenizer = Tokenizer::new(words_trie, bigram_model.clone());
        let lines_tokenizer = Tokenizer::new(lines_trie, bigram_model);
        let linebreaker = LineBreaker::new(lines_tokenizer);

        info!(
            words_dict_total = words_dict_count,
            lines_dict_total = lines_dict_count,
            "ThaiBreak dual-mode FST engine initialized successfully"
        );

        Self {
            words_tokenizer,
            linebreaker,
            base_word_count: words_dict_count,
            words_dict_count,
            lines_dict_count,
        }
    }

    fn load_fst(
        custom_path: Option<&str>,
        fallback_candidates: &[&str],
        embedded_bytes: &'static [u8],
        mode_label: &str,
    ) -> (ThaiTrie, usize) {
        // 1. Try custom path if provided
        if let Some(path_str) = custom_path {
            let path = Path::new(path_str);
            if path.exists() {
                match ThaiTrie::load_fst_file(path) {
                    Ok(trie) => {
                        let count = trie.len();
                        info!(mode = mode_label, path = path_str, count, "Loaded custom FST dictionary file");
                        return (trie, count);
                    }
                    Err(e) => {
                        tracing::warn!(mode = mode_label, path = path_str, error = %e, "Failed to load custom FST dictionary, trying candidates");
                    }
                }
            }
        }

        // 2. Try common relative filesystem paths
        for candidate in fallback_candidates {
            let path = Path::new(candidate);
            if let Ok(trie) = ThaiTrie::load_fst_file(path) {
                let count = trie.len();
                info!(mode = mode_label, path = candidate, count, "Loaded FST dictionary from filesystem candidate");
                return (trie, count);
            }
        }

        // 3. Fallback to embedded FST binary
        let trie = ThaiTrie::from_fst_bytes(Arc::from(embedded_bytes))
            .expect("Embedded FST binary must be valid TBF1 dictionary");
        let count = trie.len();
        info!(mode = mode_label, count, "Loaded embedded FST dictionary");
        (trie, count)
    }

    fn load_bigrams(custom_path: Option<&str>) -> Option<BigramModel> {
        if let Some(path_str) = custom_path {
            let path = Path::new(path_str);
            if let Ok(b) = BigramModel::load_tsv_file(path, 2.0) {
                info!(path = path_str, "Loaded bigram model");
                return Some(b);
            }
        }

        // NOTE: only committed dictionaries are auto-discovered here.
        // LST20-derived bigrams (local/bigrams-lst20.tsv) are measurement-only
        // and must be passed explicitly via --bigram-path / THAIBREAK_BIGRAM_PATH.
        let candidates = [
            "data/bigrams.tsv",
            "/data/bigrams.tsv",
            "/app/data/bigrams.tsv",
            "../data/bigrams.tsv",
            "../PHPThaiNLP/data/bigrams.tsv",
        ];
        for candidate in candidates {
            let path = Path::new(candidate);
            if let Ok(b) = BigramModel::load_tsv_file(path, 2.0) {
                info!(path = candidate, "Loaded bigram model from filesystem");
                return Some(b);
            }
        }

        None
    }

    pub fn word_count(&self) -> usize {
        self.words_dict_count
    }

    pub fn base_word_count(&self) -> usize {
        self.base_word_count
    }

    pub fn words_dict_count(&self) -> usize {
        self.words_dict_count
    }

    pub fn lines_dict_count(&self) -> usize {
        self.lines_dict_count
    }

    /// Normalize Thai text using the engine's dictionary for repetition verification.
    pub fn normalize(&self, text: &str) -> String {
        thaibreak::normalize_text_with_dict(text, self.words_tokenizer.trie())
    }

    /// Tokenize text into words using the semantic Words dictionary.
    /// If `is_html` is true, HTML tags/comments are preserved or ignored, tokenizing only the textual content.
    pub fn words(&self, text: &str, keep_whitespace: bool, is_html: bool, normalize: bool) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let normalized_text: String;
        let text = if normalize {
            normalized_text = self.normalize(text);
            &normalized_text
        } else {
            text
        };

        if !is_html && !PAT_HTML_DETECT.is_match(text) {
            return self.words_tokenizer.tokenize(text, keep_whitespace);
        }

        // HTML aware tokenization: extract and tokenize textual segments between tags
        let mut tokens = Vec::new();
        let mut last_idx = 0;

        for m in PAT_HTML_TAGS.find_iter(text) {
            let start = m.start();
            let end = m.end();

            if start > last_idx {
                let plain = &text[last_idx..start];
                let sub_tokens = self.words_tokenizer.tokenize(plain, keep_whitespace);
                tokens.extend(sub_tokens);
            }
            last_idx = end;
        }

        if last_idx < text.len() {
            let plain = &text[last_idx..];
            let sub_tokens = self.words_tokenizer.tokenize(plain, keep_whitespace);
            tokens.extend(sub_tokens);
        }

        tokens
    }

    /// Insert typographic line break markers using the layout Lines dictionary.
    pub fn lines(&self, text: &str, marker: Option<&str>, is_html: bool, normalize: bool) -> String {
        let normalized_text: String;
        let text = if normalize {
            normalized_text = self.normalize(text);
            &normalized_text
        } else {
            text
        };
        let m = marker.unwrap_or(DEFAULT_BREAK_MARKER);
        self.linebreaker.insert_line_breaks(text, m, is_html)
    }

    /// Hard-wrap text into lines based on visual column display width using the layout Lines dictionary.
    pub fn wrap(&self, text: &str, width: usize, is_html: bool, normalize: bool) -> String {
        let normalized_text: String;
        let text = if normalize {
            normalized_text = self.normalize(text);
            &normalized_text
        } else {
            text
        };
        self.linebreaker.wrap(text, width, is_html)
    }

    /// Join words with a custom delimiter
    pub fn join(&self, text: &str, delimiter: &str, is_html: bool, normalize: bool) -> String {
        if is_html || PAT_HTML_DETECT.is_match(text) {
            self.lines(text, Some(delimiter), true, normalize)
        } else {
            let words = self.words(text, false, false, normalize);
            words.join(delimiter)
        }
    }
}

pub type SharedEngine = Arc<ThaiBreakEngine>;
