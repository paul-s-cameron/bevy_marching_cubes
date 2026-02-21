use bevy::prelude::*;
use nalgebra::point;

use crate::{
    mesh::MarchMesh,
    types::{CompiledFunction, Point, Value},
};

#[derive(Component)]
#[require(Transform)]
pub struct Chunk {
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub scale: Value,
    pub threshold: Value,
    pub values: Vec<Vec<Vec<Value>>>,
    pub grid_points: Vec<Vec<Vec<Point>>>,
    pub mesh: Option<MarchMesh>,
}

impl Chunk {
    pub fn new(size_x: usize, size_y: usize, size_z: usize) -> Self {
        let scale = 1.;
        let values = vec![vec![vec![0.; size_x + 1]; size_y + 1]; size_z + 1];
        let grid_points = vec![vec![vec![point![0., 0., 0.]; size_x + 1]; size_y + 1]; size_z + 1];
        Self {
            size_x,
            size_y,
            size_z,
            scale,
            threshold: 0.,
            values,
            grid_points,
            mesh: None,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

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

    pub fn get(&self, x: usize, y: usize, z: usize) -> Value {
        self.values[z][y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, v: Value) {
        self.values[z][y][x] = v
    }

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

    pub fn fill(&mut self, function: &CompiledFunction) {
        (0..=self.size_x).for_each(|x| {
            (0..=self.size_y).for_each(|y| {
                (0..=self.size_z).for_each(|z| {
                    let xf = x as Value * self.scale;
                    let yf = y as Value * self.scale;
                    let zf = z as Value * self.scale;
                    let p = point![xf, yf, zf];
                    self.set(x as usize, y as usize, z as usize, function(p))
                })
            })
        });
    }
}
