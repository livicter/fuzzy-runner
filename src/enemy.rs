use bevy::prelude::*;
use fuzzy_runner::{
    is_caught, lane_scale, AnimationIndices, AnimationTimer, Chaser, Enemy, GameState,
    OnGameScreen, Player, RunStats, CHASER_FAR_GAP, CHASER_NEAR_GAP, ENEMY_SIZE, GROUND_Y,
    PLATFORM_THICKNESS,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_chaser)
            .add_systems(
                Update,
                (follow_player, animate_chaser)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_chaser(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    existing: Query<Entity, With<Chaser>>,
) {
    if existing.get_single().is_ok() {
        return;
    }

    let texture: Handle<Image> = asset_server.load("zombie_tilesheet.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(80.0, 110.0), 9, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let y = GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (ENEMY_SIZE.y / 2.0);

    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 9,
            },
            sprite: Sprite {
                color: Color::rgb(1.0, 0.55, 0.55),
                ..default()
            },
            transform: Transform::from_xyz(-180.0, y, 2.4).with_scale(Vec3::splat(0.86)),
            ..default()
        },
        Enemy {
            velocity: Vec2::ZERO,
            is_grounded: true,
        },
        Chaser,
        AnimationIndices { first: 9, last: 10 },
        AnimationTimer(Timer::from_seconds(0.09, TimerMode::Repeating)),
        OnGameScreen,
    ));
}

fn follow_player(
    player_query: Query<(&Transform, &Player), Without<Chaser>>,
    mut chaser_query: Query<(&mut Transform, &mut Sprite), With<Chaser>>,
    stats: Res<RunStats>,
) {
    let Ok((player_transform, player)) = player_query.get_single() else {
        return;
    };
    let Ok((mut transform, mut sprite)) = chaser_query.get_single_mut() else {
        return;
    };

    let gap = CHASER_FAR_GAP + (CHASER_NEAR_GAP - CHASER_FAR_GAP) * stats.threat;
    transform.translation.x = player_transform.translation.x - gap;
    transform.translation.y = player_transform
        .translation
        .y
        .max(GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (ENEMY_SIZE.y / 2.0));
    transform.translation.z = player_transform.translation.z - 0.2;
    transform.scale = Vec3::splat(lane_scale(player.lane) * 1.12);

    let heat = 0.55 + stats.threat * 0.45;
    sprite.color = Color::rgb(1.0, 1.0 - stats.threat * 0.55, 1.0 - stats.threat * 0.55);
    sprite
        .color
        .set_a(if is_caught(stats.threat) { 1.0 } else { heat });
}

fn animate_chaser(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationIndices,
            &mut AnimationTimer,
            &mut TextureAtlas,
        ),
        With<Chaser>,
    >,
) {
    for (mut indices, mut timer, mut atlas) in &mut query {
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
