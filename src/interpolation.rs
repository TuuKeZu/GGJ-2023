#[deprecated]
pub fn ease_old(x: f32) -> f32 {
    0.5 - (x.max(0.).min(1.) * std::f32::consts::PI).cos() / 2.
}

pub fn ease(mut x: f32) -> f32 {
    x = x.max(0.).min(1.);
    x.powi(2) * (x - 2.).powi(2)
}
