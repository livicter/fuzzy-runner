use crate::assets::GameAssets;
use bevy::prelude::*;
use fuzzy_runner::{
    lane_scale, lane_z, plan_segment, Bob, Coin, GameConfig, GameState, Obstacle, ObstacleKind,
    OnGameScreen, Platform, PlatformQueue, Player, PowerUp, PowerUpKind, RunStats, SegmentPlan,
    TrackCursor, GROUND_Y, INITIAL_TRACK_END, PLATFORM_THICKNESS, TRACK_LOOKAHEAD, VIEWPORT_WIDTH,
};

const TILE: f32 = 70.0;

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
    assets: Res<GameAssets>,
) {
    if platform_query.iter().next().is_some() {
        return;
    }

    platform_queue.0.clear();
    cursor.next_x = INITIAL_TRACK_END;
    let first = spawn_platform(&mut commands, &assets, Vec2::new(300.0, GROUND_Y), 1200.0);
    platform_queue.0.push_back(first);
}

pub fn spawn_platform(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec2,
    width: f32,
) -> Entity {
    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
            Platform {
                size: Vec2::new(width, PLATFORM_THICKNESS),
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            let tiles = ((width / TILE).ceil() as i32).max(1);
            let start_x = -width / 2.0 + TILE / 2.0;
            for i in 0..tiles {
                let x = start_x + i as f32 * TILE;
                parent.spawn(SpriteBundle {
                    texture: assets.roof.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE + 1.0, 28.0)),
                        color: Color::rgb(0.62, 0.88, 1.0),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, 4.0, 0.15),
                    ..default()
                });
                parent.spawn(SpriteBundle {
                    texture: assets.wall.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE + 1.0, 210.0)),
                        color: Color::rgb(0.38, 0.22, 0.55),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, -118.0, -0.2),
                    ..default()
                });
            }
        })
        .id()
}

fn spawn_obstacle(
    commands: &mut Commands,
    assets: &GameAssets,
    x: f32,
    kind: ObstacleKind,
    lane: i32,
) {
    let (size, texture, y, tint) = match kind {
        ObstacleKind::LowBarrier => (
            Vec2::new(52.0, 52.0),
            assets.crate_box.clone(),
            GROUND_Y + 36.0,
            Color::WHITE,
        ),
        ObstacleKind::HighBarrier => (
            Vec2::new(72.0, 36.0),
            assets.crate_warn.clone(),
            GROUND_Y + 90.0,
            Color::WHITE,
        ),
        ObstacleKind::LaneBlock => (
            Vec2::new(48.0, 70.0),
            assets.crate_boom.clone(),
            GROUND_Y + 46.0,
            Color::WHITE,
        ),
    };
    let scale = lane_scale(lane);

    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                custom_size: Some(size * scale),
                color: tint,
                ..default()
            },
            transform: Transform::from_xyz(x, y, lane_z(lane)),
            ..default()
        },
        Obstacle { kind, lane, size },
        OnGameScreen,
    ));

    if kind == ObstacleKind::LaneBlock {
        commands.spawn((
            SpriteBundle {
                texture: assets.spikes.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(48.0, 28.0) * scale),
                    ..default()
                },
                transform: Transform::from_xyz(x, GROUND_Y + 18.0, lane_z(lane) + 0.05),
                ..default()
            },
            OnGameScreen,
        ));
    }
}

fn spawn_coin(commands: &mut Commands, assets: &GameAssets, x: f32, lane: i32, airborne: bool) {
    let y = if airborne {
        GROUND_Y + 110.0
    } else {
        GROUND_Y + 50.0
    };
    let scale = lane_scale(lane);
    commands.spawn((
        SpriteBundle {
            texture: assets.coin.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(34.0 * scale)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, lane_z(lane) + 0.05),
            ..default()
        },
        Coin { value: 1, lane },
        Bob {
            base_y: y,
            phase: x * 0.04,
        },
        OnGameScreen,
    ));
}

fn spawn_power_up(
    commands: &mut Commands,
    assets: &GameAssets,
    x: f32,
    lane: i32,
    kind: PowerUpKind,
) {
    let (texture, y) = match kind {
        PowerUpKind::Magnet => (assets.gem_blue.clone(), GROUND_Y + 68.0),
        PowerUpKind::Shield => (assets.gem_green.clone(), GROUND_Y + 68.0),
        PowerUpKind::Multiplier => (assets.gem_yellow.clone(), GROUND_Y + 68.0),
        PowerUpKind::Boost => (assets.gem_red.clone(), GROUND_Y + 68.0),
    };
    let scale = lane_scale(lane);
    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                custom_size: Some(Vec2::splat(40.0 * scale)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, lane_z(lane) + 0.08),
            ..default()
        },
        PowerUp { kind, lane },
        Bob {
            base_y: y,
            phase: x * 0.05,
        },
        OnGameScreen,
    ));
}

fn ensure_platform(
    commands: &mut Commands,
    assets: &GameAssets,
    queue: &mut PlatformQueue,
    start_x: f32,
    end_x: f32,
) {
    let width = (end_x - start_x).max(80.0);
    let center = start_x + width / 2.0;
    let entity = spawn_platform(commands, assets, Vec2::new(center, GROUND_Y), width);
    queue.push_back(entity);
}

fn spawn_plan(
    commands: &mut Commands,
    assets: &GameAssets,
    cursor: &mut TrackCursor,
    queue: &mut PlatformQueue,
    plan: SegmentPlan,
) {
    match plan {
        SegmentPlan::Safe { length } => {
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            cursor.next_x += length;
        }
        SegmentPlan::Coins { lane, count } => {
            let length = 90.0 + count as f32 * 38.0;
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            for i in 0..count {
                spawn_coin(
                    commands,
                    assets,
                    cursor.next_x + 50.0 + i as f32 * 38.0,
                    lane,
                    false,
                );
            }
            cursor.next_x += length;
        }
        SegmentPlan::LowBarriers { lanes } => {
            let length = 260.0;
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            for lane in lanes {
                spawn_obstacle(
                    commands,
                    assets,
                    cursor.next_x + 150.0,
                    ObstacleKind::LowBarrier,
                    lane,
                );
            }
            cursor.next_x += length;
        }
        SegmentPlan::HighBars { lanes } => {
            let length = 260.0;
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            for lane in lanes {
                spawn_obstacle(
                    commands,
                    assets,
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
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            spawn_power_up(commands, assets, cursor.next_x + 140.0, lane, kind);
            cursor.next_x += length;
        }
        SegmentPlan::JumpSet {
            obstacle_lane,
            coin_lane,
        } => {
            let length = 300.0;
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            spawn_obstacle(
                commands,
                assets,
                cursor.next_x + 170.0,
                ObstacleKind::LowBarrier,
                obstacle_lane,
            );
            for i in 0..4 {
                spawn_coin(
                    commands,
                    assets,
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
            ensure_platform(
                commands,
                assets,
                queue,
                cursor.next_x,
                cursor.next_x + length,
            );
            spawn_obstacle(
                commands,
                assets,
                cursor.next_x + 170.0,
                ObstacleKind::HighBarrier,
                obstacle_lane,
            );
            for i in 0..4 {
                spawn_coin(
                    commands,
                    assets,
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
    platform_query: Query<(&Transform, &Platform)>,
    item_query: Query<(Entity, &Transform), Or<(With<Obstacle>, With<Coin>, With<PowerUp>)>>,
    stats: Res<RunStats>,
    config: Res<GameConfig>,
    assets: Res<GameAssets>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_x = player_transform.translation.x;

    if let Some(&first_platform_entity) = platform_queue.front() {
        if let Ok((platform_transform, platform)) = platform_query.get(first_platform_entity) {
            let platform_right_edge = platform_transform.translation.x + (platform.size.x / 2.0);
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
        spawn_plan(
            &mut commands,
            &assets,
            &mut cursor,
            &mut platform_queue,
            plan,
        );
        guard += 1;
    }
}
