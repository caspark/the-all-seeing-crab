use rgb::RGB8;

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
