use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{mouse::MousePos, window::Window};

/// Component for UI elements that are killed when clicked off of.
#[derive(Component)]
#[require(RelativeCursorPosition)]
pub struct KillOnClickOff;

pub fn kill_on_click_off(
    mut commands: Commands,
    ui_query: Query<(Entity, &RelativeCursorPosition), (With<KillOnClickOff>, Without<Window>)>,
    window_query: Query<(&RelativeCursorPosition, &Window)>,
    parent_window_query: Query<Entity, (With<Window>, With<KillOnClickOff>)>,
) {
    for (entity, cursor_pos) in ui_query {
        if !cursor_pos.mouse_over() {
            commands.entity(entity).insert(Despawning);
        }
    }

    'outer: for e_parent in parent_window_query {
        let mut windows_to_check: Vec<Entity> = vec![e_parent];

        while let Some(e_window) = windows_to_check.pop() {
            let (cursor_pos, window) = window_query.get(e_window).unwrap();

            if cursor_pos.mouse_over() {
                continue 'outer;
            }

            for new_child in &window.subwindows {
                windows_to_check.push(*new_child);
            }
        }

        // If the above didn't continue to 'outer, no subwindows are hovered over and we kill this window
        commands.entity(e_parent).insert(Despawning);
    }
}

/// Marker component for entities despawning this frame.
#[derive(Component)]
pub struct Despawning;

pub fn despawn(mut commands: Commands, query: Query<Entity, With<Despawning>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
