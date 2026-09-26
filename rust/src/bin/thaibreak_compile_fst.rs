use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thaibreak::ThaiTrie;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: thaibreak_compile_fst <input_tsv> [output_fst]");
        eprintln!("Example: thaibreak_compile_fst ../data/words.txt ../data/words.fst");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        input_path.with_extension("fst")
    };

    println!("Compiling {} -> {} ...", input_path.display(), output_path.display());
    let t0 = Instant::now();
    ThaiTrie::compile_tsv_file_to_fst(input_path, &output_path)?;
    let elapsed = t0.elapsed();

    let in_size = std::fs::metadata(input_path)?.len();
    let out_size = std::fs::metadata(&output_path)?.len();

    println!("Done in {:.2} ms!", elapsed.as_secs_f64() * 1000.0);
    println!("  Input size : {} bytes ({:.2} KB)", in_size, in_size as f64 / 1024.0);
    println!("  Output size: {} bytes ({:.2} KB)", out_size, out_size as f64 / 1024.0);
    println!("  Compression: {:.1}% of original size", (out_size as f64 / in_size as f64) * 100.0);

    // Verify it loads correctly
    let trie = ThaiTrie::load_fst_file(&output_path)?;
    println!("  Verified FST dictionary loaded with {} words, total_weight={}", trie.len(), trie.total_weight());

    Ok(())
}
