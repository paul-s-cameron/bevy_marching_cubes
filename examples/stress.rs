use bevy::{pbr::wireframe::Wireframe, prelude::*};
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk};

use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use noiz::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MarchingCubesPlugin::default(),
            PanOrbitCameraPlugin,
            #[cfg(not(target_arch = "wasm32"))]
            bevy::pbr::wireframe::WireframePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, debug)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
        Transform::from_xyz(20., 150., 20.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            ..Default::default()
        },
        Transform::default().with_rotation(Quat::from_rotation_x(-45.0_f32.to_radians())),
    ));

    const CHUNK_SIZE: usize = 16;

    let mut noise = Noise::<
        LayeredNoise<
            Normed<f32>,
            Persistence,
            Octave<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>>,
        >,
    >::default();
    noise.set_frequency(0.06);

    for x in -4..4 {
        for z in -4..4 {
            let translation: Vec3 = Vec3::new(
                x as f32 * CHUNK_SIZE as f32,
                0.,
                z as f32 * CHUNK_SIZE as f32,
            );

            let mut chunk = Chunk::new(CHUNK_SIZE, 48, CHUNK_SIZE).with_threshold(0.);
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
                Wireframe,
            ));
        }
    }
}

fn debug(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<Chunk>>) {
    for transform in query.iter() {
        gizmos.sphere(
            Isometry3d::from_translation(transform.translation()),
            1.,
            Color::WHITE,
        );
        gizmos.line(
            transform.translation(),
            transform.translation() + Vec3::X * 10.0,
            Color::Srgba(Srgba::new(1., 0., 0., 1.)),
        );
        gizmos.line(
            transform.translation(),
            transform.translation() + Vec3::Z * 10.0,
            Color::Srgba(Srgba::new(0., 0., 1., 1.)),
        );
    }
}
