use std::path::{Path, PathBuf};
use std::{env, fs};

const DICT_FILES: [&str; 2] = ["words.fst", "word-weights.tsv"];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for name in DICT_FILES {
        println!("cargo:rerun-if-changed=../data/{name}");
        println!("cargo:rerun-if-changed=data/{name}");
    }

    if env::var_os("CARGO_FEATURE_EMBEDDED_DICT").is_none() {
        return;
    }

    // Repository checkout: ../data. Published crate: data/, staged by
    // tools/sync_dict.sh before `cargo package`.
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for name in DICT_FILES {
        let source = [manifest.join("../data").join(name), manifest.join("data").join(name)]
            .into_iter()
            .find(|p| Path::new(p).is_file())
            .unwrap_or_else(|| {
                panic!(
                    "{name} not found (looked in ../data and data). Run tools/sync_dict.sh, \
                     or build with --no-default-features to exclude the embedded dictionary."
                )
            });
        fs::copy(&source, out_dir.join(name)).unwrap_or_else(|e| panic!("failed to copy {name} into OUT_DIR: {e}"));
    }
}
