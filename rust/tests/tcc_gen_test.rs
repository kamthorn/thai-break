//! The generated TCC tables of the other ports must match the rules in `src/tcc.rs`. Run
//! `cargo run --bin thaibreak_gen_tcc` after changing the rules.

use std::path::Path;
use thaibreak::tcc_gen;

fn check(relative: &str, generated: String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(relative);
    // Absent outside the repository (the published crate has no Go or TypeScript sources)
    let Ok(committed) = std::fs::read_to_string(&path) else {
        return;
    };
    assert!(committed == generated, "{relative} is out of date: run `cargo run --bin thaibreak_gen_tcc`");
}

#[test]
fn go_tables_are_up_to_date() {
    check("go/tcc_dfa.go", tcc_gen::go_source());
}

#[test]
fn typescript_tables_are_up_to_date() {
    check("typescript/src/tcc_dfa.ts", tcc_gen::typescript_source());
}
