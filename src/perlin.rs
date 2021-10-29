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
        let i = ((4.0 * p.x) as i32 & 255) as usize;
        let j = ((4.0 * p.y) as i32 & 255) as usize;
        let k = ((4.0 * p.z) as i32 & 255) as usize;

        let index = (self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize;
        self.ran_float[index]
    }
}
