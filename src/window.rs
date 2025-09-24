use std::collections::HashSet;

use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{
    click_off::{Despawning, KillOnClickOff},
    format_text::FormatText,
    pointer_capture::CapturesPointer,
    ui::UIWorldPosition,
};

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {
    pub subwindows: HashSet<Entity>,
    pub parent_window: Option<Entity>,
}

impl Window {
    fn new() -> Window {
        Window {
            subwindows: HashSet::new(),
            parent_window: None,
        }
    }

    pub fn with_parent(mut self, parent: Option<Entity>) -> Window {
        self.parent_window = parent;
        self
    }
}

/// Helper struct for building a [`Window`]
#[derive(Clone)]
pub struct WindowBuilder {
    width: Val,
    height: Val,
    entries: Vec<EntryBuilder>,
    world_anchor: Option<UIWorldPosition>,
    click_off: bool,
    parent_window: Option<Entity>,
    left: Val,
    top: Val,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            width: Val::ZERO,
            height: Val::ZERO,
            entries: Vec::new(),
            world_anchor: None,
            click_off: false,
            parent_window: None,
            left: Val::ZERO,
            top: Val::ZERO,
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

    pub fn click_off(mut self) -> WindowBuilder {
        self.click_off = true;
        self
    }

    pub fn parent(mut self, parent: Entity) -> WindowBuilder {
        self.parent_window = Some(parent);
        self
    }

    pub fn left(mut self, left: Val) -> WindowBuilder {
        self.left = left;
        self
    }

    pub fn top(mut self, top: Val) -> WindowBuilder {
        self.top = top;
        self
    }

    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        let e_window = commands
            .spawn((
                Window::new().with_parent(self.parent_window),
                Node {
                    width: self.width,
                    height: self.height,
                    border: UiRect::all(Val::Px(2.0)),
                    display: Display::Block,
                    padding: UiRect::all(Val::Px(5.0)),
                    left: self.left,
                    top: self.top,
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
            ))
            .id();

        if self.click_off {
            commands.entity(e_window).insert(KillOnClickOff);
        }

        if let Some(anchor) = self.world_anchor.clone() {
            commands.entity(e_window).insert(anchor);
        }

        for entry in &self.entries {
            let e_entry = entry.spawn(commands);
            commands.entity(e_window).add_child(e_entry);
        }

        e_window
    }
}

/// Helper struct for building a [`WindowEntry`]
#[derive(Clone)]
pub struct EntryBuilder {
    entry_type: EntryType,
    centered: bool,
    subwindow: Option<WindowBuilder>,
}

impl EntryBuilder {
    pub fn text(text: &str) -> EntryBuilder {
        EntryBuilder {
            entry_type: EntryType::Text {
                text: text.to_string(),
            },
            centered: false,
            subwindow: None,
        }
    }

    pub fn formatted_text(text: FormatText) -> EntryBuilder {
        EntryBuilder {
            entry_type: EntryType::FormatText { text },
            centered: false,
            subwindow: None,
        }
    }

    pub fn button(text: &str) -> EntryBuilder {
        EntryBuilder {
            entry_type: EntryType::Button {
                text: text.to_string(),
            },
            centered: false,
            subwindow: None,
        }
    }

    pub fn centered(mut self) -> EntryBuilder {
        self.centered = true;
        self
    }

    /// Open another window when this is clicked
    pub fn open_subwindow(mut self, window: WindowBuilder) -> EntryBuilder {
        self.subwindow = Some(window);
        self
    }

    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        let new_node = Node { ..Node::default() };
        let layout = TextLayout::new_with_justify(if self.centered {
            JustifyText::Center
        } else {
            JustifyText::Left
        });
        let e_entry = match &self.entry_type {
            EntryType::Text { text } => commands.spawn((Text::new(text), new_node, layout)).id(),
            EntryType::FormatText { text } => text.spawn(commands, (new_node, layout)),
            EntryType::Button { text } => commands
                .spawn((Button, new_node, layout, children![Text::new(text)]))
                .id(),
        };

        if let Some(window) = &self.subwindow {
            commands.entity(e_entry).insert(WindowTogglerButton {
                entity: None,
                window: window.clone(),
            });
        }

        e_entry
    }
}

#[derive(Clone)]
enum EntryType {
    /// Displays a fixed line of text.
    Text { text: String },

    /// Displays reactive text based on a template.
    FormatText { text: FormatText },

    /// Displays a button with a text label
    Button { text: String },
}

/// Component for a UI button that toggles another window's visibility
#[derive(Component)]
pub struct WindowTogglerButton {
    entity: Option<Entity>,
    window: WindowBuilder,
}

pub fn toggle_visibility_buttons(
    mut commands: Commands,
    button_query: Query<
        (Entity, &ChildOf, &mut WindowTogglerButton, &Interaction),
        Changed<Interaction>,
    >,
    mut window_query: Query<&mut Window>,
) {
    for (e_button, child_of, mut button, interaction) in button_query {
        match interaction {
            Interaction::Pressed => {
                match button.entity {
                    Some(entity) => {
                        commands.entity(entity).insert(Despawning);
                        button.entity = None;
                    }
                    None => {
                        let e_window = button
                            .window
                            .clone()
                            .parent(child_of.parent())
                            .spawn(&mut commands);
                        button.entity = Some(e_window);

                        window_query
                            .get_mut(child_of.parent())
                            .unwrap()
                            .subwindows
                            .insert(e_window);
                    }
                };
            }
            _ => {}
        }
    }
}

pub fn despawn_subwindows(
    mut commands: Commands,
    window_query: Query<&Window>,
    despawning_query: Query<Entity, (With<Window>, With<Despawning>)>,
) {
    for e_parent in despawning_query {
        let mut windows_to_check: Vec<Entity> = vec![e_parent];

        while let Some(e_window) = windows_to_check.pop() {
            let window = window_query.get(e_window).unwrap();
            for child in &window.subwindows {
                windows_to_check.push(*child);
            }
            commands.entity(e_window).insert(Despawning);
        }
    }
}

pub fn update_parent_subwindows(
    mut window_query: Query<&mut Window, Without<Despawning>>,
    despawning_query: Query<(Entity, &Window), With<Despawning>>,
) {
    for (e_window, window) in despawning_query {
        let mut curr_window = window;

        while let Some(parent) = curr_window.parent_window {
            if let Ok(mut parent_window) = window_query.get_mut(parent) {
                parent_window.subwindows.remove(&e_window);
            } else {
                break;
            }
            curr_window = window_query.get(parent).unwrap();
        }

        if let Some(parent) = window.parent_window {
            if let Ok(mut parent_window) = window_query.get_mut(parent) {
                parent_window.subwindows.remove(&e_window);
            }
        }
    }
}
