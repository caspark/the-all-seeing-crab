extern crate derive_more;

mod ray;
mod vec3;

use vec3::{Color, Point3, Vec3};

fn main() {
    let image_width = 256;
    let image_height = 256;

    println!(
        "P3\n{width} {height}\n255",
        width = image_width,
        height = image_height
    );

    for j in (0..(image_height - 1)).rev() {
        eprintln!("Scanlines remaining: {j}", j = j);
        for i in 0..image_width {
            let pixel: Color = Color::new(
                i as f64 / (image_width as f64 - 1f64),
                j as f64 / (image_height as f64 - 1f64),
                0.25,
            );
            print!("{}", pixel.as_color());
        }
    }
    eprintln!("Done.");
}
