use crate::{
    perlin::Perlin,
    vec3::{Color, Vec3},
};

use derive_more::Constructor;
use rgb::RGB8;

pub(crate) trait Texture: std::fmt::Debug + Sync + Send {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Color;
}

/// A texture that is a single solid color.
#[derive(Clone, Copy, Debug, Constructor)]
pub(crate) struct ColorTexture {
    pub color_value: Color,
}

impl ColorTexture {
    pub(crate) fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            color_value: Color::new(r, g, b),
        }
    }
}

impl Texture for ColorTexture {
    fn value(&self, _u: f64, _v: f64, _p: Vec3) -> Color {
        self.color_value
    }
}

/// A texture that is checkered; looks cool, but also useful for partially overlaying a debug texture.
#[derive(Debug, Clone, Constructor)]
pub(crate) struct CheckerTexture<O: Texture, E: Texture> {
    scale: f64,
    odd: Box<O>,
    even: Box<E>,
}

impl CheckerTexture<ColorTexture, ColorTexture> {
    pub(crate) fn from_colors(scale: f64, a: Color, b: Color) -> Self {
        Self {
            scale,
            odd: Box::new(ColorTexture::new(a)),
            even: Box::new(ColorTexture::new(b)),
        }
    }
}

impl<O: Texture, E: Texture> Texture for CheckerTexture<O, E> {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Color {
        let sines = (self.scale * p.x).sin() * (self.scale * p.y).sin() * (self.scale * p.z).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

/// A texture that is colored based on the position that it is struck by a ray (in world coordinates).
#[derive(Debug, Constructor, Clone, Copy)]
pub(crate) struct PositionTexture {}

impl Texture for PositionTexture {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Color {
        p
    }
}

/// A texture which is colored based on a provided noise source.
#[derive(Debug, Constructor, Clone)]
pub(crate) struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Color {
        Color::one() * 0.5 * (1.0 + self.noise.sample_noise(self.scale * p))
    }
}

/// A texture which is colored based on a provided noise source providing turbulence (multiple
/// layers of noise).
#[derive(Debug, Constructor, Clone)]
pub(crate) struct TurbulenceTexture {
    noise: Perlin,
    scale: f64,
    depth: i32,
}

impl Texture for TurbulenceTexture {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Color {
        Color::one() * self.noise.sample_turbulence(self.scale * p, self.depth)
    }
}

/// A procedural marble texture based on turbulated noise.
#[derive(Debug, Constructor, Clone)]
pub(crate) struct MarbleTexture {
    noise: Perlin,
    scale: f64,
    depth: i32,
}

impl Texture for MarbleTexture {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Color {
        Color::one()
            * 0.5
            * (1.0
                + (self.scale * p.z
                    + 5.0 * self.noise.sample_turbulence(self.scale * p, self.depth))
                .sin())
    }
}

/// A texture based on an image.
#[derive(Debug, Constructor, Clone)]
pub(crate) struct ImageTexture {
    data: Vec<RGB8>,
    width: usize,
    height: usize,
}

impl ImageTexture {
    pub(crate) fn load_from_png<S: AsRef<str>>(path: S) -> lodepng::Result<Self> {
        lodepng::decode24_file(path.as_ref()).map(|bitmap| ImageTexture {
            data: bitmap.buffer,
            width: bitmap.width,
            height: bitmap.height,
        })
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Vec3) -> Color {
        // Clamp input texture coordinates to [0,1] x [1,0]
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // flip V to image coordinates

        let mut i = (u * self.width as f64) as usize;
        let mut j = (v * self.height as f64) as usize;

        // Clamp integer mapping, since actual coordinates should be less than 1.0
        if i >= self.width {
            i = self.width - 1;
        };
        if j >= self.height {
            j = self.height - 1;
        }

        let color_scale = 1.0 / 255.0;

        let idx = j * self.width + i;
        Color::new(
            self.data[idx].r as f64 * color_scale,
            self.data[idx].g as f64 * color_scale,
            self.data[idx].b as f64 * color_scale,
        )
    }
}
