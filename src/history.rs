use bevy::{
    platform::collections::HashMap, prelude::*, render::render_resource::encase::private::Length,
};
use chrono::NaiveDate;

use crate::{city::City, map::Sector, utils::bezier_pathfind};

/// Information about the history of the whole map.
#[derive(Component)]
pub struct History {
    events: Vec<HistoricalEvent>,
}

/// Information about the society and culture of a city.
#[derive(Component)]
pub struct CultureInfo {
    neighbors: HashMap<Entity, NeighborInfo>,
}

impl CultureInfo {
    fn new(neighbors: HashMap<Entity, NeighborInfo>) -> Self {
        CultureInfo { neighbors }
    }
}

/// Information about a single event in history.
/// Intended to be a singleton component.
pub struct HistoricalEvent {
    event_type: EventType,
    date: NaiveDate,

    /// List of involved cities
    involved: Vec<Entity>,
}

/// Type of historical event.
pub enum EventType {}

/// Information about a city's relationship with one of its neighbors.
#[derive(Debug)]
pub struct NeighborInfo {
    neighbor: Entity,
    distance: f32,
    road_distance: f32,
}

impl NeighborInfo {
    fn new(entity: Entity, distance: f32, road_distance: f32) -> Self {
        NeighborInfo {
            neighbor: entity,
            distance,
            road_distance,
        }
    }
}

/// Get info about a city's neighbors.
fn find_neighbors(
    this_e_city: Entity,
    q_city: &Query<(Entity, &City)>,
    q_sector: &Query<&Sector>,
) -> HashMap<Entity, NeighborInfo> {
    let (_, this_city) = q_city.get(this_e_city).unwrap();

    let mut neighbors = HashMap::new();

    for (e_city, city) in q_city {
        if e_city == this_e_city {
            continue;
        }

        let this_e_sector = this_city.sector;
        let e_sector = city.sector;

        let this_sector = q_sector.get(this_e_sector).unwrap();
        let sector = q_sector.get(e_sector).unwrap();

        let (curve, _) = bezier_pathfind(this_e_sector, e_sector, &q_sector).unwrap();

        neighbors.insert(
            e_city,
            NeighborInfo::new(
                e_city,
                this_sector.centroid.distance(sector.centroid),
                curve.segments().length() as f32,
            ),
        );
    }

    neighbors
}

/// Prepare to run the history sim.
pub fn setup_history_sim(
    mut commands: Commands,
    q_city: Query<(Entity, &City)>,
    q_sector: Query<&Sector>,
) {
    for (e_city, _) in q_city {
        let neighbors = find_neighbors(e_city, &q_city, &q_sector);
        commands.entity(e_city).insert(CultureInfo::new(neighbors));
    }

    commands.spawn(History { events: Vec::new() });
}

/// Run the history simulation.
pub fn run_history_sim(
    mut q_history: Query<&mut History>,
    q_culture: Query<(Entity, &CultureInfo)>,
) {
    let mut history = q_history.single_mut().unwrap();
}
