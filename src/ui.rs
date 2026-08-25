use bevy::prelude::*;
use fuzzy_runner::{
    despawn_screen, CoinText, DeathEvent, Difficulty, DistanceText, GameConfig, GameState,
    HighScores, LastRun, OnGameOverMenu, OnGameScreen, OnMainMenu, OnPauseMenu, OnSettingsMenu,
    Player, PowerUpCollected, RunStats, ScoreText, SettingsOrigin, StatusText, NEON_CYAN,
    NEON_GOLD, NEON_LIME, NEON_MAGENTA,
};

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Resume,
    Reset,
    Settings,
    BackToMenu,
}

#[derive(Component)]
enum SettingsButtonAction {
    SetDifficulty(Difficulty),
    Back,
}

#[derive(Component)]
struct DifficultyLabel;

#[derive(Component)]
struct MenuButton;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_main_menu)
            .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMainMenu>)
            .add_systems(OnEnter(GameState::Playing), setup_game_ui)
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_screen::<OnPauseMenu>)
            .add_systems(
                OnEnter(GameState::GameOver),
                (record_last_run, setup_game_over_screen),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                despawn_screen::<OnGameOverMenu>,
            )
            .add_systems(OnEnter(GameState::SettingsMenu), setup_settings_menu)
            .add_systems(
                OnExit(GameState::SettingsMenu),
                despawn_screen::<OnSettingsMenu>,
            )
            .add_systems(
                Update,
                (
                    toggle_pause_state,
                    handle_settings_menu_actions.run_if(in_state(GameState::SettingsMenu)),
                    handle_menu_button_actions.run_if(in_button_menu_state),
                    update_difficulty_label.run_if(in_state(GameState::SettingsMenu)),
                    update_hud.run_if(in_state(GameState::Playing)),
                    show_power_up_status,
                    paint_menu_buttons,
                    start_from_keyboard.run_if(in_state(GameState::Menu)),
                    restart_from_keyboard.run_if(in_state(GameState::GameOver)),
                ),
            );
    }
}

fn in_button_menu_state(state: Res<State<GameState>>) -> bool {
    matches!(
        state.get(),
        GameState::Paused | GameState::Menu | GameState::GameOver
    )
}

fn record_last_run(
    mut last: ResMut<LastRun>,
    mut highs: ResMut<HighScores>,
    stats: Res<RunStats>,
    mut deaths: EventReader<DeathEvent>,
) {
    if let Some(death) = deaths.read().last() {
        last.reason = death.reason;
    }
    last.distance = stats.distance;
    last.coins = stats.coins;
    last.score = stats.score();
    last.new_high_score = highs.submit(last.score, last.coins, last.distance);
}

fn handle_settings_menu_actions(
    mut interaction_query: Query<
        (&Interaction, &SettingsButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    origin: Res<SettingsOrigin>,
) {
    for (interaction, action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                SettingsButtonAction::SetDifficulty(difficulty) => {
                    config.difficulty = *difficulty;
                }
                SettingsButtonAction::Back => {
                    next_state.set(origin.0);
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
            GameState::Paused => next_state.set(GameState::Playing),
            GameState::SettingsMenu => {}
            GameState::GameOver
            | GameState::Menu
            | GameState::Restart
            | GameState::ReturnToMenu => {}
        }
    }
}

fn update_difficulty_label(
    config: Res<GameConfig>,
    mut query: Query<&mut Text, With<DifficultyLabel>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!("DIFFICULTY: {}", config.difficulty.label());
    }
}

fn handle_menu_button_actions(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    current: Res<State<GameState>>,
    mut origin: ResMut<SettingsOrigin>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Play | MenuButtonAction::Resume => {
                    game_state.set(GameState::Playing);
                }
                MenuButtonAction::Reset => {
                    game_state.set(GameState::Restart);
                }
                MenuButtonAction::Settings => {
                    origin.0 = *current.get();
                    game_state.set(GameState::SettingsMenu);
                }
                MenuButtonAction::BackToMenu => {
                    game_state.set(GameState::ReturnToMenu);
                }
            }
        }
    }
}

fn start_from_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
}

fn restart_from_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Restart);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::ReturnToMenu);
    }
}

fn neon_button_style() -> Style {
    Style {
        width: Val::Px(280.0),
        height: Val::Px(56.0),
        margin: UiRect::all(Val::Px(8.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn heading_style() -> TextStyle {
    TextStyle {
        font_size: 64.0,
        color: NEON_CYAN,
        ..default()
    }
}

fn body_style() -> TextStyle {
    TextStyle {
        font_size: 22.0,
        color: Color::rgb(0.86, 0.90, 1.0),
        ..default()
    }
}

fn button_text_style() -> TextStyle {
    TextStyle {
        font_size: 28.0,
        color: Color::WHITE,
        ..default()
    }
}

fn spawn_menu_button(parent: &mut ChildBuilder, action: MenuButtonAction, label: &str) {
    parent
        .spawn((
            ButtonBundle {
                style: neon_button_style(),
                background_color: Color::rgb(0.07, 0.08, 0.16).into(),
                border_color: BorderColor(NEON_CYAN),
                ..default()
            },
            action,
            MenuButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(label, button_text_style()));
        });
}

fn setup_main_menu(mut commands: Commands, highs: Res<HighScores>) {
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
                background_color: Color::rgba(0.02, 0.0, 0.08, 0.42).into(),
                ..default()
            },
            OnMainMenu,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("CYBER TEMPLE", heading_style())
                    .with_style(Style {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    }),
            );
            parent.spawn(
                TextBundle::from_section(
                    "NEON ROOFTOP RUN",
                    TextStyle {
                        font_size: 28.0,
                        color: NEON_MAGENTA,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(18.0)),
                    ..default()
                }),
            );
            parent.spawn(
                TextBundle::from_section(
                    format!(
                        "BEST  {}    {}m    {} coins",
                        highs.score,
                        (highs.distance / 10.0) as u64,
                        highs.coins
                    ),
                    TextStyle {
                        font_size: 22.0,
                        color: NEON_GOLD,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                }),
            );

            spawn_menu_button(parent, MenuButtonAction::Play, "RUN");
            spawn_menu_button(parent, MenuButtonAction::Settings, "SETTINGS");

            parent.spawn(
                TextBundle::from_section(
                    "AUTO-RUN   A/D lanes   W jump   S slide   ESC pause\nSwipe the mouse or a finger the same way",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::rgb(0.75, 0.8, 0.95),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(22.0)),
                    ..default()
                }),
            );
        });
}

fn setup_settings_menu(mut commands: Commands, config: Res<GameConfig>) {
    let chip_style = Style {
        width: Val::Px(140.0),
        height: Val::Px(48.0),
        margin: UiRect::horizontal(Val::Px(8.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
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
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.82).into(),
                ..default()
            },
            OnSettingsMenu,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("DIFFICULTY: {}", config.difficulty.label()),
                    heading_style(),
                ),
                DifficultyLabel,
            ));
            parent.spawn(
                TextBundle::from_section(
                    "Speed, obstacle density, and how fast the horde closes in.",
                    body_style(),
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(18.0)),
                    ..default()
                }),
            );

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        margin: UiRect::bottom(Val::Px(24.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: chip_style.clone(),
                                    background_color: Color::rgb(0.07, 0.08, 0.16).into(),
                                    border_color: BorderColor(NEON_CYAN),
                                    ..default()
                                },
                                SettingsButtonAction::SetDifficulty(difficulty),
                                MenuButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    difficulty.label(),
                                    button_text_style(),
                                ));
                            });
                    }
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: neon_button_style(),
                        background_color: Color::rgb(0.07, 0.08, 0.16).into(),
                        border_color: BorderColor(NEON_MAGENTA),
                        ..default()
                    },
                    SettingsButtonAction::Back,
                    MenuButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("BACK", button_text_style()));
                });
        });
}

fn setup_pause_menu(mut commands: Commands) {
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
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.55).into(),
                ..default()
            },
            OnPauseMenu,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("PAUSED", heading_style()).with_style(Style {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                }),
            );
            spawn_menu_button(parent, MenuButtonAction::Resume, "RESUME");
            spawn_menu_button(parent, MenuButtonAction::Reset, "NEW RUN");
            spawn_menu_button(parent, MenuButtonAction::Settings, "SETTINGS");
            spawn_menu_button(parent, MenuButtonAction::BackToMenu, "MENU");
        });
}

fn setup_game_ui(mut commands: Commands, existing: Query<Entity, With<ScoreText>>) {
    if existing.iter().next().is_some() {
        return;
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    top: Val::Px(18.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "SCORE 0",
                    TextStyle {
                        font_size: 36.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ScoreText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "0m",
                    TextStyle {
                        font_size: 24.0,
                        color: NEON_CYAN,
                        ..default()
                    },
                ),
                DistanceText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "COINS 0",
                    TextStyle {
                        font_size: 24.0,
                        color: NEON_GOLD,
                        ..default()
                    },
                ),
                CoinText,
            ));
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 20.0,
                        color: NEON_LIME,
                        ..default()
                    },
                ),
                StatusText,
            ));
        });
}

fn update_hud(
    stats: Res<RunStats>,
    mut score_query: Query<
        &mut Text,
        (
            With<ScoreText>,
            Without<DistanceText>,
            Without<CoinText>,
            Without<StatusText>,
        ),
    >,
    mut distance_query: Query<
        &mut Text,
        (
            With<DistanceText>,
            Without<ScoreText>,
            Without<CoinText>,
            Without<StatusText>,
        ),
    >,
    mut coin_query: Query<
        &mut Text,
        (
            With<CoinText>,
            Without<ScoreText>,
            Without<DistanceText>,
            Without<StatusText>,
        ),
    >,
    mut status_query: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<ScoreText>,
            Without<DistanceText>,
            Without<CoinText>,
        ),
    >,
    player_query: Query<&Player>,
) {
    if let Ok(mut text) = score_query.get_single_mut() {
        text.sections[0].value = format!("SCORE {}", stats.score());
    }
    if let Ok(mut text) = distance_query.get_single_mut() {
        text.sections[0].value = format!("{}m", (stats.distance / 10.0) as u64);
    }
    if let Ok(mut text) = coin_query.get_single_mut() {
        text.sections[0].value = format!("COINS {}", stats.coins);
    }
    if let Ok(mut text) = status_query.get_single_mut() {
        let mut parts = Vec::new();
        if stats.multiplier_active() {
            parts.push(format!("x2 {:.0}s", stats.multiplier_timer));
        }
        if stats.magnet_active() {
            parts.push(format!("MAGNET {:.0}s", stats.magnet_timer));
        }
        if stats.boost_active() {
            parts.push(format!("BOOST {:.0}s", stats.boost_timer));
        }
        if player_query
            .get_single()
            .map(|player| player.has_shield)
            .unwrap_or(false)
        {
            parts.push("SHIELD".to_string());
        }
        if stats.threat > 0.35 {
            parts.push("HORDE CLOSING".to_string());
        }
        text.sections[0].value = parts.join("   ");
    }
}

fn show_power_up_status(
    mut events: EventReader<PowerUpCollected>,
    mut status_query: Query<&mut Text, With<StatusText>>,
) {
    if let Some(event) = events.read().last() {
        if let Ok(mut text) = status_query.get_single_mut() {
            text.sections[0].value = event.kind.label().to_string();
        }
    }
}

fn setup_game_over_screen(mut commands: Commands, last: Res<LastRun>) {
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
                background_color: Color::rgba(0.04, 0.0, 0.06, 0.82).into(),
                ..default()
            },
            OnGameOverMenu,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(last.reason.headline(), heading_style()).with_style(
                    Style {
                        margin: UiRect::bottom(Val::Px(12.0)),
                        ..default()
                    },
                ),
            );
            if last.new_high_score {
                parent.spawn(
                    TextBundle::from_section(
                        "NEW HIGH SCORE",
                        TextStyle {
                            font_size: 32.0,
                            color: NEON_GOLD,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    }),
                );
            }
            parent.spawn(TextBundle::from_section(
                format!(
                    "Score {}    {}m    {} coins",
                    last.score,
                    (last.distance / 10.0) as u64,
                    last.coins
                ),
                body_style(),
            ));
            spawn_menu_button(parent, MenuButtonAction::Reset, "RUN AGAIN");
            spawn_menu_button(parent, MenuButtonAction::BackToMenu, "MENU");
        });
}

fn paint_menu_buttons(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
) {
    for (interaction, mut background, mut border) in &mut query {
        match interaction {
            Interaction::Pressed => {
                *background = Color::rgb(0.0, 0.55, 0.65).into();
                *border = BorderColor(Color::WHITE);
            }
            Interaction::Hovered => {
                *background = Color::rgb(0.08, 0.22, 0.32).into();
                *border = BorderColor(NEON_LIME);
            }
            Interaction::None => {
                *background = Color::rgb(0.07, 0.08, 0.16).into();
                *border = BorderColor(NEON_CYAN);
            }
        }
    }
}
