//! Spielstand-Persistenz.
//!
//! Schreibt eine einfache textbasierte Datei `save.dat` ins aktuelle Verzeichnis.
//! Format (key=value, eine Zeile pro Feld) — kein Bincode, kein serde, keine
//! externen Crates nötig.

use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SaveData {
    pub coins: u32,
    pub hearts: i32,
    pub max_hearts: i32,
    pub player_x: f32,
    pub player_y: f32,
    pub purchases: u32,
    pub checkpoint_filiale: usize,
    pub roman_artifact: bool,
    pub cordobar_unlocked: bool,
    pub boss_defeated: bool,
    pub game_seconds: f32,
    /// Bitmask: 0=Polariom, 1=Forst, 2=Cordobar, 3=Parsberg.
    pub crystals: u8,
    pub lake_swim_done: bool,
    /// 0=Start, 1=Klaus erzählt, 2=Artefakt, 3=Kristalle gesammelt, 4=Boss besiegt.
    pub quest_stage: u8,
    pub klaus_tour_done: bool,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            coins: 0,
            hearts: 3,
            max_hearts: 10,
            player_x: 100.0 * 16.0,
            player_y: 72.0 * 16.0,
            purchases: 0,
            checkpoint_filiale: 0,
            roman_artifact: false,
            cordobar_unlocked: false,
            boss_defeated: false,
            game_seconds: 0.0,
            crystals: 0,
            lake_swim_done: false,
            quest_stage: 0,
            klaus_tour_done: false,
        }
    }
}

const SAVE_PATH: &str = "save.dat";

pub fn save(data: &SaveData) {
    let text = format!(
        "coins={}\nhearts={}\nmax_hearts={}\npx={}\npy={}\npurchases={}\ncheckpoint={}\nroman={}\ncordobar={}\nboss_defeated={}\ngame_seconds={}\ncrystals={}\nlake_swim_done={}\nquest_stage={}\nklaus_tour_done={}\n",
        data.coins,
        data.hearts,
        data.max_hearts,
        data.player_x,
        data.player_y,
        data.purchases,
        data.checkpoint_filiale,
        data.roman_artifact as u8,
        data.cordobar_unlocked as u8,
        data.boss_defeated as u8,
        data.game_seconds,
        data.crystals,
        data.lake_swim_done as u8,
        data.quest_stage,
        data.klaus_tour_done as u8,
    );
    let _ = fs::write(SAVE_PATH, text);
}

pub fn load() -> Option<SaveData> {
    if !Path::new(SAVE_PATH).exists() {
        return None;
    }
    let text = fs::read_to_string(SAVE_PATH).ok()?;
    let mut d = SaveData::default();
    for line in text.lines() {
        let mut iter = line.splitn(2, '=');
        let key = iter.next()?;
        let val = iter.next()?;
        match key {
            "coins" => d.coins = val.parse().unwrap_or(0),
            "hearts" => d.hearts = val.parse().unwrap_or(3),
            "max_hearts" => d.max_hearts = val.parse().unwrap_or(10),
            "px" => d.player_x = val.parse().unwrap_or(0.0),
            "py" => d.player_y = val.parse().unwrap_or(0.0),
            "purchases" => d.purchases = val.parse().unwrap_or(0),
            "checkpoint" => d.checkpoint_filiale = val.parse().unwrap_or(0),
            "roman" => d.roman_artifact = val.parse::<u8>().unwrap_or(0) != 0,
            "cordobar" => d.cordobar_unlocked = val.parse::<u8>().unwrap_or(0) != 0,
            "boss_defeated" => d.boss_defeated = val.parse::<u8>().unwrap_or(0) != 0,
            "game_seconds" => d.game_seconds = val.parse().unwrap_or(0.0),
            "crystals" => d.crystals = val.parse().unwrap_or(0),
            "lake_swim_done" => d.lake_swim_done = val.parse::<u8>().unwrap_or(0) != 0,
            "quest_stage" => d.quest_stage = val.parse().unwrap_or(0),
            "klaus_tour_done" => d.klaus_tour_done = val.parse::<u8>().unwrap_or(0) != 0,
            _ => {}
        }
    }
    Some(d)
}

pub fn exists() -> bool {
    Path::new(SAVE_PATH).exists()
}
