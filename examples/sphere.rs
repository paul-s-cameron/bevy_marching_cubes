use std::f32::consts::PI;

use bevy::{pbr::wireframe::Wireframe, prelude::*};
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk, types::Point};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MarchingCubesPlugin::default(),
            #[cfg(not(target_arch = "wasm32"))]
            bevy::pbr::wireframe::WireframePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10., 12., -10.).looking_at(Vec3::Y * 6., Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            ..Default::default()
        },
        Transform::default().with_rotation(Quat::from_rotation_x(-PI / 4.)),
    ));

    let function = |_p: Point| {
        let distance = (_p.x - 4.).hypot(_p.y - 4.).hypot(_p.z - 4.);
        distance - 2.
    };

    commands.spawn((
        Chunk::new(8, 8, 8).fill(&function),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0., 0.),
            perceptual_roughness: 1.,
            ..Default::default()
        })),
        Wireframe,
    ));
}
