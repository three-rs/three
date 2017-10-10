extern crate includedir_codegen;

fn main() {
    includedir_codegen::start("FILES")
        .dir("data/shaders", includedir_codegen::Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
