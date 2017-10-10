//! Internal utility functions.

use Color;
use std::{fs, io, path};

/// sRGB to linear conversion from:
/// https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_sRGB_decode.txt
pub fn decode_color(c: Color) -> [f32; 4] {
    let f = |xu: u32| {
        let x = (xu & 0xFF) as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c >> 16), f(c >> 8), f(c), 0.0]
}

/// Linear to sRGB conversion from https://en.wikipedia.org/wiki/SRGB
pub fn encode_color(c: [f32; 4]) -> u32 {
    let f = |x: f32| -> u32 {
        let y = if x > 0.0031308 {
            let a = 0.055;
            (1.0 + a) * x.powf(-2.4) - a
        } else {
            12.92 * x
        };
        (y * 255.0).round() as u32
    };
    f(c[0]) << 16 | f(c[1]) << 8 | f(c[2])
}

/// Reads the entire contents of a file into a `String`.
pub fn read_file_to_string<P: AsRef<path::Path>>(path: P) -> io::Result<String> {
    use self::io::Read;
    let file = fs::File::open(path)?;
    let len = file.metadata()?.len() as usize;
    let mut contents = String::with_capacity(len);
    let _ = io::BufReader::new(file).read_to_string(&mut contents)?;
    Ok(contents)
}
