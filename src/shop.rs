//! Ihle Shop-Logik & UI.
//!
//! Die 4 Filialen teilen sich dieselbe Item-Liste, nur Filiale 4 zeigt
//! das legendäre MEGA-IHLE-BREZE. Filiale 3 hat Frühaufsteher-Bonus
//! (-20%) zwischen 5 und 7 Uhr morgens.

use crate::consts::*;
use crate::player::Player;
use crate::world::{clock_string, time_of_day, TimeOfDay};

#[derive(Clone, Debug)]
pub struct ShopState {
    pub filiale_idx: usize,
    pub cursor: usize,
    pub message: String,
    pub message_t: f32,
}

impl ShopState {
    pub fn new(filiale_idx: usize) -> Self {
        Self {
            filiale_idx,
            cursor: 0,
            message: String::new(),
            message_t: 0.0,
        }
    }

    /// Welche Items zeigt diese Filiale?
    pub fn visible_items(&self) -> Vec<usize> {
        let f = &FILIALEN[self.filiale_idx];
        SHOP_ITEMS
            .iter()
            .enumerate()
            .filter(|(_, it)| !it.legendary || f.nummer == 4)
            .map(|(i, _)| i)
            .collect()
    }

    /// Ermittelt den effektiven Preis (Genusspass + Filiale-Discount + Tageszeit).
    pub fn effective_price(
        &self,
        item_idx: usize,
        player: &Player,
        game_secs: f32,
        purchases: u32,
    ) -> u32 {
        let it = &SHOP_ITEMS[item_idx];
        let f = &FILIALEN[self.filiale_idx];
        let mut p = it.preis as f32;

        // Genusspass: dauerhaft -15% nach 10 Käufen
        if purchases >= 10 {
            p *= 0.85;
        }

        // Filiale 3 Frühaufsteher-Bonus (Morgen, vor 7 Uhr)
        if f.nummer == 3 && time_of_day(game_secs) == TimeOfDay::Morgen {
            p *= 1.0 - f.discount_morgen;
        }

        // Filiale 4 generelle 10% Reduktion (Gewerbegebiet — billiger)
        if f.nummer == 4 {
            p *= 0.9;
        }

        let _ = player; // unused warning
        p.round() as u32
    }

    /// Versucht zu kaufen. Liefert true bei Erfolg.
    pub fn try_buy(
        &mut self,
        player: &mut Player,
        game_secs: f32,
        purchases: &mut u32,
    ) -> bool {
        let items = self.visible_items();
        if self.cursor >= items.len() {
            return false;
        }
        let idx = items[self.cursor];
        let price = self.effective_price(idx, player, game_secs, *purchases);
        if player.coins < price {
            self.message = format!("Zu wenig Münzen! ({} Münzen)", price);
            self.message_t = 2.0;
            return false;
        }
        player.coins -= price;
        *purchases += 1;
        self.apply_effect(idx, player);
        let it = &SHOP_ITEMS[idx];
        self.message = format!("Servus! {} - {}", it.name, it.effekt);
        self.message_t = 2.5;
        true
    }

    fn apply_effect(&self, idx: usize, player: &mut Player) {
        match idx {
            0 => player.heal(1),
            1 => {
                player.heal(1);
                player.powerups.speed = player.powerups.speed.max(8.0);
            }
            2 => player.heal(2),
            3 => player.heal(3),
            4 => {
                player.max_hearts = (player.max_hearts + 1).min(PLAYER_MAX_HEARTS_CAP);
                player.heal(1);
            }
            5 => {
                player.max_hearts = (player.max_hearts + 1).min(PLAYER_MAX_HEARTS_CAP);
                player.powerups.double_coin = player.powerups.double_coin.max(30.0);
                player.heal(1);
            }
            6 => {
                player.heal_full();
                player.powerups.invuln = player.powerups.invuln.max(15.0);
            }
            7 => {
                player.heal(5);
                player.powerups.fly = player.powerups.fly.max(25.0);
                player.powerups.double_coin = player.powerups.double_coin.max(20.0);
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.message_t = (self.message_t - dt).max(0.0);
        if self.message_t == 0.0 {
            self.message.clear();
        }
    }
}

/// Liefert Filiale-Index, in dessen Türzone der Spieler steht.
pub fn filiale_at(player_x: f32, player_y: f32) -> Option<usize> {
    for (i, f) in FILIALEN.iter().enumerate() {
        let door_x = f.tile_x as f32 * TILE_SIZE;
        let door_y = (f.tile_y + 2) as f32 * TILE_SIZE;
        let dx = player_x - door_x;
        let dy = player_y - door_y;
        if dx * dx + dy * dy < 24.0 * 24.0 {
            return Some(i);
        }
    }
    None
}

/// Ist die gegebene Filiale zur aktuellen Spielzeit geöffnet?
pub fn filiale_is_open(filiale_idx: usize, game_secs: f32) -> bool {
    let hour = ((game_secs / DAY_LENGTH_SECONDS) * 24.0 + 5.0) as i32;
    let hour = hour.rem_euclid(24) as u8;
    let (open, close) = FILIALE_OPEN_HOURS[filiale_idx];
    hour >= open && hour < close
}

/// Liefert die Zeile, die im Shop oben angezeigt wird (Filiale + Uhrzeit).
pub fn header_line(shop: &ShopState, game_secs: f32) -> String {
    let f = &FILIALEN[shop.filiale_idx];
    format!("{} - {} - {}", f.name, f.adresse, clock_string(game_secs))
}
