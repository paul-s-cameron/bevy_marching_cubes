use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, MarchingCubesError>;

#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum MarchingCubesError {
    InvalidCorners,
    EmptyMesh,
}

impl std::error::Error for MarchingCubesError {}
