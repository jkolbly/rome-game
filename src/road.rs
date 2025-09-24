use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    city::City,
    map::Sector,
    resource::ResourceNode,
    settings::DisplaySettings,
    utils::{self, bezier_path, line_mesh},
};

#[derive(Component)]
pub struct Road {
    pub start_sector: Entity,
    pub end_sector: Entity,
    pub path: Vec<Entity>,
}

pub fn spawn_node_roads(
    mut commands: Commands,
    node_query: Query<&ResourceNode>,
    city_query: Query<&City>,
    sector_query: Query<&Sector>,
) {
    for city in city_query {
        for e_node in &city.resource_nodes {
            let node = node_query.get(*e_node).unwrap();
            let path = utils::pathfind(city.sector, node.sector, &sector_query).unwrap();
            commands.spawn((
                Road {
                    start_sector: city.sector,
                    end_sector: node.sector,
                    path,
                },
                Visibility::Visible,
            ));
        }
    }
}

pub fn add_road_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    road_query: Query<(Entity, &Road), Without<Mesh2d>>,
    sector_query: Query<&Sector>,
    settings: Res<DisplaySettings>,
) {
    for (e_road, road) in road_query {
        let path = road
            .path
            .iter()
            .map(|e| sector_query.get(*e).unwrap().centroid)
            .collect();
        let bezier = bezier_path(path, 150);

        let mesh = line_mesh(&bezier, settings.road_width / 2.0);
        let mesh_handle = meshes.add(mesh);
        let mesh_entity = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(materials.add(Color::srgb_u8(123, 63, 0))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();
        commands.entity(e_road).add_child(mesh_entity);
    }
}

pub fn debug_roads(mut gizmos: Gizmos, sector_query: Query<&Sector>, road_query: Query<&Road>) {
    for road in road_query {
        let mut path_iter = road.path.iter();
        let mut prev = path_iter.next().unwrap();
        let mut prev_coords = sector_query.get(*prev).unwrap().centroid;

        while let Some(next) = path_iter.next() {
            let next_coords = sector_query.get(*next).unwrap().centroid;

            gizmos.line_2d(prev_coords, next_coords, Color::srgb_u8(255, 0, 0));

            prev = next;
            prev_coords = next_coords;
        }
    }
}
