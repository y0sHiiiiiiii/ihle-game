//! AABB-Kollision und Tile-Properties-Helfer.

use crate::consts::TILE_SIZE;
use crate::world::{Tile, World};

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Aabb {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

/// Liefert das Tile an Welt-Pixelposition.
pub fn tile_at(world: &World, px: f32, py: f32) -> Tile {
    let tx = (px / TILE_SIZE) as i32;
    let ty = (py / TILE_SIZE) as i32;
    world.get(tx, ty)
}

/// Prüft, ob ein gegebenes AABB mit nicht-begehbaren Tiles kollidiert.
pub fn aabb_vs_world(world: &World, b: Aabb) -> bool {
    // Sample die 4 Ecken — kleine Entities reichen damit aus.
    let pts = [
        (b.x + 1.0, b.y + 1.0),
        (b.x + b.w - 1.0, b.y + 1.0),
        (b.x + 1.0, b.y + b.h - 1.0),
        (b.x + b.w - 1.0, b.y + b.h - 1.0),
    ];
    for (px, py) in pts {
        let t = tile_at(world, px, py);
        if !t.walkable() {
            return true;
        }
    }
    false
}

/// Bewegt ein AABB um (dx,dy), löst auf Achsen-getrennt mit Tile-Welt auf.
/// Liefert die tatsächliche Verschiebung zurück.
pub fn move_aabb(world: &World, b: &mut Aabb, dx: f32, dy: f32) -> (f32, f32) {
    let mut moved_x = 0.0;
    let mut moved_y = 0.0;

    // X-Achse
    let try_x = Aabb { x: b.x + dx, ..*b };
    if !aabb_vs_world(world, try_x) {
        b.x += dx;
        moved_x = dx;
    } else {
        // Krieche heran in kleinen Schritten
        let step = dx.signum() * 0.5;
        let mut remaining = dx.abs();
        while remaining > 0.0 {
            let trial = Aabb { x: b.x + step, ..*b };
            if aabb_vs_world(world, trial) {
                break;
            }
            b.x += step;
            moved_x += step;
            remaining -= 0.5;
        }
    }

    // Y-Achse
    let try_y = Aabb { y: b.y + dy, ..*b };
    if !aabb_vs_world(world, try_y) {
        b.y += dy;
        moved_y = dy;
    } else {
        let step = dy.signum() * 0.5;
        let mut remaining = dy.abs();
        while remaining > 0.0 {
            let trial = Aabb { y: b.y + step, ..*b };
            if aabb_vs_world(world, trial) {
                break;
            }
            b.y += step;
            moved_y += step;
            remaining -= 0.5;
        }
    }

    (moved_x, moved_y)
}
