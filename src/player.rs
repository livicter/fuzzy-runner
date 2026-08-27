use bevy::prelude::*;
use fuzzy_runner::{
    aabb_overlap, clamp_lane, lane_from_blend, lane_scale, lane_z, next_lane, player_collision_pos,
    player_hitbox, resolve_hit, run_speed, tick_combo, AnimationIndices, AnimationTimer,
    CameraImpulse, Countdown, DeathEvent, DeathReason, Distance, GameConfig, GameState, HitOutcome,
    IgnoreSwipe, OnGameScreen, PendingCommands, Platform, Player, PlayerState, PlayerStumbled,
    RunnerCommand, RunStats, CAMERA_LOOKAHEAD, GRAVITY, GROUND_Y, INVINCIBLE_DURATION,
    LANE_SWITCH_SPEED, PLATFORM_THICKNESS, PLAYER_JUMP_STRENGTH, PLAYER_SIZE, SHAKE_DECAY,
    SLIDE_DURATION, STUMBLE_DURATION,
};

const COYOTE_TIME_SECONDS: f32 = 0.12;
const JUMP_BUFFER_SECONDS: f32 = 0.12;
const SWIPE_THRESHOLD: f32 = 42.0;

#[derive(Resource, Default)]
struct SwipeTracker {
    origin: Option<Vec2>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SwipeTracker>()
            .add_event::<DeathEvent>()
            .add_event::<PlayerStumbled>()
            .add_systems(OnEnter(GameState::Playing), (reset_run_start, spawn_player).chain())
            .add_systems(
                Update,
                (
                    tick_run_timers,
                    handle_actions,
                    update_player_state,
                    animate_sprite.after(update_player_state),
                    apply_forces,
                    apply_velocity.before(check_collisions),
                    check_collisions,
                    camera_follow_player.after(apply_velocity),
                    flash_invincibility,
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
    if player_query.get_single().is_ok() {
        return;
    }

    let texture: Handle<Image> = asset_server.load("player_tilesheet.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(80.0, 110.0), 9, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_start_y = GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (PLAYER_SIZE.y / 2.0);

    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 9,
            },
            transform: Transform::from_xyz(80.0, player_start_y, lane_z(1))
                .with_scale(Vec3::splat(lane_scale(1))),
            ..default()
        },
        Player {
            velocity: Vec2::ZERO,
            is_grounded: true,
            coyote_time: Timer::from_seconds(COYOTE_TIME_SECONDS, TimerMode::Once),
            jump_buffer: Timer::from_seconds(JUMP_BUFFER_SECONDS, TimerMode::Once),
            slide_timer: Timer::from_seconds(SLIDE_DURATION, TimerMode::Once),
            stumble_timer: Timer::from_seconds(STUMBLE_DURATION, TimerMode::Once),
            invincible_timer: Timer::from_seconds(INVINCIBLE_DURATION, TimerMode::Once),
            state: PlayerState::Running,
            lane: 1,
            lane_blend: 1.0,
            target_lane: 1,
            has_shield: false,
        },
        AnimationIndices { first: 9, last: 10 },
        AnimationTimer(Timer::from_seconds(0.08, TimerMode::Repeating)),
        OnGameScreen,
    ));
}

fn tick_run_timers(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut stats: ResMut<RunStats>,
    mut countdown: ResMut<Countdown>,
) {
    let dt = time.delta_seconds();
    if countdown.active() {
        countdown.remaining = (countdown.remaining - dt).max(0.0);
    }
    stats.multiplier_timer = (stats.multiplier_timer - dt).max(0.0);
    stats.magnet_timer = (stats.magnet_timer - dt).max(0.0);
    stats.boost_timer = (stats.boost_timer - dt).max(0.0);
    let (combo, timer) = tick_combo(stats.combo, stats.combo_timer, dt);
    stats.combo = combo;
    stats.combo_timer = timer;
    if stats.threat < 1.0 {
        stats.threat =
            fuzzy_runner::threat_recover(stats.threat, dt, config.difficulty.recover_rate());
    }
}

fn swipe_action(start: Vec2, end: Vec2) -> Option<RunnerCommand> {
    let delta = end - start;
    if delta.length() < SWIPE_THRESHOLD {
        return None;
    }
    if delta.x.abs() > delta.y.abs() {
        if delta.x < 0.0 {
            Some(RunnerCommand::LaneLeft)
        } else {
            Some(RunnerCommand::LaneRight)
        }
    } else if delta.y < 0.0 {
        // Screen space: y grows downward, so a swipe up has negative y.
        Some(RunnerCommand::Jump)
    } else {
        Some(RunnerCommand::Slide)
    }
}

fn reset_run_start(
    player_query: Query<Entity, With<Player>>,
    mut countdown: ResMut<Countdown>,
    mut shake: ResMut<CameraImpulse>,
) {
    if player_query.get_single().is_ok() {
        return;
    }
    countdown.reset();
    shake.trauma = 0.0;
}

fn handle_actions(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    touches: Res<Touches>,
    mut swipe: ResMut<SwipeTracker>,
    mut pending: ResMut<PendingCommands>,
    mut ignore_swipe: ResMut<IgnoreSwipe>,
    mut player_query: Query<&mut Player>,
    time: Res<Time>,
) {
    let Ok(mut player) = player_query.get_single_mut() else {
        return;
    };

    player.coyote_time.tick(time.delta());
    player.jump_buffer.tick(time.delta());
    player.slide_timer.tick(time.delta());
    player.stumble_timer.tick(time.delta());
    player.invincible_timer.tick(time.delta());

    if player.state == PlayerState::Stumbling && !player.stumble_timer.finished() {
        pending.take();
        return;
    }

    let mut queued = pending.take();
    if keyboard_input.just_pressed(KeyCode::KeyA) || keyboard_input.just_pressed(KeyCode::ArrowLeft)
    {
        queued.push(RunnerCommand::LaneLeft);
    }
    if keyboard_input.just_pressed(KeyCode::KeyD)
        || keyboard_input.just_pressed(KeyCode::ArrowRight)
    {
        queued.push(RunnerCommand::LaneRight);
    }
    if keyboard_input.just_pressed(KeyCode::KeyW)
        || keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::Space)
    {
        queued.push(RunnerCommand::Jump);
    }
    if keyboard_input.just_pressed(KeyCode::KeyS) || keyboard_input.just_pressed(KeyCode::ArrowDown)
    {
        queued.push(RunnerCommand::Slide);
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.get_single() {
            swipe.origin = window.cursor_position();
        }
    }
    if mouse_input.just_released(MouseButton::Left) {
        let skip = ignore_swipe.0;
        ignore_swipe.0 = false;
        if !skip {
            if let (Some(start), Ok(window)) = (swipe.origin.take(), windows.get_single()) {
                if let Some(end) = window.cursor_position() {
                    if let Some(action) = swipe_action(start, end) {
                        queued.push(action);
                    }
                }
            }
        } else {
            swipe.origin = None;
        }
    }
    if !ignore_swipe.0 {
        for touch in touches.iter_just_released() {
            if let Some(action) = swipe_action(touch.start_position(), touch.position()) {
                queued.push(action);
            }
        }
    }

    for action in queued {
        match action {
            RunnerCommand::LaneLeft => {
                player.target_lane = next_lane(player.target_lane, -1);
            }
            RunnerCommand::LaneRight => {
                player.target_lane = next_lane(player.target_lane, 1);
            }
            RunnerCommand::Jump => {
                player.jump_buffer.reset();
            }
            RunnerCommand::Slide => {
                if player.is_grounded {
                    player.slide_timer.reset();
                    player.state = PlayerState::Sliding;
                }
            }
        }
    }

    if !player.jump_buffer.finished() && !player.coyote_time.finished() {
        player.velocity.y = PLAYER_JUMP_STRENGTH;
        player.is_grounded = false;
        player.slide_timer.tick(std::time::Duration::from_secs(2));
        player.jump_buffer.tick(std::time::Duration::from_secs(2));
        player.coyote_time.tick(std::time::Duration::from_secs(2));
    }
}

fn update_player_state(mut query: Query<&mut Player>) {
    if let Ok(mut player) = query.get_single_mut() {
        if !player.stumble_timer.finished() {
            player.state = PlayerState::Stumbling;
            return;
        }
        if !player.slide_timer.finished() && player.is_grounded {
            player.state = PlayerState::Sliding;
            return;
        }
        player.state = if !player.is_grounded {
            if player.velocity.y > 0.0 {
                PlayerState::Jumping
            } else {
                PlayerState::Falling
            }
        } else {
            PlayerState::Running
        };
        player.lane = clamp_lane(player.target_lane);
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
        let speed_factor = if player.state == PlayerState::Running {
            0.07
        } else {
            0.10
        };
        timer.set_duration(std::time::Duration::from_secs_f32(speed_factor));
        timer.tick(time.delta());
        if timer.just_finished() {
            let (first, last) = match player.state {
                PlayerState::Idle => (0, 0),
                PlayerState::Running => (9, 10),
                PlayerState::Jumping => (1, 1),
                PlayerState::Falling => (2, 2),
                PlayerState::Sliding => (3, 3),
                PlayerState::Stumbling => (4, 4),
            };

            if indices.first != first || indices.last != last {
                indices.first = first;
                indices.last = last;
                atlas.index = first;
            } else if atlas.index == indices.last {
                atlas.index = indices.first;
            } else {
                atlas.index += 1;
            }
        }
    }
}

fn update_distance(
    player_query: Query<&Transform, With<Player>>,
    mut distance: ResMut<Distance>,
    mut stats: ResMut<RunStats>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if player_transform.translation.x > distance.0 {
            distance.0 = player_transform.translation.x;
        }
        if player_transform.translation.x > stats.distance {
            stats.distance = player_transform.translation.x;
        }
    }
}

fn check_for_death(
    player_query: Query<(&Transform, &Player)>,
    camera_query: Query<&Transform, With<Camera>>,
    stats: Res<RunStats>,
    mut next_state: ResMut<NextState<GameState>>,
    mut deaths: EventWriter<DeathEvent>,
) {
    if let Ok((player_transform, _player)) = player_query.get_single() {
        if let Ok(camera_transform) = camera_query.get_single() {
            let screen_bottom_edge = camera_transform.translation.y - 420.0;
            if player_transform.translation.y < screen_bottom_edge {
                deaths.send(DeathEvent {
                    reason: DeathReason::Fell,
                });
                next_state.set(GameState::GameOver);
            }
        }
        if fuzzy_runner::is_caught(stats.threat) {
            deaths.send(DeathEvent {
                reason: DeathReason::Caught,
            });
            next_state.set(GameState::GameOver);
        }
    }
}

fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut shake: ResMut<CameraImpulse>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            shake.trauma = (shake.trauma - time.delta_seconds() * SHAKE_DECAY).max(0.0);
            let mag = shake.trauma * shake.trauma * 16.0;
            let ox = (rand::random::<f32>() - 0.5) * 2.0 * mag;
            let oy = (rand::random::<f32>() - 0.5) * 2.0 * mag;
            camera_transform.translation.x = player_transform.translation.x + CAMERA_LOOKAHEAD + ox;
            camera_transform.translation.y = 20.0 + oy;
        }
    }
}

fn apply_forces(
    mut player_query: Query<&mut Player>,
    time: Res<Time>,
    config: Res<GameConfig>,
    stats: Res<RunStats>,
    countdown: Res<Countdown>,
) {
    if let Ok(mut player) = player_query.get_single_mut() {
        if !player.is_grounded {
            player.velocity.y -= GRAVITY * time.delta_seconds();
        }
        let mut speed = run_speed(stats.distance, config.difficulty, stats.boost_active());
        if player.state == PlayerState::Stumbling {
            speed *= 0.55;
        }
        if countdown.active() {
            speed = 0.0;
        }
        player.velocity.x = speed;

        let target = player.target_lane as f32;
        player.lane_blend +=
            (target - player.lane_blend) * LANE_SWITCH_SPEED * time.delta_seconds();
        player.lane = lane_from_blend(player.lane_blend);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Player)>, time: Res<Time>) {
    for (mut transform, player) in &mut query {
        transform.translation.x += player.velocity.x * time.delta_seconds();
        transform.translation.y += player.velocity.y * time.delta_seconds();
        transform.translation.z = lane_z(lane_from_blend(player.lane_blend));
        let scale = lane_scale(lane_from_blend(player.lane_blend));
        transform.scale = Vec3::splat(scale);
    }
}

fn flash_invincibility(time: Res<Time>, mut query: Query<(&Player, &mut Sprite)>) {
    if let Ok((player, mut sprite)) = query.get_single_mut() {
        if !player.invincible_timer.finished() {
            let pulse = ((time.elapsed_seconds() * 16.0).sin() * 0.5 + 0.5).clamp(0.25, 1.0);
            sprite.color.set_a(pulse);
        } else if player.has_shield {
            sprite.color = Color::rgb(0.65, 1.0, 1.0);
            sprite.color.set_a(1.0);
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn check_collisions(
    mut player_query: Query<(&mut Transform, &mut Player)>,
    platform_query: Query<(&Transform, &Platform), Without<Player>>,
    obstacle_query: Query<(&Transform, &fuzzy_runner::Obstacle), Without<Player>>,
    mut stats: ResMut<RunStats>,
    config: Res<GameConfig>,
    mut shake: ResMut<CameraImpulse>,
    mut next_state: ResMut<NextState<GameState>>,
    mut deaths: EventWriter<DeathEvent>,
    mut stumbled: EventWriter<PlayerStumbled>,
) {
    let Ok((mut player_transform, mut player)) = player_query.get_single_mut() else {
        return;
    };

    let was_grounded = player.is_grounded;
    player.is_grounded = false;
    let standing_size = PLAYER_SIZE;
    let player_pos = player_transform.translation;

    for (platform_transform, platform) in &platform_query {
        let platform_size = platform.size;
        let platform_pos = platform_transform.translation;
        if aabb_overlap(player_pos, standing_size, platform_pos, platform_size)
            && player.velocity.y <= 0.0
        {
            let penetration =
                (platform_pos.y + platform_size.y / 2.0) - (player_pos.y - standing_size.y / 2.0);
            if penetration > 0.0 && penetration < standing_size.y {
                player_transform.translation.y += penetration;
                player.velocity.y = 0.0;
                player.is_grounded = true;
            }
        }
    }

    if player.is_grounded && !was_grounded {
        player.coyote_time.reset();
    }

    if !player.invincible_timer.finished() {
        return;
    }

    let player_size = player_hitbox(&player);
    let player_pos = player_collision_pos(player_transform.translation, &player);
    for (obstacle_transform, obstacle) in &obstacle_query {
        if obstacle.lane != player.lane {
            continue;
        }
        if !aabb_overlap(
            player_pos,
            player_size,
            obstacle_transform.translation,
            obstacle.size,
        ) {
            continue;
        }

        let jumping = player.state == PlayerState::Jumping || player.velocity.y > 40.0;
        let sliding = player.state == PlayerState::Sliding;
        if fuzzy_runner::obstacle_cleared(obstacle.kind, jumping, sliding) {
            continue;
        }

        match resolve_hit(player.state == PlayerState::Stumbling, player.has_shield) {
            HitOutcome::Shielded => {
                player.has_shield = false;
                player.invincible_timer.reset();
            }
            HitOutcome::Stumble => {
                player.stumble_timer.reset();
                player.invincible_timer.reset();
                player.state = PlayerState::Stumbling;
                shake.add(0.85);
                stats.combo = 0;
                stats.combo_timer = 0.0;
                stats.threat = fuzzy_runner::threat_after_stumble(
                    stats.threat,
                    config.difficulty.stumble_threat(),
                );
                stumbled.send(PlayerStumbled);
                if fuzzy_runner::is_caught(stats.threat) {
                    deaths.send(DeathEvent {
                        reason: DeathReason::Caught,
                    });
                    next_state.set(GameState::GameOver);
                }
            }
            HitOutcome::Death => {
                deaths.send(DeathEvent {
                    reason: DeathReason::Crash,
                });
                next_state.set(GameState::GameOver);
            }
        }
        break;
    }
}
