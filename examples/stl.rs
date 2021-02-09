extern crate three;

use std::env;
use three::Object;

fn main() {
    let mut args = env::args();
    let obj_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/sample.stl");
    let path = args.nth(1).unwrap_or(obj_path.into());

    match stlv::parser::load_file(path.as_str()) {
        Ok(model) => {
            for triangle in (*model).iter() {
                let vertices = triangle.vertices();
                println!("{:?}", vertices);
            }
        }
        _ => panic!("Failed to parse the STL file {}", path),
    }
}
