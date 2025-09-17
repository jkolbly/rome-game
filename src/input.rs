use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
pub struct MousePos {
    pub pos: Vec2,
    pub pos_scaled: Vec2,
    pub pos_window: Vec2,
    pub delta: Vec2,
    pub delta_scaled: Vec2,
    pub delta_window: Vec2,
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

    position.delta_window = cursor_pos - position.pos_window;
    position.pos_window = cursor_pos;

    position.delta_scaled = position.delta_window * ortho.scale * vec2(1.0, -1.0);
    position.pos_scaled = vec2(cursor_pos.x, 1.0 - cursor_pos.y) * ortho.scale;

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

pub fn scroll_events(
    position: Res<MousePos>,
    mut evr_scroll: EventReader<MouseWheel>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Projection, &mut Transform, &Camera, &GlobalTransform)>,
) {
    let window = window_query.single().unwrap();
    let (mut projection, mut transform, camera, global_transform) =
        camera_query.single_mut().unwrap();
    let Projection::Orthographic(ortho) = &mut *projection else {
        return;
    };
    for event in evr_scroll.read() {
        // Window size (world) before zoom
        let old_size = window.size() * ortho.scale;

        ortho.scale *= (-event.y * 0.075).exp();

        // Window size (world) after zoom
        let new_size = window.size() * ortho.scale;

        // Mouse position (world) after zoom
        let new_mouse_pos = camera
            .viewport_to_world_2d(global_transform, position.pos_window)
            .unwrap();

        // Ratios of pixels to be added/subtracted to left/right or top/bottom of cursor
        let side_ratio = (new_mouse_pos - transform.translation.xy()) / new_size;

        // Difference in number of pixels between new and old zoom
        let pixel_diff = new_size - old_size;

        transform.translation -= (pixel_diff * side_ratio).extend(0.0);
    }
}
