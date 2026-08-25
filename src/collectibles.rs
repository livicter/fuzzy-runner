use crate::assets::GameAssets;
use bevy::prelude::*;
use fuzzy_runner::{
    aabb_overlap, award_coins, lane_from_blend, player_collision_pos, player_hitbox, Bob, Coin,
    CoinCollected, FloatingPopup, GameState, OnGameScreen, ParticleBurst, Player, PowerUp,
    PowerUpCollected, RunStats, MAGNET_RADIUS, NEON_GOLD,
};
use std::f32::consts::TAU;

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CoinCollected>()
            .add_event::<PowerUpCollected>()
            .add_systems(
                Update,
                (
                    bob_pickups,
                    attract_coins,
                    collect_coins,
                    collect_power_ups,
                    spawn_coin_feedback,
                    tick_popups,
                    tick_particles,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn bob_pickups(time: Res<Time>, stats: Res<RunStats>, mut query: Query<(&mut Transform, &Bob)>) {
    let t = time.elapsed_seconds();
    for (mut transform, bob) in &mut query {
        if !stats.magnet_active() {
            transform.translation.y = bob.base_y + (t * 3.4 + bob.phase).sin() * 7.0;
        }
        let pulse = 0.82 + 0.18 * (t * 5.0 + bob.phase).sin().abs();
        transform.scale.x = pulse;
        transform.scale.y = 1.0;
    }
}

fn attract_coins(
    time: Res<Time>,
    stats: Res<RunStats>,
    player_query: Query<(&Transform, &Player)>,
    mut coins: Query<(&mut Transform, &Coin), Without<Player>>,
) {
    if !stats.magnet_active() {
        return;
    }
    let Ok((player_transform, player)) = player_query.get_single() else {
        return;
    };

    for (mut transform, coin) in &mut coins {
        let same_lane_bonus = if coin.lane == player.lane { 1.4 } else { 1.0 };
        let delta = player_transform.translation - transform.translation;
        if delta.length() < MAGNET_RADIUS * same_lane_bonus {
            transform.translation += delta.normalize_or_zero() * 520.0 * time.delta_seconds();
        }
    }
}

fn collect_coins(
    mut commands: Commands,
    mut stats: ResMut<RunStats>,
    mut collected: EventWriter<CoinCollected>,
    player_query: Query<(&Transform, &Player)>,
    coins: Query<(Entity, &Transform, &Coin), Without<Player>>,
) {
    let Ok((player_transform, player)) = player_query.get_single() else {
        return;
    };
    let hitbox = player_hitbox(player);
    let origin = player_collision_pos(player_transform.translation, player);

    for (entity, transform, coin) in &coins {
        let magnet_grab = stats.magnet_active()
            && (transform.translation - player_transform.translation).length() < 36.0;
        let lane_ok = coin.lane == lane_from_blend(player.lane_blend) || stats.magnet_active();
        if !lane_ok && !magnet_grab {
            continue;
        }
        if aabb_overlap(origin, hitbox, transform.translation, Vec2::splat(28.0)) || magnet_grab {
            let award = award_coins(coin.value, stats.multiplier_active());
            stats.coins += award.coins;
            stats.coin_points += award.points;
            collected.send(CoinCollected {
                amount: award.coins,
                points: award.points,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn collect_power_ups(
    mut commands: Commands,
    mut stats: ResMut<RunStats>,
    mut player_query: Query<(&Transform, &mut Player)>,
    mut collected: EventWriter<PowerUpCollected>,
    power_ups: Query<(Entity, &Transform, &PowerUp), Without<Player>>,
) {
    let Ok((player_transform, mut player)) = player_query.get_single_mut() else {
        return;
    };
    let hitbox = player_hitbox(&player);
    let origin = player_collision_pos(player_transform.translation, &player);

    for (entity, transform, power_up) in &power_ups {
        if power_up.lane != player.lane {
            continue;
        }
        if !aabb_overlap(origin, hitbox, transform.translation, Vec2::splat(32.0)) {
            continue;
        }

        match power_up.kind {
            fuzzy_runner::PowerUpKind::Magnet => {
                stats.magnet_timer = power_up.kind.duration();
            }
            fuzzy_runner::PowerUpKind::Multiplier => {
                stats.multiplier_timer = power_up.kind.duration();
            }
            fuzzy_runner::PowerUpKind::Boost => {
                stats.boost_timer = power_up.kind.duration();
                player.invincible_timer.reset();
            }
            fuzzy_runner::PowerUpKind::Shield => {
                player.has_shield = true;
            }
        }
        collected.send(PowerUpCollected {
            kind: power_up.kind,
        });
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_coin_feedback(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut events: EventReader<CoinCollected>,
    assets: Res<GameAssets>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    for event in events.read() {
        let pos = player_transform.translation + Vec3::new(8.0, 46.0, 8.0);
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("+{}", event.points),
                    TextStyle {
                        font: assets.font_hud.clone(),
                        font_size: 26.0,
                        color: NEON_GOLD,
                    },
                ),
                transform: Transform::from_translation(pos),
                ..default()
            },
            FloatingPopup {
                timer: Timer::from_seconds(0.55, TimerMode::Once),
            },
            OnGameScreen,
        ));

        for i in 0..8 {
            let angle = i as f32 / 8.0 * TAU;
            let dir = Vec2::new(angle.cos(), angle.sin());
            commands.spawn((
                SpriteBundle {
                    texture: match i % 3 {
                        0 => assets.particle_star.clone(),
                        1 => assets.particle_spark.clone(),
                        _ => assets.particle_circle.clone(),
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(16.0)),
                        color: NEON_GOLD,
                        ..default()
                    },
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                ParticleBurst {
                    velocity: dir * (120.0 + (i as f32) * 8.0),
                    timer: Timer::from_seconds(0.38, TimerMode::Once),
                },
                OnGameScreen,
            ));
        }
    }
}

fn tick_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut FloatingPopup, &mut Text)>,
) {
    for (entity, mut transform, mut popup, mut text) in &mut query {
        popup.timer.tick(time.delta());
        transform.translation.y += 70.0 * time.delta_seconds();
        let alpha = 1.0 - popup.timer.fraction();
        if let Some(section) = text.sections.first_mut() {
            section.style.color.set_a(alpha);
        }
        if popup.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn tick_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut ParticleBurst)>,
) {
    for (entity, mut transform, mut sprite, mut burst) in &mut query {
        burst.timer.tick(time.delta());
        transform.translation.x += burst.velocity.x * time.delta_seconds();
        transform.translation.y += burst.velocity.y * time.delta_seconds();
        let fade = 1.0 - burst.timer.fraction();
        sprite.color.set_a(fade);
        transform.scale = Vec3::splat(0.6 + fade * 0.6);
        if burst.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
