#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod chunk;
pub mod error;
pub mod interp;
pub mod mesh;
pub mod plugin;
pub mod tables;
pub mod types;
pub mod utils;

pub use mesh::GeneratedMesh;
pub use plugin::{MarchingCubesConfig, MarchingCubesPlugin, MarchingCubesSet, QueuedChunk};
