//! Write the TCC DFA tables into the sources of the Go and TypeScript ports, and optionally of
//! the OpenSearch plugin.
//!
//! Usage: thaibreak_gen_tcc [<thai-break repository root>] [--java <path of ThaiTccDfa.java>]

use std::path::PathBuf;
use thaibreak::tcc_gen;

fn write(path: PathBuf, source: String) {
    std::fs::write(&path, source).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    println!("wrote {}", path.display());
}

fn main() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut java = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--java" {
            java = Some(PathBuf::from(args.next().expect("--java needs a path")));
        } else {
            root = PathBuf::from(arg);
        }
    }
    write(root.join("go/tcc_dfa.go"), tcc_gen::go_source());
    write(root.join("typescript/src/tcc_dfa.ts"), tcc_gen::typescript_source());
    if let Some(path) = java {
        write(path, tcc_gen::java_source());
    }
}
