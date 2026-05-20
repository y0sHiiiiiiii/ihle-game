//! Welt-Modul: Tilemap, Areas, Kamera, Tag/Nacht-Zyklus.

use crate::consts::*;
use macroquad::prelude::*;

// ------------------------------------------------------------------------
//  Tile-Typen
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Grass,
    Dirt,
    Road,
    Sidewalk,
    RedThread,
    Water,
    Sand,
    Ice,
    Forest,
    Building,
    IhleWall,
    BossFloor,
    MoldSlow,
    Brunnen,
}

impl Tile {
    /// Kann der Spieler hier laufen?
    pub fn walkable(&self) -> bool {
        !matches!(
            self,
            Tile::Building | Tile::IhleWall | Tile::Forest | Tile::Brunnen
        )
    }

    pub fn is_water(&self) -> bool {
        matches!(self, Tile::Water)
    }

    pub fn is_ice(&self) -> bool {
        matches!(self, Tile::Ice)
    }

    pub fn is_slow(&self) -> bool {
        matches!(self, Tile::MoldSlow)
    }
}

// ------------------------------------------------------------------------
//  Areas — die 16 echten Germering-Gebiete als Rechtecke
// ------------------------------------------------------------------------

#[derive(Clone)]
pub struct Area {
    pub name: &'static str,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Area {
    pub fn contains_tile(&self, tx: i32, ty: i32) -> bool {
        tx >= self.x0 && tx < self.x1 && ty >= self.y0 && ty < self.y1
    }

    pub fn contains_world(&self, px: f32, py: f32) -> bool {
        let tx = (px / TILE_SIZE) as i32;
        let ty = (py / TILE_SIZE) as i32;
        self.contains_tile(tx, ty)
    }
}

pub fn areas() -> Vec<Area> {
    vec![
        Area { name: AREA_GERMSEE,    x0: 5,   y0: 5,   x1: 55,  y1: 40  },
        Area { name: AREA_BAHNHOF,    x0: 95,  y0: 5,   x1: 130, y1: 25  },
        Area { name: AREA_WWK,        x0: 60,  y0: 5,   x1: 90,  y1: 28  },
        Area { name: AREA_CEWESTR,    x0: 160, y0: 20,  x1: 200, y1: 45  },
        Area { name: AREA_STADTPARK,  x0: 90,  y0: 30,  x1: 130, y1: 50  },
        Area { name: AREA_PARSBERG,   x0: 5,   y0: 55,  x1: 30,  y1: 85  },
        Area { name: AREA_GEP,        x0: 40,  y0: 50,  x1: 70,  y1: 75  },
        Area { name: AREA_STADTHALLE, x0: 75,  y0: 50,  x1: 95,  y1: 65  },
        Area { name: AREA_STADTMITTE, x0: 80,  y0: 50,  x1: 130, y1: 80  },
        Area { name: AREA_MUSEUM,     x0: 110, y0: 53,  x1: 130, y1: 65  },
        Area { name: AREA_FRIEDENSTR, x0: 90,  y0: 65,  x1: 120, y1: 90  },
        Area { name: AREA_FREIBAD,    x0: 130, y0: 50,  x1: 155, y1: 75  },
        Area { name: AREA_POLARIOM,   x0: 155, y0: 75,  x1: 195, y1: 100 },
        Area { name: AREA_KAUFHOF,    x0: 85,  y0: 85,  x1: 115, y1: 105 },
        Area { name: AREA_CORDOBAR,   x0: 10,  y0: 90,  x1: 40,  y1: 120 },
        Area { name: AREA_FORST,      x0: 30,  y0: 95,  x1: 150, y1: 120 },
    ]
}

// ------------------------------------------------------------------------
//  World
// ------------------------------------------------------------------------

pub struct World {
    pub tiles: Vec<Tile>,
    pub w: i32,
    pub h: i32,
    pub areas: Vec<Area>,
}

impl World {
    pub fn new() -> Self {
        let w = MAP_W;
        let h = MAP_H;
        let mut tiles = vec![Tile::Grass; (w * h) as usize];
        let areas = areas();

        // ---- Hilfsfunktion ----
        let idx = |x: i32, y: i32| (y * w + x) as usize;

        // 1) Basisterrain pro Area
        for a in &areas {
            let base = match a.name {
                s if s == AREA_GERMSEE => Tile::Water,
                s if s == AREA_FORST => Tile::Forest,
                s if s == AREA_POLARIOM => Tile::Ice,
                s if s == AREA_FREIBAD => Tile::Sand,
                s if s == AREA_STADTPARK => Tile::Grass,
                s if s == AREA_PARSBERG => Tile::Forest,
                s if s == AREA_KAUFHOF => Tile::BossFloor,
                s if s == AREA_CORDOBAR => Tile::Dirt,
                _ => Tile::Grass,
            };
            for y in a.y0.max(0)..a.y1.min(h) {
                for x in a.x0.max(0)..a.x1.min(w) {
                    tiles[idx(x, y)] = base;
                }
            }
        }

        // 2) Strand um den See
        let lake = &areas[0];
        for y in (lake.y0 - 1).max(0)..(lake.y1 + 1).min(h) {
            for x in (lake.x0 - 1).max(0)..(lake.x1 + 1).min(w) {
                if tiles[idx(x, y)] == Tile::Grass {
                    // Nur am Rand vom See → Sand
                    let mut neighbor_water = false;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = x + dx;
                            let ny = y + dy;
                            if nx >= 0 && ny >= 0 && nx < w && ny < h
                                && tiles[idx(nx, ny)] == Tile::Water
                            {
                                neighbor_water = true;
                            }
                        }
                    }
                    if neighbor_water {
                        tiles[idx(x, y)] = Tile::Sand;
                    }
                }
            }
        }

        // 3) Freibad-Innenpool (kleines Wasser in Sand)
        for y in 55..70 {
            for x in 135..150 {
                if tiles[idx(x, y)] == Tile::Sand {
                    tiles[idx(x, y)] = Tile::Water;
                }
            }
        }
        // Rand wieder Sand
        for y in 54..71 {
            tiles[idx(135, y)] = Tile::Sand;
            tiles[idx(149, y)] = Tile::Sand;
        }
        for x in 135..150 {
            tiles[idx(x, 55)] = Tile::Sand;
            tiles[idx(x, 69)] = Tile::Sand;
        }

        // 4) Straßennetz — horizontal & vertikal
        let h_roads = [20, 40, 60, 80, 100];
        let v_roads = [25, 50, 75, 105, 130, 160];
        for &y in &h_roads {
            for x in 0..w {
                if tiles[idx(x, y)] == Tile::Grass
                    || tiles[idx(x, y)] == Tile::Dirt
                    || tiles[idx(x, y)] == Tile::Forest
                {
                    tiles[idx(x, y)] = Tile::Road;
                    if y + 1 < h && tiles[idx(x, y + 1)] != Tile::Water {
                        tiles[idx(x, y + 1)] = Tile::Sidewalk;
                    }
                    if y > 0 && tiles[idx(x, y - 1)] != Tile::Water {
                        tiles[idx(x, y - 1)] = Tile::Sidewalk;
                    }
                }
            }
        }
        for &x in &v_roads {
            for y in 0..h {
                if tiles[idx(x, y)] == Tile::Grass
                    || tiles[idx(x, y)] == Tile::Dirt
                    || tiles[idx(x, y)] == Tile::Forest
                    || tiles[idx(x, y)] == Tile::Sidewalk
                {
                    tiles[idx(x, y)] = Tile::Road;
                    if x + 1 < w && tiles[idx(x + 1, y)] != Tile::Water {
                        if tiles[idx(x + 1, y)] != Tile::Road {
                            tiles[idx(x + 1, y)] = Tile::Sidewalk;
                        }
                    }
                    if x > 0 && tiles[idx(x - 1, y)] != Tile::Water {
                        if tiles[idx(x - 1, y)] != Tile::Road {
                            tiles[idx(x - 1, y)] = Tile::Sidewalk;
                        }
                    }
                }
            }
        }

        // 5) Roter Faden — folgt von Filiale 1 → 2 → 3 → 4
        // Pfad-Punkte (Tile-Koordinaten)
        let red_path: &[(i32, i32)] = &[
            (100, 70), // Filiale 1 (Friedenstr.)
            (100, 60),
            (90, 60),
            (75, 60),
            (55, 60), // Filiale 2 (GEP)
            (75, 60),
            (95, 60),
            (110, 60),
            (110, 40),
            (110, 25),
            (110, 18), // Filiale 3 (Bahnhof)
            (130, 25),
            (160, 25),
            (170, 28), // Filiale 4 (Cewestr.)
        ];
        for win in red_path.windows(2) {
            let (mut x, mut y) = win[0];
            let (tx, ty) = win[1];
            while x != tx || y != ty {
                if x >= 0 && y >= 0 && x < w && y < h {
                    let t = tiles[idx(x, y)];
                    if t == Tile::Sidewalk || t == Tile::Road || t == Tile::Grass {
                        tiles[idx(x, y)] = Tile::RedThread;
                    }
                }
                if x < tx { x += 1; } else if x > tx { x -= 1; }
                else if y < ty { y += 1; } else if y > ty { y -= 1; }
            }
        }

        // 6) Gebäude in Stadtgebieten — sauber zwischen die Straßen gesetzt.
        //    h_roads: y=20,40,60,80,100  ·  v_roads: x=25,50,75,105,130,160
        //    Jedes Gebäude liegt komplett in einem Block zwischen zwei Straßen.
        let buildings: &[(i32, i32, i32, i32)] = &[
            // WWK Hochhäuser (Block y=5..18, zwischen x=52..73, oberhalb y=20-Straße)
            (54, 7, 4, 12), (60, 5, 4, 13), (66, 8, 5, 11),
            // Bahnhof — Bahnsteig-Gebäude (zwischen v=75 und v=105, y=8..18)
            (78, 8, 5, 11), (85, 10, 5, 9), (92, 8, 5, 11), (99, 10, 5, 9),
            // weitere Bahnhof-Gebäude (zwischen v=105 und v=130, y=8..18)
            (108, 8, 5, 11), (115, 10, 6, 9), (123, 8, 5, 11),
            // GEP-Einkaufszentrum (zwischen v=25..50, y=52..58 oberhalb y=60-Straße)
            (28, 52, 6, 7), (36, 52, 5, 7), (43, 52, 5, 7),
            // GEP süd (y=62..73, zwischen v=25..50)
            (28, 62, 6, 8), (36, 62, 5, 8), (43, 64, 5, 8),
            // Stadthalle (zwischen v=75..105, y=52..58)
            (77, 52, 7, 7), (86, 52, 6, 7), (94, 52, 5, 7),
            // Stadtmuseum ZEIT+RAUM (zwischen v=105..130, y=52..58)
            (108, 52, 6, 7), (116, 52, 6, 7),
            // St. Jakobskirche neben dem Jakobusbrunnen (Brunnen liegt bei 115,35)
            (113, 30, 5, 5),
            // Stadtmitte Wohnhäuser (zwischen y=62..78, zwischen v=75..105)
            (78, 63, 4, 5), (85, 63, 4, 5), (92, 63, 4, 5), (99, 63, 4, 5),
            (78, 71, 5, 7), (86, 71, 5, 7), (94, 73, 5, 5),
            // Cewestr Gewerbe (zwischen v=160 und Map-Rand, y=22..38)
            (162, 22, 6, 7), (170, 22, 6, 7), (178, 22, 6, 7), (186, 22, 6, 7),
            (162, 31, 6, 8), (170, 31, 6, 8), (178, 31, 6, 8), (186, 31, 6, 8),
            // Cordobar Ruine (zerfallene Gebäude in Cordobar-Area, abseits der Straßen)
            (14, 92, 4, 4), (20, 95, 3, 3), (29, 92, 4, 5), (15, 102, 4, 3),
        ];
        for &(bx, by, bw, bh) in buildings {
            for y in by..(by + bh).min(h) {
                for x in bx..(bx + bw).min(w) {
                    if y < h && x < w {
                        // NIEMALS Straßen oder Gehsteige überschreiben.
                        let cur = tiles[idx(x, y)];
                        if matches!(cur, Tile::Road | Tile::Sidewalk) {
                            continue;
                        }
                        tiles[idx(x, y)] = Tile::Building;
                    }
                }
            }
        }

        // 7) Ihle-Filialen als spezielle Wände (begehbar nur durch Door-Logik)
        for f in &FILIALEN {
            for dy in -1..=1_i32 {
                for dx in -2..=2_i32 {
                    let x = f.tile_x + dx;
                    let y = f.tile_y + dy;
                    if x >= 0 && y >= 0 && x < w && y < h {
                        tiles[idx(x, y)] = Tile::IhleWall;
                    }
                }
            }
            // Eingangstür → Sidewalk vor dem Gebäude
            let dx = f.tile_x;
            let dy = f.tile_y + 2;
            if dy < h {
                tiles[idx(dx, dy)] = Tile::Sidewalk;
            }
        }

        // 8) Germarbrunnen in der Stadtmitte
        if 100 < w && 65 < h {
            tiles[idx(100, 60)] = Tile::Brunnen;
        }

        // 9) Boss-Kaufhof Eingang
        for x in 95..105 {
            tiles[idx(x, 86)] = Tile::Sidewalk;
        }

        // 10) Cordobar-Easter-Egg-Schild (als Brunnen-Marker)
        tiles[idx(20, 105)] = Tile::Brunnen;

        // 11) Jakobusbrunnen (Stadtpark)
        tiles[idx(115, 35)] = Tile::Brunnen;

        // 12) Mariensäule (Augsburger Straße — wir nehmen Stadtmitte-Position)
        tiles[idx(85, 65)] = Tile::Brunnen;

        // 13) Römischer Ziegelbrennofen (Richtung Alling — Westrand)
        tiles[idx(8, 50)] = Tile::Brunnen;

        World { tiles, w, h, areas }
    }

    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return Tile::Building; // Map-Rand = Wand
        }
        self.tiles[(y * self.w + x) as usize]
    }

    pub fn area_at(&self, tx: i32, ty: i32) -> Option<&Area> {
        // Spezifischere (kleinere) Areas zuerst, damit Filiale > Stadtmitte
        let mut best: Option<&Area> = None;
        for a in &self.areas {
            if a.contains_tile(tx, ty) {
                if let Some(b) = best {
                    let asize = (a.x1 - a.x0) * (a.y1 - a.y0);
                    let bsize = (b.x1 - b.x0) * (b.y1 - b.y0);
                    if asize < bsize {
                        best = Some(a);
                    }
                } else {
                    best = Some(a);
                }
            }
        }
        best
    }
}

// ------------------------------------------------------------------------
//  Kamera
// ------------------------------------------------------------------------

pub struct GameCamera {
    pub x: f32,
    pub y: f32,
    /// Aktueller Shake-Offset, wird pro Frame neu berechnet.
    pub shake_x: f32,
    pub shake_y: f32,
    /// Restamplitude des Shake-Effekts.
    pub shake_intensity: f32,
}

impl GameCamera {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            shake_x: 0.0,
            shake_y: 0.0,
            shake_intensity: 0.0,
        }
    }

    /// Folgt dem Spieler mit Lerp und Clamping an die Map-Grenzen.
    pub fn follow(&mut self, target_x: f32, target_y: f32) {
        let dst_x = target_x - VIRTUAL_W as f32 / 2.0;
        let dst_y = target_y - VIRTUAL_H as f32 / 2.0;
        self.x += (dst_x - self.x) * 0.08;
        self.y += (dst_y - self.y) * 0.08;

        let max_x = (MAP_W as f32 * TILE_SIZE) - VIRTUAL_W as f32;
        let max_y = (MAP_H as f32 * TILE_SIZE) - VIRTUAL_H as f32;
        self.x = self.x.clamp(0.0, max_x.max(0.0));
        self.y = self.y.clamp(0.0, max_y.max(0.0));
    }

    /// Stößt einen Screen-Shake an (additiv, größere Werte gewinnen).
    pub fn add_shake(&mut self, intensity: f32) {
        self.shake_intensity = self.shake_intensity.max(intensity);
    }

    /// Aktualisiert Shake-Offset (sin-basiert, deterministisch ohne RNG).
    pub fn update_shake(&mut self, dt: f32, t: f32) {
        // Exponentielles Abklingen
        self.shake_intensity *= (1.0 - 9.0 * dt).max(0.0);
        if self.shake_intensity > 0.1 {
            self.shake_x = (t * 137.0).sin() * self.shake_intensity;
            self.shake_y = (t * 211.0).cos() * self.shake_intensity;
        } else {
            self.shake_intensity = 0.0;
            self.shake_x = 0.0;
            self.shake_y = 0.0;
        }
    }

    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (wx - self.x + self.shake_x, wy - self.y + self.shake_y)
    }
}

// ------------------------------------------------------------------------
//  Tag-Nacht-Zyklus
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeOfDay {
    Morgen,
    Tag,
    Abend,
    Nacht,
}

impl TimeOfDay {
    /// Tint-Overlay für die ganze Szene.
    pub fn tint(&self) -> Color {
        match self {
            TimeOfDay::Morgen => Color::new(1.0, 0.78, 0.55, 0.18),
            TimeOfDay::Tag => Color::new(0.0, 0.0, 0.0, 0.0),
            TimeOfDay::Abend => Color::new(0.3, 0.3, 0.7, 0.20),
            TimeOfDay::Nacht => Color::new(0.05, 0.05, 0.2, 0.55),
        }
    }
}

/// Konvertiert Spielzeit (Sekunden) in eine Tageszeit-Variante.
pub fn time_of_day(secs: f32) -> TimeOfDay {
    let game_hour = ((secs / DAY_LENGTH_SECONDS) * 24.0 + 5.0) % 24.0;
    match game_hour as i32 {
        5..=7 => TimeOfDay::Morgen,
        8..=17 => TimeOfDay::Tag,
        18..=21 => TimeOfDay::Abend,
        _ => TimeOfDay::Nacht,
    }
}

/// Liefert die fiktive Uhrzeit als "HH:MM Uhr".
pub fn clock_string(secs: f32) -> String {
    let game_hour_f = (secs / DAY_LENGTH_SECONDS) * 24.0 + 5.0;
    let hh = (game_hour_f as i32).rem_euclid(24);
    let mm = ((game_hour_f.fract() * 60.0) as i32).rem_euclid(60);
    format!("{:02}:{:02} Uhr", hh, mm)
}
