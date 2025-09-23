use std::collections::{BinaryHeap, HashMap};

use bevy::{
    asset::RenderAssetUsages,
    ecs::query::QueryEntityError,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::map::Sector;

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

pub fn vec2_equals(a: &Vec2, b: &Vec2) -> bool {
    return a.x == b.x && a.y == b.y;
}

#[derive(Debug)]
pub enum PathfindingError {
    NoPathFound,
    EntityIsNotSector,
}

impl From<QueryEntityError> for PathfindingError {
    fn from(value: QueryEntityError) -> Self {
        Self::EntityIsNotSector
    }
}

/// Return a path of sectors from the start to the end.
/// The returned path includes both the start and the end.
/// Uses A* algorithm for pathfinding (working backwards).
pub fn pathfind(
    start: Entity,
    end: Entity,
    sector_query: &Query<&Sector>,
) -> Result<Vec<Entity>, PathfindingError> {
    let start_point = sector_query.get(start)?.centroid;
    let heuristic = |e_sector: Entity| -> Result<f32, PathfindingError> {
        let sector = sector_query.get(e_sector)?;
        Ok(start_point.distance(sector.centroid))
    };

    let actual_cost = |sector0: &Sector, sector1: &Sector| -> f32 {
        let dist = sector0.centroid.distance(sector1.centroid);
        0.5 * dist * (sector0.cost + sector1.cost)
    };

    // Maps sectors we've searched to the next sector in the path.
    let mut previous: HashMap<Entity, Entity> = HashMap::new();

    // Maps sectors we've searched to the cost to get to them.
    let mut costs_so_far: HashMap<Entity, f32> = HashMap::new();
    costs_so_far.insert(end, 0.0);

    // The current sectors to search, ordered by the heuristic cost estimate
    // for a path to the start starting from there.
    let mut border: BinaryHeap<BorderSorter> = BinaryHeap::new();
    border.push(BorderSorter {
        estimate: heuristic(end)?,
        e_sector: end,
    });

    while !border.is_empty() {
        let BorderSorter {
            estimate: _,
            e_sector,
        } = border.pop().unwrap();

        if e_sector == start {
            let mut path = vec![start];
            let mut curr = start;

            while let Some(prev) = previous.get(&curr) {
                path.push(*prev);
                curr = *prev;
            }

            return Ok(path);
        }

        let sector = sector_query.get(e_sector)?;
        let cost = *costs_so_far.get(&e_sector).unwrap();

        for neighbor in &sector.neighbors {
            let neighbor_sector = sector_query.get(*neighbor)?;

            // The cost to get to the neighbor through the current sector
            let new_cost = cost + actual_cost(sector, neighbor_sector);

            // The previous best cost to the neighbor (or infinity if none found yet)
            let old_cost = *costs_so_far.get(neighbor).unwrap_or(&f32::INFINITY);

            if new_cost < old_cost {
                previous.insert(*neighbor, e_sector);
                costs_so_far.insert(*neighbor, new_cost);
                border.push(BorderSorter {
                    estimate: new_cost + heuristic(*neighbor)?,
                    e_sector: *neighbor,
                });
            }
        }
    }

    Err(PathfindingError::NoPathFound)
}

struct BorderSorter {
    estimate: f32,
    e_sector: Entity,
}

impl Ord for BorderSorter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.estimate.total_cmp(&self.estimate)
    }
}

impl PartialOrd for BorderSorter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BorderSorter {
    fn eq(&self, other: &Self) -> bool {
        self.e_sector == other.e_sector
    }
}

impl Eq for BorderSorter {}

/// Creates a line mesh from a set of points
pub fn line_mesh(path: &Vec<Vec2>, width: f32) -> Mesh {
    let point_neighbors = |point: Vec2, facing: Vec2| -> (Vec2, Vec2) {
        let perp = facing.perp().normalize() * width;

        // Output is (left, right)
        (point + perp, point - perp)
    };

    // Data to be added to map mesh
    let mut positions: Vec<Vec3> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();

    // let mut path_iter = road.path.iter().peekable();
    let mut path_iter = path.iter().peekable();
    let mut prev_coords = *path_iter.next().unwrap();

    let (first_left, first_right) = {
        let second_coords = **path_iter.peek().unwrap();
        point_neighbors(prev_coords, second_coords - prev_coords)
    };
    positions.push(first_left.extend(0.0));
    positions.push(first_right.extend(0.0));
    triangles.push(0);
    triangles.push(1);

    while let Some(curr_coords) = path_iter.next() {
        let (curr_left, curr_right) = point_neighbors(*curr_coords, curr_coords - prev_coords);

        if let Some(next_coords) = path_iter.peek() {
            let (next_left, next_right) = point_neighbors(*curr_coords, *next_coords - curr_coords);

            let curr_left_dir = curr_left - curr_coords;
            let next_left_dir = next_left - curr_coords;
            let left_hand = (curr_left_dir.x * next_left_dir.y - next_left_dir.x * curr_left_dir.y)
                .total_cmp(&0.0);
            match left_hand {
                std::cmp::Ordering::Greater => {
                    // Acute angle on the left
                    let curr_left_adj =
                        (curr_left_dir + next_left_dir).normalize() * width + curr_coords;
                    triangles.push(positions.len() as u32);
                    positions.push(curr_left_adj.extend(0.0));
                    triangles.push(positions.len() as u32);
                    positions.push(curr_right.extend(0.0));
                    triangles.push(positions.len() as u32 - 2);
                    triangles.push(positions.len() as u32);
                    positions.push(next_right.extend(0.0));
                }
                std::cmp::Ordering::Equal => {
                    // Lines are parallel
                    triangles.push(positions.len() as u32);
                    positions.push(curr_left.extend(0.0));
                    triangles.push(positions.len() as u32);
                    positions.push(curr_right.extend(0.0));
                }
                std::cmp::Ordering::Less => {
                    // Acute angle on the right
                    let curr_right_dir = curr_right - curr_coords;
                    let next_right_dir = next_right - curr_coords;
                    let curr_right_adj =
                        (curr_right_dir + next_right_dir).normalize() * width + curr_coords;
                    triangles.push(positions.len() as u32);
                    positions.push(curr_left.extend(0.0));
                    triangles.push(positions.len() as u32);
                    positions.push(curr_right_adj.extend(0.0));
                    triangles.push(positions.len() as u32);
                    positions.push(next_left.extend(0.0));
                    triangles.push(positions.len() as u32 - 2);
                }
            };

            prev_coords = *curr_coords;
        } else {
            // This is the end, add the last two vertices
            triangles.push(positions.len() as u32);
            positions.push(curr_left.extend(0.0));
            triangles.push(positions.len() as u32);
            positions.push(curr_right.extend(0.0));
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(triangles))
}

/// Create a new path by using an existing path as Bezier control points.
pub fn bezier_path(mut path: Vec<Vec2>, subdivisions: usize) -> Vec<Vec2> {
    let last = *path.last().unwrap();
    let mut new_path: Vec<Vec2> = vec![path[0], path[0]];
    new_path.append(&mut path);
    new_path.push(last);
    new_path.push(last);
    CubicBSpline::new(new_path)
        .to_curve()
        .unwrap()
        .iter_positions(subdivisions)
        .collect()
}
