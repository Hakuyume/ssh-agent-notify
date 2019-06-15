use std::env;
use std::path::PathBuf;

fn main() {
    bindgen::Builder::default()
        .header("wrapper.h")
        .whitelist_var("SSH2?_.+")
        .generate()
        .unwrap()
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap();
}
