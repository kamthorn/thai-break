pub mod bigram;
mod linebreak_data;
pub mod linebreaker;
pub mod tcc;
pub mod tokenizer;
pub mod trie;
mod uax14;

#[cfg(feature = "c-ffi")]
pub mod c_ffi;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "typst-plugin")]
pub mod typst_plugin;

use std::sync::RwLock;
use once_cell::sync::Lazy;

pub use bigram::BigramModel;
pub use linebreaker::{can_break_between, thai_display_width, LineBreaker, DEFAULT_BREAK_MARKER};
pub use tcc::tcc_pos_array;
pub use tokenizer::Tokenizer;
pub use trie::{PrefixMatch, ThaiTrie};
pub use linebreak_data::UNICODE_VERSION;

static DEFAULT_TOKENIZER: Lazy<RwLock<Option<Tokenizer>>> = Lazy::new(|| RwLock::new(None));
static DEFAULT_BREAKER: Lazy<RwLock<Option<LineBreaker>>> = Lazy::new(|| RwLock::new(None));

fn ensure_default_loaded() {
    let r = DEFAULT_TOKENIZER.read().unwrap();
    if r.is_some() {
        return;
    }
    drop(r);

    let mut w = DEFAULT_TOKENIZER.write().unwrap();
    if w.is_some() {
        return;
    }

    let candidates = [
        "data/words.txt",
        "../data/words.txt",
        "../../data/words.txt",
        "data/wordlist.txt",
        "../data/wordlist.txt",
    ];
    let mut trie = ThaiTrie::new();
    let mut bigrams = None;

    for &c in &candidates {
        if let Ok(t) = ThaiTrie::load_tsv_file(c) {
            trie = t;
            let b_path = std::path::Path::new(c)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("bigrams.tsv");
            if let Ok(b) = BigramModel::load_tsv_file(b_path, 0.15) {
                bigrams = Some(b);
            }
            break;
        }
    }

    #[cfg(feature = "typst-plugin")]
    if trie.is_empty() {
        static EMBEDDED_WORDS: &str = include_str!("../../data/words.txt");
        if let Ok(t) = ThaiTrie::load_tsv(EMBEDDED_WORDS.as_bytes()) {
            trie = t;
        }
    }

    let tokenizer = Tokenizer::new(trie, bigrams);
    let breaker = LineBreaker::new(tokenizer.clone());

    *w = Some(tokenizer);
    *DEFAULT_BREAKER.write().unwrap() = Some(breaker);
}

/// Configure the default shared Tokenizer and LineBreaker.
pub fn set_default(tokenizer: Tokenizer) {
    let breaker = LineBreaker::new(tokenizer.clone());
    *DEFAULT_TOKENIZER.write().unwrap() = Some(tokenizer);
    *DEFAULT_BREAKER.write().unwrap() = Some(breaker);
}

/// Tokenize text into words using the default shared dictionary.
pub fn words(text: &str) -> Vec<String> {
    ensure_default_loaded();
    let r = DEFAULT_TOKENIZER.read().unwrap();
    r.as_ref().unwrap().tokenize(text, false)
}

/// Insert line break opportunities (default: ZWSP U+200B) into text.
pub fn lines(text: &str, marker: &str, is_html: bool) -> String {
    ensure_default_loaded();
    let r = DEFAULT_BREAKER.read().unwrap();
    r.as_ref().unwrap().insert_line_breaks(text, marker, is_html)
}

/// Hard-wrap text into lines with maximum visual display width.
pub fn wrap(text: &str, width: usize, is_html: bool) -> String {
    ensure_default_loaded();
    let r = DEFAULT_BREAKER.read().unwrap();
    r.as_ref().unwrap().wrap(text, width, is_html)
}

/// Calculate visual terminal / column display width of Thai text.
pub fn display_width(text: &str) -> usize {
    thai_display_width(text)
}
