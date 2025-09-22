use bevy::prelude::*;

use crate::{city::City, map::Sector, resource::ResourceNode, utils};

#[derive(Component)]
pub struct Road {
    pub start_sector: Entity,
    pub end_sector: Entity,
}

pub fn spawn_roads(
    mut commands: Commands,
    node_query: Query<&ResourceNode>,
    city_query: Query<&City>,
) {
    for city in city_query {
        for e_node in &city.resource_nodes {
            let node = node_query.get(*e_node).unwrap();
            commands.spawn(Road {
                start_sector: city.sector,
                end_sector: node.sector,
            });
        }
    }
}

pub fn debug_roads(mut gizmos: Gizmos, sector_query: Query<&Sector>, road_query: Query<&Road>) {
    for road in road_query {
        let mut path = utils::pathfind(road.start_sector, road.end_sector, &sector_query).unwrap();
        let mut prev = path.pop().unwrap();
        let mut prev_coords = sector_query.get(prev).unwrap().centroid;

        while let Some(next) = path.pop() {
            let next_coords = sector_query.get(next).unwrap().centroid;

            gizmos.line_2d(prev_coords, next_coords, Color::srgb_u8(255, 0, 0));

            prev = next;
            prev_coords = next_coords;
        }
    }
}
