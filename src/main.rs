#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use bevy::prelude::*;

mod assets;
mod background;
mod collectibles;
mod enemy;
mod fx;
mod platform;
mod player;
mod ui;

use crate::assets::AssetsPlugin;
use crate::enemy::EnemyPlugin;
use crate::fx::FxPlugin;
use background::BackgroundPlugin;
use collectibles::CollectiblesPlugin;
use fuzzy_runner::{
    try_despawn, CameraImpulse, Countdown, Distance, GameConfig, GameState, HighScores,
    IgnoreSwipe, LastRun, OnGameScreen, PendingCommands, PlatformQueue, RunStats, SettingsOrigin,
    TrackCursor,
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
        .init_resource::<Countdown>()
        .init_resource::<CameraImpulse>()
        .init_resource::<PendingCommands>()
        .init_resource::<IgnoreSwipe>()
        .insert_resource(GameConfig::default())
        .add_plugins((
            AssetsPlugin,
            PlayerPlugin,
            PlatformPlugin,
            CollectiblesPlugin,
            FxPlugin,
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
    countdown: &mut Countdown,
    shake: &mut CameraImpulse,
    pending: &mut PendingCommands,
) {
    for entity in game_screen_entities {
        try_despawn(commands, entity);
    }
    distance.0 = 0.0;
    platform_queue.0.clear();
    stats.reset();
    *track_cursor = TrackCursor::default();
    countdown.reset();
    shake.trauma = 0.0;
    pending.commands.clear();
}

fn cleanup_then_replay(
    mut commands: Commands,
    game_screen_entities: Query<Entity, With<OnGameScreen>>,
    mut distance: ResMut<Distance>,
    mut platform_queue: ResMut<PlatformQueue>,
    mut stats: ResMut<RunStats>,
    mut track_cursor: ResMut<TrackCursor>,
    mut countdown: ResMut<Countdown>,
    mut shake: ResMut<CameraImpulse>,
    mut pending: ResMut<PendingCommands>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    reset_session(
        &mut commands,
        &game_screen_entities,
        &mut distance,
        &mut platform_queue,
        &mut stats,
        &mut track_cursor,
        &mut countdown,
        &mut shake,
        &mut pending,
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
    mut countdown: ResMut<Countdown>,
    mut shake: ResMut<CameraImpulse>,
    mut pending: ResMut<PendingCommands>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    reset_session(
        &mut commands,
        &game_screen_entities,
        &mut distance,
        &mut platform_queue,
        &mut stats,
        &mut track_cursor,
        &mut countdown,
        &mut shake,
        &mut pending,
    );
    next_state.set(GameState::Menu);
}
