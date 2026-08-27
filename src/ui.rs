use crate::assets::GameAssets;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, ImageScaleMode, SliceScaleMode, TextureSlicer};
use fuzzy_runner::{
    combo_timer_fraction, countdown_label, despawn_screen, displayed_meters, milestone_crossed,
    safe_timer, set_text, try_despawn, AnimationIndices, AnimationTimer, ChaserWarn, CoinCollected,
    CoinHudPunch, CoinText, ComboFill, ComboText, Countdown, DeathEvent, Difficulty, DistanceText,
    GameConfig, GameState, GoSplash, HighScores, IgnoreSwipe, LastRun, MilestoneToast,
    OnGameOverMenu, OnGameScreen, OnMainMenu, OnPauseMenu, OnSettingsMenu, PauseButton,
    PendingCommands, Player, PlayerStumbled, PowerChip, PowerUpCollected, PowerUpKind,
    RunnerCommand, RunStats, ScoreText, ScreenFlash, SettingsOrigin, StatusText, ThreatFill,
    TitlePreview, TouchControl, Vignette, COMBO_WINDOW, GROUND_Y, NEON_CYAN, NEON_GOLD, NEON_LIME,
    NEON_MAGENTA, PLATFORM_THICKNESS, PLAYER_SIZE,
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

#[derive(Component)]
struct CountdownLabel;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(GameState::Menu),
                (setup_main_menu, setup_title_preview),
            )
            .add_systems(
                OnExit(GameState::Menu),
                (
                    despawn_screen::<OnMainMenu>,
                    despawn_screen::<TitlePreview>,
                ),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (setup_game_ui, setup_go_splash).chain(),
            )
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_screen::<OnPauseMenu>)
            .add_systems(
                OnEnter(GameState::GameOver),
                (record_last_run, setup_game_over_screen).chain(),
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
                    tick_go_splash.run_if(in_state(GameState::Playing)),
                    announce_milestones.run_if(in_state(GameState::Playing)),
                    tick_toasts.run_if(in_state(GameState::Playing)),
                    flash_on_stumble.run_if(in_state(GameState::Playing)),
                    tick_screen_flash.run_if(in_state(GameState::Playing)),
                    punch_coin_hud.run_if(in_state(GameState::Playing)),
                    paint_menu_buttons,
                    start_from_keyboard.run_if(in_state(GameState::Menu)),
                    restart_from_keyboard.run_if(in_state(GameState::GameOver)),
                ),
            )
            .add_systems(
                Update,
                (
                    update_combo_hud.run_if(in_state(GameState::Playing)),
                    handle_touch_controls.run_if(in_state(GameState::Playing)),
                    animate_title_preview.run_if(in_state(GameState::Menu)),
                    pin_title_preview.run_if(in_state(GameState::Menu)),
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
    last.best_combo = stats.best_combo;
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
        set_text(&mut text, format!("DIFFICULTY: {}", config.difficulty.label()));
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

fn spawn_power_chip(parent: &mut ChildBuilder, assets: &GameAssets, kind: PowerUpKind) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(3.0),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            PowerChip { kind },
        ))
        .with_children(|chip| {
            chip.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                image: UiImage::new(assets.power_icon(kind)),
                ..default()
            });
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
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        }),
                    );
                    panel.spawn(
                        TextBundle::from_section(
                            "TAP TO RUN",
                            hud_style(&assets, 16.0, NEON_CYAN),
                        )
                        .with_style(Style {
                            margin: UiRect::bottom(Val::Px(14.0)),
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
                                assets.star_badge.clone(),
                                format!("{}", highs.score),
                                NEON_GOLD,
                            );
                            spawn_icon_label(
                                row,
                                &assets,
                                assets.icon_trophy.clone(),
                                format!("{}m", displayed_meters(highs.distance)),
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
                                image: UiImage::new(assets.icon_power.clone()),
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
                    spawn_icon_label(
                        panel,
                        &assets,
                        assets.icon_info.clone(),
                        "Speed, density, and how fast the horde closes in.",
                        Color::rgb(0.86, 0.9, 1.0),
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
                            banner.spawn(ImageBundle {
                                style: Style {
                                    width: Val::Px(22.0),
                                    height: Val::Px(22.0),
                                    margin: UiRect::right(Val::Px(8.0)),
                                    ..default()
                                },
                                image: UiImage::new(assets.icon_locked.clone()),
                                background_color: Color::rgb(0.25, 0.12, 0.06).into(),
                                ..default()
                            });
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
                    row.spawn((
                        ImageBundle {
                            style: Style {
                                width: Val::Px(36.0),
                                height: Val::Px(36.0),
                                ..default()
                            },
                            image: UiImage::new(assets.icon_coin.clone()),
                            ..default()
                        },
                        CoinHudPunch {
                            timer: safe_timer(0.01, TimerMode::Once),
                        },
                    ));
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
                    TextBundle::from_section("", hud_style(&assets, 16.0, Color::rgb(1.0, 0.7, 0.2))),
                    ComboText,
                ));
                left.spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(120.0),
                            height: Val::Px(8.0),
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
                            background_color: Color::rgba(1.0, 0.78, 0.2, 0.92).into(),
                            ..default()
                        },
                        ComboFill,
                    ));
                });
                left.spawn((
                    TextBundle::from_section("", hud_style(&assets, 15.0, NEON_LIME)),
                    StatusText,
                ));
                left.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    spawn_power_chip(row, &assets, PowerUpKind::Magnet);
                    spawn_power_chip(row, &assets, PowerUpKind::Shield);
                    spawn_power_chip(row, &assets, PowerUpKind::Multiplier);
                    spawn_power_chip(row, &assets, PowerUpKind::Boost);
                });
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
                        row.spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    ..default()
                                },
                                image: UiImage::new(assets.icon_exclaim.clone()),
                                background_color: Color::rgba(1.0, 0.3, 0.3, 0.0).into(),
                                ..default()
                            },
                            ChaserWarn,
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
            compass
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(3.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|row| {
                    for icon in [
                        assets.icon_arrow_left.clone(),
                        assets.icon_arrow_up.clone(),
                        assets.icon_arrow_down.clone(),
                        assets.icon_arrow_right.clone(),
                    ] {
                        row.spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(14.0),
                                height: Val::Px(14.0),
                                ..default()
                            },
                            image: UiImage::new(icon),
                            background_color: NEON_CYAN.into(),
                            ..default()
                        });
                    }
                });
            compass.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(26.0),
                    height: Val::Px(26.0),
                    ..default()
                },
                image: UiImage::new(assets.icon_dpad.clone()),
                background_color: NEON_CYAN.into(),
                ..default()
            });
            compass.spawn(TextBundle::from_section(
                "SWIPE OR TAP",
                hud_style(&assets, 12.0, Color::rgb(0.78, 0.86, 1.0)),
            ));
            compass.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(22.0),
                    height: Val::Px(22.0),
                    ..default()
                },
                image: UiImage::new(assets.icon_mouse.clone()),
                background_color: Color::WHITE.into(),
                ..default()
            });
            compass.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(22.0),
                    height: Val::Px(22.0),
                    ..default()
                },
                image: UiImage::new(assets.icon_tilt.clone()),
                background_color: Color::WHITE.into(),
                ..default()
            });
        });

    spawn_vignette(&mut commands);
    spawn_touch_pads(&mut commands, &assets);
}

fn spawn_touch_pads(commands: &mut Commands, assets: &GameAssets) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    bottom: Val::Px(10.0),
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            OnGameScreen,
        ))
        .with_children(|row| {
            for (command, icon, image) in [
                (
                    RunnerCommand::LaneLeft,
                    assets.icon_arrow_left.clone(),
                    assets.button.clone(),
                ),
                (
                    RunnerCommand::Jump,
                    assets.icon_arrow_up.clone(),
                    assets.button_blue.clone(),
                ),
                (
                    RunnerCommand::Slide,
                    assets.icon_arrow_down.clone(),
                    assets.button_yellow.clone(),
                ),
                (
                    RunnerCommand::LaneRight,
                    assets.icon_arrow_right.clone(),
                    assets.button_green.clone(),
                ),
            ] {
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(78.0),
                            height: Val::Px(78.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        image: UiImage::new(image),
                        background_color: Color::rgba(1.0, 1.0, 1.0, 0.82).into(),
                        z_index: ZIndex::Global(10),
                        ..default()
                    },
                    TouchControl(command),
                ))
                .with_children(|btn| {
                    btn.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            ..default()
                        },
                        image: UiImage::new(icon),
                        ..default()
                    });
                });
            }
        });
}

fn spawn_vignette(commands: &mut Commands) {
    for (width, height, left, right, top, bottom) in [
        (
            Val::Percent(100.0),
            Val::Px(90.0),
            Val::Px(0.0),
            Val::Auto,
            Val::Px(0.0),
            Val::Auto,
        ),
        (
            Val::Percent(100.0),
            Val::Px(110.0),
            Val::Px(0.0),
            Val::Auto,
            Val::Auto,
            Val::Px(0.0),
        ),
        (
            Val::Px(70.0),
            Val::Percent(100.0),
            Val::Px(0.0),
            Val::Auto,
            Val::Px(0.0),
            Val::Auto,
        ),
        (
            Val::Px(70.0),
            Val::Percent(100.0),
            Val::Auto,
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Auto,
        ),
    ] {
        commands.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width,
                    height,
                    left,
                    right,
                    top,
                    bottom,
                    ..default()
                },
                background_color: Color::rgba(0.04, 0.0, 0.08, 0.12).into(),
                z_index: ZIndex::Global(-1),
                ..default()
            },
            Vignette,
            OnGameScreen,
        ));
    }
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
    mut chips: Query<(&PowerChip, &mut Visibility)>,
    mut warn: Query<&mut BackgroundColor, With<ChaserWarn>>,
    player_query: Query<&Player>,
) {
    if let Ok(mut text) = score_query.get_single_mut() {
        set_text(&mut text, format!("SCORE {}", stats.score()));
    }
    if let Ok(mut text) = distance_query.get_single_mut() {
        set_text(&mut text, format!("{}m", displayed_meters(stats.distance)));
    }
    if let Ok(mut text) = coin_query.get_single_mut() {
        set_text(&mut text, format!("{}", stats.coins));
    }
    if let Ok(mut fill) = threat_query.get_single_mut() {
        fill.width = Val::Percent((stats.threat * 100.0).clamp(0.0, 100.0));
    }
    let has_shield = player_query
        .get_single()
        .map(|player| player.has_shield)
        .unwrap_or(false);
    for (chip, mut visibility) in &mut chips {
        let on = match chip.kind {
            PowerUpKind::Magnet => stats.magnet_active(),
            PowerUpKind::Shield => has_shield,
            PowerUpKind::Multiplier => stats.multiplier_active(),
            PowerUpKind::Boost => stats.boost_active(),
        };
        *visibility = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut color) = warn.get_single_mut() {
        *color = if stats.threat > 0.45 {
            Color::rgba(1.0, 0.25, 0.25, 0.35 + stats.threat * 0.65).into()
        } else {
            Color::rgba(1.0, 0.3, 0.3, 0.0).into()
        };
    }
    if let Ok(mut text) = status_query.get_single_mut() {
        set_text(
            &mut text,
            if stats.threat > 0.35 {
                "CLOSING IN".to_string()
            } else {
                String::new()
            },
        );
    }
}

fn show_power_up_status(
    mut events: EventReader<PowerUpCollected>,
    mut status_query: Query<&mut Text, With<StatusText>>,
) {
    if let Some(event) = events.read().last() {
        if let Ok(mut text) = status_query.get_single_mut() {
            set_text(&mut text, event.kind.label());
        }
    }
}

fn setup_game_over_screen(
    mut commands: Commands,
    last: Res<LastRun>,
    highs: Res<HighScores>,
    assets: Res<GameAssets>,
) {
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
                        format!("{}m", displayed_meters(last.distance)),
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
                            assets.icon_check.clone(),
                            "NEW HIGH SCORE",
                            NEON_GOLD,
                        );
                    }
                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(28.0),
                                margin: UiRect::vertical(Val::Px(14.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_run_column(
                                row,
                                &assets,
                                assets.icon_medal.clone(),
                                "THIS RUN",
                                last.score,
                                displayed_meters(last.distance),
                                last.coins,
                            );
                            spawn_run_column(
                                row,
                                &assets,
                                assets.icon_board.clone(),
                                "BEST",
                                highs.score,
                                displayed_meters(highs.distance),
                                highs.coins,
                            );
                        });
                    if last.best_combo >= 3 {
                        spawn_icon_label(
                            panel,
                            &assets,
                            assets.icon_star.clone(),
                            format!("BEST COMBO x{}", last.best_combo),
                            NEON_GOLD,
                        );
                    }
                    spawn_image_button(
                        panel,
                        &assets,
                        assets.button_blue.clone(),
                        assets.icon_repeat.clone(),
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

fn spawn_run_column(
    parent: &mut ChildBuilder,
    assets: &GameAssets,
    icon: Handle<Image>,
    title: &str,
    score: u64,
    meters: u64,
    coins: u32,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                min_width: Val::Px(180.0),
                ..default()
            },
            ..default()
        })
        .with_children(|col| {
            spawn_icon_label(col, assets, icon, title, NEON_CYAN);
            col.spawn(TextBundle::from_section(
                format!("{score}"),
                hud_style(assets, 22.0, Color::WHITE),
            ));
            col.spawn(TextBundle::from_section(
                format!("{meters}m   {coins} coins"),
                hud_style(assets, 16.0, NEON_GOLD),
            ));
        });
}

fn setup_go_splash(mut commands: Commands, stats: Res<RunStats>, assets: Res<GameAssets>) {
    if stats.distance > 8.0 {
        return;
    }
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.22).into(),
                ..default()
            },
            GoSplash {
                timer: safe_timer(3.2, TimerMode::Once),
            },
            OnGameScreen,
        ))
        .with_children(|root| {
            root.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(280.0),
                        height: Val::Px(90.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: UiImage::new(assets.banner_hanging.clone()),
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                nine_slice(),
            ))
            .with_children(|banner| {
                banner.spawn((
                    TextBundle::from_section(
                        "3",
                        title_style(&assets, 48.0, Color::rgb(0.18, 0.08, 0.04)),
                    ),
                    CountdownLabel,
                ));
            });
        });
}

fn tick_go_splash(
    mut commands: Commands,
    countdown: Res<Countdown>,
    mut query: Query<(Entity, &mut BackgroundColor), With<GoSplash>>,
    mut labels: Query<&mut Text, With<CountdownLabel>>,
) {
    if let Some(label) = countdown_label(countdown.remaining) {
        for mut text in &mut labels {
            set_text(&mut text, label);
        }
        let fade = if label == "GO!" { 0.10 } else { 0.22 };
        for (_, mut background) in &mut query {
            *background = Color::rgba(0.0, 0.0, 0.0, fade).into();
        }
    } else {
        for (entity, _) in &query {
            try_despawn(&mut commands, entity);
        }
    }
}

fn announce_milestones(
    mut commands: Commands,
    stats: Res<RunStats>,
    assets: Res<GameAssets>,
    mut last_distance: Local<f32>,
) {
    if stats.distance + 1.0 < *last_distance {
        *last_distance = stats.distance;
    }
    if let Some(meters) = milestone_crossed(*last_distance, stats.distance, 500) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: Val::Px(96.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                MilestoneToast {
                    timer: safe_timer(1.4, TimerMode::Once),
                },
                OnGameScreen,
            ))
            .with_children(|root| {
                root.spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(240.0),
                            height: Val::Px(58.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },
                        image: UiImage::new(assets.banner.clone()),
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                    nine_slice(),
                ))
                .with_children(|banner| {
                    banner.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(22.0),
                            height: Val::Px(22.0),
                            ..default()
                        },
                        image: UiImage::new(assets.world_star.clone()),
                        ..default()
                    });
                    banner.spawn(TextBundle::from_section(
                        format!("{meters}m"),
                        title_style(&assets, 26.0, Color::rgb(0.2, 0.1, 0.06)),
                    ));
                });
            });
    }
    *last_distance = stats.distance;
}

fn tick_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MilestoneToast)>,
) {
    for (entity, mut toast) in &mut query {
        toast.timer.tick(time.delta());
        if toast.timer.finished() {
            try_despawn(&mut commands, entity);
        }
    }
}

fn flash_on_stumble(
    mut commands: Commands,
    mut events: EventReader<PlayerStumbled>,
    existing: Query<Entity, With<ScreenFlash>>,
) {
    if events.read().last().is_none() {
        return;
    }
    if existing.iter().next().is_some() {
        return;
    }
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: Color::rgba(1.0, 0.15, 0.2, 0.32).into(),
            ..default()
        },
        ScreenFlash {
            timer: safe_timer(0.28, TimerMode::Once),
        },
        OnGameScreen,
    ));
}

fn tick_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
) {
    for (entity, mut flash, mut background) in &mut query {
        flash.timer.tick(time.delta());
        let fade = 1.0 - flash.timer.fraction();
        *background = Color::rgba(1.0, 0.15, 0.2, 0.32 * fade).into();
        if flash.timer.finished() {
            try_despawn(&mut commands, entity);
        }
    }
}

fn punch_coin_hud(
    mut events: EventReader<CoinCollected>,
    time: Res<Time>,
    mut query: Query<(&mut Style, &mut CoinHudPunch)>,
) {
    let punched = events.read().last().is_some();
    for (mut style, mut punch) in &mut query {
        if punched {
            punch.timer = safe_timer(0.22, TimerMode::Once);
        }
        punch.timer.tick(time.delta());
        let t = if punch.timer.finished() {
            0.0
        } else {
            1.0 - punch.timer.fraction()
        };
        let size = 36.0 + 10.0 * t;
        style.width = Val::Px(size);
        style.height = Val::Px(size);
    }
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

fn update_combo_hud(
    stats: Res<RunStats>,
    mut query: Query<&mut Text, With<ComboText>>,
    mut fill: Query<&mut Style, With<ComboFill>>,
) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };
    let active = stats.combo >= 3;
    set_text(
        &mut text,
        if active {
            format!("COMBO x{}", stats.combo)
        } else {
            String::new()
        },
    );
    if let Ok(mut style) = fill.get_single_mut() {
        style.width = Val::Percent(if active {
            combo_timer_fraction(stats.combo_timer, COMBO_WINDOW) * 100.0
        } else {
            0.0
        });
    }
}

fn handle_touch_controls(
    query: Query<(&Interaction, &TouchControl), (Changed<Interaction>, With<Button>)>,
    mut pending: ResMut<PendingCommands>,
    mut ignore_swipe: ResMut<IgnoreSwipe>,
) {
    for (interaction, control) in &query {
        if *interaction == Interaction::Pressed {
            pending.push(control.0);
            ignore_swipe.0 = true;
        }
    }
}

fn setup_title_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    existing: Query<Entity, With<TitlePreview>>,
) {
    if existing.iter().next().is_some() {
        return;
    }
    let texture: Handle<Image> = asset_server.load("player_tilesheet.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(80.0, 110.0), 9, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let y = GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (PLAYER_SIZE.y / 2.0);
    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 9,
            },
            transform: Transform::from_xyz(430.0, y, 6.0).with_scale(Vec3::splat(1.2)),
            ..default()
        },
        TitlePreview,
        AnimationIndices { first: 9, last: 10 },
        AnimationTimer(safe_timer(0.10, TimerMode::Repeating)),
    ));
}

fn animate_title_preview(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &AnimationIndices, &mut TextureAtlas), With<TitlePreview>>,
) {
    for (mut timer, indices, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if atlas.index >= indices.last {
                atlas.index = indices.first;
            } else {
                atlas.index += 1;
            }
        }
    }
}

fn pin_title_preview(
    camera: Query<&Transform, (With<Camera>, Without<TitlePreview>)>,
    mut preview: Query<&mut Transform, With<TitlePreview>>,
) {
    let Ok(camera) = camera.get_single() else {
        return;
    };
    let Ok(mut transform) = preview.get_single_mut() else {
        return;
    };
    transform.translation.x = camera.translation.x + 430.0;
    transform.translation.y = GROUND_Y + (PLATFORM_THICKNESS / 2.0) + (PLAYER_SIZE.y / 2.0);
    transform.translation.z = 6.0;
}
