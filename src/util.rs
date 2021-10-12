pub(crate) fn random_int(min: i32, max: i32) -> i32 {
    min + (max - min) * rand::random::<i32>()
}

pub(crate) fn random_double(min: f64, max: f64) -> f64 {
    min + (max - min) * rand::random::<f64>()
}

pub(crate) fn random_double_unit() -> f64 {
    random_double(0.0, 1.0)
}

pub(crate) fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}
