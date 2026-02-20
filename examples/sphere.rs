use bevy::{
    pbr::wireframe::{Wireframe, WireframeConfig},
    prelude::*,
};
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk, types::Point};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // #[cfg(not(target_arch = "wasm32"))]
            bevy::pbr::wireframe::WireframePlugin::default(),
            MarchingCubesPlugin::default(),
        ))
        .insert_resource(WireframeConfig {
            global: true,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    const RESOLUTION: u32 = 16;

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            RESOLUTION as f32 * -1.2,
            RESOLUTION as f32 * 1.4,
            RESOLUTION as f32 * -1.2,
        )
        .looking_at(Vec3::Y * RESOLUTION as f32 * 0.8, Vec3::Y),
    ));

    let function = |_p: Point| {
        let distance = (_p.x - (RESOLUTION / 2) as f64)
            .hypot(_p.y - (RESOLUTION / 2) as f64)
            .hypot(_p.z - (RESOLUTION / 2) as f64);
        distance - (RESOLUTION / 4) as f64
    };

    commands.spawn((
        Chunk::new(
            RESOLUTION as usize,
            RESOLUTION as usize,
            RESOLUTION as usize,
        )
        .fill(&function),
        Wireframe,
    ));
}
