use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::{InputSystem, common_conditions::input_just_pressed},
    prelude::*,
};
use bevy_common_assets::csv::CsvAssetPlugin;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use clap::Parser;

mod biome;
mod city;
mod city_names;
mod click_off;
mod clickable;
mod exposer_tags;
mod format_text;
mod keyboard;
mod map;
mod mouse;
mod pointer_capture;
mod states;
mod ui;
mod utils;
mod window;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
pub struct Args {
    /// Display debug gizmos.
    #[arg(short, long)]
    debug: bool,

    /// Display performance metrics.
    #[arg(short, long)]
    performance: bool,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(states::AppState::Loading)
            .init_resource::<mouse::MousePos>()
            .init_resource::<pointer_capture::IsPointerCaptured>()
            .add_systems(
                OnEnter(states::AppState::InGame),
                (
                    map::create_map,
                    map::generate_map,
                    city::spawn_cities,
                    city::add_city_meshes,
                )
                    .chain(),
            )
            .add_systems(Startup, (add_camera, city_names::load_name_list))
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
                    click_off::kill_on_click_off.run_if(input_just_pressed(MouseButton::Left)),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    ui::update_world_ui_positions,
                    city::expose_cities.before(format_text::update_text_segments),
                    format_text::update_text_segments,
                ),
            )
            .add_systems(
                Update,
                states::check_loaded.run_if(in_state(states::AppState::Loading)),
            );

        let args = Args::parse();

        if args.debug {
            app.add_systems(
                Update,
                map::draw_debug.run_if(in_state(states::AppState::InGame)),
            );
        }

        if args.performance {
            app.add_plugins((
                LogDiagnosticsPlugin::default(),
                FrameTimeDiagnosticsPlugin::default(),
                bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            ));
        }
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
        .add_plugins((
            DefaultPlugins,
            EntropyPlugin::<WyRand>::default(),
            CsvAssetPlugin::<city_names::CityName>::new(&["csv"]),
            GamePlugin,
        ))
        .run();
}
