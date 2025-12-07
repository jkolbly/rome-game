use std::time::Duration;

use bevy::prelude::*;

use crate::settings::SaveSettings;

/// Component that manages saving game data on a regular interval.
/// Intended to be a singleton component.
#[derive(Component)]
pub struct SaveManager {
    timer: Timer,
}

pub fn tick_save_manager(time: Res<Time>, mut query: Query<&mut SaveManager>) {
    let Ok(mut saver) = query.single_mut() else {
        return;
    };
    saver.timer.tick(time.delta());

    if saver.timer.finished() {
        println!("Saving...");
    }
}

pub fn spawn_save_manager(mut commands: Commands, settings: Res<SaveSettings>) {
    commands.spawn(SaveManager {
        timer: Timer::new(
            Duration::from_secs_f32(settings.save_interval),
            TimerMode::Repeating,
        ),
    });
}
