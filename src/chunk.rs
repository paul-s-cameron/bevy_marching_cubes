use std::sync::Arc;

use bevy::prelude::*;

use crate::types::{CompiledFunction, Value};

/// A voxel grid that holds scalar field values and produces a marching cubes mesh.
///
/// The grid has `(size_x + 1) × (size_y + 1) × (size_z + 1)` corner points
/// and `size_x × size_y × size_z` voxels.
///
/// Values are stored in a flat `Vec<f32>` with strides:
/// ```text
/// index(x, y, z) = z * stride_z + y * stride_y + x
/// stride_y = size_x + 1
/// stride_z = (size_y + 1) * stride_y
/// ```
///
/// `values` is wrapped in an [`Arc`] so the async mesh-generation task can hold a reference
/// to the grid without copying it.
#[derive(Component)]
#[require(Transform)]
pub struct Chunk {
    /// Number of voxels along X.
    pub size_x: usize,
    /// Number of voxels along Y.
    pub size_y: usize,
    /// Number of voxels along Z.
    pub size_z: usize,
    /// World-space size of each voxel edge.
    pub scale: Value,
    /// Iso-surface threshold — corners ≤ threshold are "inside".
    pub threshold: Value,
    /// Flat scalar field — index with `idx(x, y, z)`.
    pub values: Arc<Vec<Value>>,
    /// `size_x + 1`
    pub stride_y: usize,
    /// `(size_y + 1) * stride_y`
    pub stride_z: usize,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            size_x: 0,
            size_y: 0,
            size_z: 0,
            scale: 1.,
            threshold: 0.,
            values: Arc::new(vec![]),
            stride_y: 1,
            stride_z: 1,
        }
    }
}

impl Chunk {
    /// Creates a new chunk with the given voxel dimensions.
    ///
    /// All values are initialised to `0.0`. The grid has `(size + 1)` corners
    /// per axis so that every voxel has a full set of 8 corners.
    pub fn new(size_x: usize, size_y: usize, size_z: usize) -> Self {
        let stride_y = size_x + 1;
        let stride_z = (size_y + 1) * stride_y;
        let len = (size_z + 1) * stride_z;
        Self {
            size_x,
            size_y,
            size_z,
            values: Arc::new(vec![0.; len]),
            stride_y,
            stride_z,
            ..Default::default()
        }
    }

    /// Returns the flat index for corner `(x, y, z)`.
    #[inline]
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.stride_z + y * self.stride_y + x
    }

    /// Sets the world-space size of each voxel edge.
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the iso-surface threshold.
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Returns a mutable reference to the inner values vec.
    ///
    /// If the Arc is shared this will clone the data first (copy-on-write).
    fn values_mut(&mut self) -> &mut Vec<Value> {
        Arc::make_mut(&mut self.values)
    }

    /// Calls `f(x, y, z, &mut value)` for every corner in the grid.
    ///
    /// Coordinates are integer voxel indices, not world-space positions.
    pub fn for_each_corner<F>(&mut self, mut f: F)
    where
        F: FnMut(f32, f32, f32, &mut Value),
    {
        let (size_x, size_y, size_z) = (self.size_x, self.size_y, self.size_z);
        let (stride_y, stride_z) = (self.stride_y, self.stride_z);
        let values = self.values_mut();
        for z in 0..=size_z {
            for y in 0..=size_y {
                for x in 0..=size_x {
                    let i = z * stride_z + y * stride_y + x;
                    f(x as f32, y as f32, z as f32, &mut values[i]);
                }
            }
        }
    }

    /// Like [`for_each_corner`](Chunk::for_each_corner), but scales each index by
    /// [`scale`](Chunk::scale) and adds `min_point` before passing to `f`.
    ///
    /// Coordinates passed to `f` are true world-space positions, so the closure
    /// can sample a noise function or SDF directly without needing to know the scale.
    pub fn for_each_corner_offset<F>(&mut self, min_point: Vec3, mut f: F)
    where
        F: FnMut(f32, f32, f32, &mut Value),
    {
        let (size_x, size_y, size_z) = (self.size_x, self.size_y, self.size_z);
        let (stride_y, stride_z) = (self.stride_y, self.stride_z);
        let scale = self.scale;
        let values = self.values_mut();
        for z in 0..=size_z {
            for y in 0..=size_y {
                for x in 0..=size_x {
                    let i = z * stride_z + y * stride_y + x;
                    f(
                        min_point.x + x as f32 * scale,
                        min_point.y + y as f32 * scale,
                        min_point.z + z as f32 * scale,
                        &mut values[i],
                    );
                }
            }
        }
    }

    /// Returns the scalar field value at corner `(x, y, z)`.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Value {
        self.values[self.idx(x, y, z)]
    }

    /// Sets the scalar field value at corner `(x, y, z)`.
    pub fn set(&mut self, x: usize, y: usize, z: usize, v: Value) {
        let i = self.idx(x, y, z);
        self.values_mut()[i] = v;
    }

    /// Returns the 8 corner indices `[x, y, z]` of the voxel at `(x, y, z)`.
    ///
    /// Corners are ordered to match the standard marching cubes convention:
    ///
    /// ```text
    ///     6----7          Y
    ///    /|   /|          |
    ///   2----3 |          *-- X
    ///   | 4--|-5         /
    ///   |/   |/         Z
    ///   0----1
    ///
    ///  0 = (x,   y,   z  )    4 = (x,   y,   z+1)
    ///  1 = (x+1, y,   z  )    5 = (x+1, y,   z+1)
    ///  2 = (x+1, y+1, z  )    6 = (x+1, y+1, z+1)
    ///  3 = (x,   y+1, z  )    7 = (x,   y+1, z+1)
    /// ```
    #[inline]
    pub fn voxel_corner_indices(&self, x: usize, y: usize, z: usize) -> [[usize; 3]; 8] {
        [
            [x, y, z],
            [x + 1, y, z],
            [x + 1, y + 1, z],
            [x, y + 1, z],
            [x, y, z + 1],
            [x + 1, y, z + 1],
            [x + 1, y + 1, z + 1],
            [x, y + 1, z + 1],
        ]
    }

    /// Fills the chunk by evaluating `function` at every corner.
    ///
    /// Coordinates passed to `function` are scaled by [`scale`](Chunk::scale).
    pub fn fill(&mut self, function: &CompiledFunction) {
        let (size_x, size_y, size_z) = (self.size_x, self.size_y, self.size_z);
        let (stride_y, stride_z) = (self.stride_y, self.stride_z);
        let scale = self.scale;
        let values = self.values_mut();
        for z in 0..=size_z {
            for y in 0..=size_y {
                for x in 0..=size_x {
                    let i = z * stride_z + y * stride_y + x;
                    values[i] = function(x as Value * scale, y as Value * scale, z as Value * scale);
                }
            }
        }
    }
}
