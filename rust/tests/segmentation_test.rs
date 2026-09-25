use std::path::Path;
use thaibreak::{set_default, words, ThaiTrie, Tokenizer};

fn init_test_dict() {
    let dict_path = "../data/words.txt";
    if Path::new(dict_path).exists() {
        let trie = ThaiTrie::load_tsv_file(dict_path).expect("Failed to load wordlist");
        set_default(Tokenizer::new(trie, None));
    }
}

/// (name, input, expected words joined by '|')
const SEGMENTATION_CASES: &[(&str, &str, &str)] = &[
    ("a word before a full stop is not cut into an abbreviation", "เขากินข้าว.", "เขา|กิน|ข้าว|."),
    ("a word before a full stop, then more text", "ฉันรักเธอ.ไปเที่ยวกัน", "ฉัน|รัก|เธอ|.|ไป|เที่ยว|กัน"),
    ("abbreviations are still recognized", "เมื่อ 5 มิ.ย. ที่ จ.พิษณุโลก", "เมื่อ|5|มิ.ย.|ที่|จ.|พิษณุโลก"),
    ("an abbreviation after a word keeps the word whole", "ในเขตจ.พิจิตร", "ใน|เขต|จ.|พิจิตร"),
    ("an abbreviation after a word that ends like one", "ในเดือนพ.ย.", "ใน|เดือน|พ.ย."),
    ("ties keep the earlier word whole", "บอกว่าอึดอัด", "บอก|ว่า|อึดอัด"),
    ("ties keep the earlier word whole (2)", "ลาออกจากรองประธาน", "ลาออก|จาก|รอง|ประธาน"),
];

#[test]
fn test_long_text_is_segmented_to_the_end() {
    // Longer than the 50,000-edge limit that used to leave the rest of the text as one token
    init_test_dict();
    let sentence = "การประชุมสามัญผู้ถือหุ้นประจำปีจัดขึ้นที่โรงแรมในกรุงเทพมหานคร";
    let result = words(&sentence.repeat(2000));
    assert_eq!(result.len(), 2000 * words(sentence).len());
    assert!(result.iter().all(|w| w.chars().count() < 20));
}

#[test]
fn test_segmentation() {
    init_test_dict();
    for (name, input, expected) in SEGMENTATION_CASES {
        assert_eq!(words(input).join("|"), *expected, "{name}");
    }
}
