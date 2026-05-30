//! HUD: Timer, Score, Muenzen, Leben-Icons, Auftrag, Minimap, Feedback-Banner.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::assets::{set_px, ts_body, GameAssets, UiFonts};
use crate::game::delivery::{
    ActiveDelivery, DeliveryFeedback, DeliveryPhase, DeliveryStats, LateNotification,
    INTERACT_RADIUS,
};
use crate::game::gamestate::GameState;
use crate::game::map::{GameMap, MAP_HEIGHT, MAP_WIDTH};
use crate::game::navi::{
    tile_color_for_minimap, NaviBoostText, NaviDirectionText, NaviDistanceText,
};
use crate::game::player::Player;

pub const MINIMAP_PX: u32 = 128;
pub const MINIMAP_DISPLAY_PX: f32 = 180.0;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct CoinsText;

#[derive(Component)]
pub struct LivesRow;

#[derive(Component)]
pub struct LifeIcon;

#[derive(Component)]
pub struct OrderText;

#[derive(Component)]
pub struct FeedbackBanner;

#[derive(Component)]
pub struct LateBanner;

#[derive(Component)]
pub struct MinimapContainer;

#[derive(Component)]
pub struct MinimapPlayerDot;

#[derive(Component)]
pub struct MinimapPickupDot;

#[derive(Component)]
pub struct MinimapDropoffDot;

#[derive(Component)]
pub struct NitroBarFill;

#[derive(Component)]
pub struct NitroLabel;

#[derive(Component)]
pub struct InteractHint;

#[derive(Resource)]
pub struct MinimapImage {
    #[allow(dead_code)]
    pub handle: Handle<Image>,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(OnExit(GameState::Playing), despawn_hud)
            .add_systems(
                Update,
                (
                    update_timer,
                    update_score_coins,
                    update_lives,
                    update_order_text,
                    update_feedback,
                    update_late_banner,
                    update_minimap_dots,
                    update_nitro_bar,
                    update_interact_hint,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (update_minimap_dots,).run_if(in_state(GameState::Shopping)),
            );
    }
}

fn setup_hud(
    mut commands: Commands,
    assets: Res<GameAssets>,
    fonts: Res<UiFonts>,
    map: Res<GameMap>,
    mut images: ResMut<Assets<Image>>,
    existing: Query<Entity, With<HudRoot>>,
) {
    for e in &existing {
        commands.entity(e).despawn_recursive();
    }

    let mut buf = vec![0u8; (MINIMAP_PX * MINIMAP_PX * 4) as usize];
    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let t = map.tile_at(tx, ty);
            let mx = (tx as f32 / MAP_WIDTH as f32 * MINIMAP_PX as f32) as i32;
            let my = ((MAP_HEIGHT - 1 - ty) as f32 / MAP_HEIGHT as f32 * MINIMAP_PX as f32) as i32;
            let c = tile_color_for_minimap(t);
            set_px(&mut buf, MINIMAP_PX, mx, my, c);
        }
    }
    for i in 0..MINIMAP_PX as i32 {
        set_px(&mut buf, MINIMAP_PX, 0, i, (0, 0, 0, 255));
        set_px(&mut buf, MINIMAP_PX, MINIMAP_PX as i32 - 1, i, (0, 0, 0, 255));
        set_px(&mut buf, MINIMAP_PX, i, 0, (0, 0, 0, 255));
        set_px(&mut buf, MINIMAP_PX, i, MINIMAP_PX as i32 - 1, (0, 0, 0, 255));
    }

    let img = Image::new(
        Extent3d {
            width: MINIMAP_PX,
            height: MINIMAP_PX,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        buf,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let mm_handle = images.add(img);
    commands.insert_resource(MinimapImage {
        handle: mm_handle.clone(),
    });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(78.0),
                    padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(10.0), Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::srgba(0.05, 0.07, 0.12, 0.86).into(),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|top| {
            top.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    TextBundle::from_section(
                        "ZEIT 60",
                        ts_body(&fonts, 28.0, Color::srgb(0.95, 0.95, 0.95)),
                    ),
                    TimerText,
                ));
                row.spawn((
                    TextBundle::from_section(
                        "Score: 0",
                        ts_body(&fonts, 24.0, Color::srgb(0.95, 0.95, 0.95)),
                    ),
                    ScoreText,
                ));
                row.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|coins_box| {
                    coins_box.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        image: UiImage::new(assets.coin_icon.clone()),
                        ..default()
                    });
                    coins_box.spawn((
                        TextBundle::from_section(
                            "8 Münzen",
                            ts_body(&fonts, 22.0, Color::srgb(1.0, 0.85, 0.25)),
                        ),
                        CoinsText,
                    ));
                });

                row.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    LivesRow,
                ));
            });

            top.spawn((
                TextBundle::from_section(
                    "Auftrag: -",
                    ts_body(&fonts, 17.0, Color::srgb(0.85, 0.95, 1.0)),
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                }),
                OrderText,
            ));
        });

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(96.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-220.0)),
                width: Val::Px(440.0),
                padding: UiRect::all(Val::Px(8.0)),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
            ..default()
        },
        FeedbackBanner,
        HudRoot,
    ))
    .with_children(|p| {
        p.spawn(TextBundle::from_section(
            "",
            ts_body(&fonts, 22.0, Color::srgb(1.0, 1.0, 1.0)),
        ));
    });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(28.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-260.0)),
                    width: Val::Px(520.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    display: Display::None,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::srgba(0.45, 0.05, 0.05, 0.92).into(),
                border_color: Color::srgb(1.0, 0.7, 0.3).into(),
                ..default()
            },
            LateBanner,
            HudRoot,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "ZU SPÄT! Kunde wartet nicht mehr!",
                ts_body(&fonts, 28.0, Color::srgb(1.0, 0.95, 0.95)),
            ));
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(18.0),
                    right: Val::Px(24.0),
                    width: Val::Px(MINIMAP_DISPLAY_PX + 24.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                background_color: Color::srgba(0.07, 0.09, 0.13, 0.9).into(),
                border_color: Color::srgb(0.85, 0.78, 0.25).into(),
                ..default()
            },
            HudRoot,
            MinimapContainer,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "NAVI - GERMERING",
                ts_body(&fonts, 15.0, Color::srgb(0.85, 0.78, 0.25)),
            ));
            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(MINIMAP_DISPLAY_PX),
                    height: Val::Px(MINIMAP_DISPLAY_PX),
                    margin: UiRect::top(Val::Px(2.0)),
                    position_type: PositionType::Relative,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                border_color: Color::srgb(0.25, 0.25, 0.35).into(),
                ..default()
            })
            .with_children(|q| {
                q.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(MINIMAP_DISPLAY_PX),
                        height: Val::Px(MINIMAP_DISPLAY_PX),
                        ..default()
                    },
                    image: UiImage::new(mm_handle),
                    ..default()
                });
                q.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(8.0),
                            height: Val::Px(8.0),
                            ..default()
                        },
                        background_color: Color::srgb(1.0, 1.0, 1.0).into(),
                        border_color: Color::srgb(0.0, 0.0, 0.0).into(),
                        ..default()
                    },
                    MinimapPlayerDot,
                ));
                q.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(10.0),
                            height: Val::Px(10.0),
                            display: Display::None,
                            ..default()
                        },
                        background_color: Color::srgb(0.25, 0.55, 1.0).into(),
                        ..default()
                    },
                    MinimapPickupDot,
                ));
                q.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(12.0),
                            height: Val::Px(12.0),
                            display: Display::None,
                            ..default()
                        },
                        background_color: Color::srgb(1.0, 0.25, 0.25).into(),
                        ..default()
                    },
                    MinimapDropoffDot,
                ));
            });

            p.spawn((
                TextBundle::from_section(
                    "-> GERADEAUS",
                    ts_body(&fonts, 18.0, Color::srgb(0.4, 0.95, 0.4)),
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                }),
                NaviDirectionText,
            ));

            p.spawn((
                TextBundle::from_section(
                    "0m",
                    ts_body(&fonts, 15.0, Color::srgb(0.95, 0.95, 0.95)),
                ),
                NaviDistanceText,
            ));

            p.spawn((
                TextBundle::from_section(
                    "BOOST AKTIV",
                    ts_body(&fonts, 15.0, Color::srgb(1.0, 0.85, 0.2)),
                )
                .with_style(Style {
                    display: Display::None,
                    ..default()
                }),
                NaviBoostText,
            ));
        });

    // Nitro meter, bottom-left.
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(22.0),
                    left: Val::Px(24.0),
                    width: Val::Px(220.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                background_color: Color::srgba(0.07, 0.09, 0.13, 0.9).into(),
                border_color: Color::srgb(0.85, 0.78, 0.25).into(),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|p| {
            p.spawn((
                TextBundle::from_section(
                    "NITRO",
                    ts_body(&fonts, 15.0, Color::srgb(0.85, 0.78, 0.25)),
                ),
                NitroLabel,
            ));
            // Track.
            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: Color::srgb(0.12, 0.13, 0.18).into(),
                border_color: Color::srgb(0.3, 0.32, 0.4).into(),
                ..default()
            })
            .with_children(|track| {
                // Fill.
                track.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        background_color: Color::srgb(0.3, 0.85, 1.0).into(),
                        ..default()
                    },
                    NitroBarFill,
                ));
            });
        });

    // Interaction prompt at pickup / dropoff (mirrors Jannick's hint).
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(158.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-150.0)),
                    width: Val::Px(300.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    display: Display::None,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: Color::srgba(0.05, 0.06, 0.1, 0.92).into(),
                border_color: Color::srgb(0.4, 0.9, 0.5).into(),
                ..default()
            },
            HudRoot,
            InteractHint,
        ))
        .with_children(|p| {
            p.spawn(TextBundle::from_section(
                "[E] Paket abholen",
                ts_body(&fonts, 20.0, Color::srgb(0.5, 1.0, 0.6)),
            ));
        });
}

fn despawn_hud(mut commands: Commands, q: Query<Entity, With<HudRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn update_nitro_bar(
    time: Res<Time>,
    player_q: Query<&Player>,
    mut fill_q: Query<&mut Style, With<NitroBarFill>>,
    mut bar_color_q: Query<&mut BackgroundColor, With<NitroBarFill>>,
    mut label_q: Query<&mut Text, With<NitroLabel>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    let active = player.nitro_timer > 0.0;
    let ready = player.nitro >= 1.0;
    let frac = if active { 1.0 } else { player.nitro.clamp(0.0, 1.0) };

    if let Ok(mut style) = fill_q.get_single_mut() {
        style.width = Val::Percent(frac * 100.0);
    }
    if let Ok(mut bg) = bar_color_q.get_single_mut() {
        let pulse = (time.elapsed_seconds() * 8.0).sin() * 0.5 + 0.5;
        *bg = if active {
            Color::srgb(1.0, 0.55 + pulse * 0.3, 0.2).into()
        } else if ready {
            Color::srgb(0.4 + pulse * 0.5, 0.95, 0.5).into()
        } else {
            Color::srgb(0.3, 0.75, 1.0).into()
        };
    }
    if let Ok(mut text) = label_q.get_single_mut() {
        let (msg, color) = if active {
            ("NITRO!  ", Color::srgb(1.0, 0.7, 0.25))
        } else if ready {
            ("NITRO BEREIT - [LEERTASTE]", Color::srgb(0.5, 1.0, 0.6))
        } else {
            ("NITRO lädt...", Color::srgb(0.85, 0.78, 0.25))
        };
        text.sections[0].value = msg.to_string();
        text.sections[0].style.color = color;
    }
}

fn update_timer(
    active: Option<Res<ActiveDelivery>>,
    mut q: Query<&mut Text, With<TimerText>>,
    time: Res<Time>,
) {
    let Ok(mut text) = q.get_single_mut() else {
        return;
    };
    let Some(active) = active else {
        return;
    };
    let secs = active.time_remaining.max(0.0);
    let mm = (secs / 60.0).floor() as i32;
    let ss = (secs % 60.0).floor() as i32;
    text.sections[0].value = format!("ZEIT {:02}:{:02}", mm, ss);
    let red_threshold = 10.0;
    text.sections[0].style.color = if secs < red_threshold {
        let blink = (time.elapsed_seconds() * 6.0).sin() * 0.5 + 0.5;
        Color::srgb(1.0, 0.25 + blink * 0.2, 0.25)
    } else if secs < 20.0 {
        Color::srgb(1.0, 0.85, 0.3)
    } else {
        Color::srgb(0.95, 0.95, 0.95)
    };
}

fn update_score_coins(
    stats: Res<DeliveryStats>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<CoinsText>)>,
    mut coins_q: Query<&mut Text, (With<CoinsText>, Without<ScoreText>)>,
) {
    if let Ok(mut text) = score_q.get_single_mut() {
        text.sections[0].value = format!("Score: {}", stats.score);
    }
    if let Ok(mut text) = coins_q.get_single_mut() {
        text.sections[0].value = format!("{} Münzen", stats.coins);
    }
}

fn update_lives(
    mut commands: Commands,
    assets: Res<GameAssets>,
    stats: Res<DeliveryStats>,
    lives_row_q: Query<Entity, With<LivesRow>>,
    icons_q: Query<Entity, With<LifeIcon>>,
) {
    let Ok(row) = lives_row_q.get_single() else {
        return;
    };
    let existing_count = icons_q.iter().count() as u32;
    if existing_count == stats.lives {
        return;
    }
    for e in &icons_q {
        commands.entity(e).despawn_recursive();
    }
    for _ in 0..stats.lives {
        let icon = commands
            .spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(32.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    image: UiImage::new(assets.sprinter_icon.clone()),
                    ..default()
                },
                LifeIcon,
            ))
            .id();
        commands.entity(row).push_children(&[icon]);
    }
}

fn update_order_text(
    active: Option<Res<ActiveDelivery>>,
    mut q: Query<&mut Text, With<OrderText>>,
) {
    let Ok(mut text) = q.get_single_mut() else {
        return;
    };
    if let Some(active) = active {
        let label = match active.phase {
            DeliveryPhase::GoToPickup => format!(
                "Auftrag: Hin zu {} -> {}",
                active.pickup.name, active.dropoff.name
            ),
            DeliveryPhase::GoToDropoff => format!(
                "Auftrag: {} -> {}  (PAKET GELADEN)",
                active.pickup.name, active.dropoff.name
            ),
        };
        text.sections[0].value = label;
    } else {
        text.sections[0].value = "Auftrag: -".to_string();
    }
}

fn update_feedback(
    feedback: Res<DeliveryFeedback>,
    mut banner_q: Query<(&mut Style, &Children), With<FeedbackBanner>>,
    mut text_q: Query<&mut Text>,
) {
    let Ok((mut style, children)) = banner_q.get_single_mut() else {
        return;
    };
    if feedback.timer <= 0.0 {
        style.display = Display::None;
        return;
    }
    style.display = Display::Flex;
    for child in children.iter() {
        if let Ok(mut text) = text_q.get_mut(*child) {
            text.sections[0].value = feedback.text.clone();
            text.sections[0].style.color = feedback.color;
        }
    }
}

fn update_late_banner(
    time: Res<Time>,
    late: Res<LateNotification>,
    mut banner_q: Query<(&mut Style, &mut BackgroundColor), With<LateBanner>>,
) {
    let Ok((mut style, mut bg)) = banner_q.get_single_mut() else {
        return;
    };
    if late.timer <= 0.0 {
        style.display = Display::None;
        return;
    }
    style.display = Display::Flex;
    let blink = ((time.elapsed_seconds() * 8.0).sin() * 0.5 + 0.5) * 0.5 + 0.5;
    *bg = Color::srgba(0.55 + blink * 0.3, 0.05, 0.05, 0.92).into();
}

/// Shows a clear `[E]` call-to-action when the van is in range of the current
/// pickup or dropoff — the same affordance Jannick's shop already has.
fn update_interact_hint(
    active: Option<Res<ActiveDelivery>>,
    player_q: Query<(&Transform, &Player)>,
    mut hint_q: Query<(&mut Style, &Children), With<InteractHint>>,
    mut text_q: Query<&mut Text>,
) {
    let Ok((mut style, children)) = hint_q.get_single_mut() else {
        return;
    };
    let (Some(active), Ok((player_tr, player))) = (active, player_q.get_single()) else {
        style.display = Display::None;
        return;
    };
    let pos = player_tr.translation.truncate();
    let on_foot = player.is_on_foot();
    let (target, msg) = match active.phase {
        DeliveryPhase::GoToPickup => (
            GameMap::tile_to_world(active.pickup.interact_tile),
            if on_foot {
                "[F] Einsteigen zum Abholen"
            } else {
                "[E] Paket abholen"
            },
        ),
        DeliveryPhase::GoToDropoff => (
            GameMap::tile_to_world(active.dropoff.tile),
            if on_foot {
                "[E] Hier abliefern"
            } else {
                "[F] Aussteigen zum Abliefern"
            },
        ),
    };
    if pos.distance(target) <= INTERACT_RADIUS {
        style.display = Display::Flex;
        for child in children.iter() {
            if let Ok(mut text) = text_q.get_mut(*child) {
                text.sections[0].value = msg.to_string();
            }
        }
    } else {
        style.display = Display::None;
    }
}

fn update_minimap_dots(
    time: Res<Time>,
    active: Option<Res<ActiveDelivery>>,
    player_q: Query<&Transform, With<Player>>,
    mut player_dot_q: Query<
        &mut Style,
        (
            With<MinimapPlayerDot>,
            Without<MinimapPickupDot>,
            Without<MinimapDropoffDot>,
        ),
    >,
    mut pickup_dot_q: Query<
        (&mut Style, &mut BackgroundColor),
        (
            With<MinimapPickupDot>,
            Without<MinimapPlayerDot>,
            Without<MinimapDropoffDot>,
        ),
    >,
    mut dropoff_dot_q: Query<
        (&mut Style, &mut BackgroundColor),
        (
            With<MinimapDropoffDot>,
            Without<MinimapPlayerDot>,
            Without<MinimapPickupDot>,
        ),
    >,
) {
    let Ok(player_tr) = player_q.get_single() else {
        return;
    };
    let world = player_tr.translation.truncate();
    let player_tile = GameMap::world_to_tile(world);
    let mm_size = MINIMAP_DISPLAY_PX;

    let to_mm = |tx: i32, ty: i32| -> (f32, f32) {
        let nx = tx.clamp(0, MAP_WIDTH - 1) as f32 / MAP_WIDTH as f32;
        let ny = (MAP_HEIGHT - 1 - ty.clamp(0, MAP_HEIGHT - 1)) as f32 / MAP_HEIGHT as f32;
        (nx * mm_size, ny * mm_size)
    };

    if let Ok(mut style) = player_dot_q.get_single_mut() {
        let (x, y) = to_mm(player_tile.x, player_tile.y);
        style.left = Val::Px(x - 4.0);
        style.top = Val::Px(y - 4.0);
    }

    if let Ok((mut style, mut bg)) = pickup_dot_q.get_single_mut() {
        if let Some(act) = active.as_ref() {
            if act.phase == DeliveryPhase::GoToPickup {
                let (x, y) = to_mm(act.pickup.tile.x, act.pickup.tile.y);
                style.left = Val::Px(x - 5.0);
                style.top = Val::Px(y - 5.0);
                style.display = Display::Flex;
                let pulse = (time.elapsed_seconds() * 4.0).sin() * 0.4 + 0.6;
                *bg = Color::srgb(0.25, 0.4 + pulse * 0.6, 1.0).into();
            } else {
                style.display = Display::None;
            }
        } else {
            style.display = Display::None;
        }
    }

    if let Ok((mut style, mut bg)) = dropoff_dot_q.get_single_mut() {
        if let Some(act) = active.as_ref() {
            if act.phase == DeliveryPhase::GoToDropoff {
                let (x, y) = to_mm(act.dropoff.tile.x, act.dropoff.tile.y);
                style.left = Val::Px(x - 6.0);
                style.top = Val::Px(y - 6.0);
                style.display = Display::Flex;
                let pulse = (time.elapsed_seconds() * 5.0).sin() * 0.5 + 0.5;
                *bg = Color::srgb(1.0, 0.2 + pulse * 0.2, 0.2 + pulse * 0.2).into();
            } else {
                style.display = Display::None;
            }
        } else {
            style.display = Display::None;
        }
    }
}
