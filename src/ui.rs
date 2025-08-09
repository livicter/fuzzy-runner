use bevy::prelude::*;
use fuzzy_runner::{
    despawn_screen, Distance, DistanceText, GameConfig, GameState, HealthBar, OnGameScreen,
    OnPauseMenu, OnSettingsMenu, Player,
};

#[derive(Resource)]
struct GameOverTimer(Timer);

#[derive(Component)]
enum MenuButtonAction {
    Resume,
    Reset,
    Settings,
}

#[derive(Component)]
enum SettingsButtonAction {
    IncrementEnemies,
    DecrementEnemies,
    Back,
}

#[derive(Component)]
struct EnemyCountText;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_game_ui)
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_screen::<OnPauseMenu>)
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_screen)
            .add_systems(OnEnter(GameState::SettingsMenu), setup_settings_menu)
            .add_systems(OnExit(GameState::GameOver), despawn_screen::<OnPauseMenu>)
            .add_systems(
                OnExit(GameState::SettingsMenu),
                despawn_screen::<OnSettingsMenu>,
            )
            .add_systems(
                Update,
                (
                    toggle_pause_state,
                    handle_settings_menu_actions.run_if(in_state(GameState::SettingsMenu)),
                    handle_menu_button_actions.run_if(in_state(GameState::Paused)),
                    update_enemy_count_text.run_if(in_state(GameState::SettingsMenu)),
                    game_over_reset_timer.run_if(in_state(GameState::GameOver)),
                    update_health_bar.run_if(in_state(GameState::Playing)),
                    update_distance_text.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn handle_settings_menu_actions(
    mut interaction_query: Query<
        (&Interaction, &SettingsButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                SettingsButtonAction::IncrementEnemies => {
                    config.max_enemies = (config.max_enemies + 1).min(10); // Cap at 10
                }
                SettingsButtonAction::DecrementEnemies => {
                    config.max_enemies = (config.max_enemies - 1).max(1); // Minimum 1
                }
                SettingsButtonAction::Back => {
                    next_state.set(GameState::Paused);
                }
            }
        }
    }
}

fn toggle_pause_state(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused | GameState::Restart => next_state.set(GameState::Playing),
            GameState::SettingsMenu => next_state.set(GameState::Paused),
            GameState::GameOver => {}
        }
    }
}

fn update_enemy_count_text(
    config: Res<GameConfig>,
    mut query: Query<&mut Text, With<EnemyCountText>>,
) {
    if config.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            text.sections[0].value = config.max_enemies.to_string();
        }
    }
}

fn handle_menu_button_actions(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Resume => {
                    game_state.set(GameState::Playing);
                }
                MenuButtonAction::Reset => {
                    game_state.set(GameState::Restart);
                }
                MenuButtonAction::Settings => {
                    game_state.set(GameState::SettingsMenu);
                }
            }
        }
    }
}

fn setup_settings_menu(mut commands: Commands, config: Res<GameConfig>) {
    let button_style = Style {
        width: Val::Px(50.0),
        height: Val::Px(50.0),
        margin: UiRect::horizontal(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let text_style = TextStyle {
        font_size: 32.0,
        color: Color::WHITE,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                ..default()
            },
            OnSettingsMenu,
        ))
        .with_children(|parent| {
            // Enemy Count Editor
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Enemies: ", text_style.clone()));

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: Color::DARK_GRAY.into(),
                                ..default()
                            },
                            SettingsButtonAction::DecrementEnemies,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("-", text_style.clone()));
                        });

                    parent.spawn((
                        TextBundle::from_section(
                            config.max_enemies.to_string(),
                            text_style.clone(),
                        ),
                        EnemyCountText,
                    ));

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: Color::DARK_GRAY.into(),
                                ..default()
                            },
                            SettingsButtonAction::IncrementEnemies,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("+", text_style.clone()));
                        });
                });

            // Back Button
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..button_style
                        },
                        background_color: Color::GRAY.into(),
                        ..default()
                    },
                    SettingsButtonAction::Back,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Back", text_style.clone()));
                });
        });
}

fn setup_pause_menu(mut commands: Commands) {
    let button_style = Style {
        width: Val::Px(200.0),
        height: Val::Px(50.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 32.0,
        color: Color::BLACK,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
                ..default()
            },
            OnPauseMenu,
        ))
        .with_children(|parent| {
            // Resume Button
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::GRAY.into(),
                        ..default()
                    },
                    MenuButtonAction::Resume,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Resume",
                        button_text_style.clone(),
                    ));
                });

            // Reset Button
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::GRAY.into(),
                        ..default()
                    },
                    MenuButtonAction::Reset,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Reset", button_text_style.clone()));
                });

            // Settings button
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: Color::GRAY.into(),
                        ..default()
                    },
                    MenuButtonAction::Settings,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Settings", button_text_style));
                });
        });
}

fn setup_game_ui(mut commands: Commands) {
    // Health Bar Background
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(20.0),
                    top: Val::Px(20.0),
                    width: Val::Px(200.0),
                    height: Val::Px(30.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                border_color: BorderColor(Color::WHITE),
                background_color: Color::BLACK.into(),
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            // Health Bar Filler
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::GREEN.into(),
                    ..default()
                },
                HealthBar,
            ));
        });

    // Distance Text
    commands.spawn((
        TextBundle::from_section(
            "Distance: 0",
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(60.0),
            ..default()
        }),
        DistanceText,
        OnGameScreen,
    ));
}

fn update_health_bar(
    player_query: Query<&Player>,
    mut health_bar_query: Query<&mut Style, With<HealthBar>>,
) {
    if let Ok(player) = player_query.get_single() {
        if let Ok(mut health_bar_style) = health_bar_query.get_single_mut() {
            health_bar_style.width = Val::Percent(player.health);
        }
    }
}

fn update_distance_text(
    distance: Res<Distance>,
    mut distance_text_query: Query<&mut Text, With<DistanceText>>,
) {
    if let Ok(mut text) = distance_text_query.get_single_mut() {
        text.sections[0].value = format!("Distance: {:.0}m", distance.0 / 10.0);
    }
}

fn setup_game_over_screen(mut commands: Commands) {
    commands.insert_resource(GameOverTimer(Timer::from_seconds(2.0, TimerMode::Once)));

    let text_style = TextStyle {
        font_size: 80.0,
        color: Color::WHITE,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                ..default()
            },
            OnPauseMenu,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("You Died", text_style));
        });
}

fn game_over_reset_timer(
    time: Res<Time>,
    mut timer: ResMut<GameOverTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        next_state.set(GameState::Restart);
    }
}
