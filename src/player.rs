//! Player-Modul: Max Huber, Bäckerlehrling.
//! Bewegung, Physik-Varianten (normal/Schwimmen/Eis), Powerups, Schaden.

use crate::collision::{move_aabb, tile_at, Aabb};
use crate::consts::*;
use crate::world::World;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Down,
    Up,
    Left,
    Right,
}

#[derive(Clone, Default, Debug)]
pub struct Powerups {
    pub speed: f32,         // Restzeit Speedboost
    pub invuln: f32,        // Restzeit Unverwundbarkeit
    pub fly: f32,           // Restzeit Fliegen
    pub double_coin: f32,   // Restzeit Doppelmünzen
    pub shield: bool,       // Brezel-Schild (absorbiert 1 Treffer)
    pub attack_bonus: i32,  // Vom Museum-Artefakt
}

impl Powerups {
    pub fn tick(&mut self, dt: f32) {
        self.speed = (self.speed - dt).max(0.0);
        self.invuln = (self.invuln - dt).max(0.0);
        self.fly = (self.fly - dt).max(0.0);
        self.double_coin = (self.double_coin - dt).max(0.0);
    }
}

pub struct Player {
    pub aabb: Aabb,
    pub vx: f32,
    pub vy: f32,
    pub facing: Facing,
    pub anim_t: f32,
    pub hearts: i32,
    pub max_hearts: i32,
    pub coins: u32,
    pub powerups: Powerups,
    pub attack_cooldown: f32,
    pub damage_flash: f32,
    pub swim_air: f32, // Sekunden ohne Brezel — bei 0 ertrinkt man
    pub on_ice: bool,
    pub on_water: bool,
    pub on_slow: bool,
}

impl Player {
    pub fn new(start_x: f32, start_y: f32) -> Self {
        Self {
            aabb: Aabb::new(start_x, start_y, 12.0, 14.0),
            vx: 0.0,
            vy: 0.0,
            facing: Facing::Down,
            anim_t: 0.0,
            hearts: PLAYER_START_HEARTS,
            max_hearts: PLAYER_MAX_HEARTS_INIT,
            coins: 0,
            powerups: Powerups::default(),
            attack_cooldown: 0.0,
            damage_flash: 0.0,
            swim_air: 10.0,
            on_ice: false,
            on_water: false,
            on_slow: false,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        self.aabb.center()
    }

    pub fn update(&mut self, world: &World, dt: f32) {
        self.powerups.tick(dt);
        self.anim_t += dt;
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        self.damage_flash = (self.damage_flash - dt).max(0.0);

        // Tile unter dem Spieler bestimmen
        let (cx, cy) = self.aabb.center();
        let here = tile_at(world, cx, cy);
        self.on_ice = here.is_ice();
        self.on_water = here.is_water();
        self.on_slow = here.is_slow();

        // Input
        let mut ix: f32 = 0.0;
        let mut iy: f32 = 0.0;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            iy -= 1.0;
            self.facing = Facing::Up;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            iy += 1.0;
            self.facing = Facing::Down;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            ix -= 1.0;
            self.facing = Facing::Left;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            ix += 1.0;
            self.facing = Facing::Right;
        }
        // Diagonale normalisieren
        let len = (ix * ix + iy * iy).sqrt();
        if len > 0.0 {
            ix /= len;
            iy /= len;
        }

        // Geschwindigkeit
        let mut speed = PLAYER_SPEED;
        if self.on_water { speed = PLAYER_SWIM_SPEED; }
        if self.on_slow { speed *= 0.5; }
        if self.powerups.speed > 0.0 { speed *= 1.8; }
        if self.powerups.fly > 0.0 { speed *= 1.3; }

        // Eis-Physik: behaltene velocity, niedrige Reibung
        if self.on_ice && self.powerups.fly <= 0.0 {
            self.vx += ix * speed * dt * 4.0;
            self.vy += iy * speed * dt * 4.0;
            self.vx *= 1.0 - PLAYER_ICE_FRICTION;
            self.vy *= 1.0 - PLAYER_ICE_FRICTION;
            let max_v = speed * 1.4;
            self.vx = self.vx.clamp(-max_v, max_v);
            self.vy = self.vy.clamp(-max_v, max_v);
        } else {
            // Normale Bewegung — Lerp zur Zielgeschwindigkeit (top-down feel).
            let target_vx = ix * speed;
            let target_vy = iy * speed;
            // blend = 0..1 wie schnell wir uns dem target nähern; höher = snappier
            let blend = (12.0 * dt).clamp(0.0, 1.0);
            self.vx = self.vx + (target_vx - self.vx) * blend;
            self.vy = self.vy + (target_vy - self.vy) * blend;
            if ix == 0.0 && iy == 0.0 {
                let stop = (PLAYER_NORMAL_FRICTION * 10.0 * dt).clamp(0.0, 1.0);
                self.vx *= 1.0 - stop;
                self.vy *= 1.0 - stop;
            }
        }

        // Bewegung anwenden (mit Kollision)
        let dx = self.vx * dt;
        let dy = self.vy * dt;
        if self.powerups.fly > 0.0 {
            // Fliegen ignoriert Mauern? Nein, nur Wasser/Eis-Effekte — Bäume bleiben Bäume.
            // Wir machen Fliegen halb-durchlässig (ignoriert Wasser).
            self.aabb.x += dx;
            self.aabb.y += dy;
            // Map-Bounds
            let max_x = MAP_W as f32 * TILE_SIZE - self.aabb.w;
            let max_y = MAP_H as f32 * TILE_SIZE - self.aabb.h;
            self.aabb.x = self.aabb.x.clamp(0.0, max_x);
            self.aabb.y = self.aabb.y.clamp(0.0, max_y);
        } else {
            let _ = move_aabb(world, &mut self.aabb, dx, dy);
        }

        // Ertrinken im Wasser
        if self.on_water && self.powerups.fly <= 0.0 {
            self.swim_air -= dt;
            if self.swim_air <= 0.0 {
                self.take_damage(1);
                self.swim_air = 5.0;
            }
        } else {
            self.swim_air = 10.0;
        }
    }

    /// Schaden zufügen — Schild & Unverwundbarkeit berücksichtigen.
    pub fn take_damage(&mut self, amt: i32) {
        if self.powerups.invuln > 0.0 {
            return;
        }
        if self.powerups.shield {
            self.powerups.shield = false;
            self.damage_flash = 0.3;
            return;
        }
        self.hearts = (self.hearts - amt).max(0);
        self.damage_flash = 0.4;
    }

    pub fn heal(&mut self, amt: i32) {
        self.hearts = (self.hearts + amt).min(self.max_hearts);
    }

    pub fn heal_full(&mut self) {
        self.hearts = self.max_hearts;
    }

    pub fn add_coins(&mut self, base: u32) {
        let final_amt = if self.powerups.double_coin > 0.0 { base * 2 } else { base };
        self.coins += final_amt;
    }

    pub fn alive(&self) -> bool {
        self.hearts > 0
    }
}
