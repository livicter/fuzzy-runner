use crate::assets::GameAssets;
use bevy::prelude::*;
use fuzzy_runner::{
    is_near_miss, run_speed, safe_timer, try_despawn, Aura, AuraKind, CoinCollected, DustPuff,
    GameConfig, GameState, Obstacle, OnGameScreen, Orbit, ParticleBurst, Player, PlayerState,
    RunStats, SkyProp, Spin, TorchFlicker, Vignette, GROUND_Y,
};

#[derive(Component)]
struct BoostPack;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_sky)
            .add_systems(Update, pin_sky)
            .add_systems(OnEnter(GameState::Playing), (setup_auras, setup_boost_pack))
            .add_systems(
                Update,
                (
                    spawn_runner_dust,
                    tick_dust,
                    spawn_speed_streaks,
                    spawn_coin_trails,
                    follow_auras,
                    orbit_magnet_coins,
                    flicker_torches,
                    spin_hazards,
                    lane_switch_afterimage,
                    landing_puff,
                    near_miss_spark,
                    follow_boost_pack,
                    update_vignette,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_auras(
    mut commands: Commands,
    assets: Res<GameAssets>,
    existing: Query<Entity, With<Aura>>,
) {
    if existing.iter().next().is_some() {
        return;
    }

    commands.spawn((
        SpriteBundle {
            texture: assets.particle_circle.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(90.0)),
                color: Color::rgba(0.35, 1.0, 1.0, 0.0),
                ..default()
            },
            transform: Transform::from_xyz(80.0, GROUND_Y + 50.0, 3.6),
            visibility: Visibility::Hidden,
            ..default()
        },
        Aura {
            kind: AuraKind::Shield,
        },
        OnGameScreen,
    ));

    for i in 0..3 {
        commands.spawn((
            SpriteBundle {
                texture: assets.coin.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(18.0)),
                    color: Color::rgba(1.0, 0.9, 0.3, 0.0),
                    ..default()
                },
                transform: Transform::from_xyz(80.0, GROUND_Y + 50.0, 3.7),
                visibility: Visibility::Hidden,
                ..default()
            },
            Aura {
                kind: AuraKind::Magnet,
            },
            Orbit {
                angle: i as f32 * 2.094,
                radius: 42.0,
            },
            OnGameScreen,
        ));
    }
}

fn setup_sky(mut commands: Commands, assets: Res<GameAssets>, existing: Query<(), With<SkyProp>>) {
    if existing.iter().next().is_some() {
        return;
    }

    commands.spawn((
        SpriteBundle {
            texture: assets.moon.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(96.0)),
                color: Color::rgb(1.0, 0.92, 0.78),
                ..default()
            },
            transform: Transform::from_xyz(-340.0, 236.0, -7.4),
            ..default()
        },
        SkyProp {
            offset: Vec3::new(-340.0, 236.0, -7.4),
            bob: 0.28,
            drift: 0.0,
        },
    ));
    commands.spawn((
        SpriteBundle {
            texture: assets.cloud_big.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(220.0, 88.0)),
                color: Color::rgba(1.0, 0.86, 0.78, 0.52),
                ..default()
            },
            transform: Transform::from_xyz(-90.0, 206.0, -7.2),
            ..default()
        },
        SkyProp {
            offset: Vec3::new(-90.0, 206.0, -7.2),
            bob: 0.48,
            drift: 18.0,
        },
    ));
    commands.spawn((
        SpriteBundle {
            texture: assets.cloud_long.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(280.0, 70.0)),
                color: Color::rgba(1.0, 0.8, 0.72, 0.4),
                ..default()
            },
            transform: Transform::from_xyz(240.0, 178.0, -7.1),
            ..default()
        },
        SkyProp {
            offset: Vec3::new(240.0, 178.0, -7.1),
            bob: 0.62,
            drift: -14.0,
        },
    ));
}

fn pin_sky(
    time: Res<Time>,
    camera: Query<&Transform, (With<Camera>, Without<SkyProp>)>,
    mut skies: Query<(&SkyProp, &mut Transform), Without<Camera>>,
) {
    let Ok(cam) = camera.get_single() else {
        return;
    };
    let t = time.elapsed_seconds();
    for (prop, mut transform) in &mut skies {
        let bob = (t * prop.bob + prop.offset.x * 0.01).sin() * 8.0;
        let drift = (t * 0.08).sin() * prop.drift;
        transform.translation.x = cam.translation.x + prop.offset.x + drift;
        transform.translation.y = cam.translation.y + prop.offset.y + bob;
        transform.translation.z = prop.offset.z;
    }
}

fn setup_boost_pack(
    mut commands: Commands,
    assets: Res<GameAssets>,
    existing: Query<Entity, With<BoostPack>>,
) {
    if existing.iter().next().is_some() {
        return;
    }
    commands.spawn((
        SpriteBundle {
            texture: assets.jetpack.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(34.0, 42.0)),
                ..default()
            },
            transform: Transform::from_xyz(80.0, GROUND_Y + 50.0, 3.5),
            visibility: Visibility::Hidden,
            ..default()
        },
        BoostPack,
        OnGameScreen,
    ));
}

fn follow_auras(
    time: Res<Time>,
    player: Query<(&Transform, &Player)>,
    mut auras: Query<(&mut Transform, &mut Sprite, &mut Visibility, &Aura), Without<Player>>,
) {
    let Ok((player_tf, player)) = player.get_single() else {
        return;
    };
    let pulse = 0.55 + 0.45 * (time.elapsed_seconds() * 6.0).sin().abs();
    for (mut transform, mut sprite, mut visibility, aura) in &mut auras {
        transform.translation.x = player_tf.translation.x;
        transform.translation.y = player_tf.translation.y;
        transform.translation.z = player_tf.translation.z + 0.15;
        match aura.kind {
            AuraKind::Shield => {
                let on = player.has_shield;
                *visibility = if on {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                sprite.color = Color::rgba(0.3, 1.0, 1.0, if on { 0.28 * pulse } else { 0.0 });
                transform.scale = Vec3::splat(0.9 + pulse * 0.25);
            }
            AuraKind::Magnet => {}
        }
    }
}

fn orbit_magnet_coins(
    time: Res<Time>,
    stats: Res<RunStats>,
    player: Query<&Transform, With<Player>>,
    mut coins: Query<
        (&mut Transform, &mut Sprite, &mut Visibility, &mut Orbit),
        (With<Aura>, Without<Player>),
    >,
) {
    let Ok(player_tf) = player.get_single() else {
        return;
    };
    let on = stats.magnet_active();
    for (mut transform, mut sprite, mut visibility, mut orbit) in &mut coins {
        orbit.angle += time.delta_seconds() * 4.2;
        *visibility = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        sprite.color.set_a(if on { 0.9 } else { 0.0 });
        transform.translation = player_tf.translation
            + Vec3::new(
                orbit.angle.cos() * orbit.radius,
                orbit.angle.sin() * orbit.radius * 0.55 + 8.0,
                0.2,
            );
    }
}

fn flicker_torches(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut query: Query<(&mut Handle<Image>, &mut Sprite, &mut TorchFlicker)>,
) {
    for (mut texture, mut sprite, mut flicker) in &mut query {
        flicker.timer.tick(time.delta());
        if flicker.timer.just_finished() {
            *texture = if rand::random::<bool>() {
                assets.torch_on.clone()
            } else {
                assets.torch_on2.clone()
            };
            sprite.color = Color::rgb(1.0, 0.85 + rand::random::<f32>() * 0.15, 0.65);
        }
    }
}

fn spin_hazards(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>) {
    for (mut transform, spin) in &mut query {
        transform.rotate_z(spin.speed * time.delta_seconds());
    }
}

fn lane_switch_afterimage(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player: Query<(&Transform, &Player)>,
    mut last_blend: Local<f32>,
) {
    let Ok((transform, player)) = player.get_single() else {
        return;
    };
    if (player.lane_blend - *last_blend).abs() > 0.18 {
        commands.spawn((
            SpriteBundle {
                texture: assets.dust.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(34.0)),
                    color: Color::rgba(0.6, 1.0, 1.0, 0.45),
                    ..default()
                },
                transform: Transform::from_xyz(
                    transform.translation.x - 8.0,
                    transform.translation.y - 10.0,
                    transform.translation.z - 0.05,
                ),
                ..default()
            },
            DustPuff {
                timer: safe_timer(0.22, TimerMode::Once),
            },
            OnGameScreen,
        ));
        *last_blend = player.lane_blend;
    }
}

fn landing_puff(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player: Query<(&Transform, &Player)>,
    mut was_airborne: Local<bool>,
) {
    let Ok((transform, player)) = player.get_single() else {
        return;
    };
    if *was_airborne && player.is_grounded {
        for i in 0..5 {
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            let texture = if i % 2 == 0 {
                assets.smoke.clone()
            } else {
                assets.dust.clone()
            };
            commands.spawn((
                SpriteBundle {
                    texture,
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(32.0)),
                        color: Color::rgba(0.88, 0.74, 0.56, 0.72),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        transform.translation.x + side * 10.0 * i as f32,
                        transform.translation.y - 18.0,
                        transform.translation.z + 0.08,
                    ),
                    ..default()
                },
                DustPuff {
                    timer: safe_timer(0.28 + i as f32 * 0.04, TimerMode::Once),
                },
                OnGameScreen,
            ));
        }
    }
    *was_airborne = !player.is_grounded;
}

fn near_miss_spark(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player: Query<(&Transform, &Player)>,
    obstacles: Query<(&Transform, &Obstacle), Without<Player>>,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    *cooldown = (*cooldown - time.delta_seconds()).max(0.0);
    if *cooldown > 0.0 {
        return;
    }
    let Ok((player_tf, player)) = player.get_single() else {
        return;
    };
    if !player.is_grounded {
        return;
    }
    for (obstacle_tf, obstacle) in &obstacles {
        if is_near_miss(
            player_tf.translation.x,
            player.lane,
            obstacle_tf.translation.x,
            obstacle.lane,
        ) {
            commands.spawn((
                SpriteBundle {
                    texture: assets.particle_lightning.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(28.0)),
                        color: Color::rgba(1.0, 0.95, 0.55, 0.88),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        player_tf.translation.x + 8.0,
                        player_tf.translation.y + 16.0,
                        player_tf.translation.z + 0.2,
                    ),
                    ..default()
                },
                DustPuff {
                    timer: safe_timer(0.16, TimerMode::Once),
                },
                OnGameScreen,
            ));
            *cooldown = 0.45;
            break;
        }
    }
}

fn follow_boost_pack(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    stats: Res<RunStats>,
    player: Query<&Transform, With<Player>>,
    mut pack: Query<(&mut Transform, &mut Visibility), (With<BoostPack>, Without<Player>)>,
    mut trail_acc: Local<f32>,
) {
    let Ok(player_tf) = player.get_single() else {
        return;
    };
    let Ok((mut transform, mut visibility)) = pack.get_single_mut() else {
        return;
    };
    let on = stats.boost_active();
    *visibility = if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    transform.translation = player_tf.translation + Vec3::new(-16.0, -2.0, -0.06);
    if !on {
        *trail_acc = 0.0;
        return;
    }
    *trail_acc += time.delta_seconds();
    if *trail_acc >= 0.05 {
        *trail_acc = 0.0;
        commands.spawn((
            SpriteBundle {
                texture: assets.particle_flare.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(22.0)),
                    color: Color::rgba(1.0, 0.55, 0.2, 0.75),
                    ..default()
                },
                transform: Transform::from_xyz(
                    player_tf.translation.x - 28.0,
                    player_tf.translation.y - 6.0,
                    player_tf.translation.z - 0.04,
                ),
                ..default()
            },
            DustPuff {
                timer: safe_timer(0.18, TimerMode::Once),
            },
            OnGameScreen,
        ));
    }
}

fn spawn_runner_dust(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    player: Query<(&Transform, &Player)>,
    mut cooldown: Local<f32>,
) {
    *cooldown -= time.delta_seconds();
    let Ok((transform, player)) = player.get_single() else {
        return;
    };
    if !player.is_grounded || player.state == PlayerState::Sliding {
        return;
    }
    if *cooldown > 0.0 {
        return;
    }
    *cooldown = 0.07;
    let puff = if time.elapsed_seconds() as i32 % 2 == 0 {
        assets.dust.clone()
    } else {
        assets.dust2.clone()
    };
    commands.spawn((
        SpriteBundle {
            texture: puff,
            sprite: Sprite {
                custom_size: Some(Vec2::splat(28.0)),
                color: Color::rgba(1.0, 0.9, 0.75, 0.55),
                ..default()
            },
            transform: Transform::from_xyz(
                transform.translation.x - 18.0,
                GROUND_Y + 18.0,
                transform.translation.z - 0.1,
            ),
            ..default()
        },
        DustPuff {
            timer: safe_timer(0.28, TimerMode::Once),
        },
        OnGameScreen,
    ));
}

fn tick_dust(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut DustPuff)>,
) {
    for (entity, mut transform, mut sprite, mut dust) in &mut query {
        dust.timer.tick(time.delta());
        transform.translation.y += 18.0 * time.delta_seconds();
        transform.translation.x -= 30.0 * time.delta_seconds();
        transform.scale += Vec3::splat(1.4 * time.delta_seconds());
        sprite.color.set_a(1.0 - dust.timer.fraction());
        if dust.timer.finished() {
            try_despawn(&mut commands, entity);
        }
    }
}

fn spawn_speed_streaks(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    stats: Res<RunStats>,
    config: Res<GameConfig>,
    player: Query<&Transform, With<Player>>,
    mut cooldown: Local<f32>,
) {
    *cooldown -= time.delta_seconds();
    let speed = run_speed(stats.distance, config.difficulty, stats.boost_active());
    if speed < 380.0 || *cooldown > 0.0 {
        return;
    }
    *cooldown = 0.05;
    let Ok(transform) = player.get_single() else {
        return;
    };
    let y_off = (rand::random::<f32>() - 0.5) * 80.0;
    let boosting = stats.boost_active();
    let (texture, size, color) = if boosting {
        (
            assets.fireball.clone(),
            Vec2::new(36.0, 18.0),
            Color::rgba(1.0, 0.7, 0.35, 0.7),
        )
    } else {
        (
            assets.particle_spark.clone(),
            Vec2::new(42.0, 6.0),
            Color::rgba(0.7, 1.0, 1.0, 0.4),
        )
    };
    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                custom_size: Some(size),
                color,
                ..default()
            },
            transform: Transform::from_xyz(
                transform.translation.x + 40.0,
                transform.translation.y + y_off,
                4.5,
            ),
            ..default()
        },
        DustPuff {
            timer: safe_timer(0.18, TimerMode::Once),
        },
        OnGameScreen,
    ));
}

fn spawn_coin_trails(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player: Query<&Transform, With<Player>>,
    mut events: EventReader<CoinCollected>,
) {
    let Ok(transform) = player.get_single() else {
        return;
    };
    for event in events.read() {
        let sparks = if event.combo >= 8 { 6 } else { 3 };
        for i in 0..sparks {
            let drift = -18.0 - i as f32 * 10.0;
            commands.spawn((
                SpriteBundle {
                    texture: assets.particle_spark.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(10.0 + i as f32 * 2.0)),
                        color: Color::rgba(1.0, 0.85, 0.35, 0.8),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        transform.translation.x + drift,
                        transform.translation.y - 6.0 + (i as f32 * 4.0),
                        transform.translation.z + 0.2,
                    ),
                    ..default()
                },
                ParticleBurst {
                    velocity: Vec2::new(-80.0 - i as f32 * 12.0, 40.0),
                    timer: safe_timer(0.28, TimerMode::Once),
                },
                OnGameScreen,
            ));
        }
    }
}

fn update_vignette(
    stats: Res<RunStats>,
    config: Res<GameConfig>,
    mut query: Query<&mut BackgroundColor, With<Vignette>>,
) {
    let speed = run_speed(stats.distance, config.difficulty, stats.boost_active());
    let heat = ((speed - 340.0) / 380.0).clamp(0.0, 1.0);
    let boost = if stats.boost_active() { 0.12 } else { 0.0 };
    let alpha = 0.10 + heat * 0.28 + boost;
    for mut color in &mut query {
        *color = Color::rgba(0.04, 0.0, 0.08, alpha).into();
    }
}
