use bevy::prelude::*;
use fuzzy_runner::{
    AnimationIndices, AnimationTimer, Distance, GameState, OnGameScreen, Platform, Player,
    PlayerState, GRAVITY, PLATFORM_THICKNESS, PLAYER_JUMP_STRENGTH, PLAYER_SIZE, PLAYER_SPEED,
};

const PLAYER_ACCELERATION: f32 = 2000.0;
const PLAYER_DAMPING: f32 = 0.9;
const COYOTE_TIME_SECONDS: f32 = 0.1;
const JUMP_BUFFER_SECONDS: f32 = 0.1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnEnter(GameState::Paused), zero_player_velocity_on_pause)
            .add_systems(
                Update,
                (
                    update_player_state,
                    animate_sprite.after(update_player_state),
                    handle_input,
                    apply_forces,
                    apply_velocity.before(check_collisions),
                    check_collisions,
                    camera_follow_player.after(apply_velocity),
                    check_for_death.after(check_collisions),
                    update_distance,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_query: Query<Entity, With<Player>>,
) {
    if player_query.get_single().is_err() {
        let texture: Handle<Image> = asset_server.load("player_tilesheet.png");
        let layout = TextureAtlasLayout::from_grid(Vec2::new(80.0, 110.0), 9, 3, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let ground_y = -250.0;
        let player_start_y = ground_y + (PLATFORM_THICKNESS / 2.0) + (PLAYER_SIZE.y / 2.0);

        commands.spawn((
            SpriteSheetBundle {
                texture,
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0, // Start at the idle frame
                },
                transform: Transform::from_xyz(100.0, player_start_y, 1.0)
                    .with_scale(Vec3::new(0.70, 0.70, 1.0)), // Scale the sprite down a bit
                ..default()
            },
            Player {
                velocity: Vec2::ZERO,
                is_grounded: true,
                coyote_time: Timer::from_seconds(COYOTE_TIME_SECONDS, TimerMode::Once),
                jump_buffer: Timer::from_seconds(JUMP_BUFFER_SECONDS, TimerMode::Once),
                health: 100.0,
                state: PlayerState::Idle,
            },
            AnimationIndices { first: 0, last: 0 },
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // Faster timer for more frames
            OnGameScreen,
        ));
    }
}

fn update_player_state(mut query: Query<&mut Player>) {
    if let Ok(mut player) = query.get_single_mut() {
        // Determine the new state based on a clear priority:
        // 1. In the air (Jumping/Falling)
        // 2. On the ground and moving (Running)
        // 3. On the ground and still (Idle)
        let new_state = if !player.is_grounded {
            if player.velocity.y > 0.0 {
                PlayerState::Jumping
            } else {
                PlayerState::Falling
            }
        } else if player.velocity.x.abs() > 0.0 {
            PlayerState::Running
        } else {
            PlayerState::Idle
        };

        // Only update the state if it has changed
        if player.state != new_state {
            player.state = new_state;
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlas,
        &Player,
    )>,
) {
    for (mut indices, mut timer, mut atlas, player) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let (first, last) = match player.state {
                PlayerState::Idle => (0, 0),
                PlayerState::Running => (9, 10),
                PlayerState::Jumping => (1, 1),
                PlayerState::Falling => (2, 2),
            };

            if indices.first != first || indices.last != last {
                indices.first = first;
                indices.last = last;
                atlas.index = first;
            } else {
                if atlas.index == indices.last {
                    atlas.index = indices.first;
                } else {
                    atlas.index += 1;
                }
            }
        }
    }
}

fn update_distance(player_query: Query<&Transform, With<Player>>, mut distance: ResMut<Distance>) {
    if let Ok(player_transform) = player_query.get_single() {
        if player_transform.translation.x > distance.0 {
            distance.0 = player_transform.translation.x;
        }
    }
}

fn check_for_death(
    player_query: Query<(&Transform, &Player)>,
    camera_query: Query<&Transform, With<Camera>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok((player_transform, player)) = player_query.get_single() {
        if let Ok(camera_transform) = camera_query.get_single() {
            let screen_bottom_edge = camera_transform.translation.y - 400.0;

            if player_transform.translation.y < screen_bottom_edge || player.health <= 0.0 {
                next_state.set(GameState::GameOver);
            }
        }
    }
}

fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = player_transform.translation.x;
        }
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Sprite)>,
    time: Res<Time>,
) {
    if let Ok((mut player, mut sprite)) = player_query.get_single_mut() {
        player.coyote_time.tick(time.delta());
        player.jump_buffer.tick(time.delta());

        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
            sprite.flip_x = false; // Face right
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
            sprite.flip_x = true; // Face left
        }

        player.velocity.x += direction * PLAYER_ACCELERATION * time.delta_seconds();

        if keyboard_input.just_pressed(KeyCode::KeyW)
            || keyboard_input.just_pressed(KeyCode::ArrowUp)
            || keyboard_input.just_pressed(KeyCode::Space)
        {
            player.jump_buffer.reset();
        }

        if !player.jump_buffer.finished() && !player.coyote_time.finished() {
            player.velocity.y = PLAYER_JUMP_STRENGTH;
            player.is_grounded = false;
            player.jump_buffer.finished();
            player.coyote_time.finished();
        }
    }
}

fn zero_player_velocity_on_pause(mut query: Query<&mut Player>) {
    if let Ok(mut player) = query.get_single_mut() {
        player.velocity = Vec2::ZERO;
    }
}

fn apply_forces(mut player_query: Query<&mut Player>, time: Res<Time>) {
    if let Ok(mut player) = player_query.get_single_mut() {
        if !player.is_grounded {
            player.velocity.y -= GRAVITY * time.delta_seconds();
        }
        // Apply damping to the horizontal velocity
        player.velocity.x *= PLAYER_DAMPING;
        // Clamp the velocity to the maximum speed
        player.velocity.x = player.velocity.x.clamp(-PLAYER_SPEED, PLAYER_SPEED);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Player)>, time: Res<Time>) {
    for (mut transform, player) in &mut query {
        transform.translation.x += player.velocity.x * time.delta_seconds();
        transform.translation.y += player.velocity.y * time.delta_seconds();
    }
}

fn check_collisions(
    mut player_query: Query<(&mut Transform, &mut Player)>,
    platform_query: Query<&Transform, (With<Platform>, Without<Player>)>,
) {
    if let Ok((mut player_transform, mut player)) = player_query.get_single_mut() {
        let was_grounded = player.is_grounded;
        player.is_grounded = false;

        let player_size = PLAYER_SIZE;
        let player_pos = player_transform.translation;

        for platform_transform in &platform_query {
            let platform_size = Vec2::new(platform_transform.scale.x, platform_transform.scale.y);
            let platform_pos = platform_transform.translation;

            let x_collision = (player_pos.x - player_size.x / 2.0)
                < (platform_pos.x + platform_size.x / 2.0)
                && (player_pos.x + player_size.x / 2.0) > (platform_pos.x - platform_size.x / 2.0);
            let y_collision = (player_pos.y - player_size.y / 2.0)
                < (platform_pos.y + platform_size.y / 2.0)
                && (player_pos.y + player_size.y / 2.0) > (platform_pos.y - platform_size.y / 2.0);

            if x_collision && y_collision && player.velocity.y <= 0.0 {
                let penetration =
                    (platform_pos.y + platform_size.y / 2.0) - (player_pos.y - player_size.y / 2.0);
                if penetration > 0.0 {
                    player_transform.translation.y += penetration;
                    player.velocity.y = 0.0;
                    player.is_grounded = true;
                }
            }
        }

        // If we just landed, reset the coyote time timer
        if player.is_grounded && !was_grounded {
            player.coyote_time.reset();
        }
    }
}
