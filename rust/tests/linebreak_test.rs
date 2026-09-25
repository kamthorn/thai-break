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
    ("LB25 no break inside a date", "วันที่ 1/2/2567 นะ", "วัน|ที่ 1/2/2567 นะ"),
    ("LB25 no break inside a time", "เวลา 10:30 น.", "เวลา 10:30 น."),
    ("LB25 no break inside a range or a signed number", "ช่วง 10-20 คน ลบ -5 องศา", "ช่วง 10-20 คน ลบ -5 องศา"),
    ("LB25 no break after a prefix or before a postfix", "ราคา $(5) ลด 40%", "ราคา $(5) ลด 40%"),
    ("LB25 no break between IS and NU", "พ.ศ.2567", "พ.ศ.2567"),
    ("LB29 no break after a full stop before a letter", "รพ.ศิริราช", "รพ.ศิริราช"),
    ("LB29 no break inside an abbreviation", "e.g.ไทย", "e.g.ไทย"),
    ("LB29 no break after an ellipsis of full stops", "ทดสอบ...ต่อ", "ทด|สอบ...ต่อ"),
    ("LB28 no break between Thai and Latin letters", "ภาษาPHPเป็น", "ภาษาPHPเป็น"),
    ("LB28 no break inside an email address", "ติดต่อ user@example.com ได้", "ติดต่อ user@example.com ได้"),
    ("LB28 no break around # (AL)", "แท็ก#ไทยดี", "แท็ก#ไทย|ดี"),
    ("LB28 no break inside a Latin word with a combining mark", "café́ ไทย", "café́ ไทย"),
    ("LB23 no break between Thai letters and digits", "ราคา100บาท", "ราคา100บาท"),
    ("LB23 no break between Thai letters and Thai digits", "ปี๒๕๖๗นะ", "ปี๒๕๖๗นะ"),
    ("LB23 no break between Latin letters and digits", "เอกสารWP01ของ", "เอกสารWP01ของ"),
    ("LB30 no break between a letter and an opening parenthesis", "ประเทศไทย(สยาม)เป็นประเทศ", "ประเทศ|ไทย(สยาม)เป็น|ประเทศ"),
    ("LB30 no break between a closing bracket and a letter", "[หมายเหตุ]ข้อความ", "[หมายเหตุ]ข้อความ"),
    ("LB19 no break around quotation marks", "ไทย‘คำ’ไทย", "ไทย‘คำ’ไทย"),
    ("LB19a no break around guillemets outside East Asian text", "«ไทย»ไทย", "«ไทย»ไทย"),
    ("LB15a/LB15b quotes stay with the quoted text", "เขาพูดว่า “สวัสดี” แล้ว", "เขา|พูด|ว่า “สวัสดี” แล้ว"),
    ("LB12 no break around a no-break space", "ราคา 100 บาท", "ราคา 100 บาท"),
    ("LB11 no break at a word joiner", "ไทย⁠ไทย", "ไทย⁠ไทย"),
    ("LB30b emoji modifier stays with its base", "ดี👍🏽มาก", "ดี|👍🏽|มาก"),
    ("LB30a regional indicators pair into flags", "🇹🇭🇯🇵", "🇹🇭|🇯🇵"),
    ("LB24 backslash (PR) behaves the same in every port", "A\\B", "A\\B"),
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
    ("LB14 no break after an opening parenthesis, even after spaces", "ข้อความ ( ไทย ) ต่อ", 8, &["ข้อความ", "( ไทย )", "ต่อ"]),
    ("mai yamok never starts a line, even after a space", "เด็ก ๆ เล่น", 4, &["เด็ก ๆ", "เล่น"]),
];

#[test]
fn test_wrap_uax14() {
    init_test_dict();
    for (name, input, width, expected) in WRAP_CASES {
        assert_eq!(wrap(input, *width, false).split('\n').collect::<Vec<_>>(), *expected, "{name}");
    }
}
