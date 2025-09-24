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
mod demographic;
mod exposer_tags;
mod format_text;
mod keyboard;
mod map;
mod mouse;
mod pointer_capture;
mod resource;
mod road;
mod settings;
mod states;
mod ui;
mod utils;
mod window;

#[derive(Parser, Debug, Resource)]
#[command(about, long_about = None)]
pub struct Args {
    /// Display debug gizmos.
    #[arg(short, long)]
    debug: bool,

    /// Display node connections.
    #[arg(short = 'r', long = "debug-relations")]
    debug_relations: bool,

    /// Display road gizmos.
    #[arg(long = "debug-roads")]
    debug_roads: bool,

    /// Display performance metrics.
    #[arg(short, long)]
    performance: bool,

    /// The seed to use for map generation.
    #[arg(short, long)]
    seed: u64,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let args = Args::parse();

        if args.debug {
            app.add_systems(
                Update,
                map::draw_debug.run_if(in_state(states::AppState::InGame)),
            );
        }

        if args.debug_relations {
            app.add_systems(
                Update,
                resource::debug_relations.run_if(in_state(states::AppState::InGame)),
            );
        }

        if args.debug_roads {
            app.add_systems(
                Update,
                road::debug_roads.run_if(in_state(states::AppState::InGame)),
            );
        }

        if args.performance {
            app.add_plugins((
                LogDiagnosticsPlugin::default(),
                FrameTimeDiagnosticsPlugin::default(),
                bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            ));
        }

        app.insert_state(states::AppState::Loading)
            .init_resource::<mouse::MousePos>()
            .init_resource::<pointer_capture::IsPointerCaptured>()
            .insert_resource(args)
            .insert_resource(settings::MapGenSettings {
                size: vec2(1000.0, 500.0),

                sector_num: 16000,

                lloyd_iters: 5,
                generator_border: 20.0,
                altitude_perlin_scale: 0.004,
                water_cutoff: -0.6,

                biome_seed_num: 120,

                city_num: 10,
                city_min_spacing: 100.0,
                city_start_pop_range: 10..1000,
                city_deadzone: 20.0,

                nodes_per_city_range: 1..3,
                node_city_max_dist: 100.0,
                node_city_min_dist: 20.0,
                node_min_spacing: 70.0,
                node_deadzone: 10.0,
            })
            .insert_resource(settings::DisplaySettings { road_width: 4.0 })
            .add_systems(
                OnEnter(states::AppState::InGame),
                (
                    map::create_map,
                    map::generate_map,
                    biome::generate_biomes,
                    (map::add_map_mesh, city::spawn_cities),
                    (city::add_city_meshes, resource::spawn_resource_nodes),
                    resource::add_node_meshes,
                    road::spawn_node_roads,
                    road::add_road_meshes,
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
                    demographic::update_demographics,
                    demographic::update_city_pop.after(demographic::update_demographics),
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
