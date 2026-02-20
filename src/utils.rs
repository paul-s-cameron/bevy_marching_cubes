use nalgebra::{Point3, point};

use crate::{
    error::{MarchingCubesError, Result},
    interp::{find_t, interpolate_points},
    tables::TRI_TABLE,
    types::{Point, Value, Vector},
};

pub fn triangle_verts_from_state(
    edge_points: [Option<[Value; 3]>; 12],
    state: usize,
) -> Vec<Point> {
    // triangles (TRI_TABLE[state])
    // Example: [7, 3, 2, 6, 7, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]
    // Triangles: [p7, p3, p2], [p6, p7, p2]
    let new_verts = TRI_TABLE[state]
        .iter()
        .filter(|v| v != &&-1)
        .map(|t| {
            let idx = *t as usize;
            let arr = edge_points[idx].expect("edge midpoint missing for triangle vertex");
            Point3::new(arr[2], arr[1], arr[0])
        })
        .collect::<Vec<Point>>();
    new_verts
}

// Get the point coordinates at the 8 vertices of the cube
pub fn get_corner_positions(
    min_point: Point,
    x: usize,
    y: usize,
    z: usize,
    scale: Value,
) -> Vec<Point> {
    let xf = scale * x as Value;
    let yf = scale * y as Value;
    let zf = scale * z as Value;

    // could be consolidated/more idiomatic
    let p0 = point![xf, yf, zf];
    let p1 = point![xf + scale, yf, zf];
    let p2 = point![xf + scale, yf + scale, zf];
    let p3 = point![xf, yf + scale, zf];
    let p4 = point![xf, yf, zf + scale];
    let p5 = point![xf + scale, yf, zf + scale];
    let p6 = point![xf + scale, yf + scale, zf + scale];
    let p7 = point![xf, yf + scale, zf + scale];

    let mut corner_points = vec![p0, p1, p2, p3, p4, p5, p6, p7];

    // Translating points to bounding box space
    corner_points = corner_points
        .iter()
        .map(|p| add_points(*p, min_point))
        .collect();

    corner_points
}

pub fn add_points(p1: Point, p2: Point) -> Point {
    point![p1.x + p2.x, p1.y + p2.y, p1.z + p2.z]
}

// Return min and max bounding box points from a center point and box dimensions
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
pub fn get_state(eval_corners: &Vec<Value>, threshold: Value) -> Result<usize> {
    // Make sure eval_corners contains exactly 8 values
    if eval_corners.len() != 8 {
        return Err(MarchingCubesError::InvalidCorners);
    }

    // Build an integer bitmask state: set bit i when corner i satisfies the `state_function`
    let mut state: usize = 0;
    for (i, &v) in eval_corners.iter().enumerate() {
        if state_function(v, threshold) {
            state |= 1 << i;
        }
    }

    Ok(state)
}

// Function to determine state of each corner
pub fn state_function(v: Value, threshold: Value) -> bool {
    // Preserve original behavior: true when value <= threshold
    v <= threshold
}

// Get the midpoints of the edges of the cube
pub fn get_edge_midpoints(
    endpoint_indices: Vec<[i8; 2]>,
    edges_to_use: Vec<usize>,
    corner_positions: Vec<Point>,
    corner_values: Vec<Value>,
    threshold: Value,
) -> [Option<[Value; 3]>; 12] {
    let (mut pair, mut edge);
    let (mut pi, mut pf);
    let (mut vi, mut vf, mut t);

    let mut edge_points: [Option<[Value; 3]>; 12] = [None; 12];

    for i in 0..endpoint_indices.len() {
        pair = endpoint_indices[i];
        edge = edges_to_use[i];
        if pair.len() > 0 {
            // finding points corresponding to endpoint indices
            vi = corner_values[pair[0] as usize];
            vf = corner_values[pair[1] as usize];
            pi = corner_positions[pair[0] as usize];
            pf = corner_positions[pair[1] as usize];

            t = find_t(vi, vf, threshold);

            let pe = interpolate_points(pi, pf, t); // midpoint/interpolated point (Vec<Value>)
            edge_points[edge] = Some([pe[0], pe[1], pe[2]]);
        }
    }
    edge_points
}

/// Return pairs of endpoints per edge of the cube
pub fn get_edge_endpoints(
    edges_mask: u16,
    point_indices: &[[i8; 2]; 12],
) -> (Vec<[i8; 2]>, Vec<usize>) {
    // returns the endpoints of edges from bitmask
    let mut edge_points = Vec::new();

    // get list of edge indices from bitmask
    let edges_to_use = edges_from_lookup(edges_mask);
    for e in &edges_to_use {
        edge_points.push(point_indices[*e]);
    }

    (edge_points, edges_to_use)
}

/// Return the edges that contain triangle vertices
pub fn edges_from_lookup(edges_mask: u16) -> Vec<usize> {
    // Interpret the lower 12 bits of `edges_mask` as edge flags.
    let mut edges_to_use = Vec::new();
    for i in 0..12_usize {
        if (edges_mask & (1 << i)) != 0 {
            edges_to_use.push(i);
        }
    }
    edges_to_use
}
