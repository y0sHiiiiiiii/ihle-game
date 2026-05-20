//! Gegner, Münzen, Projektile.
//!
//! Gegner-State-Machine: Idle → Patrol → Chase → Attack → (Dead).

use crate::collision::{move_aabb, Aabb};
use crate::consts::*;
use crate::player::Player;
use crate::world::World;
use macroquad::prelude::*;

// ------------------------------------------------------------------------
//  Münzen
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum CoinKind {
    Copper,
    Silver,
    Gold,
    Brezel,
}

impl CoinKind {
    pub fn value(&self) -> u32 {
        match self {
            CoinKind::Copper => COIN_COPPER,
            CoinKind::Silver => COIN_SILVER,
            CoinKind::Gold => COIN_GOLD,
            CoinKind::Brezel => COIN_BREZEL,
        }
    }
}

pub struct Coin {
    pub x: f32,
    pub y: f32,
    pub kind: CoinKind,
    pub alive: bool,
    pub respawn_t: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub bob_phase: f32,
}

impl Coin {
    pub fn new(x: f32, y: f32, kind: CoinKind) -> Self {
        Self {
            x,
            y,
            kind,
            alive: true,
            respawn_t: 0.0,
            spawn_x: x,
            spawn_y: y,
            bob_phase: (x * 0.13 + y * 0.07) % 6.28,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.x + 2.0, self.y + 2.0, 12.0, 12.0)
    }

    pub fn update(&mut self, dt: f32) {
        self.bob_phase += dt * 3.0;
        if !self.alive {
            self.respawn_t -= dt;
            if self.respawn_t <= 0.0 {
                self.alive = true;
                self.x = self.spawn_x;
                self.y = self.spawn_y;
            }
        }
    }

    /// Y-Offset für die schwebende Sin-Animation.
    pub fn bob_y(&self) -> f32 {
        self.bob_phase.sin() * 2.0
    }
}

// ------------------------------------------------------------------------
//  Gegner
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Mold,
    Blob,
    Rat,
    Beat,
    Ice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Dead,
}

pub struct Enemy {
    pub kind: EnemyKind,
    pub aabb: Aabb,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub state: EnemyState,
    pub patrol_t: f32,
    pub attack_cool: f32,
    pub anim_t: f32,
    pub hit_flash: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub beat_t: f32, // für Takt-Pilz
    pub ice_drop_t: f32, // für Eis-Schimmel
    /// Restzeit Knockback (KI-override).
    pub knockback_t: f32,
    pub knockback_vx: f32,
    pub knockback_vy: f32,
}

impl Enemy {
    pub fn new(kind: EnemyKind, x: f32, y: f32) -> Self {
        let hp = match kind {
            EnemyKind::Mold => 2,
            EnemyKind::Blob => 3,
            EnemyKind::Rat => 1,
            EnemyKind::Beat => 3,
            EnemyKind::Ice => 4,
        };
        Self {
            kind,
            aabb: Aabb::new(x, y, 14.0, 14.0),
            vx: 0.0,
            vy: 0.0,
            hp,
            state: EnemyState::Patrol,
            patrol_t: 0.0,
            attack_cool: 0.0,
            anim_t: 0.0,
            hit_flash: 0.0,
            spawn_x: x,
            spawn_y: y,
            beat_t: 0.0,
            ice_drop_t: 0.0,
            knockback_t: 0.0,
            knockback_vx: 0.0,
            knockback_vy: 0.0,
        }
    }

    pub fn damage(&self) -> i32 {
        match self.kind {
            EnemyKind::Mold => 1,
            EnemyKind::Blob => 2,
            EnemyKind::Rat => 1,
            EnemyKind::Beat => 2,
            EnemyKind::Ice => 3,
        }
    }

    /// Drop bei Tod.
    pub fn drop(&self) -> Vec<(CoinKind, i32)> {
        match self.kind {
            EnemyKind::Mold => vec![(CoinKind::Copper, 2)],
            EnemyKind::Blob => vec![(CoinKind::Silver, 1)],
            EnemyKind::Rat => vec![(CoinKind::Copper, 3)],
            EnemyKind::Beat => vec![(CoinKind::Silver, 1)],
            EnemyKind::Ice => vec![(CoinKind::Silver, 2)],
        }
    }

    pub fn alive(&self) -> bool {
        self.state != EnemyState::Dead && self.hp > 0
    }

    pub fn update(
        &mut self,
        world: &World,
        player: &Player,
        dt: f32,
        ice_spawns: &mut Vec<(f32, f32, f32)>,
    ) {
        if !self.alive() {
            return;
        }
        self.anim_t += dt;
        self.attack_cool = (self.attack_cool - dt).max(0.0);
        self.hit_flash = (self.hit_flash - dt).max(0.0);
        self.knockback_t = (self.knockback_t - dt).max(0.0);

        // Während Knockback überschreibt der Impuls die KI-Bewegung.
        if self.knockback_t > 0.0 {
            self.aabb.x += self.knockback_vx * dt;
            self.aabb.y += self.knockback_vy * dt;
            self.knockback_vx *= 0.86;
            self.knockback_vy *= 0.86;
            return;
        }

        let (pcx, pcy) = player.center();
        let (ecx, ecy) = self.aabb.center();
        let dx = pcx - ecx;
        let dy = pcy - ecy;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);

        // Sichtweite — danach Chase
        let see = match self.kind {
            EnemyKind::Mold => 80.0,
            EnemyKind::Blob => 120.0,
            EnemyKind::Rat => 100.0,
            EnemyKind::Beat => 90.0,
            EnemyKind::Ice => 110.0,
        };

        if dist < see {
            self.state = EnemyState::Chase;
        } else if dist > see * 2.0 {
            self.state = EnemyState::Patrol;
        }

        match self.kind {
            EnemyKind::Mold => {
                // Patrol hin/her — entweder Chase oder Sinus-Patrol
                if self.state == EnemyState::Chase {
                    self.vx = (dx / dist) * 25.0;
                    self.vy = (dy / dist) * 25.0;
                } else {
                    self.patrol_t += dt;
                    self.vx = (self.patrol_t * 1.2).sin() * 15.0;
                    self.vy = 0.0;
                }
            }
            EnemyKind::Blob => {
                // Hüpf-Angriff alle 1.5 Sek
                self.patrol_t += dt;
                if self.patrol_t > 1.5 && self.state == EnemyState::Chase {
                    self.vx = (dx / dist) * 80.0;
                    self.vy = (dy / dist) * 80.0;
                    self.patrol_t = 0.0;
                } else {
                    self.vx *= 0.92;
                    self.vy *= 0.92;
                }
            }
            EnemyKind::Rat => {
                // Schnell auf Spieler zu, kehrt aber bei Nähe um (Hit-and-run)
                if dist < 30.0 {
                    self.vx = -(dx / dist) * 60.0;
                    self.vy = -(dy / dist) * 60.0;
                } else if self.state == EnemyState::Chase {
                    self.vx = (dx / dist) * 50.0;
                    self.vy = (dy / dist) * 50.0;
                } else {
                    self.patrol_t += dt;
                    self.vx = (self.patrol_t).cos() * 25.0;
                    self.vy = (self.patrol_t).sin() * 25.0;
                }
            }
            EnemyKind::Beat => {
                // Bewegt sich nur im Beat (0.5s Takt)
                self.beat_t += dt;
                if self.beat_t > 0.5 {
                    self.beat_t = 0.0;
                    if self.state == EnemyState::Chase {
                        self.vx = (dx / dist) * 60.0;
                        self.vy = (dy / dist) * 60.0;
                    } else {
                        // Zufalls-Hop
                        let a = (self.patrol_t * 7.13).sin();
                        self.vx = a * 30.0;
                        self.vy = a.cos() * 30.0;
                        self.patrol_t += 1.0;
                    }
                } else {
                    self.vx *= 0.7;
                    self.vy *= 0.7;
                }
            }
            EnemyKind::Ice => {
                // Lässt Eisflecken alle 2 Sek
                self.ice_drop_t += dt;
                if self.ice_drop_t > 2.0 {
                    self.ice_drop_t = 0.0;
                    ice_spawns.push((self.aabb.x, self.aabb.y, 4.0));
                }
                if self.state == EnemyState::Chase {
                    self.vx = (dx / dist) * 30.0;
                    self.vy = (dy / dist) * 30.0;
                } else {
                    self.patrol_t += dt;
                    self.vx = (self.patrol_t * 0.8).sin() * 18.0;
                    self.vy = (self.patrol_t * 0.7).cos() * 18.0;
                }
            }
        }

        // Bewegung anwenden
        let (mx, _my) = move_aabb(world, &mut self.aabb, self.vx * dt, self.vy * dt);
        if mx.abs() < 0.01 && self.kind == EnemyKind::Mold {
            // Patrol → richtung umkehren
            self.patrol_t += 1.5;
        }
    }

    pub fn hurt(&mut self, dmg: i32) {
        self.hp -= dmg;
        self.hit_flash = 0.15;
        if self.hp <= 0 {
            self.state = EnemyState::Dead;
        }
    }

    /// Stößt den Gegner kurz vom Punkt (fx, fy) weg.
    pub fn apply_knockback(&mut self, fx: f32, fy: f32, force: f32) {
        let (ex, ey) = self.aabb.center();
        let dx = ex - fx;
        let dy = ey - fy;
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        self.knockback_vx = (dx / d) * force;
        self.knockback_vy = (dy / d) * force;
        self.knockback_t = 0.18;
    }
}

// ------------------------------------------------------------------------
//  Projektile (Boss-Sporen)
// ------------------------------------------------------------------------

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub damage: i32,
}

impl Projectile {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, damage: i32) -> Self {
        Self { x, y, vx, vy, life: 4.0, damage }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.x + 5.0, self.y + 5.0, 6.0, 6.0)
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        // leichter Bogen (Gravitation)
        self.vy += 30.0 * dt;
        self.life -= dt;
        self.life > 0.0
    }
}

// ------------------------------------------------------------------------
//  Eis-Flecken (Slippery Tiles temporär)
// ------------------------------------------------------------------------

pub struct IcePatch {
    pub x: f32,
    pub y: f32,
    pub life: f32,
}

impl IcePatch {
    pub fn new(x: f32, y: f32, life: f32) -> Self {
        Self { x, y, life }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::new(self.x, self.y, 16.0, 16.0)
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.life -= dt;
        self.life > 0.0
    }
}

// ------------------------------------------------------------------------
//  Floating Text — kurze Damage- / Coin-Zahlen die aufsteigen + faden
// ------------------------------------------------------------------------

pub struct FloatingText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
}

impl FloatingText {
    pub fn new(x: f32, y: f32, text: String, color: Color) -> Self {
        Self {
            x,
            y,
            text,
            color,
            vy: -34.0,
            life: 0.85,
            max_life: 0.85,
        }
    }

    pub fn damage(x: f32, y: f32, dmg: i32) -> Self {
        Self::new(
            x,
            y,
            format!("-{}", dmg),
            Color::new(1.0, 0.30, 0.30, 1.0),
        )
    }

    pub fn coins(x: f32, y: f32, n: u32) -> Self {
        Self::new(
            x,
            y,
            format!("+{}", n),
            Color::new(1.0, 0.88, 0.30, 1.0),
        )
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.y += self.vy * dt;
        self.vy *= (1.0 - 1.8 * dt).max(0.0);
        self.life -= dt;
        self.life > 0.0
    }

    /// Alpha 0..1 für das aktuelle Leben.
    pub fn alpha(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
    }
}

// ------------------------------------------------------------------------
//  Spawning — generiert Gegner & Münzen auf der Karte
// ------------------------------------------------------------------------

pub fn spawn_world_entities(world: &World) -> (Vec<Coin>, Vec<Enemy>) {
    let mut coins = Vec::new();
    let mut enemies = Vec::new();

    // RNG-frei: deterministisches Verteilen über Tile-Koordinaten
    for y in 0..world.h {
        for x in 0..world.w {
            let t = world.get(x, y);
            // Münzen auf Sidewalk/Road verteilen
            if matches!(t, crate::world::Tile::Sidewalk | crate::world::Tile::RedThread) {
                if (x * 7 + y * 13) % 19 == 0 {
                    coins.push(Coin::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        CoinKind::Copper,
                    ));
                }
                if (x * 11 + y * 5) % 53 == 0 {
                    coins.push(Coin::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        CoinKind::Silver,
                    ));
                }
            }
            // Goldmünzen am See-Strand
            if matches!(t, crate::world::Tile::Sand) {
                if (x * 17 + y * 23) % 97 == 0 {
                    coins.push(Coin::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        CoinKind::Gold,
                    ));
                }
            }

            // Gegner-Spawning
            if let Some(area) = world.area_at(x, y) {
                let n = area.name;
                let walk = t.walkable() && !t.is_water();
                if !walk {
                    continue;
                }
                let hash = (x.wrapping_mul(31).wrapping_add(y.wrapping_mul(17))) as u32;
                let r = hash % 1000;

                if n == AREA_FORST && r < 6 {
                    enemies.push(Enemy::new(
                        EnemyKind::Mold,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ));
                } else if n == AREA_GERMSEE && r < 12 {
                    // Im See selber spawnt nichts (Wasser), aber am Strand:
                    if matches!(t, crate::world::Tile::Sand) && r < 3 {
                        enemies.push(Enemy::new(
                            EnemyKind::Blob,
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                        ));
                    }
                } else if n == AREA_FREIBAD && r < 5 {
                    enemies.push(Enemy::new(
                        EnemyKind::Blob,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ));
                } else if (n == AREA_BAHNHOF || n == AREA_CEWESTR) && r < 4 {
                    enemies.push(Enemy::new(
                        EnemyKind::Rat,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ));
                } else if n == AREA_CORDOBAR && r < 8 {
                    enemies.push(Enemy::new(
                        EnemyKind::Beat,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ));
                } else if n == AREA_POLARIOM && r < 6 {
                    enemies.push(Enemy::new(
                        EnemyKind::Ice,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ));
                } else if n == AREA_PARSBERG && r < 4 {
                    enemies.push(Enemy::new(
                        EnemyKind::Mold,
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                    ));
                }
            }
        }
    }

    // Goldmünzen explizit an Landmarks (Germarbrunnen etc.)
    coins.push(Coin::new(100.0 * TILE_SIZE, 60.0 * TILE_SIZE + 18.0, CoinKind::Gold));
    coins.push(Coin::new(115.0 * TILE_SIZE, 35.0 * TILE_SIZE + 18.0, CoinKind::Gold));
    coins.push(Coin::new(18.0 * TILE_SIZE, 60.0 * TILE_SIZE, CoinKind::Gold)); // Parsberg-Gipfel

    (coins, enemies)
}
