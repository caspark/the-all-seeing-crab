extern crate derive_more;

mod vec3;

fn main() {
    let image_width = 256;
    let image_height = 256;

    println!(
        "P3\n{image_width} {image_height}\n255",
        image_width = image_width,
        image_height = image_height
    );

    for j in (0..(image_height - 1)).rev() {
        eprintln!("Scanlines remaining: {j}", j = j);
        for i in 0..image_width {
            let r = i as f64 / (image_width as f64 - 1f64);
            let g = j as f64 / (image_height as f64 - 1f64);
            let b = 0.25;

            let ir: i32 = (255.999 * r) as i32;
            let ig: i32 = (255.999 * g) as i32;
            let ib: i32 = (255.999 * b) as i32;

            println!("{ir} {ig} {ib}", ir = ir, ig = ig, ib = ib);
        }
    }
    eprintln!("Done.");
}
