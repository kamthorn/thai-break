use std::path::Path;
use thaibreak::{lines, set_default, wrap, ThaiTrie, Tokenizer};

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
    ("LB21 no break before a hyphen", "สี-ขาว", "สี-|ขาว"),
    ("LB21 no break before an en dash (BA)", "ข้อความ–ข้อความ", "ข้อความ–|ข้อความ"),
    ("LB13 no break before CJK closing punctuation (CL)", "ไทย、ไทย", "ไทย、|ไทย"),
    ("LB13 no break before ? (EX)", "ไปไหม?ไปสิ", "ไป|ไหม?|ไป|สิ"),
    ("LB13 no break before a solidus (SY)", "ISO/IEC 29110", "ISO/|IEC 29110"),
];

#[test]
fn test_insert_line_breaks_uax14() {
    init_test_dict();
    for (name, input, expected) in LINE_BREAK_CASES {
        assert_eq!(lines(input, "|", false), *expected, "{name}");
    }
}

/// (name, input, width, expected lines)
const WRAP_CASES: &[(&str, &str, usize, &[&str])] = &[
    ("an opening bracket never ends a line", "ประชาชน (ทั่วประเทศ) ไป", 9, &["ประชาชน", "(ทั่ว", "ประเทศ)", "ไป"]),
    ("a dash never starts a line", "ภาษาไทย–อังกฤษ", 7, &["ภาษา", "ไทย–", "อังกฤษ"]),
    ("mai yamok never starts a line", "ทดสอบเด็กๆๆๆๆๆๆ", 5, &["ทดสอบ", "เด็กๆๆๆๆๆๆ"]),
    ("an opening quote never ends a line", "สวัสดีครับ “ท่านผู้ชม”", 11, &["สวัสดีครับ", "“ท่านผู้ชม”"]),
    ("indentation that does not fit is dropped", "  ย่อหน้า ใหม่ ครับ", 6, &["ย่อหน้า", "ใหม่", "ครับ"]),
];

#[test]
fn test_wrap_uax14() {
    init_test_dict();
    for (name, input, width, expected) in WRAP_CASES {
        assert_eq!(wrap(input, *width, false).split('\n').collect::<Vec<_>>(), *expected, "{name}");
    }
}
