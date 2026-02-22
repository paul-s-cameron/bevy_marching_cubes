use bevy::prelude::*;

/// The raw mesh data produced by the marching cubes algorithm for a [`Chunk`](crate::chunk::Chunk).
///
/// Inserted as a component on the chunk entity after generation completes, then removed
/// once the mesh has been uploaded to Bevy. Read it in a system ordered between
/// [`MarchingCubesSet::Generate`] and [`MarchingCubesSet::Upload`] to build physics
/// colliders or any other geometry without copying the data.
///
/// ```text
/// MarchingCubesSet::Generate  →  GeneratedMesh inserted
/// [your collider system]      →  read GeneratedMesh, build collider
/// MarchingCubesSet::Upload    →  Mesh3d inserted, GeneratedMesh removed
/// ```
#[derive(Component)]
pub struct GeneratedMesh {
    /// Flat list of vertex positions: `[[x, y, z], ...]`
    pub vertices: Vec<[f32; 3]>,

    /// Flat list of triangle indices in groups of 3: `[v0, v1, v2, v3, v4, v5, ...]`
    pub indices: Vec<u32>,

    /// Per-vertex face normals, one per vertex: `[[nx, ny, nz], ...]`
    pub normals: Vec<[f32; 3]>,
}

impl GeneratedMesh {
    /// Returns the three vertex positions of triangle `tri`.
    pub fn tri_coords(&self, tri: usize) -> [[f32; 3]; 3] {
        let base = tri * 3;
        [
            self.vertices[self.indices[base] as usize],
            self.vertices[self.indices[base + 1] as usize],
            self.vertices[self.indices[base + 2] as usize],
        ]
    }

    /// Computes the face normal for triangle `tri`.
    ///
    /// Returns the zero vector if the triangle is degenerate.
    pub fn tri_normal(&self, tri: usize) -> [f32; 3] {
        let [a, b, c] = self.tri_coords(tri);

        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];

        let cross = [
            ab[1] * bc[2] - ab[2] * bc[1],
            ab[2] * bc[0] - ab[0] * bc[2],
            ab[0] * bc[1] - ab[1] * bc[0],
        ];

        let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if len == 0.0 {
            [0.0, 0.0, 0.0]
        } else {
            [cross[0] / len, cross[1] / len, cross[2] / len]
        }
    }

    /// Returns the number of triangles.
    pub fn tri_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Builds a [`GeneratedMesh`] from a flat vertex buffer.
    ///
    /// Indices are generated sequentially (`0,1,2, 3,4,5, ...`) and face normals
    /// are computed per triangle and duplicated across each triangle's three vertices.
    pub(crate) fn build(vertices: Vec<[f32; 3]>) -> Self {
        let vert_count = vertices.len();
        let indices: Vec<u32> = (0..vert_count as u32).collect();
        let mut mesh = Self {
            vertices,
            indices,
            normals: Vec::with_capacity(vert_count),
        };
        mesh.compute_normals();
        mesh
    }

    /// Recomputes face normals for all triangles, replacing any existing normals.
    ///
    /// Each triangle's normal is pushed once per vertex (flat shading).
    /// TODO: Experiment with option for interpolated normals.
    pub fn compute_normals(&mut self) {
        self.normals.clear();
        for tri in 0..self.tri_count() {
            let n = self.tri_normal(tri);
            self.normals.push(n);
            self.normals.push(n);
            self.normals.push(n);
        }
    }
}
