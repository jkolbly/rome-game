use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{click_off::KillOnClickOff, pointer_capture::CapturesPointer};

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {}

pub fn generate_window<T: Bundle>(
    mut commands: Commands,
    width: Val,
    height: Val,
    bundle: T,
) -> Entity {
    commands
        .spawn((
            Window {},
            Node {
                width,
                height,
                border: UiRect::all(Val::Px(2.0)),
                ..Node::default()
            },
            RelativeCursorPosition::default(),
            CapturesPointer,
            BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
            BorderRadius {
                top_left: Val::Percent(3.0),
                top_right: Val::Percent(3.0),
                bottom_left: Val::Percent(3.0),
                bottom_right: Val::Percent(3.0),
            },
            BorderColor(Color::BLACK),
            KillOnClickOff,
            bundle,
        ))
        .id()
}
