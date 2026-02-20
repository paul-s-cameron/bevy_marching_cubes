use crate::{
    error::{MarchingCubesError, Result},
    types::{Point, Vector},
};

#[derive(Clone)]
pub struct MarchMesh {
    // [[x1, y1, z1], [x2, y2, z2], ...]
    pub vertices: Vec<Point>,

    // [[v0, v1, v2], [v3, v4, v5], ...]
    pub tris: Vec<[usize; 3]>, // new triangles
}

impl MarchMesh {
    //create a new empty Mesh
    pub fn new_empty() -> Self {
        Self {
            vertices: Vec::new(),
            tris: Vec::new(),
        }
    }

    //create a triangle from Point indices
    pub fn triangle_from_verts(&mut self, x: usize, y: usize, z: usize) -> Result<()> {
        // Need to make sure mesh isn't empty
        if self.vertices.len() <= x.max(y.max(z)) {
            return Err(MarchingCubesError::EmptyMesh);
        }

        // x/y/z are indices that form a triangle
        self.tris.push([x, y, z]);

        Ok(())
    }

    //return triangle Point coordinates
    pub fn tri_coords(&self, tri: usize) -> Vec<Point> {
        let va = self.vertices[self.tris[tri][0]];
        let vb = self.vertices[self.tris[tri][1]];
        let vc = self.vertices[self.tris[tri][2]];

        vec![va, vb, vc]
    }

    //return triangle normal
    pub fn tri_normal(&self, tri: usize) -> Vector {
        //tri = starting Point index

        let va = self.vertices[self.tris[tri][0]];
        let vb = self.vertices[self.tris[tri][1]];
        let vc = self.vertices[self.tris[tri][2]];

        let _a = Vector::new(va[0], va[1], va[2]);
        let _b = Vector::new(vb[0], vb[1], vb[2]);
        let _c = Vector::new(vc[0], vc[1], vc[2]);

        let v_a_b = _b - _a;
        let v_b_c = _c - _b;

        let cross = v_a_b.cross(&v_b_c);

        cross / cross.norm() //normal vector
    }

    pub fn create_triangles(&mut self) -> () {
        let mut v = 0;
        while v < self.vertices.len() {
            self.triangle_from_verts(v, v + 1, v + 2)
                .expect("Could not create triangle.");
            v += 3
        }
    }

    pub fn set_vertices(&mut self, vertices: Vec<Point>) -> () {
        self.vertices = vertices
    }
}
