use std::collections::HashMap;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use rand::Rng;

use crate::{
    city::City,
    map::{Map, Sector},
    utils,
};

/// A type of non-global resource.
pub enum Resource {
    Wheat,
    Ore,
    Lumber,
}

/// A resource production node.
#[derive(Component)]
#[require(Transform)]
pub struct ResourceNode {
    node_type: ResourceNodeType,
    produces: Resource,
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
) {
    let (map, mut rng) = map_query.single_mut().unwrap();

    let mut node_positions: Vec<Vec2> = Vec::new();
    let city_positions: Vec<Vec2> = city_query
        .iter()
        .map(|(_, _, t_city)| t_city.translation.xy())
        .collect();

    for (e_city, mut city, t_city) in city_query {
        let mut nodes_to_gen = rng.random_range(map.nodes_per_city_range.clone());

        for _ in 0..1000 {
            if nodes_to_gen <= 0 {
                break;
            }

            let rand_index = rng.random_range(0..map.sectors.len());
            let rand_sector = map.sectors[rand_index];
            let mut node_pos = sector_query.get(rand_sector).unwrap().centroid;

            // Check not too close to a city
            if city_positions
                .iter()
                .any(|pos| pos.distance(node_pos) < map.node_city_min_dist)
            {
                continue;
            }

            // Check not too close to a node
            if node_positions
                .iter()
                .any(|pos| pos.distance(node_pos) < map.node_min_spacing)
            {
                continue;
            }

            // Check close enough to this city
            if node_pos.distance(t_city.translation.xy()) > map.node_city_max_dist {
                continue;
            }

            // Check not closer to a different city

            if city_positions.iter().any(|pos| {
                !utils::vec2_equals(pos, &t_city.translation.xy())
                    && node_pos.distance(*pos) < node_pos.distance(t_city.translation.xy())
            }) {
                continue;
            }

            let node_type: ResourceNodeType = match rng.random_range(0..3) {
                0 => ResourceNodeType::Farm,
                1 => ResourceNodeType::Lumbermill,
                2 => ResourceNodeType::Mine,
                _ => unreachable!(),
            };
            let produces = match node_type {
                ResourceNodeType::Farm => Resource::Wheat,
                ResourceNodeType::Mine => Resource::Ore,
                ResourceNodeType::Lumbermill => Resource::Lumber,
            };

            node_positions.push(node_pos);

            // Compensate for the change caused by adding as a child
            node_pos = node_pos - t_city.translation.xy();

            let e_node = commands
                .spawn((
                    ResourceNode {
                        node_type,
                        produces,
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
