use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, MarchingCubesError>;

/// Errors produced during marching cubes mesh generation.
#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum MarchingCubesError {
    /// A voxel was evaluated with a corner count other than 8.
    InvalidCorners,
    /// A triangle was added referencing a vertex index that doesn't exist.
    InvalidIndex,
}

impl std::error::Error for MarchingCubesError {}
