use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../data/words.fst");
    println!("cargo:rerun-if-changed=data/words.fst");

    if env::var_os("CARGO_FEATURE_EMBEDDED_DICT").is_none() {
        return;
    }

    // Repository checkout: ../data/words.fst. Published crate: data/words.fst,
    // staged by tools/sync_dict.sh before `cargo package`.
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = [manifest.join("../data/words.fst"), manifest.join("data/words.fst")]
        .into_iter()
        .find(|p| Path::new(p).is_file())
        .unwrap_or_else(|| {
            panic!(
                "words.fst not found (looked in ../data and data). Run tools/sync_dict.sh, \
                 or build with --no-default-features to exclude the embedded dictionary."
            )
        });

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("words.fst");
    fs::copy(&source, &out).expect("failed to copy words.fst into OUT_DIR");
}
