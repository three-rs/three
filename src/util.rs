//! Internal utility functions.

use std::{fs, io, path};
use std::hash::{Hash, Hasher};

/// Reads the entire contents of a file into a `String`.
pub fn read_file_to_string<P: AsRef<path::Path>>(path: P) -> io::Result<String> {
    use self::io::Read;
    let file = fs::File::open(path)?;
    let len = file.metadata()?.len() as usize;
    let mut contents = String::with_capacity(len);
    let _ = io::BufReader::new(file).read_to_string(&mut contents)?;
    Ok(contents)
}

/// Hash f32 value using its bit interpretation.
pub fn hash_f32<H: Hasher>(value: &f32, state: &mut H) {
    value.to_bits().hash(state);
}

/// Hash slice of floats using its bit interpretation.
pub fn hash_f32_slice<H: Hasher>(value: &[f32], state: &mut H) {
    for element in value {
        element.to_bits().hash(state);
    }
}
