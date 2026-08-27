use crate::assets::GameAssets;
use bevy::prelude::*;
use fuzzy_runner::{
    lane_scale, lane_z, plan_segment, safe_timer, try_despawn, Bob, Coin, CoinSpin, Decor,
    GameConfig, GameState, Obstacle, ObstacleKind, OnGameScreen, Platform, PlatformQueue, Player,
    PowerUp, PowerUpKind, RunStats, SegmentPlan, Spin, TorchFlicker, TrackCursor, GROUND_Y,
    INITIAL_TRACK_END, PLATFORM_THICKNESS, TRACK_LOOKAHEAD, VIEWPORT_WIDTH,
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
                        color: Color::rgb(0.55, 0.72, 0.82),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, 4.0, 0.05),
                    ..default()
                });
                parent.spawn(SpriteBundle {
                    texture: assets.wall.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE + 1.0, 210.0)),
                        color: Color::rgb(0.32, 0.2, 0.42),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, -118.0, -0.25),
                    ..default()
                });
                // Three painted runner lanes, stacked in depth like Temple Run.
                let lanes = [
                    (
                        assets.lane_near.clone(),
                        8.0,
                        2.4,
                        Color::rgb(0.45, 0.85, 1.0),
                    ),
                    (
                        assets.lane_mid.clone(),
                        16.0,
                        1.15,
                        Color::rgb(1.0, 0.86, 0.35),
                    ),
                    (
                        assets.lane_far.clone(),
                        24.0,
                        0.08,
                        Color::rgb(0.55, 1.0, 0.6),
                    ),
                ];
                for (texture, y, z, tint) in lanes {
                    parent.spawn(SpriteBundle {
                        texture,
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(TILE + 1.0, 14.0)),
                            color: tint,
                            ..default()
                        },
                        transform: Transform::from_xyz(x, y, z),
                        ..default()
                    });
                }
                if i % 3 == 0 {
                    parent.spawn(SpriteBundle {
                        texture: assets.sign_arrow.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(22.0, 18.0)),
                            color: Color::rgba(1.0, 0.95, 0.55, 0.7),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, 22.0, 2.6),
                        ..default()
                    });
                }
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
    let scale = lane_scale(lane);
    match kind {
        ObstacleKind::LowBarrier => {
            commands.spawn((
                SpriteBundle {
                    texture: assets.log.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(70.0, 36.0) * scale),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, GROUND_Y + 28.0, lane_z(lane)),
                    ..default()
                },
                Obstacle {
                    kind,
                    lane,
                    size: Vec2::new(64.0, 34.0),
                },
                OnGameScreen,
            ));
        }
        ObstacleKind::HighBarrier => {
            if rand::random::<f32>() > 0.45 {
                commands.spawn((
                    SpriteBundle {
                        texture: assets.saw.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(56.0 * scale)),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, GROUND_Y + 92.0, lane_z(lane)),
                        ..default()
                    },
                    Obstacle {
                        kind,
                        lane,
                        size: Vec2::new(56.0, 40.0),
                    },
                    Spin { speed: 8.0 },
                    OnGameScreen,
                ));
            } else {
                commands.spawn((
                    SpriteBundle {
                        texture: assets.chain.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(18.0, 70.0) * scale),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, GROUND_Y + 118.0, lane_z(lane) - 0.02),
                        ..default()
                    },
                    Decor,
                    OnGameScreen,
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: assets.weight.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(48.0, 48.0) * scale),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, GROUND_Y + 86.0, lane_z(lane)),
                        ..default()
                    },
                    Obstacle {
                        kind,
                        lane,
                        size: Vec2::new(56.0, 40.0),
                    },
                    OnGameScreen,
                ));
            }
        }
        ObstacleKind::LaneBlock => {
            commands.spawn((
                SpriteBundle {
                    texture: assets.crate_boom.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(52.0, 64.0) * scale),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, GROUND_Y + 44.0, lane_z(lane)),
                    ..default()
                },
                Obstacle {
                    kind,
                    lane,
                    size: Vec2::new(48.0, 70.0),
                },
                OnGameScreen,
            ));
            commands.spawn((
                SpriteBundle {
                    texture: assets.spikes.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(48.0, 24.0) * scale),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, GROUND_Y + 16.0, lane_z(lane) + 0.05),
                    ..default()
                },
                Decor,
                OnGameScreen,
            ));
            commands.spawn((
                SpriteBundle {
                    texture: assets.bomb.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(30.0, 30.0) * scale),
                        ..default()
                    },
                    transform: Transform::from_xyz(x + 22.0, GROUND_Y + 30.0, lane_z(lane) + 0.02),
                    ..default()
                },
                Decor,
                OnGameScreen,
            ));
            commands.spawn((
                SpriteBundle {
                    texture: assets.crate_warn.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(22.0, 22.0) * scale),
                        ..default()
                    },
                    transform: Transform::from_xyz(x - 20.0, GROUND_Y + 26.0, lane_z(lane) + 0.01),
                    ..default()
                },
                Decor,
                OnGameScreen,
            ));
        }
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
        CoinSpin {
            timer: safe_timer(0.12, TimerMode::Repeating),
            frame: (x as u32 % 3) as usize,
        },
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

fn spawn_prop(
    commands: &mut Commands,
    texture: Handle<Image>,
    x: f32,
    y: f32,
    z: f32,
    size: Vec2,
    tint: Color,
) {
    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                custom_size: Some(size),
                color: tint,
                ..default()
            },
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },
        Decor,
        OnGameScreen,
    ));
}

fn spawn_gap_pit(commands: &mut Commands, assets: &GameAssets, start: f32, length: f32) {
    let tiles = ((length / 36.0).ceil() as i32).max(2);
    for i in 0..tiles {
        let x = start + 18.0 + i as f32 * 36.0;
        spawn_prop(
            commands,
            assets.fireball.clone(),
            x,
            GROUND_Y - 28.0,
            -0.4,
            Vec2::new(40.0, 28.0),
            Color::rgb(1.0, 0.45 + (i as f32 * 0.07) % 0.4, 0.15),
        );
        spawn_prop(
            commands,
            assets.spikes.clone(),
            x,
            GROUND_Y - 6.0,
            -0.35,
            Vec2::new(34.0, 18.0),
            Color::rgb(1.0, 0.55, 0.25),
        );
    }
}

fn spawn_gap_edge(commands: &mut Commands, assets: &GameAssets, x: f32) {
    spawn_prop(
        commands,
        assets.block_warn.clone(),
        x,
        GROUND_Y + 26.0,
        2.8,
        Vec2::new(42.0, 42.0),
        Color::WHITE,
    );
    spawn_prop(
        commands,
        assets.block_exclaim.clone(),
        x,
        GROUND_Y + 68.0,
        2.85,
        Vec2::new(38.0, 38.0),
        Color::WHITE,
    );
    spawn_prop(
        commands,
        assets.spikes.clone(),
        x,
        GROUND_Y + 12.0,
        2.9,
        Vec2::new(48.0, 20.0),
        Color::WHITE,
    );
    spawn_prop(
        commands,
        assets.flag_yellow.clone(),
        x + 16.0,
        GROUND_Y + 78.0,
        2.86,
        Vec2::new(36.0, 48.0),
        Color::WHITE,
    );
    spawn_prop(
        commands,
        assets.flag_red.clone(),
        x - 16.0,
        GROUND_Y + 74.0,
        2.84,
        Vec2::new(32.0, 46.0),
        Color::WHITE,
    );
}

fn spawn_scenery(commands: &mut Commands, assets: &GameAssets, x: f32, roll: f32) {
    let near_tint = Color::rgb(0.75, 0.9, 1.0);
    let far_tint = Color::rgb(0.62, 0.38, 0.88);
    let horizon = Color::rgb(0.42, 0.22, 0.62);

    match ((roll * 17.0) as i32).rem_euclid(6) {
        0 => spawn_prop(
            commands,
            assets.pyramid.clone(),
            x + 70.0,
            GROUND_Y + 150.0,
            -1.7,
            Vec2::new(260.0, 190.0),
            horizon,
        ),
        1 => spawn_prop(
            commands,
            assets.mountain.clone(),
            x + 50.0,
            GROUND_Y + 170.0,
            -1.75,
            Vec2::new(320.0, 210.0),
            horizon,
        ),
        2 => spawn_prop(
            commands,
            assets.hills.clone(),
            x + 40.0,
            GROUND_Y + 90.0,
            -1.65,
            Vec2::new(340.0, 110.0),
            Color::rgb(0.35, 0.18, 0.5),
        ),
        3 => spawn_prop(
            commands,
            assets.temple.clone(),
            x + 40.0,
            GROUND_Y + 118.0,
            -1.4,
            Vec2::new(220.0, 140.0),
            far_tint,
        ),
        4 => spawn_prop(
            commands,
            assets.castle.clone(),
            x + 20.0,
            GROUND_Y + 136.0,
            -1.45,
            Vec2::new(200.0, 176.0),
            Color::rgb(0.5, 0.35, 0.75),
        ),
        _ => spawn_prop(
            commands,
            assets.tower.clone(),
            x + 30.0,
            GROUND_Y + 158.0,
            -1.5,
            Vec2::new(72.0, 230.0),
            Color::rgb(0.55, 0.4, 0.8),
        ),
    }

    match ((roll * 11.0) as i32).rem_euclid(4) {
        0 => spawn_prop(
            commands,
            assets.tree.clone(),
            x - 30.0,
            GROUND_Y + 126.0,
            -1.2,
            Vec2::new(130.0, 190.0),
            Color::rgb(0.7, 0.3, 0.9),
        ),
        1 => spawn_prop(
            commands,
            assets.tree2.clone(),
            x - 16.0,
            GROUND_Y + 118.0,
            -1.15,
            Vec2::new(90.0, 170.0),
            Color::rgb(0.45, 0.85, 0.7),
        ),
        2 => spawn_prop(
            commands,
            assets.cloud.clone(),
            x + 80.0,
            GROUND_Y + 248.0,
            -1.85,
            Vec2::new(170.0, 72.0),
            Color::rgba(1.0, 0.82, 1.0, 0.62),
        ),
        _ => {}
    }

    match ((roll * 8.0) as i32).rem_euclid(11) {
        0 => {
            commands.spawn((
                SpriteBundle {
                    texture: assets.torch_on.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(28.0, 70.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, GROUND_Y + 58.0, 3.3),
                    ..default()
                },
                Decor,
                TorchFlicker {
                    timer: safe_timer(0.12, TimerMode::Repeating),
                },
                OnGameScreen,
            ));
        }
        1 => spawn_prop(
            commands,
            assets.flag.clone(),
            x,
            GROUND_Y + 70.0,
            3.35,
            Vec2::new(46.0, 70.0),
            near_tint,
        ),
        2 => spawn_prop(
            commands,
            assets.bush.clone(),
            x,
            GROUND_Y + 38.0,
            3.4,
            Vec2::new(64.0, 48.0),
            Color::rgb(0.45, 1.0, 0.7),
        ),
        3 => spawn_prop(
            commands,
            assets.fence.clone(),
            x,
            GROUND_Y + 36.0,
            3.2,
            Vec2::new(80.0, 40.0),
            near_tint,
        ),
        4 => spawn_prop(
            commands,
            assets.sign.clone(),
            x,
            GROUND_Y + 52.0,
            3.25,
            Vec2::new(40.0, 56.0),
            Color::WHITE,
        ),
        5 => spawn_prop(
            commands,
            assets.rock.clone(),
            x,
            GROUND_Y + 30.0,
            3.28,
            Vec2::new(52.0, 36.0),
            Color::rgb(0.8, 0.7, 1.0),
        ),
        6 => spawn_prop(
            commands,
            assets.plant.clone(),
            x,
            GROUND_Y + 40.0,
            3.3,
            Vec2::new(34.0, 50.0),
            Color::rgb(0.55, 1.0, 0.75),
        ),
        7 => spawn_prop(
            commands,
            assets.cactus.clone(),
            x,
            GROUND_Y + 44.0,
            3.22,
            Vec2::new(36.0, 56.0),
            Color::rgb(0.55, 1.0, 0.7),
        ),
        8 => spawn_prop(
            commands,
            assets.torch.clone(),
            x,
            GROUND_Y + 58.0,
            3.22,
            Vec2::new(26.0, 64.0),
            Color::WHITE,
        ),
        9 => spawn_prop(
            commands,
            assets.crate_box.clone(),
            x,
            GROUND_Y + 34.0,
            3.22,
            Vec2::new(40.0, 40.0),
            Color::rgb(0.85, 0.75, 1.0),
        ),
        _ => spawn_prop(
            commands,
            assets.plant_purple.clone(),
            x,
            GROUND_Y + 42.0,
            3.3,
            Vec2::new(36.0, 52.0),
            Color::WHITE,
        ),
    }
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
            spawn_scenery(
                commands,
                assets,
                cursor.next_x + length * 0.45,
                length * 0.01,
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
            spawn_scenery(commands, assets, cursor.next_x + 20.0, lane as f32 * 0.2);
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
            spawn_scenery(commands, assets, cursor.next_x + 40.0, 0.31);
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
            spawn_scenery(commands, assets, cursor.next_x + 48.0, 0.44);
            cursor.next_x += length;
        }
        SegmentPlan::Gap { length } => {
            spawn_gap_edge(commands, assets, cursor.next_x - 18.0);
            spawn_gap_pit(commands, assets, cursor.next_x, length);
            spawn_gap_edge(commands, assets, cursor.next_x + length + 18.0);
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
            spawn_scenery(commands, assets, cursor.next_x + 28.0, 0.62);
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
            spawn_scenery(commands, assets, cursor.next_x + 36.0, 0.73);
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
            spawn_scenery(commands, assets, cursor.next_x + 36.0, 0.81);
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
    item_query: Query<
        (Entity, &Transform),
        Or<(With<Obstacle>, With<Coin>, With<PowerUp>, With<Decor>)>,
    >,
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
                try_despawn(&mut commands, first_platform_entity);
                platform_queue.pop_front();
            }
        }
    }

    for (entity, transform) in &item_query {
        if transform.translation.x < player_x - (VIEWPORT_WIDTH / 2.0) - 80.0 {
            try_despawn(&mut commands, entity);
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
