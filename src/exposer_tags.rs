use bevy::prelude::*;

/// Tags for referencing items in a [`ValueExposer`](crate::format_text::ValueExposer).
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum ExposerTag {
    CityPopulation,
}
