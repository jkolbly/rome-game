use crate::{mouse::MousePos, pointer_capture::IsPointerCaptured};
use bevy::prelude::*;

/// A collision shape for mouse interaction.
#[derive(Component)]
#[require(Transform)]
pub enum ClickHitbox {
    /// A circle centered on the Transform.
    Circle { radius: f32 },

    /// No hitbox (not actually clickable). This is the default.
    None,
}

impl Default for ClickHitbox {
    fn default() -> Self {
        ClickHitbox::None
    }
}

impl ClickHitbox {
    /// Check whether a coordinate is within this hitbox, given the transform of this hitbox.
    fn check_hitbox(&self, coord: Vec2, transform: &Transform) -> bool {
        match self {
            ClickHitbox::Circle { radius } => {
                coord.distance_squared(transform.translation.xy()) <= radius * radius
            }
            ClickHitbox::None => false,
        }
    }
}

/// A component for all objects that can be interacted with the mouse.
#[derive(Component, Default)]
#[require(Transform, ClickHitbox)]
pub struct ClickState {
    just_pressed: bool,
    pressed: bool,
}

/// A component for the object that just started being clicked on this frame.
#[derive(Component)]
pub struct JustPressed {}

/// A component for the object that is currently being pressed.
#[derive(Component)]
pub struct Pressed {}

/// Set the data in all Clickable components.
pub fn update_clickables(
    pointer_captured: Res<IsPointerCaptured>,
    mouse_pos: Res<MousePos>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut ClickState, &Transform, &ClickHitbox)>,
) {
    for (mut state, _, _) in &mut query {
        state.just_pressed = false;
        state.pressed = false;
    }

    if pointer_captured.0 {
        return;
    }

    for (mut state, transform, hitbox) in query
        .iter_mut()
        .sort_by::<(&ClickState, &Transform, &ClickHitbox)>(
            |this: &(&ClickState, &Transform, &ClickHitbox),
             other: &(&ClickState, &Transform, &ClickHitbox)| {
                this.1
                    .translation
                    .z
                    .partial_cmp(&other.1.translation.z)
                    .unwrap()
                    .reverse()
            },
        )
    {
        if hitbox.check_hitbox(mouse_pos.pos, transform) {
            state.just_pressed = buttons.just_pressed(MouseButton::Left);
            state.pressed = buttons.pressed(MouseButton::Left);
            break;
        }
    }
}

/// Remove all click state components.
pub fn remove_click_components(mut commands: Commands, jp_query: Query<Entity, With<Pressed>>) {
    let Ok(entity) = jp_query.single() else {
        return;
    };
    commands.entity(entity).remove::<JustPressed>();
    commands.entity(entity).remove::<Pressed>();
}

/// Add click state components (JustPressed, etc.) based on Clickable components.
pub fn add_click_components(mut commands: Commands, query: Query<(Entity, &ClickState)>) {
    for (entity, state) in query {
        if state.just_pressed {
            commands.entity(entity).insert(JustPressed {});
        }
        if state.pressed {
            commands.entity(entity).insert(Pressed {});
        }
    }
}
