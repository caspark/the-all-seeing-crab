use rgb::{ComponentMap, RGB8};

use crate::vec3::Color;

pub(crate) fn color_as_rgb8(pixel_color: Color, samples_per_pixel: i32) -> RGB8 {
    let mut r = pixel_color.x;
    let mut g = pixel_color.y;
    let mut b = pixel_color.z;

    let scale = 1.0 / samples_per_pixel as f64;
    r *= scale;
    g *= scale;
    b *= scale;

    RGB8 {
        r: (256.0f64 * r.clamp(0.0, 0.999)) as u8,
        g: (256.0f64 * g.clamp(0.0, 0.999)) as u8,
        b: (256.0f64 * b.clamp(0.0, 0.999)) as u8,
    }
}

pub(crate) fn rgb8_as_terminal_char(col: RGB8) -> char {
    let uniform = col.map(|c| (c as f64 / 255.999) as f64);
    let char_index = ((uniform.r + uniform.g + uniform.b) / 3.0 * 16.0) as u32;

    std::char::from_digit(char_index, 16)
        .unwrap()
        .to_ascii_uppercase()
}
