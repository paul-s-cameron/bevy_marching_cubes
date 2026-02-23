# bevy_marching_cubes

A small Bevy plugin that generates meshes from signed-distance-field (SDF) using the Marching Cubes algorithm. ([source reference credit](https://github.com/TristanAntonsen/marching-cubes))

This crate provides a `MarchingCubesPlugin` which can generate a `Mesh` from a `Chunk` component. It is intended as a personal lightweight utility for procedural surface generation and rendering in Bevy.

**Status:** Work-in-progress — basic mesh generation and examples available.

## Quick start

Add the plugin to your Bevy `App` and spawn a `Chunk` entity. The plugin will detect new `Chunk` components, generate a mesh from the chunk's voxel data, and attach a mesh to the entity.

Example (minimal):

```rust
use bevy::prelude::*;
use bevy_marching_cubes::{MarchingCubesPlugin, chunk::Chunk, types::Point};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MarchingCubesPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut chunk = Chunk::new(8, 8, 8);
    
    // Create and fill a chunk with an SDF (example: sphere)
    let sdf = |p: Point| { (p.x - 16.0).hypot(p.y - 16.0).hypot(p.z - 16.0) - 8.0 };
    chunk.fill(&sdf);

    commands.spawn((
        chunk,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0., 0.),
            ..Default::default()
        })),
    ));
}
```

## Bevy Version Support

| bevy | bevy_marching_cubes |
|------|---------------------|
| 0.18 | 0.18                |

## Running the examples

From the crate root run the provided examples (the available example targets are in `examples/`):

```bash
cargo run --example sphere
```

# License

This project is dual licensed:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
