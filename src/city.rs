use bevy::prelude::*;
use bevy_common_assets::csv::LoadedCsv;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use rand::Rng;

use crate::{
    city_names::{CityName, NameListHandle},
    clickable::{ClickHitbox, ClickState, JustPressed},
    exposer_tags::ExposerTag,
    format_text::{FormatText, TextSegmentType, ValueExposer},
    map::{Map, Sector},
    ui::UIWorldPosition,
    window::{WindowEntry, WindowEntryType, generate_window},
};

#[derive(Component)]
#[require(Transform)]
pub struct City {
    name: String,
    population: u32,
}

pub fn click_city(
    mut commands: Commands,
    query: Query<(Entity, &City, &Transform), With<JustPressed>>,
) {
    let Ok((e_city, city, t_city)) = query.single() else {
        return;
    };

    generate_window(
        commands.reborrow(),
        Val::Percent(25.0),
        Val::Percent(25.0),
        vec![
            WindowEntry {
                entry_type: WindowEntryType::Text {
                    text: city.name.to_string(),
                },
                centered: true,
                ..WindowEntry::default()
            },
            WindowEntry {
                entry_type: WindowEntryType::FormatText {
                    text: FormatText {
                        segments: vec![
                            TextSegmentType::Text {
                                text: "Population: ".to_string(),
                            },
                            TextSegmentType::ComponentValue {
                                entity: e_city,
                                tag: ExposerTag::CityPopulation,
                            },
                        ],
                    },
                },
                ..WindowEntry::default()
            },
        ],
        UIWorldPosition {
            pos: t_city.translation.xy() + Vec2::new(10.0, -10.0),
        },
    );
}

pub fn spawn_cities(
    name_list: Res<NameListHandle>,
    names: Res<Assets<LoadedCsv<CityName>>>,
    mut commands: Commands,
    mut map_query: Query<(&Map, &mut Entropy<WyRand>)>,
    sector_query: Query<&Sector>,
) {
    let (map, mut rng) = map_query.single_mut().unwrap();

    let mut city_positions: Vec<Vec2> = Vec::new();

    let names_list_unwrapped = names.get(&name_list.0).unwrap();
    let mut unused_name_indices: Vec<usize> = (0..names_list_unwrapped.rows.len()).collect();

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

        let name_index =
            unused_name_indices.swap_remove(rng.random_range(..unused_name_indices.len()));
        let name = names_list_unwrapped
            .rows
            .get(name_index)
            .unwrap()
            .name
            .to_string();

        city_positions.push(city_pos.clone());
        commands.spawn((
            City {
                name,
                population: rng.random_range(map.city_start_pop_range.clone()),
            },
            Transform::from_xyz(city_pos.x, city_pos.y, 1.0),
            ClickState::default(),
            ClickHitbox::Circle { radius: 10.0 },
            ValueExposer::default(),
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

pub fn expose_cities(query: Query<(&mut ValueExposer, &City), Changed<City>>) {
    for (mut exposer, city) in query {
        exposer
            .tags
            .insert(ExposerTag::CityPopulation, city.population.to_string());
    }
}
