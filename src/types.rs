use nalgebra::{Point3, Vector3};

pub type Value = f32;
pub type Point = Point3<Value>;
pub type Vector = Vector3<Value>;
pub type CompiledFunction = dyn Fn(Point) -> Value + Sync;
