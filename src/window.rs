use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{
    click_off::KillOnClickOff, format_text::FormatText, pointer_capture::CapturesPointer,
    ui::UIWorldPosition,
};

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {}

/// Helper struct for building a [`Window`]
pub struct WindowBuilder {
    width: Val,
    height: Val,
    entries: Vec<EntryBuilder>,
    world_anchor: Option<UIWorldPosition>,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            width: Val::ZERO,
            height: Val::ZERO,
            entries: Vec::new(),
            world_anchor: None,
        }
    }

    pub fn width(mut self, width: Val) -> WindowBuilder {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Val) -> WindowBuilder {
        self.height = height;
        self
    }

    pub fn add_entry(mut self, entry: EntryBuilder) -> WindowBuilder {
        self.entries.push(entry);
        self
    }

    /// Anchor to a position in the world.
    pub fn anchored(mut self, pos: Vec2) -> WindowBuilder {
        self.world_anchor = Some(UIWorldPosition { pos });
        self
    }

    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let e_window = commands
            .spawn((
                Window {},
                Node {
                    width: self.width,
                    height: self.height,
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
            ))
            .id();

        if let Some(anchor) = self.world_anchor {
            commands.entity(e_window).insert(anchor);
        }

        for entry in self.entries {
            let e_entry = entry.spawn(commands);
            commands.entity(e_window).add_child(e_entry);
        }

        e_window
    }
}

/// Helper struct for building a [`WindowEntry`]
pub struct EntryBuilder {
    entry_type: EntryType,
    centered: bool,
}

impl EntryBuilder {
    pub fn text(text: &str) -> EntryBuilder {
        EntryBuilder {
            entry_type: EntryType::Text {
                text: text.to_string(),
            },
            centered: false,
        }
    }

    pub fn formatted_text(text: FormatText) -> EntryBuilder {
        EntryBuilder {
            entry_type: EntryType::FormatText { text },
            centered: false,
        }
    }

    pub fn centered(mut self) -> EntryBuilder {
        self.centered = true;
        self
    }

    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let new_node = Node { ..Node::default() };
        let layout = TextLayout::new_with_justify(if self.centered {
            JustifyText::Center
        } else {
            JustifyText::Left
        });
        match self.entry_type {
            EntryType::Text { text } => commands.spawn((Text::new(text), new_node, layout)).id(),
            EntryType::FormatText { text } => text.spawn(commands, (new_node, layout)),
        }
    }
}

enum EntryType {
    /// Displays a fixed line of text.
    Text { text: String },

    /// Displays reactive text based on a template.
    FormatText { text: FormatText },
}
