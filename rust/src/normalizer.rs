use crate::trie::ThaiTrie;

/// Configuration options for Thai text normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeOptions {
    /// Convert Sara E + Sara E to Sara Ae ('เเ' -> 'แ')
    pub normalize_sara_ae: bool,
    /// Convert Nikhahit + Sara Aa / tones to Sara Am ('ํ' + 'า' -> 'ำ', 'นํ้า' -> 'น้ำ')
    pub normalize_sara_am: bool,
    /// Reorder misplaced vowels and tone marks (e.g. tone before upper vowel -> upper vowel before tone)
    pub reorder_vowels_tone: bool,
    /// Collapse repeated vowels (e.g. 'าา' -> 'า', 'ีี' -> 'ี') and tone marks ('่่' -> '่')
    pub remove_repeat_vowels: bool,
    /// Collapse repeated consonants (e.g. 'มากกก' -> 'มาก')
    pub remove_repeat_consonants: bool,
    /// Remove zero-width characters (ZWSP, ZWNJ, etc.)
    pub remove_zero_width: bool,
    /// Remove spaces before tone marks / vowels ('พ ุ่ม' -> 'พุ่ม')
    pub remove_spaces_before_marks: bool,
    /// Remove dangling non-base marks at start of text or after spaces ('๊ก' -> 'ก')
    pub remove_dangling: bool,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        Self {
            normalize_sara_ae: true,
            normalize_sara_am: true,
            reorder_vowels_tone: true,
            remove_repeat_vowels: true,
            remove_repeat_consonants: true,
            remove_zero_width: true,
            remove_spaces_before_marks: true,
            remove_dangling: true,
        }
    }
}

#[inline]
fn is_thai_consonant(c: char) -> bool {
    matches!(c, 'ก'..='ฮ')
}

#[inline]
fn is_thai_upper_lower_vowel(c: char) -> bool {
    matches!(c, '\u{0E31}' | '\u{0E34}'..='\u{0E39}')
}

#[inline]
fn is_thai_non_base(c: char) -> bool {
    matches!(c, '\u{0E31}' | '\u{0E34}'..='\u{0E3A}' | '\u{0E47}'..='\u{0E4E}')
}

#[inline]
fn is_thai_tone(c: char) -> bool {
    matches!(c, '\u{0E48}'..='\u{0E4B}')
}

#[inline]
fn is_thai_tone_or_thanthakhat(c: char) -> bool {
    matches!(c, '\u{0E48}'..='\u{0E4C}')
}

#[inline]
fn is_thai_repeatable_vowel(c: char) -> bool {
    matches!(c, '\u{0E30}'..='\u{0E39}' | '\u{0E40}'..='\u{0E44}' | '\u{0E47}')
}

/// Normalize Thai text using default options.
pub fn normalize_text(text: &str) -> String {
    normalize_text_with_options_dict(text, &NormalizeOptions::default(), None)
}

/// Normalize Thai text using custom options.
pub fn normalize_text_with_options(text: &str, options: &NormalizeOptions) -> String {
    normalize_text_with_options_dict(text, options, None)
}

/// Normalize Thai text with dictionary verification for repeated consonants.
pub fn normalize_text_with_dict(text: &str, trie: &ThaiTrie) -> String {
    normalize_text_with_options_dict(text, &NormalizeOptions::default(), Some(trie))
}

/// Main normalization engine implementing Unicode and Thai orthographic standards.
pub fn normalize_text_with_options_dict(
    text: &str,
    options: &NormalizeOptions,
    trie: Option<&ThaiTrie>,
) -> String {
    if text.is_empty() {
        return String::new();
    }

    // Step 1: Remove zero-width characters
    let input: Vec<char> = if options.remove_zero_width {
        text.chars()
            .filter(|&c| {
                !matches!(
                    c,
                    '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{200E}' | '\u{200F}'
                )
            })
            .collect()
    } else {
        text.chars().collect()
    };

    let n = input.len();
    let mut step1: Vec<char> = Vec::with_capacity(n);

    // Step 2: Remove spaces before marks & convert Sara E + Sara E -> Sara Ae
    let mut i = 0;
    while i < n {
        let ch = input[i];

        // Sara E + Sara E -> Sara Ae
        if options.normalize_sara_ae && ch == 'เ' && i + 1 < n && input[i + 1] == 'เ' {
            step1.push('แ');
            i += 2;
            continue;
        }

        // Space before non-base mark after consonant ("พ ุ่ม" -> "พุ่ม")
        if options.remove_spaces_before_marks
            && ch == ' '
            && !step1.is_empty()
            && is_thai_consonant(*step1.last().unwrap())
        {
            let mut next_k = i + 1;
            while next_k < n && input[next_k] == ' ' {
                next_k += 1;
            }
            if next_k < n && is_thai_non_base(input[next_k]) {
                // Skip space
                i = next_k;
                continue;
            }
        }

        step1.push(ch);
        i += 1;
    }

    // Step 3: Sara Am, tone reordering, and Lakkhangyao
    let mut step2: Vec<char> = Vec::with_capacity(step1.len());
    let m = step1.len();
    let mut j = 0;
    while j < m {
        let ch = step1[j];

        // 3a. Sara Am normalization:
        // Case A: ํ (0x0E4D) + า (0x0E32) -> ำ (0x0E33)
        // Case B: ํ + tone + า -> tone + ำ ("นํ้า" -> "น้ำ")
        // Case C: tone + ํ + า -> tone + ำ
        if options.normalize_sara_am {
            // ํ + า + tone -> tone + ำ
            if ch == '\u{0E4D}'
                && j + 2 < m
                && step1[j + 1] == 'า'
                && is_thai_tone(step1[j + 2])
            {
                step2.push(step1[j + 2]);
                step2.push('ำ');
                j += 3;
                continue;
            }
            if ch == '\u{0E4D}' && j + 1 < m && step1[j + 1] == 'า' {
                step2.push('ำ');
                j += 2;
                continue;
            }
            if ch == '\u{0E4D}'
                && j + 2 < m
                && is_thai_tone(step1[j + 1])
                && step1[j + 2] == 'า'
            {
                step2.push(step1[j + 1]);
                step2.push('ำ');
                j += 3;
                continue;
            }
            if is_thai_tone(ch)
                && j + 2 < m
                && step1[j + 1] == '\u{0E4D}'
                && step1[j + 2] == 'า'
            {
                step2.push(ch);
                step2.push('ำ');
                j += 3;
                continue;
            }
        }

        // 3b. Misplaced vowels and tone marks reordering
        if options.reorder_vowels_tone {
            // Tone mark / Thanthakhat BEFORE upper/lower vowel -> vowel + tone
            if is_thai_tone_or_thanthakhat(ch)
                && j + 1 < m
                && is_thai_upper_lower_vowel(step1[j + 1])
            {
                step2.push(step1[j + 1]);
                step2.push(ch);
                j += 2;
                continue;
            }

            // Follow vowel (ะ, า, ำ, ๅ) BEFORE tone mark -> tone mark + follow vowel ("นำ้" -> "น้ำ")
            if matches!(ch, 'ะ' | 'า' | 'ำ' | 'ๅ') && j + 1 < m && is_thai_tone(step1[j + 1]) {
                step2.push(step1[j + 1]);
                let norm_v = if ch == 'ๅ' { 'า' } else { ch };
                step2.push(norm_v);
                j += 2;
                continue;
            }

            // Lakkhangyao (ๅ) to Sara Aa (า) unless preceded by Ru (ฤ) or Lu (ฦ)
            if ch == 'ๅ' {
                let prev = step2.last().copied();
                if prev != Some('ฤ') && prev != Some('ฦ') {
                    step2.push('า');
                    j += 1;
                    continue;
                }
            }
        }

        step2.push(ch);
        j += 1;
    }

    // Step 4: Dangling marks removal
    let mut step3: Vec<char> = Vec::with_capacity(step2.len());
    let mut is_after_space = true; // start of text
    for ch in step2 {
        if options.remove_dangling && is_after_space && is_thai_non_base(ch) {
            // Drop dangling non-base character at start of text or right after whitespace
            continue;
        }
        is_after_space = ch.is_whitespace();
        step3.push(ch);
    }

    // Step 5: Repetitions collapsing (vowels, tone marks, maiyamok, and consonants)
    let mut step4: Vec<char> = Vec::with_capacity(step3.len());
    let mut k = 0;
    let p = step3.len();
    while k < p {
        let ch = step3[k];

        // 5a. Repeated repeatable vowels -> collapse to 1
        if options.remove_repeat_vowels && is_thai_repeatable_vowel(ch) {
            let mut count = 1;
            while k + count < p && step3[k + count] == ch {
                count += 1;
            }
            step4.push(ch);
            k += count;
            continue;
        }

        // 5b. Repeated tone marks -> keep the last tone mark
        if options.remove_repeat_vowels && is_thai_tone_or_thanthakhat(ch) {
            let mut last_tone = ch;
            let mut count = 1;
            while k + count < p && is_thai_tone_or_thanthakhat(step3[k + count]) {
                last_tone = step3[k + count];
                count += 1;
            }
            step4.push(last_tone);
            k += count;
            continue;
        }

        // 5c. Repeated Maiyamok ('ๆๆๆ' -> 'ๆ')
        if options.remove_repeat_vowels && ch == 'ๆ' {
            let mut count = 1;
            while k + count < p && step3[k + count] == 'ๆ' {
                count += 1;
            }
            step4.push('ๆ');
            k += count;
            continue;
        }

        // 5d. Consonants repeated 3 or more times -> collapse to 1 ("มากกกก" -> "มาก")
        if options.remove_repeat_consonants && is_thai_consonant(ch) {
            let mut count = 1;
            while k + count < p && step3[k + count] == ch {
                count += 1;
            }
            if count >= 3 {
                step4.push(ch);
                k += count;
                continue;
            }
        }

        step4.push(ch);
        k += 1;
    }

    let mut result: String = step4.into_iter().collect();

    // Step 6: Dictionary-assisted reduction for trailing double consonants (e.g. "ดีมากก" -> "ดีมาก")
    if options.remove_repeat_consonants {
        if let Some(t) = trie {
            result = clean_trailing_double_consonants(&result, t);
        }
    }

    result
}

/// Helper function to clean trailing double consonants using dictionary verification.
/// E.g. "ตัวอย่างพนักงานโรงแรมให้บริการต้อนรับดีมากก"
/// "มากก" ends with doubled 'ก'. "มาก" is in dict, "มากก" is NOT in dict -> collapses to "มาก".
/// "ต้นกก": "กก" is in dict -> preserved.
fn clean_trailing_double_consonants(text: &str, trie: &ThaiTrie) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    if n < 2 {
        return text.to_string();
    }

    let mut out: Vec<char> = Vec::with_capacity(n);
    let mut i = 0;

    while i < n {
        let ch = chars[i];
        let is_double = i + 1 < n && chars[i + 1] == ch && is_thai_consonant(ch);
        let is_at_boundary = i + 2 == n || (i + 2 < n && is_boundary_char(chars[i + 2]));

        if is_double && is_at_boundary {
            // Find start of current Thai segment (lookback up to 20 chars)
            let mut start = i;
            while start > 0 && is_thai_char(chars[start - 1]) && i - start < 20 {
                start -= 1;
            }

            let cand_double: String = chars[start..=i + 1].iter().collect();

            // If the entire segment with double consonant is in dictionary (e.g. "ต้นกก", "กก"), keep it!
            if trie.contains(&cand_double) {
                out.push(ch);
                out.push(ch);
                i += 2;
                continue;
            }

            // Strip trailing consonants
            let mut head_end = i;
            while head_end > start && chars[head_end] == ch {
                head_end -= 1;
            }

            // Check if head + single consonant exists in dictionary
            let mut should_reduce = false;
            for s in start..=head_end {
                let mut cand_single: String = chars[s..=head_end].iter().collect();
                cand_single.push(ch);
                if trie.contains(&cand_single) {
                    should_reduce = true;
                    break;
                }
            }

            if should_reduce {
                out.push(ch);
                i += 2; // skip second consonant
                continue;
            }
        }

        out.push(ch);
        i += 1;
    }

    out.into_iter().collect()
}

#[inline]
fn is_thai_char(c: char) -> bool {
    let cp = c as u32;
    (0x0E00..=0x0E7F).contains(&cp)
}

#[inline]
fn is_boundary_char(c: char) -> bool {
    c.is_whitespace() || matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '\'' | '"' | '<' | '>' | '/' | '|' | 'ๆ' | 'ฯ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_width_removal() {
        let text = "สวัสดี\u{200B}ครับ\u{FEFF}";
        assert_eq!(normalize_text(text), "สวัสดีครับ");
    }

    #[test]
    fn test_spaces_before_marks() {
        assert_eq!(normalize_text("พ ุ่มดอกไม้"), "พุ่มดอกไม้");
    }

    #[test]
    fn test_sara_ae() {
        assert_eq!(normalize_text("เเปลก"), "แปลก");
    }

    #[test]
    fn test_sara_am() {
        assert_eq!(normalize_text("นํ้า"), "น้ำ");
        assert_eq!(normalize_text("กํ้า"), "ก้ำ");
        assert_eq!(normalize_text("ลําดับ"), "ลำดับ");
        // tone mark typed after Sara Am, or after decomposed Sara Am
        assert_eq!(normalize_text("น\u{0E33}\u{0E49}ตาล"), "น้ำตาล");
        assert_eq!(normalize_text("น\u{0E4D}\u{0E32}\u{0E49}ตาล"), "น้ำตาล");
    }

    #[test]
    fn test_reorder_tone_vowel() {
        // ก + ่ + ิ -> ก + ิ + ่
        let misplaced = "ก\u{0E48}\u{0E34}";
        let expected = "ก\u{0E34}\u{0E48}";
        assert_eq!(normalize_text(misplaced), expected);
    }

    #[test]
    fn test_repeat_vowels_and_tones() {
        assert_eq!(normalize_text("นานาาา"), "นานา");
        assert_eq!(normalize_text("ดีีีี"), "ดี");
        assert_eq!(normalize_text("มากๆๆๆ"), "มากๆ");
    }

    #[test]
    fn test_dangling_marks() {
        assert_eq!(normalize_text("๊ก"), "ก");
        assert_eq!(normalize_text("คำ ่ที่สอง"), "คำ ที่สอง");
    }

    #[test]
    fn test_repeat_consonants_unconditional() {
        assert_eq!(normalize_text("ดีมากกกก"), "ดีมาก");
        assert_eq!(normalize_text("สวยยยย"), "สวย");
        assert_eq!(normalize_text("จริงงงง"), "จริง");
        assert_eq!(normalize_text("อร่อยยยยยมากกก"), "อร่อยมาก");
        assert_eq!(normalize_text("สวัสดีค้าบบบ"), "สวัสดีค้าบ");
    }

    #[test]
    fn test_trailing_double_consonants_with_dict() {
        let mut trie = ThaiTrie::new();
        trie.add("มาก", 1.0);
        trie.add("ดีมาก", 1.0);
        trie.add("กก", 1.0); // "กก" is a valid word

        // "ดีมากก" -> "ดีมาก"
        let norm1 = normalize_text_with_dict("ดีมากก", &trie);
        assert_eq!(norm1, "ดีมาก");

        // "ต้นกก" -> "ต้นกก" (preserved because "กก" is in trie)
        let norm2 = normalize_text_with_dict("ต้นกก", &trie);
        assert_eq!(norm2, "ต้นกก");
    }
}
