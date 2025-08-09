use fuzzy_runner::{
    GameState, OnGameScreen, Platform, PlatformQueue, Player, PLATFORM_THICKNESS, VIEWPORT_WIDTH,
};
use bevy::prelude::*;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlatformQueue>()
            .add_systems(OnEnter(GameState::Playing), setup_platforms)
            .add_systems(
                Update,
                manage_platforms.run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_platforms(
    mut commands: Commands,
    mut platform_queue: ResMut<PlatformQueue>,
    platform_query: Query<Entity, With<Platform>>,
) {
    if platform_query.iter().next().is_none() {
        platform_queue.0.clear();

        let first_platform = spawn_platform(&mut commands, Vec2::new(0.0, -250.0), 800.0);
        platform_queue.0.push_back(first_platform);

        let second_platform = spawn_platform(&mut commands, Vec2::new(500.0, -150.0), 200.0);
        platform_queue.0.push_back(second_platform);
    }
}

pub fn spawn_platform(commands: &mut Commands, position: Vec2, width: f32) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.4, 0.1),
                    ..default()
                },
                transform: Transform {
                    translation: position.extend(0.0),
                    scale: Vec3::new(width, PLATFORM_THICKNESS, 1.0),
                    ..default()
                },
                ..default()
            },
            Platform,
            OnGameScreen,
        ))
        .id()
}

fn manage_platforms(
    mut commands: Commands,
    mut platform_queue: ResMut<PlatformQueue>,
    player_query: Query<&Transform, With<Player>>,
    platform_query: Query<&Transform, With<Platform>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        // Despawn old platforms
        if let Some(&first_platform_entity) = platform_queue.front() {
            if let Ok(platform_transform) = platform_query.get(first_platform_entity) {
                let platform_right_edge =
                    platform_transform.translation.x + (platform_transform.scale.x / 2.0);
                let screen_left_edge =
                    player_transform.translation.x - (VIEWPORT_WIDTH / 2.0) - 50.0;
                if platform_right_edge < screen_left_edge {
                    commands.entity(first_platform_entity).despawn_recursive();
                    platform_queue.pop_front();
                }
            }
        }

        // Spawn new platforms
        if let Some(&last_platform_entity) = platform_queue.back() {
            if let Ok(platform_transform) = platform_query.get(last_platform_entity) {
                let platform_right_edge =
                    platform_transform.translation.x + (platform_transform.scale.x / 2.0);
                let screen_right_edge = player_transform.translation.x + (VIEWPORT_WIDTH / 2.0);
                if platform_right_edge < screen_right_edge {
                    let new_x = platform_right_edge + (rand::random::<f32>() * 150.0) + 100.0;
                    let new_y = -250.0 + (rand::random::<f32>() * 200.0);
                    let new_width = 100.0 + (rand::random::<f32>() * 150.0);
                    let new_platform_entity =
                        spawn_platform(&mut commands, Vec2::new(new_x, new_y), new_width);
                    platform_queue.push_back(new_platform_entity);
                }
            }
        }
    }
}
