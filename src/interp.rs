use crate::types::{Point, Value};

// linearly map a number from one range to another
pub fn remap(s: Value, range_in: [Value; 2], range_out: [Value; 2]) -> Value {
    range_out[0] + (s - range_in[0]) * (range_out[1] - range_out[0]) / (range_in[1] - range_in[0])
}

// Return the interpolation factor t corresponding to iso_val
pub fn find_t(v0: Value, v1: Value, iso_val: Value) -> Value {
    (iso_val - v0) / (v1 - v0)
}

// Linear interpolation
pub fn lerp(a: Value, b: Value, t: Value) -> Value {
    a + (b - a) * t
}

// Linearly interpolate between two points by factor t
pub fn interpolate_points(p0: Point, p1: Point, t: Value) -> Vec<Value> {
    // TODO: may need to make this an array not a vector
    let pf: Vec<Value> = p0
        .iter()
        .zip(p1.iter())
        .map(|p| lerp(*p.0, *p.1, t))
        .collect();
    pf
}
