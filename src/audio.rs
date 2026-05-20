//! Prozedurale Sound-Synthese.
//!
//! Alle Sounds werden zur Laufzeit als WAV-Bytes generiert und in
//! macroquad-`Sound`-Objekte geladen. Es gibt keine externen Audio-Dateien.

use macroquad::audio::{load_sound_from_bytes, play_sound, play_sound_once, PlaySoundParams, Sound};

const SAMPLE_RATE: u32 = 22050;

// ------------------------------------------------------------------------
//  WAV-Helpers
// ------------------------------------------------------------------------

fn make_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let n = samples.len();
    let data_size = (n * 2) as u32;
    let chunk_size = 36 + data_size;
    let mut w = Vec::with_capacity(44 + n * 2);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&chunk_size.to_le_bytes());
    w.extend_from_slice(b"WAVE");
    w.extend_from_slice(b"fmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&sample_rate.to_le_bytes());
    w.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        w.extend_from_slice(&s.to_le_bytes());
    }
    w
}

/// Sinus-Burst mit Attack/Release-Hüllkurve.
fn tone_sine(freq: f32, secs: f32, amp: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE as f32 * secs) as usize;
    let mut out = Vec::with_capacity(n);
    let attack = (n as f32 * 0.05) as usize;
    let release = (n as f32 * 0.25) as usize;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let mut env = 1.0;
        if i < attack {
            env = i as f32 / attack.max(1) as f32;
        } else if i > n - release {
            env = (n - i) as f32 / release.max(1) as f32;
        }
        let v = (t * freq * std::f32::consts::TAU).sin() * amp * env;
        out.push((v * 32767.0).clamp(-32767.0, 32767.0) as i16);
    }
    out
}

/// Rechteckwelle (chiptune).
fn tone_square(freq: f32, secs: f32, amp: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE as f32 * secs) as usize;
    let mut out = Vec::with_capacity(n);
    let attack = (n as f32 * 0.02) as usize;
    let release = (n as f32 * 0.15) as usize;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let mut env = 1.0;
        if i < attack {
            env = i as f32 / attack.max(1) as f32;
        } else if i > n - release {
            env = (n - i) as f32 / release.max(1) as f32;
        }
        let phase = (t * freq).fract();
        let v = if phase < 0.5 { amp } else { -amp } * env;
        out.push((v * 32767.0).clamp(-32767.0, 32767.0) as i16);
    }
    out
}

/// Sägezahn (für Bass).
fn tone_saw(freq: f32, secs: f32, amp: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE as f32 * secs) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let phase = (t * freq).fract();
        let v = (phase * 2.0 - 1.0) * amp;
        out.push((v * 32767.0).clamp(-32767.0, 32767.0) as i16);
    }
    out
}

/// Weißes Rauschen für Schlagwerk.
fn noise(secs: f32, amp: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE as f32 * secs) as usize;
    let mut out = Vec::with_capacity(n);
    let mut seed: u32 = 0xDEADBEEF;
    for _ in 0..n {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let r = ((seed >> 8) & 0xFFFF) as i32 - 32768;
        let v = (r as f32 / 32768.0) * amp;
        out.push((v * 32767.0).clamp(-32767.0, 32767.0) as i16);
    }
    out
}

fn concat(buffers: &[Vec<i16>]) -> Vec<i16> {
    let total: usize = buffers.iter().map(|b| b.len()).sum();
    let mut out = Vec::with_capacity(total);
    for b in buffers {
        out.extend_from_slice(b);
    }
    out
}

fn mix(a: &[i16], b: &[i16]) -> Vec<i16> {
    let n = a.len().max(b.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let va = *a.get(i).unwrap_or(&0) as i32;
        let vb = *b.get(i).unwrap_or(&0) as i32;
        let s = (va + vb).clamp(-32767, 32767);
        out.push(s as i16);
    }
    out
}

// ------------------------------------------------------------------------
//  Konkrete Sound-Effekte
// ------------------------------------------------------------------------

fn coin_wav() -> Vec<u8> {
    // C5 → E5 → G5 — fröhliches Arpeggio
    let s = concat(&[
        tone_sine(523.25, 0.08, 0.4),
        tone_sine(659.25, 0.08, 0.4),
        tone_sine(783.99, 0.10, 0.45),
    ]);
    make_wav(&s, SAMPLE_RATE)
}

fn damage_wav() -> Vec<u8> {
    // Pitch-Drop von A3 → A2
    let mut buf = Vec::new();
    let n = (SAMPLE_RATE as f32 * 0.2) as usize;
    let mut freq = 220.0_f32;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        freq -= 200.0 * t * 0.05;
        let phase = (t * freq).fract();
        let v = if phase < 0.5 { 0.5 } else { -0.5 };
        let env = 1.0 - (i as f32 / n as f32);
        buf.push((v * env * 32767.0) as i16);
    }
    make_wav(&buf, SAMPLE_RATE)
}

fn buy_wav() -> Vec<u8> {
    // C5, E5, G5 schnell hintereinander
    let s = concat(&[
        tone_square(523.25, 0.10, 0.35),
        tone_square(659.25, 0.10, 0.35),
        tone_square(783.99, 0.16, 0.4),
    ]);
    make_wav(&s, SAMPLE_RATE)
}

fn enemy_kill_wav() -> Vec<u8> {
    // Absteigende Pentatonik G5 → E5 → D5 → C5 → A4
    let s = concat(&[
        tone_square(783.99, 0.06, 0.4),
        tone_square(659.25, 0.06, 0.4),
        tone_square(587.33, 0.06, 0.4),
        tone_square(523.25, 0.06, 0.4),
        tone_square(440.00, 0.10, 0.45),
    ]);
    make_wav(&s, SAMPLE_RATE)
}

fn jingle_wav() -> Vec<u8> {
    // 8-Note majestätisches Ihle-Jingle: C5 E5 G5 C6 G5 C6 E5 C5
    let s = concat(&[
        tone_sine(523.25, 0.10, 0.4),
        tone_sine(659.25, 0.10, 0.4),
        tone_sine(783.99, 0.10, 0.4),
        tone_sine(1046.50, 0.14, 0.45),
        tone_sine(783.99, 0.10, 0.4),
        tone_sine(1046.50, 0.10, 0.4),
        tone_sine(659.25, 0.10, 0.4),
        tone_sine(523.25, 0.20, 0.45),
    ]);
    make_wav(&s, SAMPLE_RATE)
}

fn boss_hit_wav() -> Vec<u8> {
    let mut s = noise(0.12, 0.5);
    let sq = tone_square(196.00, 0.12, 0.4);
    s = mix(&s, &sq);
    make_wav(&s, SAMPLE_RATE)
}

fn menu_select_wav() -> Vec<u8> {
    let s = tone_square(880.0, 0.05, 0.3);
    make_wav(&s, SAMPLE_RATE)
}

// Tag-Musik-Loop (120 BPM, 4 Sekunden) — fröhlicher Chiptune
fn day_music() -> Vec<u8> {
    let notes_lead: [f32; 16] = [
        523.25, 587.33, 659.25, 783.99,
        659.25, 587.33, 523.25, 440.00,
        523.25, 659.25, 783.99, 880.00,
        783.99, 659.25, 523.25, 587.33,
    ];
    let beat = 0.25; // 120 BPM
    let mut lead = Vec::new();
    for &f in notes_lead.iter() {
        lead.extend(tone_square(f, beat, 0.18));
    }
    let bass_notes: [f32; 8] = [130.81, 130.81, 174.61, 174.61, 196.00, 196.00, 130.81, 130.81];
    let mut bass = Vec::new();
    for &f in bass_notes.iter() {
        bass.extend(tone_saw(f, beat * 2.0, 0.18));
    }
    let s = mix(&lead, &bass);
    make_wav(&s, SAMPLE_RATE)
}

// Nacht-Musik (80 BPM, langsamer, Moll)
fn night_music() -> Vec<u8> {
    let notes: [f32; 8] = [220.0, 261.63, 329.63, 220.0, 196.00, 261.63, 329.63, 261.63];
    let beat = 0.375;
    let mut lead = Vec::new();
    for &f in notes.iter() {
        lead.extend(tone_sine(f, beat, 0.22));
    }
    let bass_notes: [f32; 4] = [110.0, 110.0, 130.81, 98.0];
    let mut bass = Vec::new();
    for &f in bass_notes.iter() {
        bass.extend(tone_saw(f, beat * 2.0, 0.15));
    }
    let s = mix(&lead, &bass);
    make_wav(&s, SAMPLE_RATE)
}

// Boss-Musik (schnell, dramatisch)
fn boss_music() -> Vec<u8> {
    let notes: [f32; 16] = [
        220.0, 233.08, 220.0, 174.61,
        220.0, 233.08, 261.63, 220.0,
        196.00, 220.0, 196.00, 174.61,
        220.0, 261.63, 293.66, 220.0,
    ];
    let beat = 0.18;
    let mut lead = Vec::new();
    for &f in notes.iter() {
        lead.extend(tone_square(f, beat, 0.22));
    }
    // Bass-Puls
    let mut bass = Vec::new();
    for _ in 0..16 {
        bass.extend(tone_saw(55.0, beat * 0.5, 0.25));
        bass.extend(tone_saw(55.0, beat * 0.5, 0.0));
    }
    let s = mix(&lead, &bass);
    make_wav(&s, SAMPLE_RATE)
}

// Cordobar Easter-Egg: "Wellenreiten"-Vibes (Surfrock-Chiptune)
fn cordobar_music() -> Vec<u8> {
    let notes: [f32; 16] = [
        329.63, 392.00, 440.00, 523.25,
        440.00, 392.00, 329.63, 293.66,
        329.63, 440.00, 523.25, 587.33,
        523.25, 440.00, 392.00, 329.63,
    ];
    let beat = 0.18;
    let mut lead = Vec::new();
    for &f in notes.iter() {
        lead.extend(tone_square(f, beat, 0.2));
    }
    make_wav(&lead, SAMPLE_RATE)
}

// ------------------------------------------------------------------------
//  Audio-Container
// ------------------------------------------------------------------------

pub struct Audio {
    pub coin: Sound,
    pub damage: Sound,
    pub buy: Sound,
    pub enemy_kill: Sound,
    pub jingle: Sound,
    pub menu_select: Sound,
    pub boss_hit: Sound,
    pub day_music: Sound,
    pub night_music: Sound,
    pub boss_music: Sound,
    pub cordobar_music: Sound,
    pub current_music: Option<&'static str>,
}

impl Audio {
    pub async fn load() -> Self {
        let coin = load_sound_from_bytes(&coin_wav()).await.unwrap();
        let damage = load_sound_from_bytes(&damage_wav()).await.unwrap();
        let buy = load_sound_from_bytes(&buy_wav()).await.unwrap();
        let enemy_kill = load_sound_from_bytes(&enemy_kill_wav()).await.unwrap();
        let jingle = load_sound_from_bytes(&jingle_wav()).await.unwrap();
        let menu_select = load_sound_from_bytes(&menu_select_wav()).await.unwrap();
        let boss_hit = load_sound_from_bytes(&boss_hit_wav()).await.unwrap();
        let day_music = load_sound_from_bytes(&day_music()).await.unwrap();
        let night_music = load_sound_from_bytes(&night_music()).await.unwrap();
        let boss_music = load_sound_from_bytes(&boss_music()).await.unwrap();
        let cordobar_music = load_sound_from_bytes(&cordobar_music()).await.unwrap();
        Self {
            coin,
            damage,
            buy,
            enemy_kill,
            jingle,
            menu_select,
            boss_hit,
            day_music,
            night_music,
            boss_music,
            cordobar_music,
            current_music: None,
        }
    }

    pub fn play_sfx(&self, s: &Sound) {
        play_sound_once(s);
    }

    pub fn play_music(&mut self, kind: &'static str, volume: f32) {
        if self.current_music == Some(kind) {
            return;
        }
        macroquad::audio::stop_sound(&self.day_music);
        macroquad::audio::stop_sound(&self.night_music);
        macroquad::audio::stop_sound(&self.boss_music);
        macroquad::audio::stop_sound(&self.cordobar_music);
        let s = match kind {
            "day" => &self.day_music,
            "night" => &self.night_music,
            "boss" => &self.boss_music,
            "cordobar" => &self.cordobar_music,
            _ => return,
        };
        play_sound(s, PlaySoundParams { looped: true, volume });
        self.current_music = Some(kind);
    }
}
