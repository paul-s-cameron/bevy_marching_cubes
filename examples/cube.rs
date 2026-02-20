use bevy::prelude::*;
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk, types::Point};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            #[cfg(not(target_arch = "wasm32"))]
            bevy::pbr::wireframe::WireframePlugin::default(),
            MarchingCubesPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    bevy::log::info!("Cube Example");

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let function = |p: Point| {
        let distance = (p.x - 16.0).hypot(p.y - 16.0).hypot(p.z - 16.0);
        distance - 8.0
    };

    commands.spawn(Chunk::new(32, 32, 32).fill(&function));
}
