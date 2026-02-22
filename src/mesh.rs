use crate::{
    error::{MarchingCubesError, Result},
    types::{Point, Value, Vector},
};

/// Intermediate mesh representation produced by the marching cubes algorithm.
///
/// Vertices are stored flat — every group of three consecutive vertices forms one triangle.
/// Call [`create_triangles`](MarchMesh::create_triangles) then
/// [`create_normals`](MarchMesh::create_normals) after populating vertices.
#[derive(Clone)]
pub struct MarchMesh {
    /// Flat list of vertex positions: `[[x, y, z], ...]`
    pub vertices: Vec<Point>,

    /// Triangle index triples into `vertices`: `[[v0, v1, v2], ...]`
    pub tris: Vec<[usize; 3]>,

    /// Per-vertex face normals: `[[nx, ny, nz], ...]`
    pub normals: Vec<[Value; 3]>,
}

impl MarchMesh {
    /// Creates an empty mesh with no vertices, triangles, or normals.
    pub fn new_empty() -> Self {
        Self {
            vertices: Vec::new(),
            tris: Vec::new(),
            normals: Vec::new(),
        }
    }

    /// Adds a triangle defined by three vertex indices.
    ///
    /// Returns [`MarchingCubesError::InvalidIndex`] if any index is out of bounds.
    pub fn triangle_from_verts(&mut self, x: usize, y: usize, z: usize) -> Result<()> {
        if self.vertices.len() <= x.max(y.max(z)) {
            return Err(MarchingCubesError::InvalidIndex);
        }
        self.tris.push([x, y, z]);
        Ok(())
    }

    /// Returns the three vertex positions of triangle `tri`.
    pub fn tri_coords(&self, tri: usize) -> Vec<Point> {
        let va = self.vertices[self.tris[tri][0]];
        let vb = self.vertices[self.tris[tri][1]];
        let vc = self.vertices[self.tris[tri][2]];
        vec![va, vb, vc]
    }

    /// Computes the face normal for triangle `tri`.
    ///
    /// Returns the zero vector if the triangle is degenerate.
    pub fn tri_normal(&self, tri: usize) -> Vector {
        let va = self.vertices[self.tris[tri][0]];
        let vb = self.vertices[self.tris[tri][1]];
        let vc = self.vertices[self.tris[tri][2]];

        let _a = Vector::new(va[0], va[1], va[2]);
        let _b = Vector::new(vb[0], vb[1], vb[2]);
        let _c = Vector::new(vc[0], vc[1], vc[2]);

        let v_a_b = _b - _a;
        let v_b_c = _c - _b;

        let cross = v_a_b.cross(&v_b_c);

        let nrm = cross.norm();
        if nrm == 0.0 {
            Vector::new(0.0, 0.0, 0.0)
        } else {
            cross / nrm
        }
    }

    /// Generates triangles by grouping every three consecutive vertices.
    ///
    /// Must be called after [`set_vertices`](MarchMesh::set_vertices).
    /// Vertex count must be a multiple of 3.
    pub fn create_triangles(&mut self) -> () {
        let mut v = 0;
        while v < self.vertices.len() {
            self.triangle_from_verts(v, v + 1, v + 2)
                .expect("Could not create triangle.");
            v += 3
        }
    }

    /// Computes and stores face normals, one per vertex (three per triangle).
    ///
    /// Replaces any previously stored normals.
    /// Must be called after [`create_triangles`](MarchMesh::create_triangles).
    pub fn create_normals(&mut self) -> () {
        self.normals.clear();
        for tri in 0..self.tris.len() {
            let normal = self.tri_normal(tri);
            let n = [normal.x, normal.y, normal.z];
            // Push the face normal once per vertex of the triangle.
            // TODO: Experiment with option for interpolated normals.
            self.normals.push(n);
            self.normals.push(n);
            self.normals.push(n);
        }
    }

    /// Replaces the vertex buffer.
    pub fn set_vertices(&mut self, vertices: Vec<Point>) -> () {
        self.vertices = vertices
    }
}
