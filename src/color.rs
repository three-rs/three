//! sRGB colors.

/// sRGB color represented by a 4-byte hexadecimal number.
///
/// ```rust
/// # #![allow(unused)]
/// let red = 0xFF0000;
/// let green = 0x00FF00;
/// let blue = 0x0000FF;
/// ```
pub type Color = u32;

/// Black.
pub const BLACK: Color = 0x000000;

/// Red.
pub const RED: Color = 0xFF0000;

/// Green.
pub const GREEN: Color = 0x00FF00;

/// Blue.
pub const BLUE: Color = 0x0000FF;

/// Yellow.
pub const YELLOW: Color = RED | GREEN;

/// Cyan.
pub const CYAN: Color = GREEN | BLUE;

/// Magenta.
pub const MAGENTA: Color = RED | BLUE;

/// White.
pub const WHITE: Color = RED | BLUE | GREEN;

/// sRGB to linear conversion.
///
/// Implementation taken from https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_sRGB_decode.txt
pub fn to_linear_rgb(c: Color) -> [f32; 3] {
    let f = |xu: u32| {
        let x = (xu & 0xFF) as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c >> 16), f(c >> 8), f(c)]
}

/// Linear to sRGB conversion.
///
/// Implementation taken from https://en.wikipedia.org/wiki/SRGB
pub fn from_linear_rgb(c: [f32; 3]) -> Color {
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
