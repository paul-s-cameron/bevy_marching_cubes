use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use noiz::prelude::*;
use std::collections::HashSet;

type TerrainNoise = Noise<
    LayeredNoise<
        Normed<f32>,
        Persistence,
        Octave<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>>,
    >,
>;

#[derive(Component)]
struct ChunkFillTask(Task<Chunk>);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MarchingCubesPlugin::default(),
            PanOrbitCameraPlugin,
            InfiniteGridPlugin,
            #[cfg(not(target_arch = "wasm32"))]
            bevy::pbr::wireframe::WireframePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_chunk_tasks, poll_chunk_tasks, debug))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            fadeout_distance: 1000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
        Transform::from_xyz(50., 200., 50.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            ..Default::default()
        },
        Transform::default().with_rotation(Quat::from_rotation_x(-45.0_f32.to_radians())),
    ));
}

fn spawn_chunk_tasks(
    mut commands: Commands,
    pan_orbit: Query<&PanOrbitCamera>,
    mut noise: Local<TerrainNoise>,
    mut spawned: Local<HashSet<IVec3>>,
) {
    const CHUNK_SIZE: i32 = 64;
    const CHUNK_HEIGHT: usize = 128;
    const CHUNK_RADIUS: i32 = 2;

    noise.set_frequency(0.06);

    let origin = pan_orbit
        .single()
        .expect("No PanOrbitCamera found")
        .target_focus;

    let origin_chunk = IVec3::new(
        (origin.x / CHUNK_SIZE as f32).floor() as i32,
        0,
        (origin.z / CHUNK_SIZE as f32).floor() as i32,
    );

    let task_pool = AsyncComputeTaskPool::get();

    for dx in -CHUNK_RADIUS..=CHUNK_RADIUS {
        for dz in -CHUNK_RADIUS..=CHUNK_RADIUS {
            if dx * dx + dz * dz > CHUNK_RADIUS * CHUNK_RADIUS {
                continue;
            }

            let coord = IVec3::new(origin_chunk.x + dx, 0, origin_chunk.z + dz);
            if !spawned.insert(coord) {
                continue;
            }

            let translation = Vec3::new(
                coord.x as f32 * CHUNK_SIZE as f32,
                0.,
                coord.z as f32 * CHUNK_SIZE as f32,
            );

            let noise_copy = *noise;

            let task = task_pool.spawn(async move {
                let mut chunk = Chunk::new(CHUNK_SIZE as usize, CHUNK_HEIGHT, CHUNK_SIZE as usize)
                    .with_threshold(0.);
                chunk.for_each_corner_offset(translation, |x, y, z, value| {
                    *value = noise_copy.sample_for(Vec3::new(x, y, z));
                });
                chunk
            });

            commands.spawn((
                ChunkFillTask(task),
                Transform::from_translation(translation),
            ));
        }
    }
}

fn poll_chunk_tasks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tasks: Query<(Entity, &mut ChunkFillTask)>,
) {
    for (entity, mut fill_task) in &mut tasks {
        if let Some(chunk) = block_on(future::poll_once(&mut fill_task.0)) {
            commands.entity(entity).remove::<ChunkFillTask>().insert((
                chunk,
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1., 0., 0.),
                    ..Default::default()
                })),
            ));
        }
    }
}

fn debug(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &Chunk)>) {
    for (transform, chunk) in query.iter() {
        let half_extents = Vec3::new(
            chunk.size_x as f32 * chunk.scale,
            chunk.size_y as f32 * chunk.scale,
            chunk.size_z as f32 * chunk.scale,
        ) / 2.0;
        let center = transform.translation() + half_extents;
        gizmos.cube(
            Transform::from_translation(center).with_scale(half_extents * 2.0),
            Color::WHITE,
        );
    }
}
