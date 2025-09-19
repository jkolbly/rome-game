use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{click_off::KillOnClickOff, format_text::FormatText, pointer_capture::CapturesPointer};

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

    /// Displays reactive text based on a template.
    FormatText { text: FormatText },
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
        let new_entity = match entry {
            WindowEntry::Text { text } => commands.spawn(Text::new(text)).id(),
            WindowEntry::FormatText { text } => text.generate(&mut commands),
        };
        commands.entity(e_window).add_child(new_entity);
    }

    e_window
}
