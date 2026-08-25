use crate::assets::GameAssets;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, ImageScaleMode, SliceScaleMode, TextureSlicer};
use fuzzy_runner::{
    despawn_screen, CoinText, DeathEvent, Difficulty, DistanceText, GameConfig, GameState,
    HighScores, LastRun, OnGameOverMenu, OnGameScreen, OnMainMenu, OnPauseMenu, OnSettingsMenu,
    PauseButton, Player, PowerUpCollected, RunStats, ScoreText, SettingsOrigin, StatusText,
    ThreatFill, NEON_CYAN, NEON_GOLD, NEON_LIME, NEON_MAGENTA,
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
struct DifficultyChip(Difficulty);

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
                    paint_difficulty_chips.run_if(in_state(GameState::SettingsMenu)),
                    update_hud.run_if(in_state(GameState::Playing)),
                    handle_pause_button.run_if(in_state(GameState::Playing)),
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

fn handle_pause_button(
    query: Query<&Interaction, (Changed<Interaction>, With<PauseButton>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Paused);
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

fn title_style(assets: &GameAssets, size: f32, color: Color) -> TextStyle {
    TextStyle {
        font: assets.font_title.clone(),
        font_size: size,
        color,
    }
}

fn body_style(assets: &GameAssets, size: f32, color: Color) -> TextStyle {
    TextStyle {
        font: assets.font_body.clone(),
        font_size: size,
        color,
    }
}

fn hud_style(assets: &GameAssets, size: f32, color: Color) -> TextStyle {
    TextStyle {
        font: assets.font_hud.clone(),
        font_size: size,
        color,
    }
}

fn nine_slice() -> ImageScaleMode {
    ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::square(18.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

fn overlay_root(marker: impl Bundle) -> (NodeBundle, impl Bundle) {
    (
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgba(0.02, 0.0, 0.08, 0.48).into(),
            ..default()
        },
        marker,
    )
}

fn spawn_image_button(
    parent: &mut ChildBuilder,
    assets: &GameAssets,
    image: Handle<Image>,
    icon: Handle<Image>,
    label: &str,
    action: impl Bundle,
    is_menu: bool,
) {
    let mut button = parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(68.0),
                margin: UiRect::vertical(Val::Px(7.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            image: UiImage::new(image),
            background_color: Color::WHITE.into(),
            ..default()
        },
        action,
    ));
    if is_menu {
        button.insert(MenuButton);
    }
    button.with_children(|parent| {
        parent.spawn(ImageBundle {
            style: Style {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                ..default()
            },
            image: UiImage::new(icon),
            ..default()
        });
        parent.spawn(TextBundle::from_section(
            label,
            body_style(assets, 26.0, Color::WHITE),
        ));
    });
}

fn spawn_icon_label(
    parent: &mut ChildBuilder,
    assets: &GameAssets,
    icon: Handle<Image>,
    label: impl Into<String>,
    color: Color,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::vertical(Val::Px(3.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(26.0),
                    height: Val::Px(26.0),
                    ..default()
                },
                image: UiImage::new(icon),
                ..default()
            });
            row.spawn(TextBundle::from_section(
                label.into(),
                hud_style(assets, 22.0, color),
            ));
        });
}

fn spawn_key_hint(parent: &mut ChildBuilder, key: Handle<Image>, wide: bool) {
    parent.spawn(ImageBundle {
        style: Style {
            width: Val::Px(if wide { 54.0 } else { 28.0 }),
            height: Val::Px(28.0),
            ..default()
        },
        image: UiImage::new(key),
        background_color: Color::WHITE.into(),
        ..default()
    });
}

fn spawn_control_chip(
    parent: &mut ChildBuilder,
    assets: &GameAssets,
    keys: &[Handle<Image>],
    wide_last: bool,
    text: &str,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                margin: UiRect::horizontal(Val::Px(8.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            let last = keys.len().saturating_sub(1);
            for (i, key) in keys.iter().enumerate() {
                spawn_key_hint(row, key.clone(), wide_last && i == last);
            }
            row.spawn(TextBundle::from_section(
                text,
                hud_style(assets, 16.0, Color::rgb(0.82, 0.88, 1.0)),
            ));
        });
}

fn setup_main_menu(mut commands: Commands, highs: Res<HighScores>, assets: Res<GameAssets>) {
    commands
        .spawn(overlay_root(OnMainMenu))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(680.0),
                            max_width: Val::Percent(92.0),
                            padding: UiRect::axes(Val::Px(36.0), Val::Px(28.0)),
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        image: UiImage::new(assets.panel_glass.clone()),
                        ..default()
                    },
                    nine_slice(),
                ))
                .with_children(|panel| {
                    panel
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(420.0),
                                    height: Val::Px(72.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::bottom(Val::Px(6.0)),
                                    ..default()
                                },
                                image: UiImage::new(assets.banner_curtain.clone()),
                                background_color: Color::WHITE.into(),
                                ..default()
                            },
                            nine_slice(),
                        ))
                        .with_children(|banner| {
                            banner.spawn(TextBundle::from_section(
                                "CYBER TEMPLE",
                                title_style(&assets, 34.0, Color::rgb(0.22, 0.12, 0.06)),
                            ));
                        });
                    panel.spawn(
                        TextBundle::from_section(
                            "NEON ROOFTOP RUN",
                            body_style(&assets, 22.0, NEON_MAGENTA),
                        )
                        .with_style(Style {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        }),
                    );

                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(22.0),
                                margin: UiRect::bottom(Val::Px(18.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_trophy.clone(),
                                format!("{}", highs.score),
                                NEON_GOLD,
                            );
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_star.clone(),
                                format!("{}m", (highs.distance / 10.0) as u64),
                                NEON_CYAN,
                            );
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_coin.clone(),
                                format!("{}", highs.coins),
                                NEON_GOLD,
                            );
                        });

                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button_blue.clone(),
                        assets.icon_play.clone(),
                        "RUN",
                        MenuButtonAction::Play,
                        true,
                    );
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button.clone(),
                        assets.icon_gear.clone(),
                        "SETTINGS",
                        MenuButtonAction::Settings,
                        true,
                    );

                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(8.0),
                                margin: UiRect::top(Val::Px(18.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_control_chip(
                                row,
                                &assets,
                                &[
                                    assets.key_a.clone(),
                                    assets.key_left.clone(),
                                    assets.key_d.clone(),
                                    assets.key_right.clone(),
                                ],
                                false,
                                "LANE",
                            );
                            spawn_control_chip(
                                row,
                                &assets,
                                &[
                                    assets.key_w.clone(),
                                    assets.key_up.clone(),
                                    assets.key_space.clone(),
                                ],
                                true,
                                "JUMP",
                            );
                            spawn_control_chip(
                                row,
                                &assets,
                                &[assets.key_s.clone(), assets.key_down.clone()],
                                false,
                                "SLIDE",
                            );
                            spawn_control_chip(
                                row,
                                &assets,
                                &[assets.key_esc.clone()],
                                true,
                                "PAUSE",
                            );
                        });
                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.0),
                                margin: UiRect::top(Val::Px(8.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn(ImageBundle {
                                style: Style {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
                                image: UiImage::new(assets.icon_info.clone()),
                                background_color: NEON_CYAN.into(),
                                ..default()
                            });
                            row.spawn(TextBundle::from_section(
                                "or swipe to steer, jump, and slide",
                                hud_style(&assets, 14.0, Color::rgb(0.7, 0.78, 0.95)),
                            ));
                        });
                });
        });
}

fn setup_settings_menu(mut commands: Commands, config: Res<GameConfig>, assets: Res<GameAssets>) {
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
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.78).into(),
                ..default()
            },
            OnSettingsMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(680.0),
                            max_width: Val::Percent(92.0),
                            padding: UiRect::all(Val::Px(32.0)),
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        image: UiImage::new(assets.panel_glass.clone()),
                        ..default()
                    },
                    nine_slice(),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        TextBundle::from_section(
                            format!("DIFFICULTY: {}", config.difficulty.label()),
                            title_style(&assets, 36.0, NEON_CYAN),
                        ),
                        DifficultyLabel,
                    ));
                    panel.spawn(
                        TextBundle::from_section(
                            "Speed, density, and how fast the horde closes in.",
                            body_style(&assets, 18.0, Color::rgb(0.86, 0.9, 1.0)),
                        )
                        .with_style(Style {
                            margin: UiRect::vertical(Val::Px(16.0)),
                            ..default()
                        }),
                    );

                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(12.0),
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            for (difficulty, image) in [
                                (Difficulty::Easy, assets.button_green.clone()),
                                (Difficulty::Normal, assets.button_yellow.clone()),
                                (Difficulty::Hard, assets.button_red.clone()),
                            ] {
                                row.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(150.0),
                                            height: Val::Px(56.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        image: UiImage::new(image),
                                        background_color: Color::WHITE.into(),
                                        ..default()
                                    },
                                    SettingsButtonAction::SetDifficulty(difficulty),
                                    DifficultyChip(difficulty),
                                ))
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        difficulty.label(),
                                        body_style(&assets, 20.0, Color::WHITE),
                                    ));
                                });
                            }
                        });

                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button.clone(),
                        assets.icon_home.clone(),
                        "BACK",
                        SettingsButtonAction::Back,
                        false,
                    );
                });
        });
}

fn setup_pause_menu(mut commands: Commands, assets: Res<GameAssets>) {
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
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(460.0),
                            padding: UiRect::all(Val::Px(28.0)),
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        image: UiImage::new(assets.panel_glass.clone()),
                        ..default()
                    },
                    nine_slice(),
                ))
                .with_children(|panel| {
                    panel
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(280.0),
                                    height: Val::Px(64.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::bottom(Val::Px(12.0)),
                                    ..default()
                                },
                                image: UiImage::new(assets.banner_hanging.clone()),
                                background_color: Color::WHITE.into(),
                                ..default()
                            },
                            nine_slice(),
                        ))
                        .with_children(|banner| {
                            banner.spawn(TextBundle::from_section(
                                "PAUSED",
                                title_style(&assets, 28.0, Color::rgb(0.22, 0.12, 0.06)),
                            ));
                        });
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button_blue.clone(),
                        assets.icon_play.clone(),
                        "RESUME",
                        MenuButtonAction::Resume,
                        true,
                    );
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button.clone(),
                        assets.icon_star.clone(),
                        "NEW RUN",
                        MenuButtonAction::Reset,
                        true,
                    );
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button.clone(),
                        assets.icon_gear.clone(),
                        "SETTINGS",
                        MenuButtonAction::Settings,
                        true,
                    );
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button.clone(),
                        assets.icon_home.clone(),
                        "MENU",
                        MenuButtonAction::BackToMenu,
                        true,
                    );
                });
        });
}

fn setup_game_ui(
    mut commands: Commands,
    existing: Query<Entity, With<ScoreText>>,
    assets: Res<GameAssets>,
) {
    if existing.iter().next().is_some() {
        return;
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|bar| {
            bar.spawn((
                ImageBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        min_width: Val::Px(180.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    image: UiImage::new(assets.panel_pixel.clone()),
                    ..default()
                },
                nine_slice(),
            ))
            .with_children(|left| {
                left.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            ..default()
                        },
                        image: UiImage::new(assets.icon_coin.clone()),
                        ..default()
                    });
                    row.spawn((
                        TextBundle::from_section("0", hud_style(&assets, 34.0, NEON_GOLD)),
                        CoinText,
                    ));
                });
                left.spawn((
                    TextBundle::from_section("SCORE 0", hud_style(&assets, 16.0, Color::WHITE)),
                    ScoreText,
                ));
                left.spawn((
                    TextBundle::from_section("", hud_style(&assets, 15.0, NEON_LIME)),
                    StatusText,
                ));
            });

            bar.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(280.0),
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    image: UiImage::new(assets.banner.clone()),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                nine_slice(),
            ))
            .with_children(|banner| {
                banner.spawn((
                    TextBundle::from_section(
                        "0m",
                        hud_style(&assets, 26.0, Color::rgb(0.2, 0.12, 0.08)),
                    ),
                    DistanceText,
                ));
            });

            bar.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(8.0),
                    min_width: Val::Px(180.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|right| {
                right
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(56.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            image: UiImage::new(assets.button_pause.clone()),
                            background_color: Color::WHITE.into(),
                            ..default()
                        },
                        PauseButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                ..default()
                            },
                            image: UiImage::new(assets.icon_pause.clone()),
                            ..default()
                        });
                    });
                right
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                            image: UiImage::new(assets.icon_heart.clone()),
                            ..default()
                        });
                        row.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                ..default()
                            },
                            image: UiImage::new(assets.icon_heart_empty.clone()),
                            ..default()
                        });
                        row.spawn(TextBundle::from_section(
                            "HORDE",
                            hud_style(&assets, 14.0, Color::rgb(1.0, 0.45, 0.45)),
                        ));
                    });
                right
                    .spawn((
                        ImageBundle {
                            style: Style {
                                width: Val::Px(160.0),
                                height: Val::Px(14.0),
                                ..default()
                            },
                            image: UiImage::new(assets.bar.clone()),
                            ..default()
                        },
                        nine_slice(),
                    ))
                    .with_children(|meter| {
                        meter.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(0.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                background_color: Color::rgba(1.0, 0.2, 0.28, 0.85).into(),
                                ..default()
                            },
                            ThreatFill,
                        ));
                    });
            });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(14.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|compass| {
            compass.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(18.0),
                    height: Val::Px(18.0),
                    ..default()
                },
                image: UiImage::new(assets.icon_arrow_up.clone()),
                background_color: NEON_CYAN.into(),
                ..default()
            });
            compass
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    row.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            ..default()
                        },
                        image: UiImage::new(assets.icon_arrow_left.clone()),
                        background_color: NEON_CYAN.into(),
                        ..default()
                    });
                    row.spawn(TextBundle::from_section(
                        "SWIPE",
                        hud_style(&assets, 12.0, Color::rgb(0.78, 0.86, 1.0)),
                    ));
                    row.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            ..default()
                        },
                        image: UiImage::new(assets.icon_arrow_right.clone()),
                        background_color: NEON_CYAN.into(),
                        ..default()
                    });
                });
            compass.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(18.0),
                    height: Val::Px(18.0),
                    ..default()
                },
                image: UiImage::new(assets.icon_arrow_down.clone()),
                background_color: NEON_CYAN.into(),
                ..default()
            });
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
    mut threat_query: Query<&mut Style, With<ThreatFill>>,
    player_query: Query<&Player>,
) {
    if let Ok(mut text) = score_query.get_single_mut() {
        text.sections[0].value = format!("SCORE {}", stats.score());
    }
    if let Ok(mut text) = distance_query.get_single_mut() {
        text.sections[0].value = format!("{}m", (stats.distance / 10.0) as u64);
    }
    if let Ok(mut text) = coin_query.get_single_mut() {
        text.sections[0].value = format!("{}", stats.coins);
    }
    if let Ok(mut fill) = threat_query.get_single_mut() {
        fill.width = Val::Percent((stats.threat * 100.0).clamp(0.0, 100.0));
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
            parts.push("CLOSING IN".to_string());
        }
        text.sections[0].value = parts.join("  ");
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

fn setup_game_over_screen(mut commands: Commands, last: Res<LastRun>, assets: Res<GameAssets>) {
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
                background_color: Color::rgba(0.05, 0.0, 0.07, 0.82).into(),
                ..default()
            },
            OnGameOverMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(360.0),
                            height: Val::Px(70.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        },
                        image: UiImage::new(assets.banner_wide.clone()),
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                    nine_slice(),
                ))
                .with_children(|banner| {
                    banner.spawn(TextBundle::from_section(
                        format!("{}m", (last.distance / 10.0) as u64),
                        title_style(&assets, 36.0, Color::rgb(0.22, 0.12, 0.06)),
                    ));
                });
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(680.0),
                            max_width: Val::Percent(94.0),
                            padding: UiRect::all(Val::Px(32.0)),
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        image: UiImage::new(assets.panel_glass.clone()),
                        ..default()
                    },
                    nine_slice(),
                ))
                .with_children(|panel| {
                    panel.spawn(
                        TextBundle::from_section(
                            last.reason.headline(),
                            title_style(&assets, 28.0, NEON_MAGENTA),
                        )
                        .with_style(Style {
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        }),
                    );
                    if last.new_high_score {
                        spawn_icon_label(
                            panel,
                            &assets,
                            assets.world_star.clone(),
                            "NEW HIGH SCORE",
                            NEON_GOLD,
                        );
                    }
                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(20.0),
                                margin: UiRect::vertical(Val::Px(14.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_star.clone(),
                                format!("{}", last.score),
                                Color::WHITE,
                            );
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_trophy.clone(),
                                format!("{}m", (last.distance / 10.0) as u64),
                                NEON_CYAN,
                            );
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_coin.clone(),
                                format!("{}", last.coins),
                                NEON_GOLD,
                            );
                        });
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button_blue.clone(),
                        assets.icon_play.clone(),
                        "RUN AGAIN",
                        MenuButtonAction::Reset,
                        true,
                    );
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button.clone(),
                        assets.icon_home.clone(),
                        "MENU",
                        MenuButtonAction::BackToMenu,
                        true,
                    );
                });
        });
}

fn paint_menu_buttons(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuButton>, With<Button>),
    >,
) {
    for (interaction, mut background) in &mut query {
        *background = match interaction {
            Interaction::Pressed => Color::rgb(0.55, 0.9, 1.0).into(),
            Interaction::Hovered => Color::rgb(0.82, 1.0, 1.0).into(),
            Interaction::None => Color::WHITE.into(),
        };
    }
}

fn paint_difficulty_chips(
    config: Res<GameConfig>,
    mut query: Query<(&DifficultyChip, &mut BackgroundColor)>,
) {
    for (chip, mut background) in &mut query {
        *background = if chip.0 == config.difficulty {
            Color::WHITE.into()
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.42).into()
        };
    }
}
