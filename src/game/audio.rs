//! Fully procedural audio — no external files (per project spec).
//!
//! A custom [`Decodable`] PCM source lets us synthesise everything at startup:
//! a looping engine hum whose pitch/volume track the van's speed, plus a small
//! bank of one-shot SFX (delivery chime, coin, crash thud, nitro whoosh, late
//! buzzer, UI blip). Volumes are deliberately gentle.

use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioSink, AudioSinkPlayback, Source};
use bevy::prelude::*;
use bevy::reflect::TypePath;

use crate::game::delivery::{DeliveryCompletedEvent, DeliveryLateEvent, PackagePickedUpEvent};
use crate::game::gamestate::GameState;
use crate::game::player::{CollisionBumpEvent, NitroActivatedEvent, Player, PLAYER_BASE_MAX_SPEED};

const SAMPLE_RATE: u32 = 44_100;
const MASTER: f32 = 0.6;

/// A pre-rendered mono PCM buffer playable through Bevy's audio graph.
#[derive(Asset, TypePath, Clone)]
pub struct Pcm {
    samples: Arc<Vec<f32>>,
}

pub struct PcmDecoder {
    samples: Arc<Vec<f32>>,
    index: usize,
}

impl Iterator for PcmDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let s = self.samples.get(self.index).copied();
        self.index += 1;
        s
    }
}

impl Source for PcmDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.index))
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / SAMPLE_RATE as f32,
        ))
    }
}

impl Decodable for Pcm {
    type DecoderItem = f32;
    type Decoder = PcmDecoder;
    fn decoder(&self) -> Self::Decoder {
        PcmDecoder {
            samples: self.samples.clone(),
            index: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct AudioBank {
    pub engine: Handle<Pcm>,
    pub chime: Handle<Pcm>,
    pub coin: Handle<Pcm>,
    pub thud: Handle<Pcm>,
    pub nitro: Handle<Pcm>,
    pub buzzer: Handle<Pcm>,
    pub blip: Handle<Pcm>,
}

#[derive(Component)]
pub struct EngineSound;

/// Fire to play the UI confirmation blip from any state (menus, pause, …).
#[derive(Event)]
pub struct UiBlipEvent;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Pcm>()
            .init_resource::<AudioBank>()
            .add_event::<UiBlipEvent>()
            .add_systems(Update, sfx_ui_blip)
            .add_systems(Startup, build_audio_bank)
            .add_systems(OnEnter(GameState::Playing), start_engine)
            .add_systems(OnExit(GameState::Playing), stop_engine)
            .add_systems(
                Update,
                (
                    drive_engine,
                    sfx_collision,
                    sfx_nitro,
                    sfx_pickup,
                    sfx_delivery,
                    sfx_late,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::Shopping), |q: Query<&AudioSink, With<EngineSound>>| {
                if let Ok(sink) = q.get_single() {
                    sink.set_volume(0.0);
                }
            });
    }
}

// ---------------------------------------------------------------------------
// Synthesis
// ---------------------------------------------------------------------------

fn build_audio_bank(mut pcm: ResMut<Assets<Pcm>>, mut bank: ResMut<AudioBank>) {
    bank.engine = pcm.add(Pcm {
        samples: Arc::new(make_engine()),
    });
    bank.chime = pcm.add(Pcm {
        samples: Arc::new(make_chime()),
    });
    bank.coin = pcm.add(Pcm {
        samples: Arc::new(make_coin()),
    });
    bank.thud = pcm.add(Pcm {
        samples: Arc::new(make_thud()),
    });
    bank.nitro = pcm.add(Pcm {
        samples: Arc::new(make_nitro()),
    });
    bank.buzzer = pcm.add(Pcm {
        samples: Arc::new(make_buzzer()),
    });
    bank.blip = pcm.add(Pcm {
        samples: Arc::new(make_blip()),
    });
}

fn n_samples(secs: f32) -> usize {
    (secs * SAMPLE_RATE as f32) as usize
}

/// Pseudo-random noise in [-1, 1] from an integer index (deterministic).
fn noise(i: usize) -> f32 {
    let mut x = (i as u32).wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    x ^= x >> 16;
    (x as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// One looping cycle of a gritty four-cylinder idle. Looped at runtime and
/// pitch-shifted by [`drive_engine`].
fn make_engine() -> Vec<f32> {
    let len = n_samples(0.5);
    let mut out = vec![0.0f32; len];
    let base = 70.0;
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let phase = t * base;
        // Sawtooth-ish fundamental + a couple of harmonics + a touch of grit.
        let saw = (phase % 1.0) * 2.0 - 1.0;
        let h2 = (phase * 2.0 * std::f32::consts::TAU).sin() * 0.3;
        let h3 = (phase * 3.0 * std::f32::consts::TAU).sin() * 0.15;
        let grit = noise(i) * 0.08;
        *s = (saw * 0.5 + h2 + h3 + grit) * 0.32;
    }
    // Crossfade the ends so the loop has no click.
    let fade = n_samples(0.02).min(len / 2);
    for j in 0..fade {
        let k = len - fade + j;
        let a = j as f32 / fade as f32;
        let blended = out[k] * (1.0 - a) + out[j] * a;
        out[k] = blended;
    }
    out
}

/// Pleasant rising two-note "ding-dong" for a completed delivery.
fn make_chime() -> Vec<f32> {
    let len = n_samples(0.55);
    let mut out = vec![0.0f32; len];
    let notes = [(0.0f32, 880.0f32), (0.16, 1174.0), (0.30, 1568.0)];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let mut v = 0.0;
        for &(start, freq) in &notes {
            if t >= start {
                let lt = t - start;
                let env = (-lt * 7.0).exp();
                v += (lt * freq * std::f32::consts::TAU).sin() * env;
            }
        }
        *s = v * 0.3;
    }
    out
}

fn make_coin() -> Vec<f32> {
    let len = n_samples(0.18);
    let mut out = vec![0.0f32; len];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = if t < 0.05 { 1320.0 } else { 1760.0 };
        let env = (-t * 16.0).exp();
        *s = (t * freq * std::f32::consts::TAU).sin() * env * 0.28;
    }
    out
}

fn make_thud() -> Vec<f32> {
    let len = n_samples(0.22);
    let mut out = vec![0.0f32; len];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-t * 22.0).exp();
        let body = (t * 90.0 * std::f32::consts::TAU).sin();
        *s = (body * 0.7 + noise(i) * 0.5) * env * 0.4;
    }
    out
}

/// Rising filtered-noise whoosh for the nitro burst.
fn make_nitro() -> Vec<f32> {
    let len = n_samples(0.5);
    let mut out = vec![0.0f32; len];
    let mut lp = 0.0f32;
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let prog = t / 0.5;
        // Low-pass the noise, opening up as the sweep rises.
        let cutoff = 0.05 + prog * 0.5;
        lp += (noise(i) - lp) * cutoff;
        let tone = (t * (300.0 + prog * 900.0) * std::f32::consts::TAU).sin() * 0.4;
        let env = (prog * std::f32::consts::PI).sin();
        *s = (lp * 0.7 + tone) * env * 0.3;
    }
    out
}

fn make_buzzer() -> Vec<f32> {
    let len = n_samples(0.6);
    let mut out = vec![0.0f32; len];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        // Harsh square with a tremolo gate.
        let sq = if (t * 150.0).fract() < 0.5 { 1.0 } else { -1.0 };
        let gate = if (t * 8.0).fract() < 0.6 { 1.0 } else { 0.0 };
        let env = (1.0 - t / 0.6).max(0.0);
        *s = sq * gate * env * 0.26;
    }
    out
}

fn make_blip() -> Vec<f32> {
    let len = n_samples(0.08);
    let mut out = vec![0.0f32; len];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-t * 30.0).exp();
        *s = (t * 660.0 * std::f32::consts::TAU).sin() * env * 0.25;
    }
    out
}

// ---------------------------------------------------------------------------
// Playback
// ---------------------------------------------------------------------------

fn play_once(commands: &mut Commands, handle: Handle<Pcm>, volume: f32) {
    commands.spawn((
        bevy::audio::AudioSourceBundle {
            source: handle,
            settings: PlaybackSettings::DESPAWN
                .with_volume(bevy::audio::Volume::new(volume * MASTER)),
        },
    ));
}

fn start_engine(mut commands: Commands, bank: Res<AudioBank>) {
    commands.spawn((
        bevy::audio::AudioSourceBundle {
            source: bank.engine.clone(),
            settings: PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::new(0.0)),
        },
        EngineSound,
    ));
}

fn stop_engine(mut commands: Commands, q: Query<Entity, With<EngineSound>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn drive_engine(
    player_q: Query<&Player>,
    sink_q: Query<&AudioSink, With<EngineSound>>,
) {
    let Ok(sink) = sink_q.get_single() else {
        return;
    };
    let speed_frac = player_q
        .get_single()
        .map(|p| (p.velocity.length() / PLAYER_BASE_MAX_SPEED).clamp(0.0, 1.6))
        .unwrap_or(0.0);
    // Idle hum that revs with speed.
    sink.set_speed(0.8 + speed_frac * 1.1);
    sink.set_volume((0.12 + speed_frac * 0.5) * MASTER);
}

fn sfx_collision(
    mut commands: Commands,
    bank: Res<AudioBank>,
    mut events: EventReader<CollisionBumpEvent>,
) {
    if events.read().next().is_some() {
        play_once(&mut commands, bank.thud.clone(), 0.7);
    }
    events.clear();
}

fn sfx_nitro(
    mut commands: Commands,
    bank: Res<AudioBank>,
    mut events: EventReader<NitroActivatedEvent>,
) {
    if events.read().next().is_some() {
        play_once(&mut commands, bank.nitro.clone(), 0.8);
    }
    events.clear();
}

fn sfx_pickup(
    mut commands: Commands,
    bank: Res<AudioBank>,
    mut events: EventReader<PackagePickedUpEvent>,
) {
    if events.read().next().is_some() {
        play_once(&mut commands, bank.blip.clone(), 0.8);
    }
    events.clear();
}

fn sfx_delivery(
    mut commands: Commands,
    bank: Res<AudioBank>,
    mut events: EventReader<DeliveryCompletedEvent>,
) {
    for ev in events.read() {
        play_once(&mut commands, bank.chime.clone(), 0.8);
        play_once(&mut commands, bank.coin.clone(), 0.6);
        let _ = ev.points;
    }
}

fn sfx_late(
    mut commands: Commands,
    bank: Res<AudioBank>,
    mut events: EventReader<DeliveryLateEvent>,
) {
    if events.read().next().is_some() {
        play_once(&mut commands, bank.buzzer.clone(), 0.7);
    }
    events.clear();
}

fn sfx_ui_blip(
    mut commands: Commands,
    bank: Res<AudioBank>,
    mut events: EventReader<UiBlipEvent>,
) {
    if events.read().next().is_some() {
        play_once(&mut commands, bank.blip.clone(), 0.7);
    }
    events.clear();
}
