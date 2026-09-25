use std::path::Path;
use thaibreak::{lines, set_default, ThaiTrie, Tokenizer};

fn init_test_dict() {
    let dict_path = "../data/words.txt";
    if Path::new(dict_path).exists() {
        let trie = ThaiTrie::load_tsv_file(dict_path).expect("Failed to load wordlist");
        set_default(Tokenizer::new(trie, None));
    }
}

/// (name, input, expected output with '|' as the break marker)
const LINE_BREAK_CASES: &[(&str, &str, &str)] = &[
    ("LB9 combining mark stays with its base", "สวัสดี\u{0301}ครับ", "สวัสดี\u{0301}|ครับ"),
];

#[test]
fn test_insert_line_breaks_uax14() {
    init_test_dict();
    for (name, input, expected) in LINE_BREAK_CASES {
        assert_eq!(lines(input, "|", false), *expected, "{name}");
    }
}
