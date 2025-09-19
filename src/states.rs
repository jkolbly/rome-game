use bevy::prelude::*;
use bevy_common_assets::csv::LoadedCsv;

use crate::city_names::{CityName, NameListHandle};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Loading,
    InGame,
}

pub fn check_loaded(
    asset_server: Res<AssetServer>,
    name_list: Res<NameListHandle>,
    names: Res<Assets<LoadedCsv<CityName>>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if asset_server.is_loaded(name_list.0.id()) {
        next_app_state.set(AppState::InGame);
    }
}
