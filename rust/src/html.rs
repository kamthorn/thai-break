//! Linear-time scanner for the markup in an HTML fragment.
//!
//! It finds the same spans as the regex
//! `(?si:<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;)`
//! but never rescans the text: with that regex, every `<script>`, `<!--` or `<a` whose
//! terminator is missing scans to the end of the text again, so crafted input takes
//! quadratic time.

/// The next position of a fixed terminator, remembered between lookups. Lookups come
/// from increasing positions, so a cached hit at or after the new start is still the
/// first one, and a cached miss stays a miss: every byte is searched at most once.
struct Terminator {
    needle: &'static [u8],
    cached: Option<Option<usize>>,
}

impl Terminator {
    const fn new(needle: &'static [u8]) -> Self {
        Self { needle, cached: None }
    }

    fn find(&mut self, hay: &[u8], from: usize) -> Option<usize> {
        match self.cached {
            Some(None) => return None,
            Some(Some(p)) if p >= from => return Some(p),
            _ => {}
        }
        let found = (from..=hay.len().saturating_sub(self.needle.len()))
            .find(|&i| hay[i..i + self.needle.len()].eq_ignore_ascii_case(self.needle));
        self.cached = Some(found);
        found
    }
}

fn starts_with_ci(hay: &[u8], at: usize, prefix: &[u8]) -> bool {
    hay.len() >= at + prefix.len() && hay[at..at + prefix.len()].eq_ignore_ascii_case(prefix)
}

/// `\b` after an ASCII letter: the next character is not a word character.
fn word_boundary_at(text: &str, at: usize) -> bool {
    text[at..]
        .chars()
        .next()
        .map_or(true, |c| !(c.is_alphanumeric() || c == '_'))
}

/// Byte ranges of the comments, script and style blocks, tags and entities in `text`.
pub fn markup_spans(text: &str) -> Vec<(usize, usize)> {
    let hay = text.as_bytes();
    let mut comment_end = Terminator::new(b"-->");
    let mut script_end = Terminator::new(b"</script>");
    let mut style_end = Terminator::new(b"</style>");
    let mut tag_end = Terminator::new(b">");

    let mut spans = Vec::new();
    let mut i = 0;
    while i < hay.len() {
        let end = match hay[i] {
            b'<' => {
                let mut end = None;
                if hay[i..].starts_with(b"<!--") {
                    end = comment_end.find(hay, i + 4).map(|j| j + 3);
                }
                for (open, close) in [(&b"<script"[..], &mut script_end), (&b"<style"[..], &mut style_end)] {
                    if end.is_none() && starts_with_ci(hay, i, open) && word_boundary_at(text, i + open.len()) {
                        end = tag_end
                            .find(hay, i + open.len())
                            .and_then(|k| close.find(hay, k + 1))
                            .map(|j| j + close.needle.len());
                    }
                }
                if end.is_none() && hay.get(i + 1).is_some_and(|&c| c.is_ascii_alphabetic() || b"/!?".contains(&c)) {
                    end = tag_end.find(hay, i + 1).map(|k| k + 1);
                }
                end
            }
            b'&' => {
                let run = hay[i + 1..].iter().take_while(|&&c| c.is_ascii_alphanumeric() || c == b'#').count();
                (run > 0 && hay.get(i + 1 + run) == Some(&b';')).then_some(i + run + 2)
            }
            _ => None,
        };
        match end {
            Some(end) => {
                spans.push((i, end));
                i = end;
            }
            None => i += 1,
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::markup_spans;
    use regex::Regex;

    static PAT_HTML_TAGS: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?si:(<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;))").unwrap()
    });

    fn regex_spans(text: &str) -> Vec<(usize, usize)> {
        PAT_HTML_TAGS.find_iter(text).map(|m| (m.start(), m.end())).collect()
    }

    #[test]
    fn matches_the_regex() {
        let pieces = [
            "<", ">", "!", "-", "/", "?", "<!--", "-->", "<script", "</script>", "<SCRIPT", "</Script>",
            "<style", "</STYLE>", "<scripts", "<script_", " ", "\n", "a", "Z", "&", "#", ";", "&amp;", "ก", "ไทย",
            "<b>", "</b>", "<ab",
        ];
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..20_000 {
            let mut text = String::new();
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let len = (seed % 12) as usize;
            for _ in 0..len {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                text.push_str(pieces[(seed % pieces.len() as u64) as usize]);
            }
            assert_eq!(markup_spans(&text), regex_spans(&text), "text: {text:?}");
        }
    }

    #[test]
    fn unterminated_markup_is_linear() {
        for piece in ["<script>", "<style x>", "<!-->", "<a", "<!--<script>"] {
            let text = piece.repeat(200_000);
            let start = std::time::Instant::now();
            markup_spans(&text);
            assert!(start.elapsed().as_secs() < 2, "{piece:?} took {:?}", start.elapsed());
        }
    }
}
