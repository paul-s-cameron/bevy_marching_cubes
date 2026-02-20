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
    pub min_point: Point,
    pub scale: Value,
    pub values: Vec<Vec<Vec<Value>>>,
    pub grid_points: Vec<Vec<Vec<Point>>>,
    pub mesh: Option<MarchMesh>,
}

impl Chunk {
    pub fn new(size_x: usize, size_y: usize, size_z: usize) -> Self {
        let scale = 1.;
        let min_point = point![0., 0., 0.];
        let values = vec![vec![vec![0.; size_x]; size_y]; size_z];
        let grid_points = vec![vec![vec![point![0., 0., 0.]; size_x]; size_y]; size_z];
        Self {
            size_x,
            size_y,
            size_z,
            min_point,
            scale,
            values,
            grid_points,
            mesh: None,
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Value {
        self.values[z][y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, v: Value) {
        self.values[z][y][x] = v
    }

    pub fn voxel_corner_indices(&self, x: usize, y: usize, z: usize) -> [[usize; 3]; 8] {
        // TODO: could be consolidated/more idiomatic
        let c0 = [x, y, z];
        let c1 = [x + 1, y, z];
        let c2 = [x + 1, y + 1, z];
        let c3 = [x, y + 1, z];
        let c4 = [x, y, z + 1];
        let c5 = [x + 1, y, z + 1];
        let c6 = [x + 1, y + 1, z + 1];
        let c7 = [x, y + 1, z + 1];
        return [c0, c1, c2, c3, c4, c5, c6, c7];
    }

    pub fn fill(&mut self, function: &CompiledFunction) {
        (0..self.size_x).for_each(|x| {
            (0..self.size_y).for_each(|y| {
                (0..self.size_z).for_each(|z| {
                    let xf = x as Value * self.scale;
                    let yf = y as Value * self.scale;
                    let zf = z as Value * self.scale;
                    let p = point![
                        xf + self.min_point.x,
                        yf + self.min_point.y,
                        zf + self.min_point.z
                    ];
                    self.set(x as usize, y as usize, z as usize, function(p))
                })
            })
        });
    }
}
