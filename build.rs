extern crate includedir_codegen;

use includedir_codegen::Compression;

fn main() {
    includedir_codegen::start("SHADERS")
        .dir("data/shaders", Compression::None)
        .build("data.rs")
        .unwrap();
}
