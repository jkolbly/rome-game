use bevy::{input::InputSystem, prelude::*};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;

mod input;
mod map;
mod utils;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<input::MousePos>()
            .add_systems(Startup, (map::create_map, map::generate_map).chain())
            .add_systems(Startup, add_camera)
            .add_systems(PreUpdate, input::update_mouse_pos.after(InputSystem))
            .add_systems(Update, input::mouse_button_input);
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
        Transform::from_xyz(250.0, 250.0, 0.0),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(GamePlugin)
        .run();
}
