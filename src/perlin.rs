use crate::{
    util,
    vec3::{Point3, Vec3},
};

#[derive(Clone, Debug)]
pub(crate) struct Perlin {
    ran_float: Vec<Vec3>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

const POINT_COUNT: usize = 256;

impl Perlin {
    pub(crate) fn new() -> Self {
        let mut ran_float = Vec::with_capacity(POINT_COUNT);
        for _ in 0..POINT_COUNT {
            ran_float.push(Vec3::random(-1.0, 1.0));
        }

        Self {
            ran_float,
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }

    fn perlin_generate_perm() -> Vec<i32> {
        let mut p = Vec::with_capacity(POINT_COUNT);
        for (i, _) in (0..POINT_COUNT).enumerate() {
            p.push(i as i32);
        }
        Perlin::permute(&mut p, POINT_COUNT);
        p
    }

    fn permute(p: &mut Vec<i32>, point_count: usize) {
        for i in (0..point_count).rev() {
            let target = util::random_int(0, i as i32) as usize;
            p.swap(i, target);
        }
    }

    pub(crate) fn sample_turbulence(&self, p: Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * self.sample_noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }
        accum.abs()
    }

    pub(crate) fn sample_noise(&self, p: Point3) -> f64 {
        let mut u = p.x - p.x.floor();
        let mut v = p.y - p.y.floor();
        let mut w = p.z - p.z.floor();

        // address Mach banding via hermitian smoothing
        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;
        let mut c = [[[Vec3::zero(); 2]; 2]; 2];

        for (di, ci) in c.iter_mut().enumerate() {
            for (dj, cij) in ci.iter_mut().enumerate() {
                for (dk, cijk) in cij.iter_mut().enumerate() {
                    let x = self.perm_x[((i + di as i32) & 255) as usize];
                    let y = self.perm_y[((j + dj as i32) & 255) as usize];
                    let z = self.perm_z[((k + dk as i32) & 255) as usize];

                    let idx = x ^ y ^ z;
                    *cijk = self.ran_float[idx as usize];
                }
            }
        }

        Perlin::trilinear_interpolate(c, u, v, w)
    }

    fn trilinear_interpolate(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for (i, ci) in c.iter().enumerate() {
            for (j, cij) in ci.iter().enumerate() {
                for (k, cijk) in cij.iter().enumerate() {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);

                    let uterm = i as f64 * u + (1.0 - i as f64) * (1.0 - u);
                    let vterm = j as f64 * v + (1.0 - j as f64) * (1.0 - v);
                    let wterm = k as f64 * w + (1.0 - k as f64) * (1.0 - w);
                    accum += uterm * vterm * wterm * cijk.dot(weight_v);
                }
            }
        }
        accum
    }
}
