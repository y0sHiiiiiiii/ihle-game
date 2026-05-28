//! Jannicks Shop — Pizza + Kaffee fuer Speed-Boost.

use bevy::prelude::*;

use crate::game::delivery::DeliveryStats;
use crate::game::gamestate::GameState;
use crate::game::player::Player;

#[derive(Event)]
pub struct ShopRequest {
    pub dialog: String,
}

#[derive(Resource, Default)]
pub struct ShopUiState {
    pub dialog: String,
}

#[derive(Component)]
pub struct ShopUiRoot;

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct ShopFeedback;

#[derive(Resource, Default)]
pub struct ShopFeedbackTimer(pub f32);

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShopRequest>()
            .init_resource::<ShopUiState>()
            .init_resource::<ShopFeedbackTimer>()
            .add_systems(
                Update,
                (open_shop_on_request,)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::Shopping), spawn_shop_ui)
            .add_systems(OnExit(GameState::Shopping), despawn_shop_ui)
            .add_systems(
                Update,
                (shop_input, update_feedback, update_coins_text)
                    .run_if(in_state(GameState::Shopping)),
            );
    }
}

fn open_shop_on_request(
    mut events: EventReader<ShopRequest>,
    mut next_state: ResMut<NextState<GameState>>,
    mut shop_state: ResMut<ShopUiState>,
) {
    if let Some(ev) = events.read().next() {
        shop_state.dialog = ev.dialog.clone();
        next_state.set(GameState::Shopping);
    }
    events.clear();
}

fn spawn_shop_ui(
    mut commands: Commands,
    state: Res<ShopUiState>,
    stats: Res<DeliveryStats>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.55).into(),
                ..default()
            },
            ShopUiRoot,
        ))
        .with_children(|outer| {
            outer
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(520.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::srgb(0.12, 0.05, 0.05).into(),
                    border_color: Color::srgb(0.95, 0.85, 0.2).into(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "JANNICKS KOELNER ECK",
                        TextStyle {
                            font_size: 32.0,
                            color: Color::srgb(0.95, 0.85, 0.2),
                            ..default()
                        },
                    ));

                    p.spawn(
                        TextBundle::from_section(
                            state.dialog.clone(),
                            TextStyle {
                                font_size: 18.0,
                                color: Color::srgb(0.95, 0.95, 0.95),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        }),
                    );

                    p.spawn((
                        TextBundle::from_section(
                            format!("Muenzen: {}", stats.coins),
                            TextStyle {
                                font_size: 22.0,
                                color: Color::srgb(1.0, 0.85, 0.25),
                                ..default()
                            },
                        ),
                        ShopCoinsText,
                    ));

                    p.spawn(
                        TextBundle::from_section(
                            "[1]  Margherita Pizza        5 Muenzen   +30s Speed-Boost (+40%)",
                            TextStyle {
                                font_size: 20.0,
                                color: Color::srgb(0.85, 0.95, 0.85),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(16.0)),
                            ..default()
                        }),
                    );

                    p.spawn(
                        TextBundle::from_section(
                            "[2]  'Geiler Kaffee'         3 Muenzen   +20s Speed-Boost (+25%)",
                            TextStyle {
                                font_size: 20.0,
                                color: Color::srgb(0.85, 0.95, 0.85),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(6.0)),
                            ..default()
                        }),
                    );

                    p.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::srgb(0.95, 0.7, 0.3),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(12.0)),
                            min_height: Val::Px(24.0),
                            ..default()
                        }),
                        ShopFeedback,
                    ));

                    p.spawn(
                        TextBundle::from_section(
                            "[ESC] verlassen",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.7, 0.7, 0.75),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(18.0)),
                            ..default()
                        }),
                    );
                });
        });
}

fn despawn_shop_ui(mut commands: Commands, q: Query<Entity, With<ShopUiRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn shop_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stats: ResMut<DeliveryStats>,
    mut player_q: Query<&mut Player>,
    mut feedback_timer: ResMut<ShopFeedbackTimer>,
    mut feedback_q: Query<&mut Text, With<ShopFeedback>>,
    mut blip: EventWriter<crate::game::audio::UiBlipEvent>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        blip.send(crate::game::audio::UiBlipEvent);
        next_state.set(GameState::Playing);
        return;
    }

    let buy1 = keys.just_pressed(KeyCode::Digit1) || keys.just_pressed(KeyCode::Numpad1);
    let buy2 = keys.just_pressed(KeyCode::Digit2) || keys.just_pressed(KeyCode::Numpad2);
    if !buy1 && !buy2 {
        return;
    }
    blip.send(crate::game::audio::UiBlipEvent);

    let mut feedback = |msg: &str| {
        if let Ok(mut text) = feedback_q.get_single_mut() {
            text.sections[0].value = msg.to_string();
        }
        feedback_timer.0 = 2.0;
    };

    if buy1 {
        if stats.coins >= 5 {
            stats.coins -= 5;
            if let Ok(mut player) = player_q.get_single_mut() {
                player.speed_boost_timer = 30.0;
                player.speed_boost_factor = 0.40;
            }
            feedback("Lecker Pizza! Schub fuer 30 Sekunden!");
        } else {
            feedback("Du hast nit genug Muenzen, kumm spaeter!");
        }
    } else if buy2 {
        if stats.coins >= 3 {
            stats.coins -= 3;
            if let Ok(mut player) = player_q.get_single_mut() {
                player.speed_boost_timer = 20.0;
                player.speed_boost_factor = 0.25;
            }
            feedback("Geiler Kaffee! 20 Sekunden Power!");
        } else {
            feedback("3 Muenzen brauchste! Schaff doch erstmal nen Auftrag!");
        }
    }
}

fn update_feedback(
    time: Res<Time>,
    mut timer: ResMut<ShopFeedbackTimer>,
    mut feedback_q: Query<&mut Text, With<ShopFeedback>>,
) {
    if timer.0 > 0.0 {
        timer.0 -= time.delta_seconds();
        if timer.0 <= 0.0 {
            timer.0 = 0.0;
            if let Ok(mut text) = feedback_q.get_single_mut() {
                text.sections[0].value.clear();
            }
        }
    }
}

fn update_coins_text(stats: Res<DeliveryStats>, mut q: Query<&mut Text, With<ShopCoinsText>>) {
    if let Ok(mut text) = q.get_single_mut() {
        text.sections[0].value = format!("Muenzen: {}", stats.coins);
    }
}
