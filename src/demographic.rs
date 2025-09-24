use bevy::{platform::collections::HashSet, prelude::*};

use crate::city::City;

/// A group of people within a city
#[derive(Component)]
pub struct Demographic {
    pub population: u32,
    pub job: JobType,
}

/// The different jobs a demographic can have
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum JobType {
    /// Politicians control the city's government
    Politician,

    /// Soldiers execute the politican's orders
    Soldier,

    Unemployed,
}

/// Set the population field for all City structs based on their demographics
pub fn update_city_pop(
    changed_query: Query<&ChildOf, Changed<Demographic>>,
    demo_query: Query<&Demographic>,
    mut city_query: Query<&mut City>,
) {
    let cities: HashSet<Entity> = changed_query
        .iter()
        .map(|childof| childof.parent())
        .collect();

    for e_city in cities {
        let mut city = city_query.get_mut(e_city).unwrap();

        let mut population = 0;
        for e_demo in city.demographics.values() {
            let demo = demo_query.get(*e_demo).unwrap();
            population += demo.population;
        }

        city.population = population;
    }
}

pub fn update_demographics(demo_query: Query<&mut Demographic>) {}
