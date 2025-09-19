use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{click_off::KillOnClickOff, format_text::FormatText, pointer_capture::CapturesPointer};

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {}

/// Component for an entry in a UI window.
#[derive(Component)]
#[require(Node)]
pub struct WindowEntry {
    pub entry_type: WindowEntryType,
    pub centered: bool,
}

impl Default for WindowEntry {
    fn default() -> Self {
        Self {
            entry_type: WindowEntryType::Text {
                text: "".to_string(),
            },
            centered: false,
        }
    }
}

pub enum WindowEntryType {
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
                display: Display::Block,
                padding: UiRect::all(Val::Px(5.0)),
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
        let new_node = Node { ..Node::default() };
        let layout = TextLayout::new_with_justify(if entry.centered {
            JustifyText::Center
        } else {
            JustifyText::Left
        });
        let new_entity = match entry.entry_type {
            WindowEntryType::Text { text } => {
                commands.spawn((Text::new(text), new_node, layout)).id()
            }
            WindowEntryType::FormatText { text } => {
                text.generate(&mut commands, (new_node, layout))
            }
        };
        commands.entity(e_window).add_child(new_entity);
    }

    e_window
}
