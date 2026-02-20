use nalgebra::{Point3, Vector3};

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;
pub type CompiledFunction = dyn Fn(Point) -> f64 + Sync;
