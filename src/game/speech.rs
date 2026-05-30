//! Wiederverwendbare Welt-Sprechblasen — von Kunden (bayerisch), Crash-Opfern
//! (wütend) und Jannick (Kölsch) genutzt. Eine kleine dunkle Box mit Pixeltext,
//! die optional einer Ziel-Entität folgt und nach kurzer Zeit ausblendet.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::game::assets::{ts_body, UiFonts};
use crate::game::gamestate::GameState;

/// Eine schwebende Sprechblase. Folgt `target`, solange diese lebt; sonst bleibt
/// sie an `anchor` stehen (z. B. wenn der Kunde nach der Lieferung verschwindet).
#[derive(Component)]
pub struct SpeechBubble {
    pub target: Option<Entity>,
    pub anchor: Vec2,
    pub timer: f32,
    /// Höhe über dem Ziel in Weltpixeln.
    pub offset: f32,
}

#[derive(Component)]
struct BubbleText;

pub struct SpeechPlugin;

impl Plugin for SpeechPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_speech_bubbles.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_all_bubbles);
    }
}

/// Spawnt eine Sprechblase über `anchor`. Wird `target` mitgegeben, folgt die
/// Blase dieser Entität (z. B. einem laufenden NPC).
pub fn spawn_speech_bubble(
    commands: &mut Commands,
    fonts: &UiFonts,
    target: Option<Entity>,
    anchor: Vec2,
    text: &str,
    color: Color,
    ttl: f32,
) {
    // Hintergrundbreite grob aus der Textlänge schätzen (Pixelfont, ~6 px/Zeichen).
    let char_w = 6.0;
    let bg_w = (text.chars().count() as f32 * char_w + 12.0).clamp(28.0, 220.0);
    let bg_h = 18.0;

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(0.05, 0.05, 0.08, 0.82),
                    custom_size: Some(Vec2::new(bg_w, bg_h)),
                    ..default()
                },
                transform: Transform::from_xyz(anchor.x, anchor.y + 24.0, 40.0),
                ..default()
            },
            SpeechBubble {
                target,
                anchor,
                timer: ttl,
                offset: 24.0,
            },
        ))
        .with_children(|p| {
            p.spawn((
                Text2dBundle {
                    text: Text::from_section(text, ts_body(fonts, 11.0, color))
                        .with_justify(JustifyText::Center),
                    text_anchor: Anchor::Center,
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                },
                BubbleText,
            ));
        });
}

fn update_speech_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut bubble_q: Query<(Entity, &mut SpeechBubble, &mut Transform, &mut Sprite, &Children)>,
    transforms: Query<&Transform, Without<SpeechBubble>>,
    mut text_q: Query<&mut Text, With<BubbleText>>,
) {
    let dt = time.delta_seconds();
    for (entity, mut bubble, mut tr, mut sprite, children) in &mut bubble_q {
        bubble.timer -= dt;
        if bubble.timer <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Dem Ziel folgen, falls es noch existiert.
        if let Some(target) = bubble.target {
            if let Ok(target_tr) = transforms.get(target) {
                bubble.anchor = target_tr.translation.truncate();
            } else {
                bubble.target = None;
            }
        }
        tr.translation.x = bubble.anchor.x;
        tr.translation.y = bubble.anchor.y + bubble.offset;

        // Sanftes Ausblenden in der letzten halben Sekunde.
        let fade = (bubble.timer / 0.5).clamp(0.0, 1.0);
        let bg_a = 0.82 * fade;
        sprite.color = Color::srgba(0.05, 0.05, 0.08, bg_a);
        for child in children.iter() {
            if let Ok(mut text) = text_q.get_mut(*child) {
                for section in &mut text.sections {
                    let c = section.style.color.to_srgba();
                    section.style.color = Color::srgba(c.red, c.green, c.blue, fade);
                }
            }
        }
    }
}

fn despawn_all_bubbles(mut commands: Commands, q: Query<Entity, With<SpeechBubble>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
