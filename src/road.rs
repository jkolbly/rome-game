use bevy::{prelude::*, render::render_resource::encase::private::Length};

use crate::{
    city::City,
    map::Sector,
    resource::ResourceNode,
    settings::DisplaySettings,
    utils::{bezier_pathfind, line_mesh},
};

#[derive(Component)]
pub struct Road {
    pub start_sector: Entity,
    pub end_sector: Entity,

    /// The list of sectors making this path.
    pub path: Vec<Entity>,

    /// The curve making up this road.
    pub curve: CubicCurve<Vec2>,

    /// The length of this road.
    pub length: f32,

    /// The multiplier to movement speed on this road.
    pub speed_multiplier: f32,
}

pub fn spawn_node_roads(
    mut commands: Commands,
    mut node_query: Query<&mut ResourceNode>,
    city_query: Query<&City>,
    sector_query: Query<&Sector>,
) {
    for city in city_query {
        for e_node in &city.resource_nodes {
            let mut node = node_query.get_mut(*e_node).unwrap();
            let (curve, path) = bezier_pathfind(city.sector, node.sector, &sector_query).unwrap();
            let length = curve.segments().length() as f32;
            let e_road = commands
                .spawn((
                    Road {
                        start_sector: city.sector,
                        end_sector: node.sector,
                        path,
                        curve,
                        length,
                        speed_multiplier: 1.0,
                    },
                    Visibility::Visible,
                ))
                .id();
            node.road = e_road;
        }
    }
}

pub fn add_road_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    road_query: Query<(Entity, &Road), Without<Mesh2d>>,
    settings: Res<DisplaySettings>,
) {
    for (e_road, road) in road_query {
        let points = road.curve.iter_positions(150).collect();

        let mesh = line_mesh(&points, settings.road_width / 2.0);
        let mesh_handle = meshes.add(mesh);
        let mesh_entity = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(materials.add(Color::srgb_u8(210, 180, 140))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();
        commands.entity(e_road).add_child(mesh_entity);
    }
}

pub fn debug_roads(mut gizmos: Gizmos, sector_query: Query<&Sector>, road_query: Query<&Road>) {
    for road in road_query {
        let mut path_iter = road.path.iter();
        let mut prev_coords = sector_query
            .get(*path_iter.next().unwrap())
            .unwrap()
            .centroid;

        while let Some(next) = path_iter.next() {
            let next_coords = sector_query.get(*next).unwrap().centroid;

            gizmos.line_2d(prev_coords, next_coords, Color::srgb_u8(255, 0, 0));

            prev_coords = next_coords;
        }
    }
}
