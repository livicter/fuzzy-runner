mod background;

use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
    SettingsMenu,
    Restart,
}

#[derive(Component, PartialEq, Clone, Copy, Debug, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Running,
    Jumping,
    Falling,
}

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub is_grounded: bool,
    pub coyote_time: Timer,
    pub jump_buffer: Timer,
    pub health: f32,
    pub state: PlayerState,
}

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct Enemy {
    pub velocity: Vec2,
    pub is_grounded: bool,
}

#[derive(Resource)]
pub struct GameConfig {
    pub max_enemies: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { max_enemies: 3 }
    }
}

#[derive(Component)]
pub struct OnSettingsMenu;

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct OnGameScreen;

#[derive(Component)]
pub struct OnPauseMenu;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct DistanceText;

#[derive(Resource, Default)]
pub struct Distance(pub f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PlatformQueue(pub VecDeque<Entity>);

// --- CONSTANTS ---
pub const GRAVITY: f32 = 1600.0;
pub const PLAYER_JUMP_STRENGTH: f32 = 650.0;
pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_SIZE: Vec2 = Vec2::new(50.0, 80.0);
pub const PLATFORM_THICKNESS: f32 = 20.0;
pub const VIEWPORT_WIDTH: f32 = 800.0;

// --- ENEMY CONSTANTS ---
pub const ENEMY_SPEED: f32 = 270.0; // A bit slower than the player
pub const ENEMY_JUMP_STRENGTH: f32 = 650.0; // Can't jump as high as the player
pub const ENEMY_SIZE: Vec2 = PLAYER_SIZE; // Same size as player

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
