use crate::assets::GameAssets;
use bevy::prelude::*;
use fuzzy_runner::{
    run_speed, DustPuff, GameConfig, GameState, OnGameScreen, Player, PlayerState, RunStats,
    GROUND_Y,
};

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_runner_dust, tick_dust, spawn_speed_streaks)
                .run_if(in_state(GameState::Playing)),
        );
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
