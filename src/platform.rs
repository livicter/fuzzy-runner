use bevy::prelude::*;
use fuzzy_runner::{
    lane_scale, lane_z, plan_segment, Coin, GameConfig, GameState, Obstacle, ObstacleKind,
    OnGameScreen, Platform, PlatformQueue, Player, PowerUp, PowerUpKind, RunStats, SegmentPlan,
    TrackCursor, GROUND_Y, INITIAL_TRACK_END, NEON_CYAN, NEON_GOLD, NEON_LIME, NEON_MAGENTA,
    NEON_PURPLE, PLATFORM_THICKNESS, ROOF_COLOR, ROOF_EDGE, TRACK_LOOKAHEAD, VIEWPORT_WIDTH,
};

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlatformQueue>()
            .init_resource::<TrackCursor>()
            .add_systems(OnEnter(GameState::Playing), setup_track)
            .add_systems(Update, manage_track.run_if(in_state(GameState::Playing)));
    }
}

fn setup_track(
    mut commands: Commands,
    mut platform_queue: ResMut<PlatformQueue>,
    mut cursor: ResMut<TrackCursor>,
    platform_query: Query<Entity, With<Platform>>,
) {
    if platform_query.iter().next().is_some() {
        return;
    }

    platform_queue.0.clear();
    cursor.next_x = INITIAL_TRACK_END;
    let first = spawn_platform(&mut commands, Vec2::new(300.0, GROUND_Y), 1200.0);
    platform_queue.0.push_back(first);
}

pub fn spawn_platform(commands: &mut Commands, position: Vec2, width: f32) -> Entity {
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: ROOF_COLOR,
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
        .with_children(|parent| {
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: ROOF_EDGE,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, 0.55, 0.1),
                    scale: Vec3::new(1.0, 0.18, 1.0),
                    ..default()
                },
                ..default()
            });
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.07, 0.05, 0.12),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, -6.5, -0.2),
                    scale: Vec3::new(1.0, 12.0, 1.0),
                    ..default()
                },
                ..default()
            });
        })
        .id()
}

fn spawn_obstacle(commands: &mut Commands, x: f32, kind: ObstacleKind, lane: i32) {
    let (size, color, y) = match kind {
        ObstacleKind::LowBarrier => (Vec2::new(42.0, 44.0), NEON_MAGENTA, GROUND_Y + 33.0),
        ObstacleKind::HighBarrier => (Vec2::new(70.0, 28.0), NEON_CYAN, GROUND_Y + 86.0),
        ObstacleKind::LaneBlock => (
            Vec2::new(36.0, 92.0),
            Color::rgb(1.0, 0.25, 0.25),
            GROUND_Y + 57.0,
        ),
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color, ..default() },
            transform: Transform {
                translation: Vec3::new(x, y, lane_z(lane)),
                scale: Vec3::new(size.x, size.y, 1.0) * lane_scale(lane),
                ..default()
            },
            ..default()
        },
        Obstacle { kind, lane, size },
        OnGameScreen,
    ));
}

fn spawn_coin(commands: &mut Commands, x: f32, lane: i32, airborne: bool) {
    let y = if airborne {
        GROUND_Y + 110.0
    } else {
        GROUND_Y + 48.0
    };
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: NEON_GOLD,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, lane_z(lane) + 0.05),
                scale: Vec3::splat(16.0 * lane_scale(lane)),
                ..default()
            },
            ..default()
        },
        Coin { value: 1, lane },
        OnGameScreen,
    ));
}

fn spawn_power_up(commands: &mut Commands, x: f32, lane: i32, kind: PowerUpKind) {
    let color = match kind {
        PowerUpKind::Magnet => NEON_PURPLE,
        PowerUpKind::Shield => NEON_CYAN,
        PowerUpKind::Multiplier => NEON_GOLD,
        PowerUpKind::Boost => NEON_LIME,
    };
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color, ..default() },
            transform: Transform {
                translation: Vec3::new(x, GROUND_Y + 68.0, lane_z(lane) + 0.08),
                scale: Vec3::splat(22.0 * lane_scale(lane)),
                ..default()
            },
            ..default()
        },
        PowerUp { kind, lane },
        OnGameScreen,
    ));
}

fn ensure_platform(commands: &mut Commands, queue: &mut PlatformQueue, start_x: f32, end_x: f32) {
    let width = (end_x - start_x).max(80.0);
    let center = start_x + width / 2.0;
    let entity = spawn_platform(commands, Vec2::new(center, GROUND_Y), width);
    queue.push_back(entity);
}

fn spawn_plan(
    commands: &mut Commands,
    cursor: &mut TrackCursor,
    queue: &mut PlatformQueue,
    plan: SegmentPlan,
) {
    match plan {
        SegmentPlan::Safe { length } => {
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            cursor.next_x += length;
        }
        SegmentPlan::Coins { lane, count } => {
            let length = 90.0 + count as f32 * 38.0;
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            for i in 0..count {
                spawn_coin(
                    commands,
                    cursor.next_x + 50.0 + i as f32 * 38.0,
                    lane,
                    false,
                );
            }
            cursor.next_x += length;
        }
        SegmentPlan::LowBarriers { lanes } => {
            let length = 260.0;
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            for lane in lanes {
                spawn_obstacle(
                    commands,
                    cursor.next_x + 150.0,
                    ObstacleKind::LowBarrier,
                    lane,
                );
            }
            cursor.next_x += length;
        }
        SegmentPlan::HighBars { lanes } => {
            let length = 260.0;
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            for lane in lanes {
                spawn_obstacle(
                    commands,
                    cursor.next_x + 150.0,
                    ObstacleKind::HighBarrier,
                    lane,
                );
            }
            cursor.next_x += length;
        }
        SegmentPlan::Gap { length } => {
            cursor.next_x += length;
        }
        SegmentPlan::PowerUp { kind, lane } => {
            let length = 240.0;
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            spawn_power_up(commands, cursor.next_x + 140.0, lane, kind);
            cursor.next_x += length;
        }
        SegmentPlan::JumpSet {
            obstacle_lane,
            coin_lane,
        } => {
            let length = 300.0;
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            spawn_obstacle(
                commands,
                cursor.next_x + 170.0,
                ObstacleKind::LowBarrier,
                obstacle_lane,
            );
            for i in 0..4 {
                spawn_coin(
                    commands,
                    cursor.next_x + 130.0 + i as f32 * 28.0,
                    coin_lane,
                    true,
                );
            }
            cursor.next_x += length;
        }
        SegmentPlan::SlideSet {
            obstacle_lane,
            coin_lane,
        } => {
            let length = 300.0;
            ensure_platform(commands, queue, cursor.next_x, cursor.next_x + length);
            spawn_obstacle(
                commands,
                cursor.next_x + 170.0,
                ObstacleKind::HighBarrier,
                obstacle_lane,
            );
            for i in 0..4 {
                spawn_coin(
                    commands,
                    cursor.next_x + 130.0 + i as f32 * 28.0,
                    coin_lane,
                    false,
                );
            }
            cursor.next_x += length;
        }
    }
}

fn manage_track(
    mut commands: Commands,
    mut platform_queue: ResMut<PlatformQueue>,
    mut cursor: ResMut<TrackCursor>,
    player_query: Query<&Transform, With<Player>>,
    platform_query: Query<&Transform, With<Platform>>,
    item_query: Query<(Entity, &Transform), Or<(With<Obstacle>, With<Coin>, With<PowerUp>)>>,
    stats: Res<RunStats>,
    config: Res<GameConfig>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_x = player_transform.translation.x;

    if let Some(&first_platform_entity) = platform_queue.front() {
        if let Ok(platform_transform) = platform_query.get(first_platform_entity) {
            let platform_right_edge =
                platform_transform.translation.x + (platform_transform.scale.x / 2.0);
            let screen_left_edge = player_x - (VIEWPORT_WIDTH / 2.0) - 80.0;
            if platform_right_edge < screen_left_edge {
                commands.entity(first_platform_entity).despawn_recursive();
                platform_queue.pop_front();
            }
        }
    }

    for (entity, transform) in &item_query {
        if transform.translation.x < player_x - (VIEWPORT_WIDTH / 2.0) - 80.0 {
            commands.entity(entity).despawn_recursive();
        }
    }

    let mut guard = 0;
    while cursor.next_x < player_x + TRACK_LOOKAHEAD && guard < 8 {
        let plan = plan_segment(
            stats.distance,
            config.difficulty,
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        );
        spawn_plan(&mut commands, &mut cursor, &mut platform_queue, plan);
        guard += 1;
    }
}
