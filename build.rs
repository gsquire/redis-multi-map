extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .file("include/redismodule.c")
        .include("include/")
        .compile("libredismodule.a");

    let bindings = bindgen::Builder::default()
        .header("include/redismodule.h")
        .generate()
        .expect("error generating bindings");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("failed to write bindings to file");
}
