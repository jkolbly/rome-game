use std::sync::Mutex;

use bevy::{asset::AssetLoader, platform::collections::HashSet, prelude::*};
use bevy_common_assets::csv::LoadedCsv;
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use lazy_static::lazy_static;
use rand::Rng;
use serde::Deserialize;

lazy_static! {
    static ref UNUSED_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

#[derive(Asset, Reflect, Deserialize)]
pub struct CityName {
    pub name: String,
}

#[derive(Resource)]
pub struct NameListHandle(pub Handle<LoadedCsv<CityName>>);

pub fn load_name_list(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(NameListHandle(asset_server.load("city-names.csv")));
}

pub fn set_unused_names(name_list: Res<NameListHandle>, names: Res<Assets<LoadedCsv<CityName>>>) {
    let names_list_unwrapped = names.get(&name_list.0).unwrap();
    let mut unused_names = UNUSED_NAMES.lock().unwrap();
    for name in &names_list_unwrapped.rows {
        unused_names.push(name.name.to_string());
    }
}

/// Return a random name and mark it as having been used.
pub fn get_name(rng: &mut Entropy<WyRand>) -> String {
    let mut names = UNUSED_NAMES.lock().unwrap();
    let index = rng.random_range(..names.len());
    let name = names[index].to_string();
    names[index] = names.last().unwrap().to_string();
    names.pop();
    name
}
