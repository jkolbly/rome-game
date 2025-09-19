use std::collections::HashMap;

use bevy::prelude::*;

use crate::exposer_tags::ExposerTag;

/// Struct for spawning reactive text made up of segments.
pub struct FormatText {
    pub segments: Vec<TextSegmentType>,
}

impl FormatText {
    pub fn generate(&self, commands: &mut Commands) -> Entity {
        commands
            .spawn(Text::default())
            .with_children(|parent| {
                for segment in &self.segments {
                    match segment {
                        TextSegmentType::Text { text } => parent.spawn(TextSpan::new(text)),
                        TextSegmentType::ComponentValue { entity, tag } => parent.spawn((
                            TextSpan::default(),
                            TextUpdater {
                                entity: *entity,
                                tag: *tag,
                            },
                        )),
                    };
                }
            })
            .id()
    }
}

/// Type of data to display in a segment of a [`FormatText`].
pub enum TextSegmentType {
    /// Displays a static piece of text.
    Text { text: String },

    /// Displays a value with a tag from a component on an entity.
    /// The entity must have an attached [`ValueExposer`] component.
    ComponentValue { entity: Entity, tag: ExposerTag },
}

#[derive(Component)]
#[require(TextSpan)]
pub struct TextUpdater {
    entity: Entity,
    tag: ExposerTag,
}

/// Component for exposing data from an entity to a [`FormatText`].
#[derive(Component, Default)]
pub struct ValueExposer {
    /// Maps tags to the data corresponding to them.
    pub tags: HashMap<ExposerTag, String>,
}

/// Update all text components.
/// Note that this is run for all [`TextUpdater`] components each frame.
pub fn update_text_segments(
    segment_query: Query<(&TextUpdater, &mut TextSpan)>,
    exposer_query: Query<&ValueExposer>,
) {
    for (updater, mut span) in segment_query {
        let exposer = exposer_query.get(updater.entity).unwrap();
        span.0 = exposer.tags.get(&updater.tag).unwrap().to_string();
    }
}
