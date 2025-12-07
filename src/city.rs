use bevy::{platform::collections::HashMap, prelude::*};
use bevy_common_assets::csv::LoadedCsv;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use rand::Rng;

use crate::{
    biome::Biome,
    city_names::{CityName, NameListHandle},
    clickable::{ClickHitbox, ClickState, JustPressed},
    demographic::{Demographic, JobType, Population},
    exposer_tags::ExposerTag,
    format_text::{FormatText, ValueExposer},
    map::{Map, Sector},
    settings::MapGenSettings,
    shipment::ShipmentReceiver,
    window::{EntryBuilder, WindowBuilder},
};

#[derive(Component)]
#[require(Transform, ShipmentReceiver)]
pub struct City {
    pub name: String,
    pub resource_nodes: Vec<Entity>,
    pub sector: Entity,

    pub demographics: HashMap<JobType, Entity>,
}

pub fn click_city(
    mut commands: Commands,
    query: Query<(Entity, &City, &Transform), With<JustPressed>>,
) {
    let Ok((e_city, city, t_city)) = query.single() else {
        return;
    };

    WindowBuilder::new()
        .width(Val::Percent(20.0))
        .height(Val::Auto)
        .add_entry(EntryBuilder::text(&city.name).centered())
        .add_entry(EntryBuilder::formatted_text(
            FormatText::new()
                .add_text("Population: ")
                .add_component_value(e_city, ExposerTag::CityPopulation),
        ))
        .add_entry(
            EntryBuilder::button("Open Subwindow")
                .open_subwindow(
                    WindowBuilder::new()
                        .width(Val::Percent(15.0))
                        .height(Val::Percent(15.0))
                        .left(Val::Percent(5.0))
                        .top(Val::Percent(10.0))
                        .add_entry(EntryBuilder::text("Subwindow!").centered())
                        .add_entry(
                            EntryBuilder::button("Another One...").open_subwindow(
                                WindowBuilder::new()
                                    .left(Val::Percent(5.0))
                                    .top(Val::Percent(30.0))
                                    .width(Val::Percent(15.0))
                                    .height(Val::Percent(15.0))
                                    .add_entry(EntryBuilder::text("Subwindow 2!").centered()),
                            ),
                        ),
                )
                .centered(),
        )
        .anchored(t_city.translation.xy() + Vec2::new(10.0, -10.0))
        .click_off()
        .spawn(&mut commands);
}

pub fn spawn_cities(
    name_list: Res<NameListHandle>,
    names: Res<Assets<LoadedCsv<CityName>>>,
    mut commands: Commands,
    mut map_query: Query<(&Map, &mut Entropy<WyRand>)>,
    sector_query: Query<&Sector>,
    settings: Res<MapGenSettings>,
) {
    let (map, mut rng) = map_query.single_mut().unwrap();

    let mut city_positions: Vec<Vec2> = Vec::new();

    let names_list_unwrapped = names.get(&name_list.0).unwrap();
    let mut unused_name_indices: Vec<usize> = (0..names_list_unwrapped.rows.len()).collect();

    for _ in 0..1000 {
        let rand_index = rng.random_range(0..map.sectors.len());
        let e_sector = map.sectors[rand_index];
        let sector = sector_query.get(e_sector).unwrap();
        let city_pos = sector.centroid;

        // Check not within deadzone
        if city_pos.x < settings.city_deadzone
            || city_pos.x > settings.size.x - settings.city_deadzone
            || city_pos.y < settings.city_deadzone
            || city_pos.y > settings.size.y - settings.city_deadzone
        {
            continue;
        }

        // Check not too close to another city
        if city_positions.iter().any(|p| {
            p.distance_squared(city_pos) < settings.city_min_spacing * settings.city_min_spacing
        }) {
            continue;
        }

        // Check in a valid biome
        match sector.biome.unwrap() {
            Biome::Forest | Biome::Mountains | Biome::Water => {
                continue;
            }
            Biome::Plains | Biome::Desert => {}
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
        let total_pop = rng.random_range(settings.city_start_pop_range.clone());
        let mut demographics = HashMap::new();
        demographics.insert(
            JobType::Unemployed,
            commands
                .spawn(Demographic {
                    population: total_pop,
                    job: JobType::Unemployed,
                })
                .id(),
        );
        let e_city = commands
            .spawn((
                City {
                    name,
                    resource_nodes: Vec::new(),
                    sector: e_sector,
                    demographics: demographics.clone(),
                },
                Population {
                    population: total_pop,
                },
                Transform::from_xyz(city_pos.x, city_pos.y, 1.0),
                ClickState::default(),
                ClickHitbox::Circle { radius: 10.0 },
                ValueExposer::default(),
                Visibility::Visible,
                ShipmentReceiver::new(),
            ))
            .id();

        for e_demo in demographics.values() {
            commands.entity(e_city).add_child(*e_demo);
        }

        if city_positions.len() as u32 >= settings.city_num {
            break;
        }
    }
}

pub fn add_city_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    city_query: Query<Entity, With<City>>,
) {
    let material = materials.add(Color::BLACK);
    let mesh = meshes.add(Circle::new(10.0));

    for e_city in city_query {
        commands
            .entity(e_city)
            .insert((Mesh2d(mesh.clone()), MeshMaterial2d(material.clone())));
    }
}

pub fn expose_cities(query: Query<(&mut ValueExposer, &Population), Changed<City>>) {
    for (mut exposer, pop) in query {
        exposer
            .tags
            .insert(ExposerTag::CityPopulation, pop.population.to_string());
    }
}

pub fn receive_city_shipments(query: Query<(&City, &mut ShipmentReceiver)>) {
    for (city, mut receiver) in query {
        while let Some(shipment) = receiver.get_shipment() {
            println!(
                "{} received shipment of {} {:?}",
                city.name, shipment.quantity, shipment.resource
            );
        }
    }
}
