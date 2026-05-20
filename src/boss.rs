//! Schimmelmeister Modrý — der Endboss im alten Kaufhof.
//!
//! 3 Phasen:
//!  1) 300→180 HP: Sporen-Projektile, parabolische Trajektorie
//!  2) 180→80 HP: Klone (3 Kopien — nur der echte verliert HP)
//!  3) 80→0 HP:   Rasend, hinterlässt Schimmel-Slow-Tiles

use crate::collision::Aabb;
use crate::consts::*;
use crate::entities::Projectile;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossPhase {
    Intro,
    P1,
    P2,
    P3,
    Dead,
}

pub struct BossClone {
    pub aabb: Aabb,
    pub real: bool,
    pub move_t: f32,
    pub vx: f32,
    pub vy: f32,
}

pub struct Boss {
    pub aabb: Aabb,
    pub hp: i32,
    pub phase: BossPhase,
    pub attack_cool: f32,
    pub move_t: f32,
    pub clones: Vec<BossClone>,
    pub taunt_t: f32,
    pub current_taunt: &'static str,
    pub hit_flash: f32,
    pub anim_t: f32,
    pub slime_drop_t: f32,
}

impl Boss {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            aabb: Aabb::new(x, y, 44.0, 44.0),
            hp: BOSS_HP_MAX,
            phase: BossPhase::Intro,
            attack_cool: 2.0,
            move_t: 0.0,
            clones: Vec::new(),
            taunt_t: 0.0,
            current_taunt: BOSS_TAUNT_P1,
            hit_flash: 0.0,
            anim_t: 0.0,
            slime_drop_t: 0.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.phase != BossPhase::Dead && self.hp > 0
    }

    pub fn center(&self) -> (f32, f32) {
        self.aabb.center()
    }

    /// Aktualisiert den Boss und gibt Projektile sowie Slow-Tile-Spawns zurück.
    pub fn update(
        &mut self,
        player_x: f32,
        player_y: f32,
        dt: f32,
        new_projectiles: &mut Vec<Projectile>,
        new_slow_spawns: &mut Vec<(f32, f32, f32)>,
    ) {
        if !self.alive() {
            return;
        }
        self.anim_t += dt;
        self.attack_cool = (self.attack_cool - dt).max(0.0);
        self.move_t += dt;
        self.taunt_t = (self.taunt_t - dt).max(0.0);
        self.hit_flash = (self.hit_flash - dt).max(0.0);

        // Phasen-Übergänge
        let new_phase = if self.hp <= 0 {
            BossPhase::Dead
        } else if self.hp <= BOSS_PHASE3_HP {
            BossPhase::P3
        } else if self.hp <= BOSS_PHASE2_HP {
            BossPhase::P2
        } else {
            BossPhase::P1
        };
        if new_phase != self.phase && new_phase != BossPhase::Intro {
            self.phase = new_phase;
            match self.phase {
                BossPhase::P1 => self.current_taunt = BOSS_TAUNT_P1,
                BossPhase::P2 => {
                    self.current_taunt = BOSS_TAUNT_P2;
                    self.spawn_clones();
                }
                BossPhase::P3 => self.current_taunt = BOSS_TAUNT_P3,
                _ => {}
            }
            self.taunt_t = 3.0;
        }

        // Intro automatisch enden lassen
        if self.phase == BossPhase::Intro && self.move_t > 2.0 {
            self.phase = BossPhase::P1;
            self.taunt_t = 3.0;
        }

        // Bewegung — Schwebe-Muster
        let cx = 100.0 * TILE_SIZE; // Mitte des Kaufhof-Areas
        let cy = 95.0 * TILE_SIZE;
        let speed_mul = match self.phase {
            BossPhase::P3 => 2.2,
            BossPhase::P2 => 1.4,
            _ => 1.0,
        };
        let radius = match self.phase {
            BossPhase::P3 => 80.0,
            _ => 50.0,
        };
        self.aabb.x = cx - 22.0 + (self.move_t * speed_mul * 0.8).cos() * radius;
        self.aabb.y = cy - 22.0 + (self.move_t * speed_mul * 0.6).sin() * radius * 0.6;

        // Angriffe
        if self.attack_cool <= 0.0 {
            let dx = player_x - (self.aabb.x + 22.0);
            let dy = player_y - (self.aabb.y + 22.0);
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);
            let speed = 90.0;
            let arc = -50.0; // negativer vy für Bogenflug
            match self.phase {
                BossPhase::P1 => {
                    new_projectiles.push(Projectile::new(
                        self.aabb.x + 18.0,
                        self.aabb.y + 18.0,
                        dx / dist * speed,
                        dy / dist * speed + arc,
                        2,
                    ));
                    self.attack_cool = 1.8;
                }
                BossPhase::P2 => {
                    // 2 Sporen im Doppelpack
                    new_projectiles.push(Projectile::new(
                        self.aabb.x + 18.0,
                        self.aabb.y + 18.0,
                        dx / dist * speed,
                        dy / dist * speed + arc,
                        3,
                    ));
                    new_projectiles.push(Projectile::new(
                        self.aabb.x + 18.0,
                        self.aabb.y + 18.0,
                        -dx / dist * speed,
                        dy / dist * speed + arc,
                        2,
                    ));
                    self.attack_cool = 1.4;
                }
                BossPhase::P3 => {
                    // 3er Streufeuer
                    for i in -1..=1 {
                        let ang = i as f32 * 0.4;
                        let vx = (dx / dist) * speed * (1.0 + 0.0) - (dy / dist) * speed * ang;
                        let vy = (dy / dist) * speed * (1.0 + 0.0) + (dx / dist) * speed * ang;
                        new_projectiles.push(Projectile::new(
                            self.aabb.x + 18.0,
                            self.aabb.y + 18.0,
                            vx,
                            vy + arc,
                            3,
                        ));
                    }
                    self.attack_cool = 1.1;
                }
                _ => {}
            }
        }

        // Phase 3: Schimmelspuren hinterlassen
        if self.phase == BossPhase::P3 {
            self.slime_drop_t += dt;
            if self.slime_drop_t > 0.6 {
                self.slime_drop_t = 0.0;
                new_slow_spawns.push((self.aabb.x, self.aabb.y + 30.0, 5.0));
            }
        }

        // Klone aktualisieren (nur in P2)
        if self.phase == BossPhase::P2 {
            for c in self.clones.iter_mut() {
                c.move_t += dt;
                c.aabb.x += c.vx * dt;
                c.aabb.y += c.vy * dt;
                if (c.move_t as i32) % 2 == 0 {
                    c.vx = -(c.move_t * 1.7).sin() * 30.0;
                    c.vy = (c.move_t * 1.3).cos() * 30.0;
                }
                // Map-Grenzen
                let min_x = 85.0 * TILE_SIZE;
                let max_x = 115.0 * TILE_SIZE - c.aabb.w;
                let min_y = 85.0 * TILE_SIZE;
                let max_y = 105.0 * TILE_SIZE - c.aabb.h;
                c.aabb.x = c.aabb.x.clamp(min_x, max_x);
                c.aabb.y = c.aabb.y.clamp(min_y, max_y);
            }
        }
    }

    fn spawn_clones(&mut self) {
        self.clones.clear();
        let (cx, cy) = self.center();
        for i in 0..3 {
            let ang = i as f32 * std::f32::consts::TAU / 3.0;
            let x = cx + ang.cos() * 60.0 - 22.0;
            let y = cy + ang.sin() * 60.0 - 22.0;
            self.clones.push(BossClone {
                aabb: Aabb::new(x, y, 44.0, 44.0),
                real: false,
                move_t: i as f32 * 0.7,
                vx: ang.cos() * 30.0,
                vy: ang.sin() * 30.0,
            });
        }
        // Echter Boss verbleibt mit drin (durch Hauptbody)
    }

    pub fn hurt(&mut self, dmg: i32, hit_box: Aabb) -> bool {
        // In P2: nur Treffer auf echten Boss (Haupt-AABB) zählen
        if self.phase == BossPhase::P2 {
            // Klon-Treffer ignorieren
            for c in &self.clones {
                if c.aabb.intersects(&hit_box) && !c.real {
                    // nur ein Hit-Flash für Klon, kein Schaden
                    return false;
                }
            }
        }
        if self.aabb.intersects(&hit_box) {
            self.hp -= dmg;
            self.hit_flash = 0.2;
            return true;
        }
        false
    }
}
