use bevy::prelude::*;

mod background;
mod enemy;
mod platform;
mod player;
mod ui;

use crate::enemy::EnemyPlugin;
use background::BackgroundPlugin;
use platform::PlatformPlugin;
use player::PlayerPlugin;
use fuzzy_runner::{Distance, GameState, GameConfig, OnGameScreen, PlatformQueue};
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Platformer!".into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<Distance>()
        .insert_resource(GameConfig::default())
        .add_plugins((
            PlayerPlugin,
            PlatformPlugin,
            UiPlugin,
            EnemyPlugin,
            BackgroundPlugin,
        ))
        .add_systems(OnEnter(GameState::Restart), cleanup_game_session)
        .run();
}

fn cleanup_game_session(
    mut commands: Commands,
    game_screen_entities: Query<Entity, With<OnGameScreen>>,
    mut distance: ResMut<Distance>,
    mut platform_queue: ResMut<PlatformQueue>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // 1. Despawn all entities from the previous game session
    for entity in &game_screen_entities {
        commands.entity(entity).despawn_recursive();
    }

    // 2. Reset game-specific resources to their default values
    distance.0 = 0.0;
    platform_queue.0.clear();

    // 3. Immediately transition to the Playing state to start a new game
    next_state.set(GameState::Playing);
}