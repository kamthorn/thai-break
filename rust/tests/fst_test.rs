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
    let on_disk = thaibreak::trie::ThaiTrie::load_fst_file(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/words.fst"))
        .expect("data/words.fst loads");
    assert_eq!(embedded.max_weight(), on_disk.max_weight());
    assert!(!embedded.is_empty());
}
