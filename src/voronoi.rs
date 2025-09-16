use std::collections::BinaryHeap;

use bevy::{asset::RenderAssetUsages, prelude::*};

use crate::map::Sector;

/// Generate a Voronoi mesh from a set of points (sites).
/// Uses Fortune's algorithm for O(nlogn) time complexity.
pub fn voronoi(points: Vec<(f32, f32)>) -> Vec<Sector> {
    let mut events: BinaryHeap<Event> = BinaryHeap::new();
    for point in points {
        events.push(Event::Site {
            x: point.0,
            y: point.1,
        });
    }

    let mut beachline: Vec<Beach> = Vec::new();

    while let Some(event) = events.pop() {
        println!("Fortune's Event: {:?}", event);

        match event {
            Event::Site { x, y } => {
                // Find which parabola the site intersects
            }
            Event::Intersection => todo!(),
        }
    }

    Vec::new()
}

#[derive(PartialEq, Debug)]
enum Event {
    /// The sweepline passes a new site.
    Site { x: f32, y: f32 },

    /// An arc disappears by two parabolas intersecting.
    Intersection,
}

impl Event {
    fn get_y(&self) -> f32 {
        match self {
            Event::Site { x, y } => *y,
            Event::Intersection => todo!(),
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_y().total_cmp(&other.get_y()).reverse()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Event {}

/// A segment of the beachline.
enum Beach {
    Parabola {
        focus_x: f32,
        focus_y: f32,
    },
    Edge {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
    },
}

// Find the index of the parabola that contains the given x coordinate.
fn parabola_binary_search(beachline: Vec<Beach>, x: f32) -> usize {
    0
}
