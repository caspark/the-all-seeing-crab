use crate::{util, vec3::Point3};

#[derive(Clone, Debug)]
pub(crate) struct Perlin {
    ran_float: Vec<f64>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

const POINT_COUNT: usize = 256;

impl Perlin {
    pub(crate) fn new() -> Self {
        let mut ran_float = Vec::with_capacity(POINT_COUNT);
        for _ in 0..POINT_COUNT {
            ran_float.push(rand::random::<f64>());
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

    pub(crate) fn sample(&self, p: Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;
        let mut c = [[[0f64; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let x = self.perm_x[((i + di as i32) & 255) as usize];
                    let y = self.perm_y[((j + dj as i32) & 255) as usize];
                    let z = self.perm_z[((k + dk as i32) & 255) as usize];

                    let idx = x ^ y ^ z;
                    c[di][dj][dk] = self.ran_float[idx as usize];
                }
            }
        }

        return Perlin::trilinear_interpolate(c, u, v, w);
    }

    fn trilinear_interpolate(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let uterm = i as f64 * u + (1.0 - i as f64) * (1.0 - u);
                    let vterm = j as f64 * v + (1.0 - j as f64) * (1.0 - v);
                    let wterm = k as f64 * w + (1.0 - k as f64) * (1.0 - w);
                    accum += uterm * vterm * wterm * c[i][j][k];
                }
            }
        }
        accum
    }
}
