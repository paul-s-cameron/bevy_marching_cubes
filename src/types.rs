use nalgebra::{Point3, Vector3};

/// Scalar field value at a point in space.
pub type Value = f32;

/// A 3D point with [`Value`] components.
pub type Point = Point3<Value>;

/// A 3D vector with [`Value`] components.
pub type Vector = Vector3<Value>;

/// A scalar field function: maps a [`Point`] to a [`Value`].
///
/// Return values **below or equal to** the chunk's threshold are considered "inside" the surface.
pub type CompiledFunction = dyn Fn(Point) -> Value + Sync;
