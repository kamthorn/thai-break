use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::bigram::BigramModel;
use crate::linebreaker::{thai_display_width, LineBreaker};
use crate::tokenizer::Tokenizer;
use crate::trie::ThaiTrie;

static GLOBAL_TOKENIZER: Lazy<RwLock<Option<Tokenizer>>> = Lazy::new(|| RwLock::new(None));
static GLOBAL_BREAKER: Lazy<RwLock<Option<LineBreaker>>> = Lazy::new(|| RwLock::new(None));

fn ensure_initialized() {
    let read_guard = GLOBAL_TOKENIZER.read().unwrap();
    if read_guard.is_some() {
        return;
    }
    drop(read_guard);

    let mut write_guard = GLOBAL_TOKENIZER.write().unwrap();
    if write_guard.is_some() {
        return;
    }

    // Attempt default lookup locations
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
            let bigram_path = std::path::Path::new(c)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("bigrams.tsv");
            if let Ok(b) = BigramModel::load_tsv_file(bigram_path, 0.15) {
                bigrams = Some(b);
            }
            break;
        }
    }

    let tokenizer = Tokenizer::new(trie, bigrams);
    let breaker = LineBreaker::new(tokenizer.clone());

    *write_guard = Some(tokenizer);
    *GLOBAL_BREAKER.write().unwrap() = Some(breaker);
}

/// Initialize ThaiBreak with custom dictionary and bigram file paths.
/// Pass NULL for bigram_path if not using bigrams.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_init(
    dict_path: *const c_char,
    bigram_path: *const c_char,
) -> c_int {
    if dict_path.is_null() {
        return -1;
    }

    let dict_str = match CStr::from_ptr(dict_path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let trie = match ThaiTrie::load_tsv_file(dict_str) {
        Ok(t) => t,
        Err(_) => return -1,
    };

    let bigrams = if !bigram_path.is_null() {
        if let Ok(b_str) = CStr::from_ptr(bigram_path).to_str() {
            BigramModel::load_tsv_file(b_str, 0.15).ok()
        } else {
            None
        }
    } else {
        None
    };

    let tokenizer = Tokenizer::new(trie, bigrams);
    let breaker = LineBreaker::new(tokenizer.clone());

    *GLOBAL_TOKENIZER.write().unwrap() = Some(tokenizer);
    *GLOBAL_BREAKER.write().unwrap() = Some(breaker);

    0
}

/// Tokenize UTF-8 Thai text.
/// Returns an array of null-terminated strings, and writes the token count into `*count`.
/// The returned array must be freed with `thaibreak_free_tokens`.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_tokenize(
    text: *const c_char,
    count: *mut usize,
) -> *mut *mut c_char {
    if text.is_null() || count.is_null() {
        return std::ptr::null_mut();
    }

    ensure_initialized();

    let text_str = match CStr::from_ptr(text).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let read_guard = GLOBAL_TOKENIZER.read().unwrap();
    let tokenizer = match read_guard.as_ref() {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };

    let tokens = tokenizer.tokenize(text_str, false);
    let n = tokens.len();
    *count = n;

    let mut c_tokens: Vec<*mut c_char> = Vec::with_capacity(n);
    for t in tokens {
        let c_string = CString::new(t).unwrap_or_default();
        c_tokens.push(c_string.into_raw());
    }

    let ptr = c_tokens.as_mut_ptr();
    std::mem::forget(c_tokens);
    ptr
}

/// Free token array allocated by `thaibreak_tokenize`.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_free_tokens(tokens: *mut *mut c_char, count: usize) {
    if tokens.is_null() {
        return;
    }
    let slice = std::slice::from_raw_parts_mut(tokens, count);
    for &mut ptr in slice.iter_mut() {
        if !ptr.is_null() {
            drop(CString::from_raw(ptr));
        }
    }
    drop(Vec::from_raw_parts(tokens, count, count));
}

/// Insert line break opportunities into UTF-8 text.
/// If `marker` is NULL, defaults to ZWSP (U+200B).
/// The returned string must be freed with `thaibreak_free_string`.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_lines(
    text: *const c_char,
    marker: *const c_char,
    is_html: c_int,
) -> *mut c_char {
    if text.is_null() {
        return std::ptr::null_mut();
    }

    ensure_initialized();

    let text_str = match CStr::from_ptr(text).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let marker_str = if !marker.is_null() {
        CStr::from_ptr(marker).to_str().unwrap_or("\u{200B}")
    } else {
        "\u{200B}"
    };

    let read_guard = GLOBAL_BREAKER.read().unwrap();
    let breaker = match read_guard.as_ref() {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };

    let result = breaker.insert_line_breaks(text_str, marker_str, is_html != 0);
    let c_string = CString::new(result).unwrap_or_default();
    c_string.into_raw()
}

/// Hard-wrap text to visual display width.
/// The returned string must be freed with `thaibreak_free_string`.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_wrap(
    text: *const c_char,
    width: usize,
    is_html: c_int,
) -> *mut c_char {
    if text.is_null() {
        return std::ptr::null_mut();
    }

    ensure_initialized();

    let text_str = match CStr::from_ptr(text).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let read_guard = GLOBAL_BREAKER.read().unwrap();
    let breaker = match read_guard.as_ref() {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };

    let result = breaker.wrap(text_str, width, is_html != 0);
    let c_string = CString::new(result).unwrap_or_default();
    c_string.into_raw()
}

/// Calculate terminal / column visual display width for Thai text.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_display_width(text: *const c_char) -> usize {
    if text.is_null() {
        return 0;
    }
    match CStr::from_ptr(text).to_str() {
        Ok(s) => thai_display_width(s),
        Err(_) => 0,
    }
}

/// Free string allocated by `thaibreak_lines` or `thaibreak_wrap`.
#[no_mangle]
pub unsafe extern "C" fn thaibreak_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
