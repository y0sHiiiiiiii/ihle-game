//! Highscore-Persistenz (JSON) + Game-Over-Screen "FEIERABEND!".

use bevy::app::AppExit;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::game::assets::{ts_body, ts_display, UiFonts};
use crate::game::delivery::DeliveryStats;
use crate::game::gamestate::GameState;

const HIGHSCORE_FILE: &str = "highscore.json";
const MAX_ENTRIES: usize = 5;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HighscoreEntry {
    pub deliveries: u32,
    pub score: u64,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Resource)]
pub struct Highscores {
    pub entries: Vec<HighscoreEntry>,
}

impl Highscores {
    pub fn load() -> Self {
        let path = Path::new(HIGHSCORE_FILE);
        if !path.exists() {
            return Self::default();
        }
        match fs::read_to_string(path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = fs::write(HIGHSCORE_FILE, s);
        }
    }

    pub fn submit(&mut self, deliveries: u32, score: u64) -> bool {
        let date = chrono_today();
        let entry = HighscoreEntry {
            deliveries,
            score,
            date,
        };
        let was_top = self.entries.first().is_none_or(|top| score > top.score);
        self.entries.push(entry);
        self.entries
            .sort_by(|a, b| b.score.cmp(&a.score).then(b.deliveries.cmp(&a.deliveries)));
        if self.entries.len() > MAX_ENTRIES {
            self.entries.truncate(MAX_ENTRIES);
        }
        self.save();
        was_top
    }
}

fn chrono_today() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = now / 86_400;
    let (year, month, day) = day_to_ymd(days as i64);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn day_to_ymd(days_from_epoch: i64) -> (i32, u32, u32) {
    let mut days = days_from_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = (days - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let mut y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    days = days_from_epoch;
    let _ = days;
    (y as i32, m as u32, d as u32)
}

#[derive(Resource, Default)]
pub struct NewRecordFlag(pub bool);

#[derive(Component)]
pub struct GameOverRoot;

#[derive(Component)]
pub struct NewRecordText;

pub struct HighscorePlugin;

impl Plugin for HighscorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Highscores::load())
            .init_resource::<NewRecordFlag>()
            .add_systems(OnEnter(GameState::GameOver), enter_game_over)
            .add_systems(OnExit(GameState::GameOver), exit_game_over)
            .add_systems(
                Update,
                (handle_game_over_input, blink_record).run_if(in_state(GameState::GameOver)),
            );
    }
}

fn enter_game_over(
    mut commands: Commands,
    stats: Res<DeliveryStats>,
    fonts: Res<UiFonts>,
    mut highscores: ResMut<Highscores>,
    mut record: ResMut<NewRecordFlag>,
    existing: Query<Entity, With<GameOverRoot>>,
    cams: Query<Entity, With<Camera>>,
) {
    for e in &existing {
        commands.entity(e).despawn_recursive();
    }
    for e in &cams {
        commands.entity(e).despawn();
    }
    commands.spawn((Camera2dBundle::default(), GameOverRoot));

    let is_new_record = highscores.submit(stats.delivery_count, stats.score);
    record.0 = is_new_record;

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
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                background_color: Color::srgb(0.04, 0.05, 0.08).into(),
                ..default()
            },
            GameOverRoot,
        ))
        .with_children(|p| {
            // Title with offset shadow copy (pixel-sign look).
            p.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Relative,
                    ..default()
                },
                ..default()
            })
            .with_children(|t| {
                t.spawn(
                    TextBundle::from_section(
                        "FEIERABEND!",
                        ts_display(&fonts, 60.0, Color::srgb(0.0, 0.0, 0.0)),
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(5.0),
                        top: Val::Px(5.0),
                        ..default()
                    }),
                );
                t.spawn(TextBundle::from_section(
                    "FEIERABEND!",
                    ts_display(&fonts, 60.0, Color::srgb(0.98, 0.82, 0.15)),
                ));
            });

            if is_new_record {
                p.spawn((
                    TextBundle::from_section(
                        "NEUER REKORD!",
                        ts_display(&fonts, 26.0, Color::srgb(1.0, 0.9, 0.3)),
                    )
                    .with_style(Style {
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    }),
                    NewRecordText,
                ));
            }

            p.spawn(
                TextBundle::from_section(
                    format!("Lieferungen: {}", stats.delivery_count),
                    ts_body(&fonts, 26.0, Color::srgb(0.95, 0.95, 0.95)),
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(24.0)),
                    ..default()
                }),
            );
            p.spawn(
                TextBundle::from_section(
                    format!("Score: {}", stats.score),
                    ts_body(&fonts, 26.0, Color::srgb(0.95, 0.95, 0.95)),
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                }),
            );

            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(24.0)),
                    padding: UiRect::all(Val::Px(18.0)),
                    row_gap: Val::Px(6.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                background_color: Color::srgba(0.07, 0.09, 0.14, 0.95).into(),
                border_color: Color::srgb(0.3, 0.5, 0.7).into(),
                ..default()
            })
            .with_children(|card| {
                card.spawn(
                    TextBundle::from_section(
                        "TOP 5 HIGHSCORES",
                        ts_body(&fonts, 26.0, Color::srgb(0.4, 0.85, 1.0)),
                    )
                    .with_style(Style {
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    }),
                );

                if highscores.entries.is_empty() {
                    card.spawn(TextBundle::from_section(
                        "Noch keine Einträge - fahr los!",
                        ts_body(&fonts, 20.0, Color::srgb(0.7, 0.72, 0.8)),
                    ));
                }

                for (idx, entry) in highscores.entries.iter().enumerate().take(MAX_ENTRIES) {
                    let rank_color = match idx {
                        0 => Color::srgb(1.0, 0.85, 0.2),
                        1 => Color::srgb(0.85, 0.85, 0.9),
                        2 => Color::srgb(0.85, 0.55, 0.25),
                        _ => Color::srgb(0.8, 0.8, 0.85),
                    };
                    card.spawn(TextBundle::from_section(
                        format!(
                            "  {}.  {:>6} Pkt    {} Lieferungen    {}",
                            idx + 1,
                            entry.score,
                            entry.deliveries,
                            entry.date
                        ),
                        ts_body(&fonts, 20.0, rank_color),
                    ));
                }
            });

            p.spawn(
                TextBundle::from_section(
                    "[LEERTASTE] Nochmal spielen        [Q] Beenden",
                    ts_body(&fonts, 24.0, Color::srgb(0.55, 0.95, 0.55)),
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(28.0)),
                    ..default()
                }),
            );
        });
}

fn exit_game_over(mut commands: Commands, q: Query<Entity, With<GameOverRoot>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn handle_game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut blip: EventWriter<crate::game::audio::UiBlipEvent>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::KeyR)
    {
        blip.send(crate::game::audio::UiBlipEvent);
        next_state.set(GameState::Playing);
    }
    if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn blink_record(time: Res<Time>, mut q: Query<&mut Text, With<NewRecordText>>) {
    let blink = (time.elapsed_seconds() * 4.0).sin() * 0.5 + 0.5;
    for mut text in &mut q {
        let intensity = 0.6 + blink * 0.4;
        text.sections[0].style.color =
            Color::srgb(intensity, intensity * 0.92, intensity * 0.25);
    }
}
