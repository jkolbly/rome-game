use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::InputSystem,
    prelude::*,
};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;

mod biome;
mod city;
mod clickable;
mod keyboard;
mod map;
mod mouse;
mod pointer_capture;
mod ui;
mod utils;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<mouse::MousePos>()
            .init_resource::<pointer_capture::IsPointerCaptured>()
            .add_systems(
                Startup,
                (
                    map::create_map,
                    map::generate_map,
                    city::spawn_cities,
                    city::add_city_meshes,
                )
                    .chain(),
            )
            .add_systems(Startup, add_camera)
            .add_systems(
                PreUpdate,
                (
                    mouse::update_mouse_pos,
                    pointer_capture::update_pointer_capture,
                    (
                        clickable::update_clickables,
                        clickable::remove_click_components,
                    ),
                    clickable::add_click_components,
                )
                    .chain()
                    .after(InputSystem),
            )
            .add_systems(
                Update,
                (
                    mouse::mouse_button_input,
                    mouse::scroll_events,
                    city::click_city,
                    ui::update_world_ui_positions,
                ),
            );
        // .add_systems(Update, map::draw_debug);
    }
}

fn add_camera(mut commands: Commands) {
    let ortho = OrthographicProjection {
        scale: 1.0,
        ..OrthographicProjection::default_2d()
    };
    commands.spawn((
        Camera2d,
        Projection::Orthographic(ortho),
        Transform::from_xyz(500.0, 250.0, 0.0),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(GamePlugin)
        .add_plugins((
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ))
        .run();
}
