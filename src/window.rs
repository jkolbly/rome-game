use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::pointer_capture::CapturesPointer;

/// Component for a standard UI window.
#[derive(Component)]
#[require(Node)]
pub struct Window {}

pub fn generate_window(width: Val, height: Val) -> impl Bundle {
    (
        Window {},
        Node {
            width,
            height,
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
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::BLACK),
    )
}
