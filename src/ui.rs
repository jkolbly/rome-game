use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::pointer_capture::CapturesPointer;

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {}

/// Bundle for elements of a Window.
/// Use `WindowBundle::new()`` to construct a Window.
#[derive(Bundle)]
pub struct WindowBundle(Window, Node, RelativeCursorPosition, CapturesPointer);

impl WindowBundle {
    pub fn new(width: Val, height: Val) -> WindowBundle {
        WindowBundle(
            Window {},
            Node {
                width,
                height,
                ..Node::default()
            },
            RelativeCursorPosition::default(),
            CapturesPointer,
        )
    }
}

/// Component for UI displayed relative to the world.
#[derive(Component)]
#[require(Node)]
pub struct UIWorldPosition {
    pub pos: Vec2,
}

pub fn update_world_ui_positions(
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_node: Query<(&UIWorldPosition, &mut Node)>,
) {
    let (camera, t_camera) = q_camera.single().unwrap();

    for (pos, mut node) in q_node {
        let screen_pos = camera
            .world_to_viewport(t_camera, pos.pos.extend(0.0))
            .unwrap();
        node.left = Val::Px(screen_pos.x);
        node.top = Val::Px(screen_pos.y);
    }
}
