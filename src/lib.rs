#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use bevy::prelude::*;
use std::collections::VecDeque;

pub mod logic;
pub mod safe;

pub use safe::{safe_duration, safe_timer, set_text, try_despawn};

pub use logic::{
    award_coins, clamp_lane, combo_multiplier, countdown_label, displayed_meters, displayed_score,
    format_highscore, is_caught, is_new_highscore, lane_from_blend, milestone_crossed, next_lane,
    obstacle_cleared, parse_highscore, plan_segment, register_combo, resolve_hit, run_speed,
    threat_after_stumble, threat_recover, tick_combo, CoinAward, DeathReason, Difficulty,
    HitOutcome, ObstacleKind, PowerUpKind, SegmentPlan,
};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
    SettingsMenu,
    Restart,
    ReturnToMenu,
}

#[derive(Component, PartialEq, Clone, Copy, Debug, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Running,
    Jumping,
    Falling,
    Sliding,
    Stumbling,
}

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub is_grounded: bool,
    pub coyote_time: Timer,
    pub jump_buffer: Timer,
    pub slide_timer: Timer,
    pub stumble_timer: Timer,
    pub invincible_timer: Timer,
    pub state: PlayerState,
    pub lane: i32,
    pub lane_blend: f32,
    pub target_lane: i32,
    pub has_shield: bool,
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

#[derive(Component)]
pub struct Chaser {
    pub rank: u8,
}

#[derive(Resource, Clone, Debug)]
pub struct GameConfig {
    pub difficulty: Difficulty,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct RunStats {
    pub distance: f32,
    pub coins: u32,
    pub coin_points: u32,
    pub multiplier_timer: f32,
    pub magnet_timer: f32,
    pub boost_timer: f32,
    pub threat: f32,
    pub combo: u32,
    pub combo_timer: f32,
    pub best_combo: u32,
}

impl RunStats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn multiplier_active(&self) -> bool {
        self.multiplier_timer > 0.0
    }

    pub fn magnet_active(&self) -> bool {
        self.magnet_timer > 0.0
    }

    pub fn boost_active(&self) -> bool {
        self.boost_timer > 0.0
    }

    pub fn score(&self) -> u64 {
        displayed_score(self.distance, self.coin_points)
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct LastRun {
    pub distance: f32,
    pub coins: u32,
    pub score: u64,
    pub reason: DeathReason,
    pub new_high_score: bool,
    pub best_combo: u32,
}

#[derive(Resource, Clone, Debug)]
pub struct HighScores {
    pub score: u64,
    pub coins: u32,
    pub distance: f32,
}

impl Default for HighScores {
    fn default() -> Self {
        Self::load()
    }
}

impl HighScores {
    pub fn path() -> std::path::PathBuf {
        std::env::var("HOME")
            .map(|home| std::path::PathBuf::from(home).join(".fuzzy_runner_highscore"))
            .unwrap_or_else(|_| std::path::PathBuf::from("fuzzy_runner_highscore.txt"))
    }

    pub fn load() -> Self {
        let Ok(contents) = std::fs::read_to_string(Self::path()) else {
            return Self {
                score: 0,
                coins: 0,
                distance: 0.0,
            };
        };
        parse_highscore(&contents)
            .map(|(score, coins, distance)| Self {
                score,
                coins,
                distance,
            })
            .unwrap_or(Self {
                score: 0,
                coins: 0,
                distance: 0.0,
            })
    }

    pub fn save(&self) {
        let _ = std::fs::write(
            Self::path(),
            format_highscore(self.score, self.coins, self.distance),
        );
    }

    pub fn submit(&mut self, score: u64, coins: u32, distance: f32) -> bool {
        if is_new_highscore(self.score, score) {
            self.score = score;
            self.coins = coins;
            self.distance = distance;
            self.save();
            true
        } else {
            false
        }
    }
}

#[derive(Resource, Default)]
pub struct SettingsOrigin(pub GameState);

#[derive(Component)]
pub struct OnSettingsMenu;

#[derive(Component)]
pub struct OnMainMenu;

#[derive(Component)]
pub struct OnGameOverMenu;

#[derive(Component)]
pub struct Platform {
    pub size: Vec2,
}

#[derive(Component)]
pub struct Bob {
    pub base_y: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct ParticleBurst {
    pub velocity: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct ThreatFill;

#[derive(Component)]
pub struct Decor;

#[derive(Component)]
pub struct CoinSpin {
    pub timer: Timer,
    pub frame: usize,
}

#[derive(Component)]
pub struct DustPuff {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PauseButton;

#[derive(Component)]
pub struct GoSplash {
    pub timer: Timer,
}

#[derive(Component)]
pub struct MilestoneToast {
    pub timer: Timer,
}

#[derive(Component)]
pub struct ScreenFlash {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PowerChip {
    pub kind: PowerUpKind,
}

#[derive(Component)]
pub struct CoinHudPunch {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Aura {
    pub kind: AuraKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AuraKind {
    Shield,
    Magnet,
}

#[derive(Component)]
pub struct Orbit {
    pub angle: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct TorchFlicker {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Spin {
    pub speed: f32,
}

#[derive(Component)]
pub struct ChaserWarn;

#[derive(Component)]
pub struct OnGameScreen;

#[derive(Component)]
pub struct OnPauseMenu;

#[derive(Component)]
pub struct DistanceText;

#[derive(Component)]
pub struct CoinText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct TitlePreview;

#[derive(Component)]
pub struct Vignette;

#[derive(Component)]
pub struct TouchControl(pub RunnerCommand);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunnerCommand {
    LaneLeft,
    LaneRight,
    Jump,
    Slide,
}

#[derive(Resource, Default)]
pub struct PendingCommands {
    pub commands: Vec<RunnerCommand>,
}

impl PendingCommands {
    pub fn push(&mut self, command: RunnerCommand) {
        self.commands.push(command);
    }

    pub fn take(&mut self) -> Vec<RunnerCommand> {
        std::mem::take(&mut self.commands)
    }
}

#[derive(Resource, Default)]
pub struct IgnoreSwipe(pub bool);

#[derive(Resource)]
pub struct Countdown {
    pub remaining: f32,
}

impl Default for Countdown {
    fn default() -> Self {
        Self {
            remaining: COUNTDOWN_DURATION,
        }
    }
}

impl Countdown {
    pub fn reset(&mut self) {
        self.remaining = COUNTDOWN_DURATION;
    }

    pub fn active(&self) -> bool {
        self.remaining > 0.0
    }
}

#[derive(Resource, Default)]
pub struct CameraImpulse {
    pub trauma: f32,
}

impl CameraImpulse {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct Coin {
    pub value: u32,
    pub lane: i32,
}

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
    pub lane: i32,
}

#[derive(Component)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub lane: i32,
    pub size: Vec2,
}

#[derive(Component)]
pub struct FloatingPopup {
    pub timer: Timer,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct DeathEvent {
    pub reason: DeathReason,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct CoinCollected {
    pub amount: u32,
    pub points: u32,
    pub combo: u32,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct PowerUpCollected {
    pub kind: PowerUpKind,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct PlayerStumbled;

#[derive(Resource, Default)]
pub struct Distance(pub f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PlatformQueue(pub VecDeque<Entity>);

#[derive(Resource)]
pub struct TrackCursor {
    pub next_x: f32,
}

impl Default for TrackCursor {
    fn default() -> Self {
        Self {
            next_x: INITIAL_TRACK_END,
        }
    }
}

// --- WORLD ---
pub const GRAVITY: f32 = 1800.0;
pub const PLAYER_JUMP_STRENGTH: f32 = 720.0;
pub const PLAYER_SIZE: Vec2 = Vec2::new(46.0, 78.0);
pub const PLAYER_SLIDE_SIZE: Vec2 = Vec2::new(54.0, 36.0);
pub const PLATFORM_THICKNESS: f32 = 22.0;
pub const VIEWPORT_WIDTH: f32 = 1280.0;
pub const GROUND_Y: f32 = -230.0;
pub const INITIAL_TRACK_END: f32 = 900.0;
pub const TRACK_LOOKAHEAD: f32 = 1100.0;
pub const LANE_COUNT: i32 = 3;
pub const LANE_Z: [f32; 3] = [3.0, 1.4, 0.2];
pub const LANE_SCALE: [f32; 3] = [0.80, 0.70, 0.60];
pub const LANE_SWITCH_SPEED: f32 = 14.0;
pub const SLIDE_DURATION: f32 = 0.55;
pub const STUMBLE_DURATION: f32 = 0.55;
pub const INVINCIBLE_DURATION: f32 = 0.85;
pub const CAMERA_LOOKAHEAD: f32 = 220.0;
pub const MAGNET_RADIUS: f32 = 170.0;
pub const CHASER_FAR_GAP: f32 = 340.0;
pub const CHASER_NEAR_GAP: f32 = 58.0;
pub const MAX_RUN_SPEED: f32 = 720.0;
pub const COUNTDOWN_DURATION: f32 = 3.2;
pub const COMBO_WINDOW: f32 = 1.25;
pub const SHAKE_DECAY: f32 = 3.2;

// --- ENEMY ---
pub const ENEMY_SPEED: f32 = 270.0;
pub const ENEMY_JUMP_STRENGTH: f32 = 650.0;
pub const ENEMY_SIZE: Vec2 = PLAYER_SIZE;

pub const NEON_CYAN: Color = Color::rgb(0.15, 0.92, 1.0);
pub const NEON_MAGENTA: Color = Color::rgb(1.0, 0.25, 0.72);
pub const NEON_GOLD: Color = Color::rgb(1.0, 0.84, 0.22);
pub const NEON_PURPLE: Color = Color::rgb(0.62, 0.28, 1.0);
pub const NEON_LIME: Color = Color::rgb(0.45, 1.0, 0.38);
pub const ROOF_COLOR: Color = Color::rgb(0.16, 0.12, 0.24);
pub const ROOF_EDGE: Color = Color::rgb(0.28, 0.86, 1.0);

pub fn lane_z(lane: i32) -> f32 {
    *LANE_Z.get(clamp_lane(lane) as usize).unwrap_or(&1.4)
}

pub fn lane_scale(lane: i32) -> f32 {
    *LANE_SCALE.get(clamp_lane(lane) as usize).unwrap_or(&0.70)
}

pub fn player_hitbox(player: &Player) -> Vec2 {
    if player.state == PlayerState::Sliding {
        PLAYER_SLIDE_SIZE
    } else {
        PLAYER_SIZE
    }
}

pub fn player_collision_pos(player_pos: Vec3, player: &Player) -> Vec3 {
    if player.state == PlayerState::Sliding {
        player_pos - Vec3::new(0.0, 20.0, 0.0)
    } else {
        player_pos
    }
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        try_despawn(&mut commands, entity);
    }
}

pub fn aabb_overlap(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    (a_pos.x - a_size.x / 2.0) < (b_pos.x + b_size.x / 2.0)
        && (a_pos.x + a_size.x / 2.0) > (b_pos.x - b_size.x / 2.0)
        && (a_pos.y - a_size.y / 2.0) < (b_pos.y + b_size.y / 2.0)
        && (a_pos.y + a_size.y / 2.0) > (b_pos.y - b_size.y / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_lookups_never_index_out_of_range() {
        for lane in [-99, 0, 1, 2, 99] {
            let _ = lane_z(lane);
            let _ = lane_scale(lane);
        }
        assert!((lane_z(1) - 1.4).abs() < f32::EPSILON);
        assert!((lane_scale(1) - 0.70).abs() < f32::EPSILON);
    }
}
