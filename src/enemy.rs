use bevy::prelude::*;
use fuzzy_runner::{
    is_caught, lane_scale, safe_duration, safe_timer, AnimationIndices, AnimationTimer, Chaser,
    Enemy, GameState, OnGameScreen, Player, RunStats, CHASER_FAR_GAP, CHASER_NEAR_GAP, ENEMY_SIZE,
    GROUND_Y, PLATFORM_THICKNESS,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_horde)
            .add_systems(
                Update,
                (follow_player, animate_chaser)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_horde(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    existing: Query<Entity, With<Chaser>>,
) {
    if existing.iter().next().is_some() {
        return;
    }

    let texture: Handle<Image> = asset_server.load("zombie_tilesheet.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(80.0, 110.0), 9, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let y = GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (ENEMY_SIZE.y / 2.0);

    for rank in 0..3u8 {
        let back = 40.0 * rank as f32;
        let lane_nudge = match rank {
            1 => 18.0,
            2 => -16.0,
            _ => 0.0,
        };
        commands.spawn((
            SpriteSheetBundle {
                texture: texture.clone(),
                atlas: TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 9,
                },
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.55 + rank as f32 * 0.08, 0.45),
                    ..default()
                },
                transform: Transform::from_xyz(-180.0 - back, y + lane_nudge, 2.4 - rank as f32 * 0.15)
                    .with_scale(Vec3::splat(0.86 - rank as f32 * 0.06)),
                ..default()
            },
            Enemy {
                velocity: Vec2::ZERO,
                is_grounded: true,
            },
            Chaser { rank },
            AnimationIndices { first: 9, last: 10 },
            AnimationTimer(safe_timer(0.08 + rank as f32 * 0.02, TimerMode::Repeating)),
            OnGameScreen,
        ));
    }
}

fn follow_player(
    player_query: Query<(&Transform, &Player), Without<Chaser>>,
    mut chaser_query: Query<(&mut Transform, &mut Sprite, &Chaser), Without<Player>>,
    stats: Res<RunStats>,
) {
    let Ok((player_transform, player)) = player_query.get_single() else {
        return;
    };

    for (mut transform, mut sprite, chaser) in &mut chaser_query {
        let pack_gap = 46.0 * chaser.rank as f32;
        let gap = CHASER_FAR_GAP + (CHASER_NEAR_GAP - CHASER_FAR_GAP) * stats.threat + pack_gap;
        let lane_nudge = match chaser.rank {
            1 => 16.0,
            2 => -14.0,
            _ => 0.0,
        };
        transform.translation.x = player_transform.translation.x - gap;
        transform.translation.y = player_transform
            .translation
            .y
            .max(GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (ENEMY_SIZE.y / 2.0))
            + lane_nudge;
        transform.translation.z = player_transform.translation.z - 0.18 - chaser.rank as f32 * 0.12;
        transform.scale = Vec3::splat(lane_scale(player.lane) * (1.14 - chaser.rank as f32 * 0.08));

        let heat = 0.55 + stats.threat * 0.45;
        sprite.color = Color::rgb(1.0, 1.0 - stats.threat * 0.55, 1.0 - stats.threat * 0.55);
        sprite
            .color
            .set_a(if is_caught(stats.threat) { 1.0 } else { heat });
    }
}

fn animate_chaser(
    time: Res<Time>,
    stats: Res<RunStats>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &mut AnimationTimer,
            &mut TextureAtlas,
        ),
        With<Chaser>,
    >,
) {
    let hurry = (0.10 - stats.threat * 0.04).max(0.05);
    for (mut indices, mut timer, mut atlas) in &mut query {
        timer.set_duration(safe_duration(hurry));
        timer.tick(time.delta());
        if timer.just_finished() {
            indices.first = 9;
            indices.last = 10;
            if atlas.index >= indices.last {
                atlas.index = indices.first;
            } else {
                atlas.index += 1;
            }
        }
    }
}
