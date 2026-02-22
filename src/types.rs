/// Scalar field value at a point in space.
pub type Value = f32;

/// A scalar field function: maps `(x, y, z)` coordinates to a [`Value`].
///
/// Return values **below or equal to** the chunk's threshold are considered "inside" the surface.
pub type CompiledFunction = dyn Fn(f32, f32, f32) -> Value + Sync;
