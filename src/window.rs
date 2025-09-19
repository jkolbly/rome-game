use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{click_off::KillOnClickOff, pointer_capture::CapturesPointer};

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {}

/// Component for an entry in a UI window.
#[derive(Component)]
#[require(Node)]
pub enum WindowEntry {
    /// Displays a fixed line of text.
    Text { text: String },
}

pub fn generate_window<T: Bundle>(
    mut commands: Commands,
    width: Val,
    height: Val,
    entries: Vec<WindowEntry>,
    bundle: T,
) -> Entity {
    let e_window = commands
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
        .id();

    for entry in entries {
        let entry_bundle = match entry {
            WindowEntry::Text { text } => (Text::new(text)),
        };

        let entry_entity = commands.spawn(entry_bundle).id();
        commands.entity(e_window).add_child(entry_entity);
    }

    e_window
}
