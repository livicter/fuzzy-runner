use bevy::prelude::*;
use fuzzy_runner::{AnimationIndices, AnimationTimer, Enemy, GameConfig, GameState, OnGameScreen, Platform, Player, ENEMY_JUMP_STRENGTH, ENEMY_SIZE, ENEMY_SPEED, GRAVITY, PLAYER_SIZE};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_zombie_population,
                animate_zombie,
                zombie_movement_ai,
                apply_velocity.before(zombie_platform_collision),
                zombie_platform_collision,
                zombie_player_collision.after(zombie_platform_collision),
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Spawns a single zombie instance.
fn spawn_zombie(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    position: Vec3,
) {
    let texture: Handle<Image> = asset_server.load("zombie_tilesheet.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(80.0, 110.0), 9, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 13, // Start with running animation
            },
            transform: Transform::from_translation(position)
                .with_scale(Vec3::new(0.70, 0.70, 1.0)), // Same scale as player
            ..default()
        },
        Enemy {
            velocity: Vec2::ZERO,
            is_grounded: false,
        },
        AnimationIndices { first: 13, last: 14 }, // Running animation
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        OnGameScreen,
    ));
}

fn zombie_movement_ai(
    mut zombie_query: Query<(&mut Enemy, &Transform, &mut Sprite), Without<Platform>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    platform_query: Query<&Transform, (With<Platform>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut zombie, zombie_transform, mut sprite) in zombie_query.iter_mut() {
            if !zombie.is_grounded {
                zombie.velocity.y -= GRAVITY * time.delta_seconds();
            }

            let direction_to_player =
                (player_transform.translation.x - zombie_transform.translation.x).signum();
            zombie.velocity.x = direction_to_player * ENEMY_SPEED;

            if direction_to_player > 0.0 {
                sprite.flip_x = false;
            } else if direction_to_player < 0.0 {
                sprite.flip_x = true;
            }

            if zombie.is_grounded {
                let mut should_jump = false;
                if player_transform.translation.y > zombie_transform.translation.y + PLAYER_SIZE.y
                {
                    should_jump = true;
                }

                let probe_distance = direction_to_player * (ENEMY_SIZE.x / 2.0 + 10.0);
                let probe_pos = Vec3::new(
                    zombie_transform.translation.x + probe_distance,
                    zombie_transform.translation.y - (ENEMY_SIZE.y / 2.0) - 5.0,
                    0.0,
                );
                let probe_size = Vec2::new(5.0, 5.0);

                let mut ground_ahead = false;
                for platform_transform in &platform_query {
                    let platform_size =
                        Vec2::new(platform_transform.scale.x, platform_transform.scale.y);
                    let platform_pos = platform_transform.translation;

                    let x_collision = (probe_pos.x - probe_size.x / 2.0)
                        < (platform_pos.x + platform_size.x / 2.0)
                        && (probe_pos.x + probe_size.x / 2.0)
                        > (platform_pos.x - platform_size.x / 2.0);
                    let y_collision = (probe_pos.y - probe_size.y / 2.0)
                        < (platform_pos.y + platform_size.y / 2.0)
                        && (probe_pos.y + probe_size.y / 2.0)
                        > (platform_pos.y - platform_size.y / 2.0);

                    if x_collision && y_collision {
                        ground_ahead = true;
                        break;
                    }
                }

                if !ground_ahead {
                    should_jump = true;
                }

                if should_jump {
                    zombie.velocity.y = ENEMY_JUMP_STRENGTH;
                    zombie.is_grounded = false;
                }
            }
        }
    }
}

/// Applies the zombie's velocity to its transform.
fn apply_velocity(mut query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut transform, zombie) in &mut query {
        transform.translation.x += zombie.velocity.x * time.delta_seconds();
        transform.translation.y += zombie.velocity.y * time.delta_seconds();
    }
}

/// Handles collision between the zombie and platforms using manual AABB checks.
fn zombie_platform_collision(
    mut zombie_query: Query<(&mut Transform, &mut Enemy), Without<Platform>>,
    platform_query: Query<&Transform, (With<Platform>, Without<Enemy>)>,
) {
    for (mut zombie_transform, mut zombie) in zombie_query.iter_mut() {
        zombie.is_grounded = false;
        let zombie_size = ENEMY_SIZE;
        let zombie_pos = zombie_transform.translation;

        for platform_transform in &platform_query {
            let platform_size =
                Vec2::new(platform_transform.scale.x, platform_transform.scale.y);
            let platform_pos = platform_transform.translation;

            let x_collision = (zombie_pos.x - zombie_size.x / 2.0)
                < (platform_pos.x + platform_size.x / 2.0)
                && (zombie_pos.x + zombie_size.x / 2.0)
                > (platform_pos.x - platform_size.x / 2.0);
            let y_collision = (zombie_pos.y - zombie_size.y / 2.0)
                < (platform_pos.y + platform_size.y / 2.0)
                && (zombie_pos.y + zombie_size.y / 2.0)
                > (platform_pos.y - platform_size.y / 2.0);

            if x_collision && y_collision && zombie.velocity.y <= 0.0 {
                let penetration = (platform_pos.y + platform_size.y / 2.0)
                    - (zombie_pos.y - zombie_size.y / 2.0);
                if penetration > 0.0 {
                    zombie_transform.translation.y += penetration;
                    zombie.velocity.y = 0.0;
                    zombie.is_grounded = true;
                }
            }
        }
    }
}

/// Handles collision between the zombie and the player using manual AABB checks.
fn zombie_player_collision(
    mut player_query: Query<(&Transform, &mut Player), Without<Enemy>>,
    zombie_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
) {
    if let Ok((player_transform, mut player)) = player_query.get_single_mut() {
        for zombie_transform in zombie_query.iter() {
            let player_pos = player_transform.translation;
            let zombie_pos = zombie_transform.translation;

            let x_collision = (player_pos.x - PLAYER_SIZE.x / 2.0)
                < (zombie_pos.x + ENEMY_SIZE.x / 2.0)
                && (player_pos.x + PLAYER_SIZE.x / 2.0)
                > (zombie_pos.x - ENEMY_SIZE.x / 2.0);
            let y_collision = (player_pos.y - PLAYER_SIZE.y / 2.0)
                < (zombie_pos.y + ENEMY_SIZE.y / 2.0)
                && (player_pos.y + PLAYER_SIZE.y / 2.0)
                > (zombie_pos.y - ENEMY_SIZE.y / 2.0);

            if x_collision && y_collision {
                player.health -= 1.0;
                // Since we are looping, break after the first hit to avoid multiple damage instances in one frame
                break;
            }
        }
    }
}

/// Animates the zombie's sprite based on its state.
fn animate_zombie(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlas,
        &Enemy,
    )>,
) {
    for (mut indices, mut timer, mut atlas, zombie) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            // TODO: Can be an index array to iterate
            let (first, last) = if !zombie.is_grounded {
                (13, 14) // Jumping/Falling frame
            } else {
                (9, 10) // Running frames
            };

            if indices.first != first || indices.last != last {
                indices.first = first;
                indices.last = last;
                atlas.index = first;
            } else {
                if atlas.index == indices.last {
                    atlas.index = indices.first;
                } else {
                    atlas.index += 1;
                }
            }
        }
    }
}

fn manage_zombie_population(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    config: Res<GameConfig>,
    zombie_query: Query<(Entity, &Transform), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let mut zombie_count = 0;
        for (zombie_entity, zombie_transform) in zombie_query.iter() {
            zombie_count += 1;
            // Despawn zombie if it falls off the screen
            if zombie_transform.translation.y < -400.0 {
                commands.entity(zombie_entity).despawn_recursive();
                zombie_count -= 1; // Decrement count after despawning
            }
        }

        // Spawn new zombies if count is less than max
        if zombie_count < config.max_enemies {
            // Add a small random offset to avoid spawning on top of each other
            let random_offset = (rand::random::<f32>() * 200.0) - 100.0;
            let spawn_pos =
                Vec3::new(player_transform.translation.x - 500.0 + random_offset, 200.0, 5.0);
            spawn_zombie(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                spawn_pos,
            );
        }
    }
}
