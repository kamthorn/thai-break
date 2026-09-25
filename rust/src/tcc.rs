use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

static TCC_GENERAL_REGEX: Lazy<Regex> = Lazy::new(|| {
    let c = "[ก-ฮ]";
    let t = "[่-๋]?";
    let d = "[ุู]";
    let k = "([ก-ฮ][ก-ฮ]?[ุูิ]?์)?";

    let raw_general_rules = [
        "c[ั]([่-๋]c)?",
        "c[ั]([่-๋]c)?k",
        "เc็ck",
        "เcctาะk",
        "เccีtยะk",
        "เ(?:c[รลว]|หc)็ck",
        "เcิc์ck",
        "เcิtck",
        "เcีtยะ?k",
        "เcืtอะk",
        "เcืtอ?k",
        "เctา?ะ?k",
        "c[ึื]tck",
        "c[ะ-ู]tk",
        "c[ิุู]์",
        "cรรc์",
        "c็",
        "ct[ะาำ]?k",
        "แc็ck",
        "แcc์k",
        "แctะk",
        "แ(?:c[รลว]|หc)็ck",
        "แccc์k",
        "โctะk",
        "[เ-ไ]ctk",
        "ก็",
        "อึ",
        "หึ",
    ];

    let patterns: Vec<String> = raw_general_rules
        .iter()
        .map(|r| {
            r.replace('k', k)
                .replace('c', c)
                .replace('t', t)
                .replace('d', d)
        })
        .collect();

    Regex::new(&format!("^(?:{})", patterns.join("|"))).expect("Failed to compile TCC general regex")
});

static TCC_LOOKAHEAD_REGEX: Lazy<Regex> = Lazy::new(|| {
    let c = "[ก-ฮ]";
    let t = "[่-๋]?";
    let d = "[ุู]";
    let k = "([ก-ฮ][ก-ฮ]?[ุูิ]?์)?";

    let raw_lookahead_rules = ["เccีtยk", "เc[ิีุู]tยk"];

    let patterns: Vec<String> = raw_lookahead_rules
        .iter()
        .map(|r| {
            r.replace('k', k)
                .replace('c', c)
                .replace('t', t)
                .replace('d', d)
        })
        .collect();

    Regex::new(&format!("^(?:{})", patterns.join("|"))).expect("Failed to compile TCC lookahead regex")
});

/// Byte length of a matched cluster. A final consonant followed by a dependent
/// vowel or mark starts the next cluster instead: "รึยัง" is "รึ" + "ยัง", not
/// "รึย" + "ัง". Except ว before ะ, which is part of the vowel -ัวะ ("ผัวะ").
fn cluster_len(matched: &str, rest: &str) -> usize {
    let mut chars = matched.chars();
    if let (Some(last), Some(next)) = (chars.next_back(), rest.chars().next()) {
        if chars.next().is_some()
            && ('ก'..='ฮ').contains(&last)
            && is_dependent_thai(next)
            && !(last == 'ว' && next == 'ะ')
        {
            return matched.len() - last.len_utf8();
        }
    }
    matched.len()
}

/// Whether a character can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎.
fn is_dependent_thai(ch: char) -> bool {
    matches!(ch as u32, 0x0E30..=0x0E3A | 0x0E45 | 0x0E47..=0x0E4E)
}

#[inline]
fn is_followed_by_lookahead_char(rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    if let Some(ch) = rest.chars().next() {
        let cp = ch as u32;
        if (0x0E01..=0x0E2E).contains(&cp) || (0x0E40..=0x0E44).contains(&cp) {
            return true;
        }
        if cp < 0x0E00 || cp > 0x0E7F {
            return true;
        }
    }
    false
}

/// Compute valid TCC break positions for a slice of characters.
///
/// Returns a boolean vector of size `chars.len() + 1` where `true` indicates
/// a safe word boundary.
pub fn tcc_pos_array(chars: &[char]) -> Vec<bool> {
    let n = chars.len();
    let mut valid = vec![false; n + 1];
    valid[0] = true;
    valid[n] = true;

    if n == 0 {
        return valid;
    }

    let text: String = chars.iter().collect();
    let byte_len = text.len();

    let mut byte_to_char_idx = HashMap::with_capacity(n + 1);
    let mut offset = 0;
    for (i, &ch) in chars.iter().enumerate() {
        byte_to_char_idx.insert(offset, i);
        offset += ch.len_utf8();
    }
    byte_to_char_idx.insert(offset, n);

    let mut byte_pos = 0;
    while byte_pos < byte_len {
        let sub = &text[byte_pos..];

        // First, check lookahead rules
        if let Some(m) = TCC_LOOKAHEAD_REGEX.find(sub) {
            let m_len = m.end();
            if is_followed_by_lookahead_char(&sub[m_len..]) {
                byte_pos += m_len;
                if let Some(&idx) = byte_to_char_idx.get(&byte_pos) {
                    valid[idx] = true;
                }
                continue;
            }
        }

        // Second, check general rules
        if let Some(m) = TCC_GENERAL_REGEX.find(sub) {
            byte_pos += cluster_len(m.as_str(), &sub[m.end()..]);
            if let Some(&idx) = byte_to_char_idx.get(&byte_pos) {
                valid[idx] = true;
            }
        } else {
            // Fallback: advance 1 char
            let ch_len = sub.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            byte_pos += ch_len;
            if let Some(&idx) = byte_to_char_idx.get(&byte_pos) {
                valid[idx] = true;
            }
        }
    }

    // Non-Thai characters are always valid boundaries
    for (i, &ch) in chars.iter().enumerate() {
        let cp = ch as u32;
        if cp < 0x0E00 || cp > 0x0E7F {
            valid[i] = true;
            valid[i + 1] = true;
        }
    }

    // Never a boundary before a dependent vowel or mark (e.g. inside "เมื่อ")
    for i in 1..n {
        if is_dependent_thai(chars[i]) {
            valid[i] = false;
        }
    }

    valid
}
