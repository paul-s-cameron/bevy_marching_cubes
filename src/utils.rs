use bevy::platform::collections::HashMap;
use nalgebra::{Point3, point};

use crate::{
    error::{MarchingCubesError, Result},
    interp::{find_t, interpolate_points, remap},
    tables::TRI_TABLE,
    types::{Point, Vector},
};

pub fn triangle_verts_from_state(
    edge_points: HashMap<usize, Vec<f64>>,
    state: usize,
) -> Vec<Point> {
    // triangles (TRI_TABLE[state])
    // Example: [7, 3, 2, 6, 7, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]
    // Triangles: [p7, p3, p2], [p6, p7, p2]
    let new_verts = TRI_TABLE[state]
        .iter()
        .filter(|v| v != &&-1)
        .map(|t| {
            let new_vert = Point3::new(
                //converting Vec to array
                edge_points[&(*t as usize)][2],
                edge_points[&(*t as usize)][1],
                edge_points[&(*t as usize)][0],
            );
            new_vert
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
    scale: f64,
) -> Vec<Point> {
    let xf = scale * x as f64;
    let yf = scale * y as f64;
    let zf = scale * z as f64;

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
pub fn get_state(eval_corners: &Vec<f64>, threshold: f64) -> Result<usize> {
    // Make sure eval_corners contains exactly 8 values
    if eval_corners.len() != 8 {
        return Err(MarchingCubesError::InvalidCorners);
    }

    // 0 if <= threshold, 1 if > threshold
    let states = eval_corners.iter().map(|x| state_function(*x, threshold));

    let mut i = 1.0;
    let mut final_state = 0.0;
    for s in states {
        final_state += s * i;
        i *= 2.0;
    }

    return Ok(final_state as usize);
}

// Function to determine state of each corner
pub fn state_function(v: f64, threshold: f64) -> f64 {
    if v <= threshold { 1.0 } else { 0.0 }
}

// Get the midpoints of the edges of the cube
pub fn get_edge_midpoints(
    endpoint_indices: Vec<[i8; 2]>,
    edges_to_use: Vec<usize>,
    corner_positions: Vec<Point>,
    corner_values: Vec<f64>,
    threshold: f64,
) -> HashMap<usize, Vec<f64>> {
    let (mut pair, mut edge);
    let (mut pi, mut pf, mut pe);
    let (mut vi, mut vf, mut t);

    let mut edge_points: HashMap<usize, Vec<f64>> = HashMap::new();

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

            pe = interpolate_points(pi, pf, t); // midpoint/interpolated point
            edge_points.insert(edge, pe);
        }
    }
    edge_points
}

// Return pairs of endpoints per edge of the cube
pub fn get_edge_endpoints(
    edges: &String,
    point_indices: &[[i8; 2]; 12],
) -> (Vec<[i8; 2]>, Vec<usize>) {
    // returns the endpoints of edges from EdgeTable lookup
    let mut edge_points = Vec::new();

    // prepare for the check to see if each character = 1
    // TODO: (doesn't seem like the right way to do this)

    // looping through binary string of yes/no for each edge
    let edges_to_use = edges_from_lookup(edges);
    for e in edges_to_use.clone() {
        edge_points.push(point_indices[e]);
    }

    (edge_points, edges_to_use)
}

// Return the edges that contain triangle vertices
pub fn edges_from_lookup(edges: &String) -> Vec<usize> {
    let use_edge = "1".chars().next().unwrap(); // edgeTable[8] = 100000001100 -> Edges 2, 3, 11 intersected
    let mut i = (edges.len() - 1) as i32;
    let mut edges_to_use = Vec::new();

    for char in edges.chars() {
        if char == use_edge {
            edges_to_use.push(i as usize)
        }
        i -= 1;
    }

    edges_to_use
}

pub fn smooth_min(a: f64, b: f64, mut k: f64) -> f64 {
    // polynomial smooth min

    if k < 0.00001 {
        k = 0.00001
    }

    let h = (k - (a - b).abs()).max(0.0) / k;

    a.min(b) - h * h * k * (1.0 / 4.0)
}

pub fn ramp(v: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    if v < in_min {
        return out_min;
    } else if v > in_max {
        return out_max;
    } else {
        return remap(v, [in_min, in_max], [out_min, out_max]);
    }
}
