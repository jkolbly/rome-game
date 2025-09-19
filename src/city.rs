use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use rand::Rng;

use crate::{
    clickable::{ClickHitbox, ClickState, JustPressed},
    map::{Map, Sector},
    ui::{self, WindowBundle},
};

#[derive(Component)]
#[require(Transform)]
pub struct City {
    population: u32,
}

pub fn click_city(mut commands: Commands, query: Query<(&City, &Transform), With<JustPressed>>) {
    let Ok((city, t_city)) = query.single() else {
        return;
    };

    commands.spawn((
        ui::UIWorldPosition {
            pos: t_city.translation.xy(),
        },
        WindowBundle::new(Val::Percent(25.0), Val::Percent(25.0)),
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        BorderRadius {
            top_left: Val::Percent(3.0),
            top_right: Val::Percent(3.0),
            bottom_left: Val::Percent(3.0),
            bottom_right: Val::Percent(3.0),
        },
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::BLACK),
    ));
}

pub fn spawn_cities(
    mut commands: Commands,
    mut map_query: Query<(&Map, &mut Entropy<WyRand>)>,
    sector_query: Query<&Sector>,
) {
    let (map, mut rng) = map_query.single_mut().unwrap();

    let mut city_positions: Vec<Vec2> = Vec::new();

    for _ in 0..1000 {
        let rand_index = rng.random_range(0..map.sectors.len());
        let rand_sector = map.sectors[rand_index];
        let city_pos = sector_query.get(rand_sector).unwrap().centroid;

        if city_pos.x < map.city_deadzone
            || city_pos.x > map.size.x - map.city_deadzone
            || city_pos.y < map.city_deadzone
            || city_pos.y > map.size.y - map.city_deadzone
        {
            continue;
        }

        if city_positions
            .iter()
            .any(|p| p.distance_squared(city_pos) < map.city_min_spacing * map.city_min_spacing)
        {
            continue;
        }

        city_positions.push(city_pos.clone());
        commands.spawn((
            City {
                population: rng.random_range(map.city_start_pop_range.clone()),
            },
            Transform::from_xyz(city_pos.x, city_pos.y, 1.0),
            ClickState::default(),
            ClickHitbox::Circle { radius: 10.0 },
        ));

        if city_positions.len() as u32 >= map.city_num {
            break;
        }
    }
}

pub fn add_city_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    city_query: Query<(Entity, &City)>,
) {
    let material = materials.add(Color::BLACK);
    let mesh = meshes.add(Circle::new(10.0));

    for (entity, city) in city_query {
        commands
            .entity(entity)
            .insert((Mesh2d(mesh.clone()), MeshMaterial2d(material.clone())));
    }
}
