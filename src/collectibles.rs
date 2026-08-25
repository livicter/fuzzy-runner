use bevy::prelude::*;
use fuzzy_runner::{
    aabb_overlap, award_coins, lane_from_blend, player_collision_pos, player_hitbox, Coin,
    CoinCollected, FloatingPopup, GameState, OnGameScreen, Player, PowerUp, PowerUpCollected,
    RunStats, MAGNET_RADIUS, NEON_GOLD,
};

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CoinCollected>()
            .add_event::<PowerUpCollected>()
            .add_systems(
                Update,
                (
                    attract_coins,
                    collect_coins,
                    collect_power_ups,
                    spawn_coin_popups,
                    tick_popups,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
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
        if aabb_overlap(origin, hitbox, transform.translation, Vec2::splat(20.0)) || magnet_grab {
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
        if !aabb_overlap(origin, hitbox, transform.translation, Vec2::splat(28.0)) {
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

fn spawn_coin_popups(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut events: EventReader<CoinCollected>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    for event in events.read() {
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("+{}", event.points),
                    TextStyle {
                        font_size: 22.0,
                        color: NEON_GOLD,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(
                    player_transform.translation.x + 10.0,
                    player_transform.translation.y + 50.0,
                    8.0,
                ),
                ..default()
            },
            FloatingPopup {
                timer: Timer::from_seconds(0.55, TimerMode::Once),
            },
            OnGameScreen,
        ));
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
