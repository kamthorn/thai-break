use std::path::Path;
use thaibreak::{
    display_width, lines, set_default, words, wrap, ThaiTrie, Tokenizer,
    DEFAULT_BREAK_MARKER,
};

fn init_test_dict() {
    let dict_path = "../data/words.txt";

    if Path::new(dict_path).exists() {
        let trie = ThaiTrie::load_tsv_file(dict_path).expect("Failed to load wordlist");
        let tokenizer = Tokenizer::new(trie, None);
        set_default(tokenizer);
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
