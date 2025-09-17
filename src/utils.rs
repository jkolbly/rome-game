use bevy::prelude::*;

/// Compute the centroid of a polygon.
pub fn centroid(vertices: &Vec<Vec2>) -> Vec2 {
    let area = area(vertices);
    let mut res = Vec2::ZERO;
    let n = vertices.len();
    for i in 0..vertices.len() {
        let factor =
            vertices[i].x * vertices[(i + 1) % n].y - vertices[(i + 1) % n].x * vertices[i].y;
        res += (vertices[i] + vertices[(i + 1) % n]) * factor;
    }
    res / (6.0 * area)
}

/// Compute a polygon's signed area.
pub fn area(vertices: &Vec<Vec2>) -> f32 {
    let n = vertices.len();
    0.5 * (0..n)
        .map(|i| vertices[i].x * vertices[(i + 1) % n].y - vertices[(i + 1) % n].x * vertices[i].y)
        .sum::<f32>()
}
