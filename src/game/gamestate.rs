//! GameState-Enum + zentrale Steuerung des Spielablaufs

use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    Shopping,
    GameOver,
}

#[derive(Component)]
pub struct MainMenuMarker;

#[derive(Component)]
pub struct PauseMenuMarker;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
            .add_systems(
                Update,
                (
                    main_menu_input.run_if(in_state(GameState::MainMenu)),
                    pause_input.run_if(in_state(GameState::Playing)),
                    unpause_input.run_if(in_state(GameState::Paused)),
                ),
            );
    }
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainMenuMarker));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: Color::srgb(0.04, 0.06, 0.11).into(),
                ..default()
            },
            MainMenuMarker,
        ))
        .with_children(|parent| {
            // Title with a dark "shadow" copy behind it for a pixel-sign feel.
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|t| {
                    t.spawn(
                        TextBundle::from_section(
                            "GERMERING DELIVERY",
                            TextStyle {
                                font_size: 76.0,
                                color: Color::srgb(0.0, 0.0, 0.0),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(4.0),
                            top: Val::Px(4.0),
                            ..default()
                        }),
                    );
                    t.spawn(TextBundle::from_section(
                        "GERMERING DELIVERY",
                        TextStyle {
                            font_size: 76.0,
                            color: Color::srgb(0.98, 0.82, 0.15),
                            ..default()
                        },
                    ));
                });

            parent.spawn(
                TextBundle::from_section(
                    "Ihle-Sprinter Lieferfahrer — Highscore-Jagd in Germering",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.55, 0.8, 1.0),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                }),
            );

            parent.spawn(
                TextBundle::from_section(
                    "[ LEERTASTE ]  —  Schicht beginnen",
                    TextStyle {
                        font_size: 34.0,
                        color: Color::srgb(0.45, 0.95, 0.5),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(26.0)),
                    ..default()
                }),
            );

            // Framed control / goal card.
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    background_color: Color::srgba(0.07, 0.09, 0.14, 0.95).into(),
                    border_color: Color::srgb(0.3, 0.34, 0.46).into(),
                    ..default()
                })
                .with_children(|card| {
                    for (line, color) in [
                        ("WASD / Pfeiltasten — Fahren", Color::srgb(0.9, 0.9, 0.95)),
                        ("E — Abholen / Abliefern / Jannick", Color::srgb(0.9, 0.9, 0.95)),
                        ("LEERTASTE — Nitro-Schub (wenn voll)", Color::srgb(0.5, 0.85, 1.0)),
                        ("ESC — Pause", Color::srgb(0.9, 0.9, 0.95)),
                    ] {
                        card.spawn(TextBundle::from_section(
                            line,
                            TextStyle {
                                font_size: 19.0,
                                color,
                                ..default()
                            },
                        ));
                    }
                    card.spawn(
                        TextBundle::from_section(
                            "Liefere so viele Pakete wie moeglich aus —\n3 verpasste Kunden und es ist FEIERABEND!",
                            TextStyle {
                                font_size: 17.0,
                                color: Color::srgb(0.7, 0.72, 0.8),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        })
                        .with_text_justify(JustifyText::Center),
                    );
                });
        });
}

fn despawn_main_menu(mut commands: Commands, q: Query<Entity, With<MainMenuMarker>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut blip: EventWriter<crate::game::audio::UiBlipEvent>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        blip.send(crate::game::audio::UiBlipEvent);
        next_state.set(GameState::Playing);
    }
}

fn pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut blip: EventWriter<crate::game::audio::UiBlipEvent>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        blip.send(crate::game::audio::UiBlipEvent);
        next_state.set(GameState::Paused);
    }
}

fn unpause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut blip: EventWriter<crate::game::audio::UiBlipEvent>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        blip.send(crate::game::audio::UiBlipEvent);
        next_state.set(GameState::Playing);
    }
}

fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.75).into(),
                ..default()
            },
            PauseMenuMarker,
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(36.0)),
                    row_gap: Val::Px(18.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                background_color: Color::srgba(0.07, 0.09, 0.14, 0.96).into(),
                border_color: Color::srgb(0.95, 0.85, 0.2).into(),
                ..default()
            })
            .with_children(|card| {
                card.spawn(TextBundle::from_section(
                    "PAUSE",
                    TextStyle {
                        font_size: 84.0,
                        color: Color::srgb(0.98, 0.82, 0.15),
                        ..default()
                    },
                ));
                card.spawn(TextBundle::from_section(
                    "[ESC]  weiterspielen",
                    TextStyle {
                        font_size: 26.0,
                        color: Color::srgb(0.5, 0.95, 0.55),
                        ..default()
                    },
                ));
            });
        });
}

fn despawn_pause_menu(mut commands: Commands, q: Query<Entity, With<PauseMenuMarker>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
