use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::mouse::MousePos;

/// True iff the mouse pointer is over a UI element that
/// prevents it from interacting with the world.
#[derive(Resource, Default)]
pub struct IsPointerCaptured(pub bool);

/// Marker component for UI elements that capture the pointer.
#[derive(Component)]
#[require(RelativeCursorPosition)]
pub struct CapturesPointer;

pub fn update_pointer_capture(
    mut pointer_captured: ResMut<IsPointerCaptured>,
    ui_query: Query<&RelativeCursorPosition, With<CapturesPointer>>,
) {
    pointer_captured.0 = ui_query.iter().any(|cursor_pos| cursor_pos.mouse_over());
    println!("Pointer captured: {}", pointer_captured.0);
}
