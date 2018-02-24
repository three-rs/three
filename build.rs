extern crate includedir_codegen;

const SHADERS: &str = "data/shaders";

fn main() {
    println!("cargo:rerun-if-changed={}/*.glsl", SHADERS);
    includedir_codegen::start("FILES")
        .dir(SHADERS, includedir_codegen::Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
