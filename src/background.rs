use bevy::prelude::*;
use bevy_parallax::{
    CreateParallaxEvent, LayerData, LayerSpeed, ParallaxCameraComponent, ParallaxMoveEvent,
    ParallaxPlugin, ParallaxSystems,
};
use fuzzy_runner::{run_speed, GameConfig, GameState, RunStats};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ParallaxPlugin)
            .add_event::<CreateParallaxEvent>()
            .add_systems(Startup, initialize_camera_system)
            .add_systems(
                Update,
                move_camera_system
                    .before(ParallaxSystems)
                    .run_if(in_scrolling_state),
            );
    }
}

fn in_scrolling_state(state: Res<State<GameState>>) -> bool {
    matches!(state.get(), GameState::Playing | GameState::Menu)
}

pub fn initialize_camera_system(
    mut commands: Commands,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
) {
    let camera = commands
        .spawn(Camera2dBundle::default())
        .insert(ParallaxCameraComponent::default())
        .id();
    let event = CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(0.9),
                path: "cyberpunk_back.png".to_string(),
                tile_size: Vec2::new(96.0, 160.0),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(4.5),
                z: -10.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.6),
                path: "cyberpunk_middle.png".to_string(),
                tile_size: Vec2::new(144.0, 160.0),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(4.5),
                z: -9.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.1),
                path: "cyberpunk_front.png".to_string(),
                tile_size: Vec2::new(272.0, 160.0),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(4.5),
                z: -8.0,
                ..default()
            },
        ],
        camera,
    };
    create_parallax.send(event);
}

pub fn move_camera_system(
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
    camera_query: Query<Entity, With<Camera>>,
    state: Res<State<GameState>>,
    stats: Res<RunStats>,
    config: Res<GameConfig>,
) {
    let Ok(camera) = camera_query.get_single() else {
        return;
    };

    let scroll = match state.get() {
        GameState::Menu => 1.2,
        GameState::Playing => (run_speed(stats.distance, config.difficulty, stats.boost_active())
            / 90.0)
            .clamp(2.2, 8.5),
        _ => 0.0,
    };

    if scroll > 0.0 {
        move_event_writer.send(ParallaxMoveEvent {
            translation: Vec2::new(scroll, 0.0),
            rotation: 0.,
            camera,
        });
    }
}
