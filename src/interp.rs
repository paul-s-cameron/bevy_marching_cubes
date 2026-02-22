use crate::types::{Point, Value};

/// Returns the interpolation factor `t` at which the linear function passing
/// through `v0` and `v1` equals `iso_val`.
///
/// ```text
/// v0 ---[t]--- v1
///        ^iso_val
/// ```
pub fn find_t(v0: Value, v1: Value, iso_val: Value) -> Value {
    (iso_val - v0) / (v1 - v0)
}

/// Linear interpolation between `a` and `b` by factor `t ∈ [0, 1]`.
pub fn lerp(a: Value, b: Value, t: Value) -> Value {
    a + (b - a) * t
}

/// Interpolates component-wise between two points by factor `t`.
///
/// Each coordinate is lerped independently:
/// ```text
/// p0 --[t]--> p1   →   (lerp(p0.x, p1.x, t), lerp(p0.y, p1.y, t), lerp(p0.z, p1.z, t))
/// ```
pub fn interpolate_points(p0: Point, p1: Point, t: Value) -> Vec<Value> {
    // TODO: may need to make this an array not a vector
    let pf: Vec<Value> = p0
        .iter()
        .zip(p1.iter())
        .map(|p| lerp(*p.0, *p.1, t))
        .collect();
    pf
}
