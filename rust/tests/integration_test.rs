use std::path::Path;
use thaibreak::{
    display_width, lines, set_default, words, wrap, ThaiTrie, Tokenizer,
    DEFAULT_BREAK_MARKER,
};

fn init_test_dict() {
    let candidates = [
        "../data/words.fst",
        "../../data/words.fst",
        "data/words.fst",
        "../data/words.txt",
        "../../data/words.txt",
        "data/words.txt",
    ];
    for &path in &candidates {
        if Path::new(path).exists() {
            let trie = if path.ends_with(".fst") {
                ThaiTrie::load_fst_file(path).expect("Failed to load fst")
            } else {
                ThaiTrie::load_tsv_file(path).expect("Failed to load wordlist")
            };
            let tokenizer = Tokenizer::new(trie, None);
            set_default(tokenizer);
            return;
        }
    }
}

#[test]
fn test_words_tokenization() {
    init_test_dict();
    let result = words("ฉันรักภาษาไทย");
    assert_eq!(result, vec!["ฉัน", "รัก", "ภาษา", "ไทย"]);
}

#[test]
fn test_typographic_rules() {
    init_test_dict();
    let text = "เดินเล่นๆ (ที่นี่ๆ) กรุงเทพฯ ฿100";
    let broken = lines(text, DEFAULT_BREAK_MARKER, false);

    assert!(
        !broken.contains(&format!("{}ๆ", DEFAULT_BREAK_MARKER)),
        "Should not break before ๆ"
    );
    assert!(
        !broken.contains(&format!("{}ฯ", DEFAULT_BREAK_MARKER)),
        "Should not break before ฯ"
    );
    assert!(
        !broken.contains(&format!("({}", DEFAULT_BREAK_MARKER)),
        "Should not break after ("
    );
    assert!(
        !broken.contains(&format!("{})", DEFAULT_BREAK_MARKER)),
        "Should not break before )"
    );
    assert!(
        !broken.contains(&format!(" {}", DEFAULT_BREAK_MARKER)),
        "Should not break after space"
    );
    assert!(
        !broken.contains(&format!("{} ", DEFAULT_BREAK_MARKER)),
        "Should not break before space"
    );
}

#[test]
fn test_html_preservation() {
    init_test_dict();
    let html = "<div class=\"title\"><b>สวัสดี</b> &amp; ประเทศไทย</div><script>var x = \"สวัสดีประเทศไทย\";</script><!-- คอมเมนต์ -->";
    let broken = lines(html, DEFAULT_BREAK_MARKER, true);

    assert!(broken.contains("<div class=\"title\">"), "Opening tag damaged");
    assert!(broken.contains("&amp;"), "Entity damaged");
    assert!(broken.contains("<script>var x = \"สวัสดีประเทศไทย\";</script>"), "Script tag damaged");
    assert!(broken.contains("<!-- คอมเมนต์ -->"), "Comment damaged");
    assert_eq!(
        broken.replace(DEFAULT_BREAK_MARKER, ""),
        html,
        "Stripped HTML matches original"
    );
}

#[test]
fn test_display_width() {
    assert_eq!(display_width("ก"), 1);
    assert_eq!(display_width("ก็"), 1);
    assert_eq!(display_width("ที่"), 1);
    assert_eq!(display_width("ภาษาไทย"), 7);
    assert_eq!(display_width("English"), 7);
    assert_eq!(display_width("ไทย"), 3);
}

#[test]
fn test_wrapping() {
    init_test_dict();
    let text = "ฉันรักภาษาไทยมากที่สุดในโลก";
    let wrapped = wrap(text, 12, false);
    let line_count = wrapped.lines().count();
    assert!(line_count >= 2, "Should wrap into multiple lines");
}

#[test]
fn test_latin_letters_and_digits_stay_together() {
    // UAX #14 LB23: no break between letters and digits.
    init_test_dict();
    let broken = lines("ตาม WP01 และมาตรฐาน ISO29110", "|", false);
    assert!(broken.contains("WP01"), "{}", broken);
    assert!(broken.contains("ISO29110"), "{}", broken);
}

#[test]
fn test_no_break_before_solidus() {
    // UAX #14 LB13: "ISO/IEC" must not become "ISO" + "/IEC".
    init_test_dict();
    let broken = lines("ISO/IEC 29110 กำหนดให้มี", "|", false);
    assert!(!broken.contains("|/"), "{}", broken);
}

#[test]
fn test_wrapping_breaks_at_spaces() {
    init_test_dict();
    let wrapped = wrap("the quick brown fox jumps over the lazy dog", 10, false);
    assert_eq!(
        wrapped.lines().collect::<Vec<_>>(),
        vec!["the quick", "brown fox", "jumps over", "the lazy", "dog"]
    );
}

#[test]
fn test_dual_engine() {
    use thaibreak::{LineBreaker, ThaiTrie, Tokenizer};

    let mut words_trie = ThaiTrie::new();
    words_trie.add("กรมการกงสุล", 1.0);
    words_trie.add("ไป", 1.0);

    let mut lines_trie = ThaiTrie::new();
    lines_trie.add("กรม", 1.0);
    lines_trie.add("การ", 1.0);
    lines_trie.add("กงสุล", 1.0);
    lines_trie.add("ไป", 1.0);

    let words_tok = Tokenizer::new(words_trie, None);
    let lines_tok = Tokenizer::new(lines_trie, None);
    let breaker = LineBreaker::new(lines_tok);

    let w = words_tok.tokenize("ไปกรมการกงสุล", false);
    assert_eq!(w, vec!["ไป", "กรมการกงสุล"]);

    let l = breaker.insert_line_breaks("ไปกรมการกงสุล", "|", false);
    assert_eq!(l, "ไป|กรม|การ|กงสุล");
}
