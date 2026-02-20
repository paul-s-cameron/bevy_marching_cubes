use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    chunk::Chunk,
    mesh::MarchMesh,
    tables::{CORNER_POINT_INDICES, EDGE_TABLE},
    types::{Point, Value},
    utils::{
        get_corner_positions, get_edge_endpoints, get_edge_midpoints, get_state,
        triangle_verts_from_state,
    },
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarchingCubesSet;

/// Marker component for chunks that are queued for processing.
#[derive(Component)]
pub struct QueuedChunk;

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

fn on_chunk_add(
    mut commands: Commands,
    query: Query<Entity, (Added<Chunk>, Without<QueuedChunk>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(QueuedChunk);
        bevy::log::info!("Added Entity {} to chunk queue", entity);
    }
}

fn process_chunk(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Chunk, &Transform), With<QueuedChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let edge_table = &EDGE_TABLE.map(|e| format!("{:b}", e));

    for (entity, mut chunk, transform) in query.iter_mut() {
        let mut mesh = MarchMesh::new_empty();
        let min_pos = Point::new(
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        );
        let vertices: Vec<Point> = (0..chunk.size_x - 1)
            .into_par_iter()
            .map(|x| {
                (0..chunk.size_y - 1)
                    .map(|y| {
                        (0..chunk.size_z - 1)
                            .map(|z| {
                                // corner positions
                                let corner_positions =
                                    get_corner_positions(min_pos, x, y, z, chunk.scale);

                                // voxel values (read from chunk)
                                let corner_indices = chunk.voxel_corner_indices(x, y, z);
                                let eval_corners: Vec<Value> = corner_indices
                                    .iter()
                                    .map(|[cx, cy, cz]| chunk.get(*cx, *cy, *cz))
                                    .collect();

                                // Calculating state
                                let state =
                                    get_state(&eval_corners, 0.).expect("Could not get state");

                                // edges
                                // Example: 11001100
                                // Edges 2, 3, 6, 7 are intersected
                                let edges_bin_string = &edge_table[state];

                                // Indices of edge endpoints (List of pairs)
                                let (endpoint_indices, edges_to_use) =
                                    get_edge_endpoints(edges_bin_string, &CORNER_POINT_INDICES);

                                // finding midpoints of edges
                                let edge_points = get_edge_midpoints(
                                    endpoint_indices,
                                    edges_to_use,
                                    corner_positions,
                                    eval_corners,
                                    0.,
                                );

                                // adding triangle verts
                                let new_verts = triangle_verts_from_state(edge_points, state);
                                new_verts
                            })
                            .flatten()
                            .collect::<Vec<Point>>()
                    })
                    .flatten()
                    .collect::<Vec<Point>>()
            })
            .flatten()
            .collect::<Vec<Point>>();

        mesh.set_vertices(vertices.clone());
        mesh.create_triangles();

        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        // Convert vertices from Point3<f64> to Vec<[f32; 3]>
        let positions: Vec<[f32; 3]> = vertices
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32])
            .collect();

        // Convert triangle indices from Vec<[usize; 3]> to Vec<u32>
        let indices: Vec<u32> = mesh
            .tris
            .iter()
            .flat_map(|tri| vec![tri[0] as u32, tri[1] as u32, tri[2] as u32])
            .collect();

        chunk.mesh = Some(mesh);

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.insert_indices(Indices::U32(indices));

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(bevy_mesh)))
            .remove::<QueuedChunk>();
        bevy::log::info!("Processed chunk {}", entity);
    }
}
