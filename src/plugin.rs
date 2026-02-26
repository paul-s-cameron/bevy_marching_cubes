use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
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
/// MarchingCubesSet::Spawn   →  [async compute]  →  MarchingCubesSet::Generate  →  [your systems]  →  MarchingCubesSet::Upload
/// ```
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarchingCubesSet {
    /// Spawns an async compute task for each queued chunk.
    Spawn,
    /// Polls async tasks and inserts [`GeneratedMesh`] on completion.
    Generate,
    /// Uploads [`GeneratedMesh`] data into a Bevy [`Mesh3d`] and removes [`GeneratedMesh`].
    Upload,
}

/// Marker component added to [`Chunk`] entities that are waiting to be processed.
///
/// Removed automatically once the chunk's mesh has been generated and uploaded.
#[derive(Component)]
pub struct QueuedChunk;

/// Holds the in-flight async compute task for a [`Chunk`].
///
/// Inserted by [`MarchingCubesSet::Spawn`], removed once the task completes
/// and [`GeneratedMesh`] has been inserted by [`MarchingCubesSet::Generate`].
#[derive(Component)]
pub struct ComputeTask(Task<GeneratedMesh>);

/// Runtime configuration for the marching cubes pipeline.
///
/// Inserted as a resource by [`MarchingCubesPlugin`]. Modify it at any time to change behaviour:
///
/// ```rust,ignore
/// app.add_plugins(MarchingCubesPlugin { max_tasks_per_frame: 8, ..default() });
///
/// // Or change it at runtime:
/// fn my_system(mut config: ResMut<MarchingCubesConfig>) {
///     config.max_tasks_per_frame = 1; // throttle while the player is in combat
/// }
/// ```
#[derive(Resource)]
pub struct MarchingCubesConfig {
    /// Maximum number of async mesh tasks spawned per frame.
    ///
    /// Higher values load chunks faster but may cause frame hitches when many chunks
    /// are queued at once. Default: `4`.
    pub max_tasks_per_frame: usize,
}

impl Default for MarchingCubesConfig {
    fn default() -> Self {
        Self {
            max_tasks_per_frame: 4,
        }
    }
}

/// Bevy plugin that drives marching cubes mesh generation.
///
/// When the `auto_queue` feature is enabled, any [`Chunk`] added to the world is
/// automatically processed. Mesh generation runs on Bevy's `AsyncComputeTaskPool`
/// so the main thread is never blocked:
///
/// ```text
/// Chunk added
///   → QueuedChunk inserted          (on_chunk_add)
///   → ComputeTask spawned           (MarchingCubesSet::Spawn)
///   → [async compute runs]
///   → GeneratedMesh inserted        (MarchingCubesSet::Generate, once task completes)
///   → [your collider systems here]
///   → Mesh3d inserted               (MarchingCubesSet::Upload)
///   → QueuedChunk + GeneratedMesh removed
/// ```
pub struct MarchingCubesPlugin {
    /// Initial value for [`MarchingCubesConfig::max_tasks_per_frame`].
    pub max_tasks_per_frame: usize,
}

impl Default for MarchingCubesPlugin {
    fn default() -> Self {
        Self {
            max_tasks_per_frame: MarchingCubesConfig::default().max_tasks_per_frame,
        }
    }
}

impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MarchingCubesConfig {
            max_tasks_per_frame: self.max_tasks_per_frame,
        });

        #[cfg(feature = "auto_queue")]
        app.configure_sets(
            Update,
            (
                MarchingCubesSet::Spawn,
                MarchingCubesSet::Generate,
                MarchingCubesSet::Upload,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                on_chunk_add,
                spawn_mesh_tasks.in_set(MarchingCubesSet::Spawn),
                poll_mesh_tasks.in_set(MarchingCubesSet::Generate),
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

/// Spawns async compute tasks for [`QueuedChunk`]s, up to [`MarchingCubesConfig::max_tasks_per_frame`] per frame.
fn spawn_mesh_tasks(
    mut commands: Commands,
    config: Res<MarchingCubesConfig>,
    query: Query<(Entity, &Chunk), (With<QueuedChunk>, Without<ComputeTask>, Without<Mesh3d>)>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, chunk) in query.iter().take(config.max_tasks_per_frame) {
        // Arc::clone is a single pointer bump — no heap allocation on the main thread.
        let size_x = chunk.size_x;
        let size_y = chunk.size_y;
        let size_z = chunk.size_z;
        let scale = chunk.scale;
        let threshold = chunk.threshold;
        let values: Arc<Vec<Vec<Vec<Value>>>> = Arc::clone(&chunk.values);

        let task = task_pool.spawn(async move {
            run_marching_cubes(size_x, size_y, size_z, scale, threshold, &values)
        });

        commands.entity(entity).insert(ComputeTask(task));
    }
}

/// Polls in-flight [`ComputeTask`]s each frame and inserts [`GeneratedMesh`] on completion.
///
/// Non-blocking: tasks that haven't finished are skipped and retried next frame.
fn poll_mesh_tasks(mut commands: Commands, mut query: Query<(Entity, &mut ComputeTask)>) {
    for (entity, mut compute_task) in query.iter_mut() {
        if let Some(generated_mesh) = block_on(future::poll_once(&mut compute_task.0)) {
            commands
                .entity(entity)
                .insert(generated_mesh)
                .remove::<ComputeTask>();
        }
    }
}

/// Uploads a [`GeneratedMesh`] into a Bevy [`Mesh3d`], then removes [`GeneratedMesh`] and [`QueuedChunk`].
///
/// The three vertex data Vecs are **moved** directly into the Bevy mesh with no copies.
fn upload_mesh(
    mut commands: Commands,
    query: Query<(Entity, &GeneratedMesh), With<QueuedChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, generated) in query.iter() {
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, generated.vertices.clone());
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, generated.normals.clone());
        bevy_mesh.insert_indices(Indices::U32(generated.indices.clone()));

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(bevy_mesh)))
            .remove::<QueuedChunk>();
    }
}

/// Runs the marching cubes algorithm over the given voxel grid.
///
/// Work is parallelised over X slices using Rayon. Returns a [`GeneratedMesh`]
/// with vertices, sequential indices, and flat-shaded normals.
///
/// ```text
/// Per voxel:
/// 1. get_corner_positions       →  8 world-space points
/// 2. values[z][y][x] (×8)      →  8 scalar values
/// 3. get_state                  →  256-entry lookup key
/// 4. EDGE_TABLE[state]          →  bitmask of intersected edges
/// 5. get_edge_midpoints         →  up to 12 interpolated points
/// 6. triangle_verts_from_state  →  triangle vertices from TRI_TABLE
/// ```
fn run_marching_cubes(
    size_x: usize,
    size_y: usize,
    size_z: usize,
    scale: Value,
    threshold: Value,
    values: &Vec<Vec<Vec<Value>>>,
) -> GeneratedMesh {
    let per_x: Vec<Vec<[f32; 3]>> = (0..size_x)
        .into_par_iter()
        .map(|x| {
            let mut local: Vec<[f32; 3]> = Vec::new();
            let per_voxel_max = 15_usize; // upper bound of vertices per voxel
            local.reserve(size_y * size_z * per_voxel_max);

            for y in 0..size_y {
                for z in 0..size_z {
                    let corner_positions = get_corner_positions(x, y, z, scale);

                    let corner_indices = voxel_corner_indices(x, y, z);
                    let eval_corners: Vec<Value> = corner_indices
                        .iter()
                        .map(|[cx, cy, cz]| values[*cz][*cy][*cx])
                        .collect();

                    let state = get_state(&eval_corners, threshold).expect("Could not get state");

                    let edges_mask = EDGE_TABLE[state] as u16;

                    let edge_points = get_edge_midpoints(
                        edges_mask,
                        &CORNER_POINT_INDICES,
                        &corner_positions,
                        &eval_corners,
                        threshold,
                    );

                    local.extend(triangle_verts_from_state(edge_points, state));
                }
            }
            local
        })
        .collect();

    // Merge per-X slices into a single vertex buffer
    let total: usize = per_x.iter().map(|v| v.len()).sum();
    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(total);
    for mut v in per_x {
        vertices.append(&mut v);
    }

    GeneratedMesh::build(vertices)
}

/// Returns the 8 corner indices `[x, y, z]` of the voxel at `(x, y, z)`.
///
/// Matches the standard marching cubes corner ordering used in `EDGE_TABLE` and `TRI_TABLE`.
#[inline]
fn voxel_corner_indices(x: usize, y: usize, z: usize) -> [[usize; 3]; 8] {
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
