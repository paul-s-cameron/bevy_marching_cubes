use crate::{
    error::{MarchingCubesError, Result},
    interp::{find_t, interpolate_points},
    tables::TRI_TABLE,
    types::Value,
};

/// Converts the active edge midpoints for a given marching cubes `state` into
/// a flat list of triangle vertices.
///
/// `TRI_TABLE[state]` contains edge indices in groups of three, terminated by `-1`:
/// ```text
/// TRI_TABLE[state] = [e0, e1, e2,  e3, e4, e5,  -1, ...]
///                     \___tri0__/   \___tri1__/
/// ```
/// Each edge index maps into `edge_points` to retrieve the interpolated midpoint.
#[inline]
pub fn triangle_verts_from_state(
    edge_points: [Option<[f32; 3]>; 12],
    state: usize,
) -> Vec<[f32; 3]> {
    TRI_TABLE[state]
        .iter()
        .take_while(|&&v| v != -1)
        .map(|&t| edge_points[t as usize].expect("edge midpoint missing"))
        .collect()
}

/// Returns the 8 world-space corner positions of the voxel at grid index `(x, y, z)`.
///
/// Corners are ordered to match the standard marching cubes convention:
/// ```text
///     6----7          Y
///    /|   /|          |
///   2----3 |          *-- X
///   | 4--|-5         /
///   |/   |/         Z
///   0----1
/// ```
#[inline]
pub fn get_corner_positions(x: usize, y: usize, z: usize, scale: Value) -> [[f32; 3]; 8] {
    let xf = scale * x as Value;
    let yf = scale * y as Value;
    let zf = scale * z as Value;

    [
        [xf,         yf,         zf        ],
        [xf + scale, yf,         zf        ],
        [xf + scale, yf + scale, zf        ],
        [xf,         yf + scale, zf        ],
        [xf,         yf,         zf + scale],
        [xf + scale, yf,         zf + scale],
        [xf + scale, yf + scale, zf + scale],
        [xf,         yf + scale, zf + scale],
    ]
}

/// Returns the `[min, max]` bounding box corners given a `center` point and box dimensions.
///
/// ```text
///  min = center - dims/2
///  max = center + dims/2
/// ```
#[inline]
pub fn center_box(center: [f32; 3], dims: [f32; 3]) -> [[f32; 3]; 2] {
    let min = [
        center[0] - dims[0] / 2.0,
        center[1] - dims[1] / 2.0,
        center[2] - dims[2] / 2.0,
    ];
    let max = [
        center[0] + dims[0] / 2.0,
        center[1] + dims[1] / 2.0,
        center[2] + dims[2] / 2.0,
    ];
    [min, max]
}

/// Computes the marching cubes state bitmask for a voxel.
///
/// Each of the 8 corners maps to one bit. A bit is set when the corner's value
/// is **at or below** the threshold (i.e. "inside" the surface):
///
/// ```text
/// corner index:  7  6  5  4  3  2  1  0
/// state bits:   [_][_][_][_][_][_][_][_]
///                                      ^-- corner 0 inside?
/// ```
///
/// Returns [`MarchingCubesError::InvalidCorners`] if `eval_corners` does not contain exactly 8 values.
#[inline]
pub fn get_state(eval_corners: &Vec<Value>, threshold: Value) -> Result<usize> {
    if eval_corners.len() != 8 {
        return Err(MarchingCubesError::InvalidCorners);
    }

    let mut state: usize = 0;
    for (i, &v) in eval_corners.iter().enumerate() {
        if v <= threshold {
            state |= 1 << i;
        }
    }

    Ok(state)
}

/// Interpolates the midpoint along each edge of the voxel that crosses the iso-surface.
///
/// `edges_mask` is a 12-bit field from `EDGE_TABLE` — a set bit means that edge is active.
///
/// For each active edge, the midpoint is found by linearly interpolating between
/// the two endpoint positions at the iso-value.
#[inline]
pub fn get_edge_midpoints(
    edges_mask: u16,
    point_indices: &[[i8; 2]; 12],
    corner_positions: &[[f32; 3]; 8],
    corner_values: &[Value],
    threshold: Value,
) -> [Option<[f32; 3]>; 12] {
    let mut edge_points: [Option<[f32; 3]>; 12] = [None; 12];

    for i in 0..12_usize {
        if (edges_mask & (1 << i)) == 0 {
            continue;
        }

        let pair = point_indices[i];
        let vi = corner_values[pair[0] as usize];
        let vf = corner_values[pair[1] as usize];
        let pi = corner_positions[pair[0] as usize];
        let pf = corner_positions[pair[1] as usize];

        let t = find_t(vi, vf, threshold);
        edge_points[i] = Some(interpolate_points(pi, pf, t));
    }

    edge_points
}
