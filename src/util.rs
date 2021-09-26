pub(crate) fn random_double(min: f64, max: f64) -> f64 {
    min + (max - min) * rand::random::<f64>()
}

pub(crate) fn random_double_unit() -> f64 {
    random_double(0.0, 1.0)
}
