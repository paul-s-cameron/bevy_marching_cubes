use bevy::prelude::*;

use crate::types::{CompiledFunction, Value};

/// A voxel grid that holds scalar field values and produces a marching cubes mesh.
///
/// The grid has `(size_x + 1) × (size_y + 1) × (size_z + 1)` corner points
/// and `size_x × size_y × size_z` voxels.
///
/// Values are stored as `values[z][y][x]`.
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
    /// Scalar field values, indexed `[z][y][x]`.
    pub values: Vec<Vec<Vec<Value>>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            size_x: 0,
            size_y: 0,
            size_z: 0,
            scale: 1.,
            threshold: 0.,
            values: vec![],
        }
    }
}

impl Chunk {
    /// Creates a new chunk with the given voxel dimensions.
    ///
    /// All values are initialised to `0.0`. The grid has `(size + 1)` corners
    /// per axis so that every voxel has a full set of 8 corners.
    pub fn new(size_x: usize, size_y: usize, size_z: usize) -> Self {
        let values = vec![vec![vec![0.; size_x + 1]; size_y + 1]; size_z + 1];
        Self {
            size_x,
            size_y,
            size_z,
            values,
            ..Default::default()
        }
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

    /// Calls `f(x, y, z, &mut value)` for every corner in the grid.
    ///
    /// Coordinates are integer voxel indices, not world-space positions.
    pub fn for_each_corner<F>(&mut self, mut f: F)
    where
        F: FnMut(f32, f32, f32, &mut Value),
    {
        for x in 0..=self.size_x {
            for y in 0..=self.size_y {
                for z in 0..=self.size_z {
                    f(x as f32, y as f32, z as f32, &mut self.values[z][y][x]);
                }
            }
        }
    }

    /// Like [`for_each_corner`](Chunk::for_each_corner), but adds `min_point` to each
    /// coordinate before passing it to `f`.
    ///
    /// Useful for filling a chunk that is offset in world space.
    pub fn for_each_corner_offset<F>(&mut self, min_point: Vec3, mut f: F)
    where
        F: FnMut(f32, f32, f32, &mut Value),
    {
        for x in 0..=self.size_x {
            for y in 0..=self.size_y {
                for z in 0..=self.size_z {
                    f(
                        min_point.x + x as f32,
                        min_point.y + y as f32,
                        min_point.z + z as f32,
                        &mut self.values[z][y][x],
                    );
                }
            }
        }
    }

    /// Returns the scalar field value at corner `(x, y, z)`.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Value {
        self.values[z][y][x]
    }

    /// Sets the scalar field value at corner `(x, y, z)`.
    pub fn set(&mut self, x: usize, y: usize, z: usize, v: Value) {
        self.values[z][y][x] = v
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
        (0..=self.size_x).for_each(|x| {
            (0..=self.size_y).for_each(|y| {
                (0..=self.size_z).for_each(|z| {
                    let xf = x as Value * self.scale;
                    let yf = y as Value * self.scale;
                    let zf = z as Value * self.scale;
                    self.set(x, y, z, function(xf, yf, zf))
                })
            })
        });
    }
}
