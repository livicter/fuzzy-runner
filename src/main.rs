use bevy::prelude::*;

mod assets;
mod background;
mod collectibles;
mod enemy;
mod platform;
mod player;
mod ui;

use crate::assets::AssetsPlugin;
use crate::enemy::EnemyPlugin;
use background::BackgroundPlugin;
use collectibles::CollectiblesPlugin;
use fuzzy_runner::{
    Distance, GameConfig, GameState, HighScores, LastRun, OnGameScreen, PlatformQueue, RunStats,
    SettingsOrigin, TrackCursor,
};
use platform::PlatformPlugin;
use player::PlayerPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cyber Temple — Rooftop Run".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<Distance>()
        .init_resource::<RunStats>()
        .init_resource::<LastRun>()
        .init_resource::<HighScores>()
        .init_resource::<SettingsOrigin>()
        .init_resource::<TrackCursor>()
        .insert_resource(GameConfig::default())
        .add_plugins((
            AssetsPlugin,
            PlayerPlugin,
            PlatformPlugin,
            CollectiblesPlugin,
            UiPlugin,
            EnemyPlugin,
            BackgroundPlugin,
        ))
        .add_systems(OnEnter(GameState::Restart), cleanup_then_replay)
        .add_systems(OnEnter(GameState::ReturnToMenu), cleanup_then_menu)
        .run();
}

fn reset_session(
    commands: &mut Commands,
    game_screen_entities: &Query<Entity, With<OnGameScreen>>,
    distance: &mut Distance,
    platform_queue: &mut PlatformQueue,
    stats: &mut RunStats,
    track_cursor: &mut TrackCursor,
) {
    for entity in game_screen_entities {
        commands.entity(entity).despawn_recursive();
    }
    distance.0 = 0.0;
    platform_queue.0.clear();
    stats.reset();
    *track_cursor = TrackCursor::default();
}

fn cleanup_then_replay(
    mut commands: Commands,
    game_screen_entities: Query<Entity, With<OnGameScreen>>,
    mut distance: ResMut<Distance>,
    mut platform_queue: ResMut<PlatformQueue>,
    mut stats: ResMut<RunStats>,
    mut track_cursor: ResMut<TrackCursor>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    reset_session(
        &mut commands,
        &game_screen_entities,
        &mut distance,
        &mut platform_queue,
        &mut stats,
        &mut track_cursor,
    );
    next_state.set(GameState::Playing);
}

fn cleanup_then_menu(
    mut commands: Commands,
    game_screen_entities: Query<Entity, With<OnGameScreen>>,
    mut distance: ResMut<Distance>,
    mut platform_queue: ResMut<PlatformQueue>,
    mut stats: ResMut<RunStats>,
    mut track_cursor: ResMut<TrackCursor>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    reset_session(
        &mut commands,
        &game_screen_entities,
        &mut distance,
        &mut platform_queue,
        &mut stats,
        &mut track_cursor,
    );
    next_state.set(GameState::Menu);
}
