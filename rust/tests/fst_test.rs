use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use thaibreak::{ThaiTrie, Tokenizer};

#[test]
fn test_fst_compilation_and_loading() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("thaibreak_fst_test_{}", now));
    std::fs::create_dir_all(&dir).unwrap();

    let tsv_path = dir.join("test_words.txt");
    let fst_path = dir.join("test_words.fst");

    // Create custom test dictionary with uniform and weighted words
    {
        let mut f = File::create(&tsv_path).unwrap();
        writeln!(f, "กงการ\t1.0").unwrap();
        writeln!(f, "กฎหมาย\t2.5").unwrap();
        writeln!(f, "การ\t1.0").unwrap();
        writeln!(f, "การบ้าน\t1.8").unwrap();
        writeln!(f, "สวัสดี\t3.0").unwrap();
    }

    // 1. Compile to FST
    ThaiTrie::compile_tsv_file_to_fst(&tsv_path, &fst_path).expect("Failed to compile TSV to FST");

    // 2. Load Flat Trie vs FST Trie
    let flat_trie = ThaiTrie::load_tsv_file(&tsv_path).expect("Failed to load TSV");
    let fst_trie = ThaiTrie::load_fst_file(&fst_path).expect("Failed to load FST");

    assert_eq!(flat_trie.len(), fst_trie.len());
    assert!((flat_trie.total_weight() - fst_trie.total_weight()).abs() < 1e-4);
    assert!((flat_trie.max_weight() - fst_trie.max_weight()).abs() < 1e-4);

    // 3. Compare prefix matches
    let text: Vec<char> = "การบ้านของฉันตามกฎหมาย".chars().collect();
    let flat_matches = flat_trie.prefixes_from_chars(&text, 0, 10);
    let fst_matches = fst_trie.prefixes_from_chars(&text, 0, 10);

    assert_eq!(flat_matches, fst_matches);
    assert_eq!(flat_matches.len(), 2); // "การ", "การบ้าน"
    assert_eq!(flat_matches[0].word, "การ");
    assert_eq!(flat_matches[1].word, "การบ้าน");
    assert!((flat_matches[1].weight - 1.8).abs() < 1e-4);

    // 4. Test Tokenizer segmentation with FST
    let flat_tok = Tokenizer::new(flat_trie, None);
    let fst_tok = Tokenizer::new(fst_trie.clone(), None);

    let sample = "สวัสดีการบ้านตามกฎหมาย";
    let res_flat = flat_tok.tokenize(sample, false);
    let res_fst = fst_tok.tokenize(sample, false);
    assert_eq!(res_flat, res_fst);

    // 5. Test Dynamic Overlay add() on FST
    let mut dynamic_fst = fst_trie;
    dynamic_fst.add("ตาม", 1.5);
    let dynamic_tok = Tokenizer::new(dynamic_fst, None);
    let res_dynamic = dynamic_tok.tokenize(sample, false);
    assert!(res_dynamic.contains(&"ตาม".to_string()));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_standard_data_words_fst_matches_txt() {
    let txt_path = "../data/words.txt";
    let fst_path = "../data/words.fst";

    if !std::path::Path::new(fst_path).exists() {
        ThaiTrie::compile_tsv_file_to_fst(txt_path, fst_path).unwrap();
    }

    let trie_txt = ThaiTrie::load_tsv_file(txt_path).expect("load txt");
    let trie_fst = ThaiTrie::load_fst_file(fst_path).expect("load fst");

    let tok_txt = Tokenizer::new(trie_txt, None);
    let tok_fst = Tokenizer::new(trie_fst, None);

    let sentences = [
        "สถาบันวิจัยดาราศาสตร์แห่งชาติกระทรวงการอุดมศึกษาวิทยาศาสตร์วิจัยและนวัตกรรม",
        "กรุงเทพมหานครอมรรัตนโกสินทร์มหินทรายุธยามหาดิลกภพนพรัตน์ราชธานีบุรีรมย์",
        "ปัญญาประดิษฐ์กำลังเปลี่ยนแปลงวิถีชีวิตและระบบการทำงานของมนุษย์ทั่วโลก",
        "ทดสอบคำสั้นคำยาว กก กกหู กงการ กฎหมาย วิทยาลัยดนตรี",
    ];

    for s in &sentences {
        let words_txt = tok_txt.tokenize(s, false);
        let words_fst = tok_fst.tokenize(s, false);
        assert_eq!(words_txt, words_fst, "Tokenization mismatch on text: {}", s);
    }
}

#[cfg(feature = "embedded-dict")]
#[test]
fn embedded_dictionary_matches_data_dir() {
    let embedded = thaibreak::trie::ThaiTrie::embedded().expect("embedded dict loads");
    let mut on_disk = thaibreak::trie::ThaiTrie::load_fst_file(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/words.fst"))
        .expect("data/words.fst loads");
    let applied = on_disk
        .apply_weights_file(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/word-weights.tsv"))
        .expect("data/word-weights.tsv loads");
    assert!(applied > 0);
    assert_eq!(embedded.max_weight(), on_disk.max_weight());
    assert_eq!(embedded.total_weight(), on_disk.total_weight());
    assert!(embedded.max_weight() > 1.0, "the embedded dictionary carries word weights");
}

#[test]
fn set_weight_replaces_the_weight_of_a_dictionary_word() {
    let tsv = "แมว\nแมวน้ำ\nหมา\n";
    let mut fst_bytes = Vec::new();
    ThaiTrie::compile_tsv_to_fst(tsv.as_bytes(), &mut fst_bytes).unwrap();
    let flat = ThaiTrie::load_tsv(tsv.as_bytes()).unwrap();
    let fst = ThaiTrie::from_fst_bytes(fst_bytes.into()).unwrap();
    for mut trie in [flat, fst] {
        assert!(trie.set_weight("แมว", 5.0));
        assert_eq!(trie.total_weight(), 7.0);
        assert!(trie.set_weight("แมว", 2.0));
        assert_eq!(trie.total_weight(), 4.0);
        assert!(!trie.set_weight("ปลา", 3.0), "a word outside the dictionary is not added");
        assert!(!trie.set_weight("แม", 3.0), "a prefix is not a word");
        assert_eq!(trie.total_weight(), 4.0);
        let chars: Vec<char> = "แมวน้ำ".chars().collect();
        let weights: Vec<(String, f64)> =
            trie.prefixes_from_chars(&chars, 0, 25).into_iter().map(|m| (m.word, m.weight)).collect();
        assert_eq!(weights, vec![("แมว".to_string(), 2.0), ("แมวน้ำ".to_string(), 1.0)]);
        assert_eq!(trie.apply_weights("# comment\nหมา\t3.5\nปลา\t2\n".as_bytes()).unwrap(), 1);
        assert_eq!(trie.total_weight(), 6.5);
        assert!(trie.apply_weights("หมา\tx\n".as_bytes()).is_err());
    }
}

/// `prefix_ends_from_chars` is `prefixes_from_chars` without the words, for every start.
fn assert_prefix_ends_match(trie: &ThaiTrie, text: &str) {
    let chars: Vec<char> = text.chars().collect();
    let mut ends = Vec::new();
    for start in 0..chars.len() {
        for max_len in [1, 2, 3, 40] {
            ends.clear();
            trie.prefix_ends_from_chars(&chars, start, max_len, &mut ends);
            let expected: Vec<(usize, f64)> =
                trie.prefixes_from_chars(&chars, start, max_len).into_iter().map(|m| (m.end, m.weight)).collect();
            assert_eq!(ends, expected, "start {start}, max_len {max_len} in {text:?}");
        }
    }
}

#[test]
fn prefix_ends_match_prefixes() {
    let tsv = "ก\nกา\nกาก\nกาแฟ\t4\nแมว\nแมวน้ำ\t2\nmail\nอีเมล\nอีเมลล์\nๆ\n";
    let text = "กาแฟกากแมวน้ำอีเมลล์ mail กๆ ก";
    let mut fst_bytes = Vec::new();
    ThaiTrie::compile_tsv_to_fst(tsv.as_bytes(), &mut fst_bytes).unwrap();
    let flat = ThaiTrie::load_tsv(tsv.as_bytes()).unwrap();
    let fst = ThaiTrie::from_fst_bytes(fst_bytes.into()).unwrap();
    for mut trie in [flat, fst] {
        assert_prefix_ends_match(&trie, text);
        trie.set_weight("แมว", 3.0);
        trie.add("กาแฟเย็น", 2.0);
        trie.add("กาแฟ", 5.0);
        trie.add("ma", 1.0);
        assert_prefix_ends_match(&trie, text);
        assert_prefix_ends_match(&trie, "กาแฟเย็นmailma");
    }
}

#[cfg(feature = "embedded-dict")]
#[test]
fn prefix_ends_match_prefixes_in_the_embedded_dictionary() {
    let trie = ThaiTrie::embedded().unwrap();
    assert_prefix_ends_match(&trie, "พระบาทสมเด็จพระเจ้าอยู่หัวเสด็จฯ ไปทรงเปิดงานที่กรุงเทพมหานคร เมื่อวันที่ 5 ธ.ค.");
    assert_prefix_ends_match(&trie, "นํ้าแข็งใส่แก้ว Eucerin ค่ะ ไม่อยากก้ไม่ต้องกิน");
}

#[test]
fn fst_header_with_impossible_weight_count_is_an_error() {
    // "TBF1", flags = weights present, total/max weight, count = u32::MAX: the weights cannot fit
    let mut bytes = b"TBF1".to_vec();
    bytes.extend_from_slice(&[1, 0, 0, 0]);
    bytes.extend_from_slice(&1f64.to_le_bytes());
    bytes.extend_from_slice(&1f64.to_le_bytes());
    bytes.extend_from_slice(&u32::MAX.to_le_bytes());
    assert!(thaibreak::ThaiTrie::from_fst_bytes(bytes.into()).is_err());
}

#[test]
fn a_prefix_of_an_added_word_is_still_a_dictionary_word() {
    // Adding "ไปตำ" puts "ไป" into the overlay as a prefix marker; "ไป" must stay a word, or the
    // full stop in "ต่อไป." is taken for an abbreviation
    let mut trie = ThaiTrie::embedded().expect("embedded dictionary");
    assert!(trie.contains("ไป"));
    assert!(!trie.contains("ไปตำ"));
    trie.add("ไปตำ", 1.0);
    assert!(trie.contains("ไป"), "a prefix marker hides a dictionary word");
    assert!(trie.contains("ไปตำ"));
    assert!(!trie.contains("ไปต"), "a prefix that is no word is not a word");
    let tok = Tokenizer::new(trie, None);
    assert_eq!(tok.tokenize("ต่อไป.", false).join("|"), "ต่อ|ไป|.");
}
