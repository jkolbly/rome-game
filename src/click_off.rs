use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::mouse::MousePos;

/// Component for UI elements that are killed when clicked off of.
#[derive(Component)]
#[require(RelativeCursorPosition)]
pub struct KillOnClickOff;

pub fn kill_on_click_off(
    mut commands: Commands,
    ui_query: Query<(Entity, &RelativeCursorPosition), With<KillOnClickOff>>,
) {
    for (entity, cursor_pos) in ui_query {
        if !cursor_pos.mouse_over() {
            commands.entity(entity).despawn();
        }
    }
}
