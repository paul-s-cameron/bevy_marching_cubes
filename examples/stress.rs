use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk};

use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use noiz::prelude::*;

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
        .add_systems(Update, (debug, spawn_chunks))
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

fn spawn_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        const CHUNK_SIZE: i32 = 32;
        const CHUNK_DIM: i32 = 16;

        let mut noise = Noise::<
            LayeredNoise<
                Normed<f32>,
                Persistence,
                Octave<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>>,
            >,
        >::default();
        noise.set_frequency(0.06);

        for x in -CHUNK_DIM..CHUNK_DIM {
            for z in -CHUNK_DIM..CHUNK_DIM {
                let translation: Vec3 = Vec3::new(
                    x as f32 * CHUNK_SIZE as f32,
                    0.,
                    z as f32 * CHUNK_SIZE as f32,
                );

                let mut chunk =
                    Chunk::new(CHUNK_SIZE as usize, 128, CHUNK_SIZE as usize).with_threshold(0.);
                chunk.for_each_corner_offset(translation, |x, y, z, value| {
                    *value = noise.sample_for(Vec3::new(x, y, z));
                });

                commands.spawn((
                    chunk,
                    Transform::from_translation(translation),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1., 0., 0.),
                        ..Default::default()
                    })),
                    // Wireframe,
                ));
            }
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
