use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    chunk::Chunk,
    mesh::GeneratedMesh,
    tables::{CORNER_POINT_INDICES, EDGE_TABLE},
    types::Value,
    utils::{get_corner_positions, get_edge_midpoints, get_state, triangle_verts_from_state},
};

/// System sets for the marching cubes pipeline.
///
/// Use these to order your own systems relative to mesh generation:
///
/// ```rust,ignore
/// // Run after geometry is ready but before it's uploaded — ideal for collider generation:
/// app.add_systems(Update, build_collider.after(MarchingCubesSet::Generate)
///                                       .before(MarchingCubesSet::Upload));
/// ```
///
/// ```text
/// MarchingCubesSet::Generate  →  [your systems]  →  MarchingCubesSet::Upload
/// ```
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarchingCubesSet {
    /// Runs marching cubes and inserts [`GeneratedMesh`] on each queued chunk.
    Generate,
    /// Uploads [`GeneratedMesh`] data into a Bevy [`Mesh3d`] and removes [`GeneratedMesh`].
    Upload,
}

/// Marker component added to [`Chunk`] entities that are waiting to be processed.
///
/// Removed automatically once the chunk's mesh has been generated and uploaded.
#[derive(Component)]
pub struct QueuedChunk;

/// Bevy plugin that drives marching cubes mesh generation.
///
/// When the `auto_queue` feature is enabled, any [`Chunk`] added to the world is
/// automatically processed within the same [`Update`] frame it is added:
///
/// ```text
/// Chunk added
///   → QueuedChunk inserted          (on_chunk_add)
///   → GeneratedMesh inserted        (MarchingCubesSet::Generate)
///   → [your collider systems here]
///   → Mesh3d inserted               (MarchingCubesSet::Upload)
///   → QueuedChunk + GeneratedMesh removed
/// ```
#[derive(Default)]
pub struct MarchingCubesPlugin;

impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "auto_queue")]
        app.configure_sets(
            Update,
            (MarchingCubesSet::Generate, MarchingCubesSet::Upload).chain(),
        )
        .add_systems(
            Update,
            (
                on_chunk_add,
                generate_mesh.in_set(MarchingCubesSet::Generate),
                upload_mesh.in_set(MarchingCubesSet::Upload),
            ),
        );
    }
}

/// Inserts [`QueuedChunk`] on every newly added [`Chunk`] that doesn't already have it.
fn on_chunk_add(
    mut commands: Commands,
    query: Query<Entity, (Added<Chunk>, Without<QueuedChunk>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(QueuedChunk);
    }
}

/// Runs marching cubes on all [`QueuedChunk`] entities and inserts a [`GeneratedMesh`].
///
/// Per-x-slice work is parallelised with Rayon. The pipeline per voxel is:
///
/// ```text
/// 1. get_corner_positions       →  8 world-space points
/// 2. chunk.get (×8)             →  8 scalar values
/// 3. get_state                  →  256-entry lookup key
/// 4. EDGE_TABLE[state]          →  bitmask of intersected edges
/// 5. get_edge_midpoints         →  up to 12 interpolated points
/// 6. triangle_verts_from_state  →  triangle vertices from TRI_TABLE
/// ```
fn generate_mesh(mut commands: Commands, query: Query<(Entity, &Chunk), With<QueuedChunk>>) {
    for (entity, chunk) in query.iter() {
        // --- Marching cubes (parallelised over X slices) ---
        let per_x: Vec<Vec<[f32; 3]>> = (0..chunk.size_x)
            .into_par_iter()
            .map(|x| {
                let mut local: Vec<[f32; 3]> = Vec::new();
                let per_voxel_max = 15_usize; // upper bound of vertices per voxel
                local.reserve(chunk.size_y * chunk.size_z * per_voxel_max);

                for y in 0..chunk.size_y {
                    for z in 0..chunk.size_z {
                        let corner_positions = get_corner_positions(x, y, z, chunk.scale);

                        let corner_indices = chunk.voxel_corner_indices(x, y, z);
                        let eval_corners: Vec<Value> = corner_indices
                            .iter()
                            .map(|[cx, cy, cz]| chunk.get(*cx, *cy, *cz))
                            .collect();

                        let state =
                            get_state(&eval_corners, chunk.threshold).expect("Could not get state");

                        let edges_mask = EDGE_TABLE[state] as u16;

                        let edge_points = get_edge_midpoints(
                            edges_mask,
                            &CORNER_POINT_INDICES,
                            &corner_positions,
                            &eval_corners,
                            chunk.threshold,
                        );

                        local.extend(triangle_verts_from_state(edge_points, state));
                    }
                }
                local
            })
            .collect();

        // --- Merge per-X slices into a single vertex buffer ---
        let total: usize = per_x.iter().map(|v| v.len()).sum();
        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(total);
        for mut v in per_x {
            vertices.append(&mut v);
        }

        commands
            .entity(entity)
            .insert(GeneratedMesh::build(vertices));
    }
}

/// Uploads a [`GeneratedMesh`] into a Bevy [`Mesh3d`], then removes [`GeneratedMesh`] and [`QueuedChunk`].
///
/// The three vertex data Vecs are **moved** directly into the Bevy mesh with no copies.
fn upload_mesh(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GeneratedMesh), With<QueuedChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut generated) in query.iter_mut() {
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        // Zero-copy move into Bevy
        let vertices = std::mem::take(&mut generated.vertices);
        let normals = std::mem::take(&mut generated.normals);
        let indices = std::mem::take(&mut generated.indices);

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.insert_indices(Indices::U32(indices));

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(bevy_mesh)))
            .remove::<GeneratedMesh>()
            .remove::<QueuedChunk>();
    }
}
