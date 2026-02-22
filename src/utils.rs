use nalgebra::{Point3, point};

use crate::{
    error::{MarchingCubesError, Result},
    interp::{find_t, interpolate_points},
    tables::TRI_TABLE,
    types::{Point, Value, Vector},
};

#[inline]
pub fn triangle_verts_from_state(
    edge_points: [Option<[Value; 3]>; 12],
    state: usize,
) -> Vec<Point> {
    // triangles (TRI_TABLE[state])
    // Example: [7, 3, 2, 6, 7, 2, -1, ...]
    TRI_TABLE[state]
        .iter()
        .take_while(|&&v| v != -1)
        .map(|&t| {
            let arr = edge_points[t as usize].expect("edge midpoint missing");
            Point3::new(arr[0], arr[1], arr[2])
        })
        .collect()
}

// Get the point coordinates at the 8 vertices of the cube
#[inline]
pub fn get_corner_positions(x: usize, y: usize, z: usize, scale: Value) -> [Point; 8] {
    let xf = scale * x as Value;
    let yf = scale * y as Value;
    let zf = scale * z as Value;

    [
        point![xf, yf, zf],
        point![xf + scale, yf, zf],
        point![xf + scale, yf + scale, zf],
        point![xf, yf + scale, zf],
        point![xf, yf, zf + scale],
        point![xf + scale, yf, zf + scale],
        point![xf + scale, yf + scale, zf + scale],
        point![xf, yf + scale, zf + scale],
    ]
}

// Return min and max bounding box points from a center point and box dimensions
#[inline]
pub fn center_box(center: Point, dims: Vector) -> [Point; 2] {
    let min_point = point![
        center.x - dims.x / 2.0,
        center.y - dims.y / 2.0,
        center.z - dims.z / 2.0
    ];
    let max_point = point![
        center.x + dims.x / 2.0,
        center.y + dims.y / 2.0,
        center.z + dims.z / 2.0
    ];
    [min_point, max_point]
}

// get the state of the 8 vertices of the cube
#[inline]
pub fn get_state(eval_corners: &Vec<Value>, threshold: Value) -> Result<usize> {
    // Make sure eval_corners contains exactly 8 values
    if eval_corners.len() != 8 {
        return Err(MarchingCubesError::InvalidCorners);
    }

    // Build an integer bitmask state
    let mut state: usize = 0;
    for (i, &v) in eval_corners.iter().enumerate() {
        if v <= threshold {
            state |= 1 << i;
        }
    }

    Ok(state)
}

// Get the midpoints of the edges of the cube
#[inline]
pub fn get_edge_midpoints(
    edges_mask: u16,
    point_indices: &[[i8; 2]; 12],
    corner_positions: &[Point; 8],
    corner_values: &[Value],
    threshold: Value,
) -> [Option<[Value; 3]>; 12] {
    let mut edge_points: [Option<[Value; 3]>; 12] = [None; 12];

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
        let pe = interpolate_points(pi, pf, t); // Vec<Value>
        edge_points[i] = Some([pe[0], pe[1], pe[2]]);
    }

    edge_points
}
