use crate::{
    perlin::Perlin,
    vec3::{Color, Vec3},
};

use derive_more::Constructor;

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
#[derive(Debug, Constructor)]
pub(crate) struct CheckerTexture {
    scale: f64,
    odd: Box<dyn Texture>,
    even: Box<dyn Texture>,
}

impl CheckerTexture {
    pub(crate) fn from_colors(scale: f64, a: Color, b: Color) -> Self {
        Self {
            scale,
            odd: Box::new(ColorTexture::new(a)),
            even: Box::new(ColorTexture::new(b)),
        }
    }
}

impl Texture for CheckerTexture {
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
#[derive(Debug, Constructor)]
pub(crate) struct PositionTexture {}

impl Texture for PositionTexture {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Color {
        p
    }
}

/// A texture which is colored based on a provided noise source.
#[derive(Debug, Constructor)]
pub(crate) struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Color {
        Color::one() * self.noise.sample(self.scale * p)
    }
}
