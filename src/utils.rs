use std::collections::{BinaryHeap, HashMap};

use bevy::{ecs::query::QueryEntityError, prelude::*};

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
