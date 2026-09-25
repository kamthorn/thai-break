use once_cell::sync::Lazy;
use regex::Regex;

use crate::tokenizer::Tokenizer;
use crate::uax14;

pub const DEFAULT_BREAK_MARKER: &str = "\u{200B}";

static PAT_NO_BREAK_AFTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[(\[{\\"“‘<«#@（【《]$"#).expect("Failed to compile PAT_NO_BREAK_AFTER")
});

static PAT_NO_BREAK_BEFORE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?:[\\"”’>»ๆฯ๏]|ฯลฯ)$"#)
        .expect("Failed to compile PAT_NO_BREAK_BEFORE")
});

static PAT_HTML_TAGS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?si:(<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[^>]+>|&[a-zA-Z0-9#]+;))")
        .expect("Failed to compile PAT_HTML_TAGS")
});


static RE_THAI_COMBINING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\x{0E31}\x{0E34}-\x{0E3A}\x{0E47}-\x{0E4E}\x{200B}]")
        .expect("Failed to compile RE_THAI_COMBINING")
});

/// Whether a line break is permissible between two adjacent tokens. The tokens
/// are treated as separate dictionary words, so a Thai|Thai junction is a word
/// boundary.
pub fn can_break_between(left: &str, right: &str) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }

    let cps: Vec<char> = left.chars().chain(right.chars()).collect();
    let at = left.chars().count();
    let mut dict = vec![false; cps.len() + 1];
    dict[at] = true;

    // Whitespace safety: never insert break adjacent to spaces
    if left.ends_with(char::is_whitespace) || right.starts_with(char::is_whitespace) {
        return false;
    }

    uax14::break_opportunities(&cps, Some(&dict))[at] == uax14::ALLOWED
        && passes_typographic_rules(left, right)
}

/// Legacy token-level rules that are not yet expressed as UAX #14 rules.
fn passes_typographic_rules(left: &str, right: &str) -> bool {
    // No break after opening symbols
    if PAT_NO_BREAK_AFTER.is_match(left) {
        return false;
    }

    // No break before closing quotes and postfixes (ๆ, ฯ)
    if PAT_NO_BREAK_BEFORE.is_match(right) {
        return false;
    }

    // Latin letters and digits (UAX #14 LB23): WP01, ISO29110, 3rd
    if let (Some(l), Some(r)) = (left.chars().last(), right.chars().next()) {
        if (l.is_ascii_alphabetic() && r.is_ascii_digit())
            || (l.is_ascii_digit() && r.is_ascii_alphabetic())
        {
            return false;
        }
    }

    true
}

pub fn thai_display_width(text: &str) -> usize {
    let stripped = RE_THAI_COMBINING.replace_all(text, "");
    let mut width = 0;
    for ch in stripped.chars() {
        let cp = ch as u32;
        if (0x2E80..=0x9FFF).contains(&cp)
            || (0xAC00..=0xD7AF).contains(&cp)
            || (0xF900..=0xFAFF).contains(&cp)
        {
            width += 2;
        } else {
            width += 1;
        }
    }
    width
}

#[derive(Clone, Debug)]
pub struct LineBreaker {
    tokenizer: Tokenizer,
}

impl LineBreaker {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self { tokenizer }
    }

    pub fn insert_line_breaks(&self, text: &str, marker: &str, is_html: bool) -> String {
        if text.is_empty() {
            return String::new();
        }

        let marker = if marker.is_empty() {
            DEFAULT_BREAK_MARKER
        } else {
            marker
        };

        if is_html || (text.contains('<') && text.contains('>')) {
            self.process_html(text, marker)
        } else {
            self.process_plain(text, marker)
        }
    }

    fn process_plain(&self, text: &str, marker: &str) -> String {
        let mut res = String::with_capacity(text.len() * 2);
        let mut prev_ends_with_break = true;
        for seg in self.break_segments(text) {
            // Breaks after spaces and ZWSP are already implicit: never put a marker next to them
            if !prev_ends_with_break {
                res.push_str(marker);
            }
            prev_ends_with_break = seg.ends_with(|c: char| c.is_whitespace() || c == '\u{200B}');
            res.push_str(&seg);
        }
        res
    }

    /// Split plain text into segments that must not be broken internally. A
    /// line may break between any two segments; spaces stay at the end of the
    /// segment they follow.
    fn break_segments(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenizer.tokenize(text, true);
        let n = tokens.len();
        if n <= 1 {
            return if text.is_empty() { Vec::new() } else { vec![text.to_string()] };
        }

        // Token ends are the dictionary word boundaries inside Thai runs
        let mut cps: Vec<char> = Vec::with_capacity(text.len());
        let mut ends = Vec::with_capacity(n);
        for tok in &tokens {
            cps.extend(tok.chars());
            ends.push(cps.len());
        }
        let mut dict = vec![false; cps.len() + 1];
        for &end in &ends {
            dict[end] = true;
        }
        let actions = uax14::break_opportunities(&cps, Some(&dict));

        let mut segments = Vec::new();
        let mut cur = String::new();
        for i in 0..n {
            cur.push_str(&tokens[i]);
            if i + 1 < n
                && actions[ends[i]] == uax14::ALLOWED
                && passes_typographic_rules(&tokens[i], &tokens[i + 1])
            {
                segments.push(std::mem::take(&mut cur));
            }
        }
        segments.push(cur);
        segments
    }

    fn process_html(&self, html: &str, marker: &str) -> String {
        let mut res = String::with_capacity(html.len());
        let mut last_idx = 0;

        for m in PAT_HTML_TAGS.find_iter(html) {
            let start = m.start();
            let end = m.end();

            // Text segment before tag
            if start > last_idx {
                let plain = &html[last_idx..start];
                res.push_str(&self.process_plain(plain, marker));
            }

            // Tag/entity preserved untouched
            res.push_str(&html[start..end]);
            last_idx = end;
        }

        if last_idx < html.len() {
            let plain = &html[last_idx..];
            res.push_str(&self.process_plain(plain, marker));
        }

        res
    }

    /// Hard-wrap text into lines of at most `width` display columns, breaking
    /// only at the same opportunities that `insert_line_breaks` marks or after
    /// spaces. Trailing spaces are trimmed.
    pub fn wrap(&self, text: &str, width: usize, is_html: bool) -> String {
        if text.is_empty() || width == 0 {
            return text.to_string();
        }

        let mut wrapped_paragraphs = Vec::new();
        for para in text.lines() {
            let segments: Vec<String> = if is_html {
                // Tags stay intact in insert_line_breaks(): break at its markers and after spaces
                let broken = self.insert_line_breaks(para, DEFAULT_BREAK_MARKER, true);
                split_break_units(&broken).into_iter().map(str::to_string).collect()
            } else {
                self.break_segments(para)
            };
            wrapped_paragraphs.push(fill_lines(&segments, width).join("\n"));
        }

        wrapped_paragraphs.join("\n")
    }
}

/// Greedily fill lines of at most `width` display columns with unbreakable segments.
fn fill_lines(segments: &[String], width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur_line = String::new();
    let mut cur_width = 0;
    for seg in segments {
        // Trailing spaces may hang past the margin, so only the visible part must fit
        let visible_width = thai_display_width(seg.trim_end());
        if !cur_line.is_empty() && cur_width + visible_width > width {
            // Indentation that does not fit is dropped rather than left as an empty line
            if !cur_line.trim_end().is_empty() {
                lines.push(cur_line.trim_end().to_string());
            }
            cur_line.clear();
            cur_width = 0;
        }
        cur_line.push_str(seg);
        cur_width += thai_display_width(seg);
    }
    if !cur_line.is_empty() {
        lines.push(cur_line.trim_end().to_string());
    }
    lines
}

/// Split a paragraph into unbreakable units. A unit ends at a break marker or
/// after a run of whitespace (spaces are break opportunities too, but markers
/// are never inserted next to them).
fn split_break_units(para: &str) -> Vec<&str> {
    let mut units = Vec::new();
    let mut start = 0;
    let mut prev_space = false;
    for (i, ch) in para.char_indices() {
        if ch == '\u{200B}' {
            units.push(&para[start..i]);
            start = i + ch.len_utf8();
            prev_space = false;
            continue;
        }
        let is_space = ch.is_whitespace();
        if prev_space && !is_space && i > start {
            units.push(&para[start..i]);
            start = i;
        }
        prev_space = is_space;
    }
    units.push(&para[start..]);
    units
}
