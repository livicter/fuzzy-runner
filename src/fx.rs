use crate::assets::GameAssets;
use bevy::prelude::*;
use fuzzy_runner::{
    run_speed, Aura, AuraKind, DustPuff, GameConfig, GameState, OnGameScreen, Orbit, Player,
    PlayerState, RunStats, Spin, TorchFlicker, GROUND_Y,
};

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_auras)
            .add_systems(
                Update,
                (
                    spawn_runner_dust,
                    tick_dust,
                    spawn_speed_streaks,
                    follow_auras,
                    orbit_magnet_coins,
                    flicker_torches,
                    spin_hazards,
                    lane_switch_afterimage,
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
                timer: Timer::from_seconds(0.22, TimerMode::Once),
            },
            OnGameScreen,
        ));
        *last_blend = player.lane_blend;
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
            timer: Timer::from_seconds(0.28, TimerMode::Once),
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
            commands.entity(entity).despawn_recursive();
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
            timer: Timer::from_seconds(0.18, TimerMode::Once),
        },
        OnGameScreen,
    ));
}
