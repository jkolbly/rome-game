use std::fmt::Debug;
use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use rand::Rng;

use crate::{
    biome::Biome,
    city::City,
    map::{Map, Sector},
    settings::{GameplaySettings, MapGenSettings},
    shipment::Shipment,
    utils,
    wagon::Wagon,
};

/// A type of non-global resource.
#[derive(Clone, Copy)]
pub enum Resource {
    Wheat,
    Ore,
    Lumber,
}

impl Debug for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wheat => write!(f, "Wheat"),
            Self::Ore => write!(f, "Ore"),
            Self::Lumber => write!(f, "Lumber"),
        }
    }
}

/// A resource production node.
#[derive(Component)]
#[require(Transform)]
pub struct ResourceNode {
    pub node_type: ResourceNodeType,
    pub produces: Shipment,
    pub sector: Entity,
    pub wagon_timer: Timer,
    pub city: Entity,
    pub road: Entity,
}

/// A type of [`ResourceNode`]
#[derive(Eq, PartialEq, Hash)]
pub enum ResourceNodeType {
    Farm,
    Mine,
    Lumbermill,
}

pub fn spawn_resource_nodes(
    mut commands: Commands,
    mut map_query: Query<(&Map, &mut Entropy<WyRand>)>,
    city_query: Query<(Entity, &mut City, &Transform)>,
    sector_query: Query<&Sector>,
    settings: Res<MapGenSettings>,
    gameplay_settings: Res<GameplaySettings>,
) {
    let (map, mut rng) = map_query.single_mut().unwrap();

    let mut node_positions: Vec<Vec2> = Vec::new();
    let city_positions: Vec<Vec2> = city_query
        .iter()
        .map(|(_, _, t_city)| t_city.translation.xy())
        .collect();

    for (e_city, mut city, t_city) in city_query {
        let mut nodes_to_gen = rng.random_range(settings.nodes_per_city_range.clone());

        for _ in 0..1000 {
            if nodes_to_gen <= 0 {
                break;
            }

            let rand_index = rng.random_range(0..map.sectors.len());
            let e_sector = map.sectors[rand_index];
            let sector = sector_query.get(e_sector).unwrap();
            let mut node_pos = sector.centroid;

            // Check not outside of deadzone
            if node_pos.x < settings.node_deadzone
                || node_pos.x > settings.size.x - settings.node_deadzone
                || node_pos.y < settings.node_deadzone
                || node_pos.y > settings.size.y - settings.node_deadzone
            {
                continue;
            }

            // Check not too close to a city
            if city_positions
                .iter()
                .any(|pos| pos.distance(node_pos) < settings.node_city_min_dist)
            {
                continue;
            }

            // Check not too close to a node
            if node_positions
                .iter()
                .any(|pos| pos.distance(node_pos) < settings.node_min_spacing)
            {
                continue;
            }

            // Check close enough to this city
            if node_pos.distance(t_city.translation.xy()) > settings.node_city_max_dist {
                continue;
            }

            // Check not closer to a different city

            if city_positions.iter().any(|pos| {
                !utils::vec2_equals(pos, &t_city.translation.xy())
                    && node_pos.distance(*pos) < node_pos.distance(t_city.translation.xy())
            }) {
                continue;
            }

            // Get node type and check not in desert
            let node_type = match sector.biome.unwrap() {
                Biome::Plains => ResourceNodeType::Farm,
                Biome::Forest => ResourceNodeType::Lumbermill,
                Biome::Mountains => ResourceNodeType::Mine,
                Biome::Desert | Biome::Water => continue,
            };
            let produces = match node_type {
                ResourceNodeType::Farm => Resource::Wheat,
                ResourceNodeType::Mine => Resource::Ore,
                ResourceNodeType::Lumbermill => Resource::Lumber,
            };

            node_positions.push(node_pos);

            // Compensate for the change caused by adding as a child
            node_pos = node_pos - t_city.translation.xy();

            let wagon_timer = Timer::from_seconds(
                gameplay_settings.node_wagon_spawn_time,
                TimerMode::Repeating,
            )
            .tick(Duration::from_secs_f32(
                rng.random_range(0.0..gameplay_settings.node_wagon_spawn_time),
            ))
            .clone();

            let e_node = commands
                .spawn((
                    ResourceNode {
                        node_type,
                        produces: Shipment {
                            resource: produces,
                            quantity: 1,
                        },
                        sector: e_sector,
                        wagon_timer,
                        city: e_city,
                        road: Entity::PLACEHOLDER,
                    },
                    Transform::from_translation(node_pos.extend(0.0)),
                ))
                .id();
            commands.entity(e_city).add_child(e_node);
            city.resource_nodes.push(e_node);

            nodes_to_gen -= 1;
        }
    }
}

pub fn add_node_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_asset: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    node_query: Query<(Entity, &ResourceNode)>,
) {
    let mut materials: HashMap<ResourceNodeType, Handle<ColorMaterial>> = HashMap::new();
    materials.insert(
        ResourceNodeType::Farm,
        materials_asset.add(Color::srgb_u8(218, 165, 32)),
    );
    materials.insert(
        ResourceNodeType::Lumbermill,
        materials_asset.add(Color::srgb_u8(139, 69, 19)),
    );
    materials.insert(
        ResourceNodeType::Mine,
        materials_asset.add(Color::srgb_u8(112, 128, 144)),
    );
    let mesh = meshes.add(Circle::new(5.0));

    for (e_node, node) in node_query {
        commands.entity(e_node).insert((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(materials.get(&node.node_type).unwrap().clone()),
        ));
    }
}

pub fn debug_relations(
    mut gizmos: Gizmos,
    node_query: Query<(&ResourceNode, &GlobalTransform)>,
    city_query: Query<(&City, &GlobalTransform)>,
) {
    for (city, t_city) in city_query {
        for node in city.resource_nodes.clone() {
            let (_, t_node) = node_query.get(node).unwrap();
            gizmos.line_2d(
                t_city.translation().xy(),
                t_node.translation().xy(),
                Color::srgb_u8(210, 105, 30),
            );
        }
    }
}

pub fn spawn_node_wagons(
    time: Res<Time>,
    mut commands: Commands,
    node_query: Query<(&mut ResourceNode, &GlobalTransform)>,
) {
    for (mut node, t_node) in node_query {
        node.wagon_timer.tick(time.delta());
        if node.wagon_timer.finished() {
            commands.spawn((
                Wagon::new(node.produces, node.road, node.city),
                Transform::from_xyz(t_node.translation().x, t_node.translation().y, 0.0),
            ));
        }
    }
}
