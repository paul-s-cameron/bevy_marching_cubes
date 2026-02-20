use crate::types::Point;

// linearly map a number from one range to another
pub fn remap(s: f64, range_in: [f64; 2], range_out: [f64; 2]) -> f64 {
    range_out[0] + (s - range_in[0]) * (range_out[1] - range_out[0]) / (range_in[1] - range_in[0])
}

// Return the interpolation factor t corresponding to iso_val
pub fn find_t(v0: f64, v1: f64, iso_val: f64) -> f64 {
    (iso_val - v0) / (v1 - v0)
}

// Linear interpolation
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

// Linearly interpolate between two points by factor t
pub fn interpolate_points(p0: Point, p1: Point, t: f64) -> Vec<f64> {
    // TODO: may need to make this an array not a vector
    let pf: Vec<f64> = p0
        .iter()
        .zip(p1.iter())
        .map(|p| lerp(*p.0, *p.1, t))
        .collect();
    pf
}
