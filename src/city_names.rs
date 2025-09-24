use bevy::{
    asset::AssetLoader,
    prelude::*,
};
use bevy_common_assets::csv::LoadedCsv;
use serde::Deserialize;

#[derive(Asset, Reflect, Deserialize)]
pub struct CityName {
    pub name: String,
}

#[derive(Resource)]
pub struct NameListHandle(pub Handle<LoadedCsv<CityName>>);

pub fn load_name_list(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(NameListHandle(asset_server.load("city-names.csv")));
}
