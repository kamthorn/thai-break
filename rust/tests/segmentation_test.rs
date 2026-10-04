use std::path::Path;
use thaibreak::{set_default, tcc_pos_array, words, ThaiTrie, Tokenizer};

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
    ("ก็ is not swallowed by the cluster before it", "ทะเลก็สวย", "ทะเล|ก็|สวย"),
    ("a final consonant before a vowel starts the next cluster", "รึยัง", "รึ|ยัง"),
    ("no-break spaces are whitespace, not words", "ราคา 100 บาท", "ราคา|100|บาท"),
    ("two sara e are matched as sara ae, the text is kept", "เเข็งเเรงมาก", "เเข็งเเรง|มาก"),
    ("nikhahit + sara aa is matched as sara am, the text is kept", "นํ้าตาลทราย", "นํ้าตาล|ทราย"),
    ("an out-of-vocabulary name made of dictionary words is kept whole", "ชวรัตน์", "ชวรัตน์"),
    ("an out-of-vocabulary name is kept whole in a sentence", "นายศุภชัยกล่าวว่ามีการประชุม", "นาย|ศุภชัย|กล่าว|ว่า|มี|การ|ประชุม"),
    ("the repetition mark is not part of an unknown word", "อื่นๆ", "อื่น|ๆ"),
    ("an unknown word does not swallow the function words after it", "มาร์คฮิวจ์สไม่ได้สามารถ", "มาร์ค|ฮิวจ์ส|ไม่|ได้|สามารถ"),
    ("an unknown word does not swallow a conjunction", "ภานุวัฒน์ จินตะและสุทธินันท์", "ภานุวัฒน์|จินตะ|และ|สุทธิ|นันท์"),
    ("a tone mark after sara am is matched before it, the text is kept", "นำ้ตาลทราย", "นำ้ตาล|ทราย"),
    ("nikhahit + sara aa + tone mark is matched as tone + sara am", "นํา้ตาลทราย", "นํา้ตาล|ทราย"),
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
fn test_tcc_never_splits_before_a_vowel_or_tone_mark() {
    for word in ["เมื่อ", "เนื้อ", "เบื่อ", "ต้น", "เกล็ด", "เหม็น", "ลั๊วะ", "รึยัง"] {
        let chars: Vec<char> = word.chars().collect();
        let valid = tcc_pos_array(&chars);
        for i in 1..chars.len() {
            let dependent = matches!(chars[i] as u32, 0x0E30..=0x0E3A | 0x0E45 | 0x0E47..=0x0E4E);
            assert!(!(dependent && valid[i]), "boundary before {} in {}", chars[i], word);
        }
    }
}

#[test]
fn test_tcc_keeps_a_mai_han_akat_syllable_whole() {
    // Mai Han-akat is always followed by a final (or -ัวะ), so the syllable is one cluster
    for word in ["ผัวะ", "จั๊วะ", "ยัง", "ยั่ง", "หัว", "สัญ", "กัณฐ์"] {
        let chars: Vec<char> = word.chars().collect();
        let valid = tcc_pos_array(&chars);
        assert!((1..chars.len()).all(|i| !valid[i]), "{} was split", word);
    }
}

#[test]
fn test_tcc_keeps_a_cluster_onset_after_sara_e_whole() {
    // เ + a true cluster or ห-led onset + -ิ / -ือ / -า is one syllable
    for word in ["เปล่า", "เหล้า", "เหงา", "เพลิง", "เหลือ", "เกลือ", "เครือ"] {
        let chars: Vec<char> = word.chars().collect();
        let valid = tcc_pos_array(&chars);
        assert!((1..chars.len()).all(|i| !valid[i]), "{} was split", word);
    }
    // a consonant pair that is not a cluster can still start the next word
    for (text, boundary) in [("เทลง", 2), ("ทะเลว่า", 4)] {
        let chars: Vec<char> = text.chars().collect();
        assert!(tcc_pos_array(&chars)[boundary], "no boundary at {} in {}", boundary, text);
    }
}

#[test]
fn test_segmentation() {
    init_test_dict();
    for (name, input, expected) in SEGMENTATION_CASES {
        assert_eq!(words(input).join("|"), *expected, "{name}");
    }
}

#[test]
fn break_iterator_follows_the_viterbi_boundaries() {
    use thaibreak::BreakIterator;
    let trie = ThaiTrie::load_tsv_file("../data/words.txt").expect("Failed to load wordlist");
    let tokenizer = Tokenizer::new(trie, None);
    // "นํ้า" is matched as น้ำ but keeps its characters, so offsets are into the original text
    let text = "นํ้าตาลทราย ราคา 100 บาท";
    let mut it = BreakIterator::new(&tokenizer, text);

    let boundaries: Vec<usize> = std::iter::once(it.current()).chain(it.clone()).collect();
    let segments: Vec<&str> = boundaries.windows(2).map(|w| &text[w[0]..w[1]]).collect();
    assert_eq!(segments.concat(), text);
    let words: Vec<&str> = segments.iter().copied().filter(|s| !s.trim().is_empty()).collect();
    assert_eq!(words, tokenizer.tokenize(text, false).iter().map(String::as_str).collect::<Vec<_>>());

    let mid = boundaries[1];
    assert!(it.is_boundary(mid));
    assert_eq!(it.current(), mid);
    // one byte into the next word is inside a character, so not a boundary
    assert!(!it.is_boundary(mid + 1));
    assert_eq!(it.current(), boundaries[2]);
    assert_eq!(it.following(mid), Some(boundaries[2]));
    assert_eq!(it.preceding(mid), Some(boundaries[0]));
    assert_eq!(it.preceding(0), None);
    assert_eq!(it.following(text.len()), None);
    assert_eq!(it.last_boundary(), text.len());
    assert_eq!(it.next_boundary(), None);
    assert_eq!(it.first_boundary(), 0);
    assert_eq!(it.previous(), None);

    let mut empty = BreakIterator::new(&tokenizer, "");
    assert_eq!((empty.current(), empty.next_boundary()), (0, None));
}

#[test]
fn break_iterator_offsets_in_code_points_and_utf16() {
    use thaibreak::BreakIterator;
    let trie = ThaiTrie::load_tsv_file("../data/words.txt").expect("Failed to load wordlist");
    let tokenizer = Tokenizer::new(trie, None);
    // 😀 is 4 bytes, 2 UTF-16 units and 1 code point
    let text = "กินข้าว😀ดี";
    let it = BreakIterator::new(&tokenizer, text);
    let bytes = it.boundaries().to_vec();
    assert_eq!(bytes.first(), Some(&0));
    assert_eq!(bytes.last(), Some(&text.len()));
    let chars = it.boundaries_in(text, false);
    let utf16 = it.boundaries_in(text, true);
    assert_eq!(chars.len(), bytes.len());
    assert_eq!(chars.last(), Some(&text.chars().count()));
    assert_eq!(utf16.last(), Some(&text.encode_utf16().count()));
    let units: Vec<u16> = text.encode_utf16().collect();
    for (i, w) in utf16.windows(2).enumerate() {
        let segment = String::from_utf16(&units[w[0]..w[1]]).unwrap();
        assert_eq!(segment, text[bytes[i]..bytes[i + 1]]);
    }
}

#[cfg(feature = "c-ffi")]
#[test]
fn c_boundaries_round_trip() {
    use std::ffi::CString;
    let text = CString::new("กินข้าว ครับ").unwrap();
    let mut count = 0usize;
    unsafe {
        let ptr = thaibreak::c_ffi::thaibreak_boundaries(text.as_ptr(), &mut count);
        assert!(!ptr.is_null());
        let offsets = std::slice::from_raw_parts(ptr, count).to_vec();
        thaibreak::c_ffi::thaibreak_free_boundaries(ptr, count);
        assert_eq!(offsets.first(), Some(&0));
        assert_eq!(offsets.last(), Some(&text.as_bytes().len()));
        assert!(offsets.windows(2).all(|w| w[0] < w[1]));
        assert!(thaibreak::c_ffi::thaibreak_boundaries(std::ptr::null(), &mut count).is_null());
    }
}
