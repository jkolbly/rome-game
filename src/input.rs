use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
pub struct MousePos {
    pub pos: Vec2,
    pub pos_scaled: Vec2,
    pub delta: Vec2,
    pub delta_scaled: Vec2,
}

pub fn update_mouse_pos(
    mut position: ResMut<MousePos>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection)>,
) {
    let window = window_query.single().unwrap();
    let (camera, transform, projection) = camera_query.single().unwrap();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Projection::Orthographic(ortho) = projection else {
        return;
    };

    let new_pos_scaled = cursor_pos * ortho.scale * vec2(1.0, -1.0);
    position.delta_scaled = new_pos_scaled - position.pos_scaled;
    position.pos_scaled = new_pos_scaled;

    let new_pos = camera.viewport_to_world_2d(transform, cursor_pos).unwrap();
    position.delta = new_pos - position.pos;
    position.pos = new_pos;
}

pub fn mouse_button_input(
    position: Res<MousePos>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let mut camera = camera_query.single_mut().unwrap();
    if buttons.pressed(MouseButton::Right) && !buttons.just_pressed(MouseButton::Right) {
        camera.translation -= position.delta_scaled.extend(0.0);
    }
}
