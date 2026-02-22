use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    chunk::Chunk,
    tables::{CORNER_POINT_INDICES, EDGE_TABLE},
    types::{Point, Value},
    utils::{get_corner_positions, get_edge_midpoints, get_state, triangle_verts_from_state},
};

/// System set containing the marching cubes processing systems.
///
/// Use this to order your systems relative to mesh generation:
/// ```rust,ignore
/// app.add_systems(Update, my_system.before(MarchingCubesSet));
/// ```
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarchingCubesSet;

/// Marker component added to [`Chunk`] entities that are waiting to be processed.
///
/// Removed automatically once the chunk's mesh has been generated.
#[derive(Component)]
pub struct QueuedChunk;

/// Bevy plugin that drives marching cubes mesh generation.
///
/// When the `auto_queue` feature is enabled, any [`Chunk`] added to the world
/// is automatically processed on the next [`Update`] frame:
///
/// ```text
/// Chunk added  →  QueuedChunk inserted  →  mesh generated  →  QueuedChunk removed
/// ```
///
/// Without `auto_queue`, call [`process_chunk`] manually or manage [`QueuedChunk`] yourself.
#[derive(Default)]
pub struct MarchingCubesPlugin;

impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "auto_queue")]
        app.add_systems(
            Update,
            (on_chunk_add, process_chunk)
                .chain()
                .in_set(MarchingCubesSet),
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

/// Processes all [`QueuedChunk`] entities: runs marching cubes and uploads the result to Bevy.
fn process_chunk(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Chunk), With<QueuedChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut chunk) in query.iter_mut() {
        // --- Marching cubes (parallelised over X slices) ---
        let per_x: Vec<Vec<Point>> = (0..chunk.size_x)
            .into_par_iter()
            .map(|x| {
                let mut local: Vec<Point> = Vec::new();
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

                        let new_verts = triangle_verts_from_state(edge_points, state);
                        local.extend(new_verts);
                    }
                }
                local
            })
            .collect();

        // --- Merge per-X slices into a single vertex buffer ---
        let total: usize = per_x.iter().map(|v| v.len()).sum();
        let mut vertices: Vec<Point> = Vec::with_capacity(total);
        for mut v in per_x {
            vertices.append(&mut v);
        }

        chunk.mesh.set_vertices(vertices);
        chunk.mesh.create_triangles();
        chunk.mesh.create_normals();

        // --- Upload to Bevy ---
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        let positions: Vec<[f32; 3]> = chunk
            .mesh
            .vertices
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32])
            .collect();

        let indices: Vec<u32> = chunk
            .mesh
            .tris
            .iter()
            .flat_map(|tri| vec![tri[0] as u32, tri[1] as u32, tri[2] as u32])
            .collect();

        let normals: Vec<[f32; 3]> = chunk
            .mesh
            .normals
            .iter()
            .map(|n| [n[0] as f32, n[1] as f32, n[2] as f32])
            .collect();

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.insert_indices(Indices::U32(indices));

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(bevy_mesh)))
            .remove::<QueuedChunk>();
    }
}
