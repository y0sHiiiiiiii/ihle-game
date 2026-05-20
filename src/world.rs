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
    Zebra,
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

    /// Fahrzeuge stoppen, wenn ein Fußgänger einen Zebrastreifen quert.
    pub fn is_zebra(&self) -> bool {
        matches!(self, Tile::Zebra)
    }

    /// Fahrzeuge fahren auf Road / RedThread / Zebra.
    pub fn is_drivable(&self) -> bool {
        matches!(self, Tile::Road | Tile::RedThread | Tile::Zebra)
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
        // See nach Süd-West verschoben, damit Gleise (y=14) frei sind.
        Area { name: AREA_GERMSEE,    x0: 5,   y0: 22,  x1: 50,  y1: 55  },
        Area { name: AREA_HARTHAUS,   x0: 55,  y0: 5,   x1: 88,  y1: 22  },
        Area { name: AREA_WWK,        x0: 55,  y0: 22,  x1: 88,  y1: 35  },
        Area { name: AREA_BAHNHOF,    x0: 105, y0: 5,   x1: 138, y1: 22  },
        Area { name: AREA_CEWESTR,    x0: 155, y0: 5,   x1: 200, y1: 35  },
        Area { name: AREA_STADTPARK,  x0: 90,  y0: 30,  x1: 130, y1: 50  },
        Area { name: AREA_RATHAUS,    x0: 95,  y0: 50,  x1: 115, y1: 62  },
        Area { name: AREA_PARSBERG,   x0: 5,   y0: 58,  x1: 30,  y1: 88  },
        Area { name: AREA_GEP,        x0: 35,  y0: 50,  x1: 68,  y1: 78  },
        Area { name: AREA_STADTHALLE, x0: 72,  y0: 50,  x1: 92,  y1: 62  },
        Area { name: AREA_STADTMITTE, x0: 80,  y0: 50,  x1: 135, y1: 80  },
        Area { name: AREA_SCHULE,     x0: 118, y0: 50,  x1: 138, y1: 62  },
        Area { name: AREA_MUSEUM,     x0: 113, y0: 62,  x1: 132, y1: 75  },
        Area { name: AREA_FRIEDENSTR, x0: 88,  y0: 65,  x1: 122, y1: 92  },
        Area { name: AREA_FREIBAD,    x0: 135, y0: 50,  x1: 158, y1: 75  },
        Area { name: AREA_KRANKENHAUS,x0: 158, y0: 50,  x1: 178, y1: 70  },
        Area { name: AREA_POLARIOM,   x0: 158, y0: 75,  x1: 198, y1: 100 },
        Area { name: AREA_KAUFHOF,    x0: 85,  y0: 90,  x1: 115, y1: 108 },
        Area { name: AREA_CORDOBAR,   x0: 8,   y0: 92,  x1: 40,  y1: 120 },
        Area { name: AREA_FORST,      x0: 40,  y0: 95,  x1: 150, y1: 120 },
    ]
}

// ------------------------------------------------------------------------
//  World
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
//  Building-Variety — pro Gebäude wird ein Kind bestimmt für sichtbare
//  Vielfalt (Dachform, Farbe, Fensteranordnung, Sondernutzung).
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    House,         // Einfamilienhaus, rotes Satteldach
    Reihenhaus,    // Reihenhausreihe, beige Wand
    Apartment,     // Mehrfamilien, flaches Dach
    Highrise,      // WWK-Hochhaus
    Shop,          // Ladengeschäft, große Fenster
    Industrial,    // Halle (Cewestr) flach + grau
    Rathaus,       // Rathaus mit Turm
    Schule,        // Schule, hell + Glockenturm
    Krankenhaus,   // weißer Bau mit rotem Kreuz
    Tankstelle,    // breite flache Tankstelle
    Ruin,          // Cordobar-Ruine
    Kirche,        // St. Jakobskirche
}

#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub kind: BuildingKind,
    /// Palette-Seed für deterministisch unterschiedliche Farben.
    pub seed: u32,
}

pub struct World {
    pub tiles: Vec<Tile>,
    pub w: i32,
    pub h: i32,
    pub areas: Vec<Area>,
    pub buildings: Vec<Building>,
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
        for y in 21..56 {
            for x in 4..51 {
                if y < 0 || y >= h || x < 0 || x >= w { continue; }
                if tiles[idx(x, y)] != Tile::Grass { continue; }
                let mut neighbor_water = false;
                'scan: for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && ny >= 0 && nx < w && ny < h
                            && tiles[idx(nx, ny)] == Tile::Water
                        {
                            neighbor_water = true;
                            break 'scan;
                        }
                    }
                }
                if neighbor_water {
                    tiles[idx(x, y)] = Tile::Sand;
                }
            }
        }

        // 3) Freibad-Innenpool — neue Position passend zu AREA_FREIBAD (x=135..158, y=50..75)
        for y in 56..72 {
            for x in 138..156 {
                if tiles[idx(x, y)] == Tile::Sand {
                    tiles[idx(x, y)] = Tile::Water;
                }
            }
        }
        for y in 55..73 {
            tiles[idx(138, y)] = Tile::Sand;
            tiles[idx(155, y)] = Tile::Sand;
        }
        for x in 138..156 {
            tiles[idx(x, 56)] = Tile::Sand;
            tiles[idx(x, 71)] = Tile::Sand;
        }

        // 4) Straßennetz — horizontal & vertikal
        let h_roads = [20, 40, 60, 80, 100];
        let v_roads = [25, 50, 75, 105, 130, 160];
        for &y in &h_roads {
            for x in 0..w {
                if matches!(tiles[idx(x, y)], Tile::Grass | Tile::Dirt | Tile::Forest) {
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
                if matches!(
                    tiles[idx(x, y)],
                    Tile::Grass | Tile::Dirt | Tile::Forest | Tile::Sidewalk
                ) {
                    tiles[idx(x, y)] = Tile::Road;
                    if x + 1 < w && tiles[idx(x + 1, y)] != Tile::Water
                        && tiles[idx(x + 1, y)] != Tile::Road
                    {
                        tiles[idx(x + 1, y)] = Tile::Sidewalk;
                    }
                    if x > 0 && tiles[idx(x - 1, y)] != Tile::Water
                        && tiles[idx(x - 1, y)] != Tile::Road
                    {
                        tiles[idx(x - 1, y)] = Tile::Sidewalk;
                    }
                }
            }
        }

        // 5) Roter Faden — folgt von Filiale 1 → 2 → 3 → 4 (neue Positionen)
        let red_path: &[(i32, i32)] = &[
            (100, 70), // Filiale 1 (Friedenstr.)
            (100, 60),
            (75, 60),
            (50, 60),
            (45, 67),  // Filiale 2 (GEP)
            (50, 60),
            (75, 60),
            (105, 60),
            (105, 40),
            (105, 20),
            (118, 20),
            (118, 26), // Filiale 3 (Bahnhof)
            (130, 20),
            (160, 20),
            (168, 28), // Filiale 4 (Cewestr.)
        ];
        for win in red_path.windows(2) {
            let (mut x, mut y) = win[0];
            let (tx, ty) = win[1];
            while x != tx || y != ty {
                if x >= 0 && y >= 0 && x < w && y < h {
                    let t = tiles[idx(x, y)];
                    if matches!(t, Tile::Sidewalk | Tile::Road | Tile::Grass) {
                        tiles[idx(x, y)] = Tile::RedThread;
                    }
                }
                if x < tx { x += 1; } else if x > tx { x -= 1; }
                else if y < ty { y += 1; } else if y > ty { y -= 1; }
            }
        }

        // 6) Gebäude — mit Sortenvielfalt
        let mut buildings: Vec<Building> = Vec::new();

        fn push_b(
            out: &mut Vec<Building>,
            x: i32, y: i32, w: i32, h: i32,
            kind: BuildingKind,
        ) {
            let seed = (x.wrapping_mul(1973).wrapping_add(y.wrapping_mul(401)).wrapping_add(w * 31 + h * 17)) as u32;
            out.push(Building { x, y, w, h, kind, seed });
        }

        fn add_row(
            out: &mut Vec<Building>,
            x0: i32, y: i32, count: i32, unit_w: i32, house_h: i32, gap: i32,
            kind: BuildingKind,
        ) {
            for i in 0..count {
                let bx = x0 + i * (unit_w + gap);
                push_b(out, bx, y, unit_w, house_h, kind);
            }
        }

        // === Bahnhof Germering Empfangsgebäude (y=6..12, x=105..138) ===
        push_b(&mut buildings, 108, 6, 10, 6, BuildingKind::Shop);     // Bahnhofsgebäude
        push_b(&mut buildings, 123, 6, 8, 5, BuildingKind::Shop);      // Kiosk / DB Service

        // === Bahnhof Harthaus Empfangsgebäude (x=60..78, y=6..12) ===
        push_b(&mut buildings, 62, 7, 7, 5, BuildingKind::Shop);
        push_b(&mut buildings, 71, 8, 5, 4, BuildingKind::Shop);

        // === WWK Hochhaussiedlung (südl. der Gleise, y=24..34, x=56..86) ===
        push_b(&mut buildings, 56, 24, 5, 10, BuildingKind::Highrise);
        push_b(&mut buildings, 63, 23, 5, 11, BuildingKind::Highrise);
        push_b(&mut buildings, 70, 24, 5, 10, BuildingKind::Highrise);
        push_b(&mut buildings, 77, 23, 5, 11, BuildingKind::Highrise);
        push_b(&mut buildings, 84, 25, 4, 9, BuildingKind::Apartment);

        // === Zwischen Bahnhof und Stadtmitte (Wohnblöcke y=22..38) ===
        add_row(&mut buildings, 90, 22, 4, 5, 6, 1, BuildingKind::House);
        add_row(&mut buildings, 90, 32, 4, 5, 6, 1, BuildingKind::House);
        add_row(&mut buildings, 113, 22, 3, 5, 6, 1, BuildingKind::Reihenhaus);
        add_row(&mut buildings, 113, 32, 3, 5, 6, 1, BuildingKind::Reihenhaus);
        add_row(&mut buildings, 132, 22, 3, 7, 7, 1, BuildingKind::Apartment);
        add_row(&mut buildings, 132, 32, 3, 7, 7, 1, BuildingKind::Apartment);

        // === Cewestraße Gewerbe (y=21..38, x=161..199) ===
        push_b(&mut buildings, 161, 22, 11, 7, BuildingKind::Industrial); // Halle 1
        push_b(&mut buildings, 173, 22, 11, 7, BuildingKind::Industrial); // Halle 2
        push_b(&mut buildings, 185, 22, 12, 7, BuildingKind::Industrial); // Halle 3
        push_b(&mut buildings, 161, 31, 8, 7, BuildingKind::Industrial);  // Halle 4
        push_b(&mut buildings, 172, 31, 12, 7, BuildingKind::Shop);       // Großhandel
        push_b(&mut buildings, 187, 31, 10, 7, BuildingKind::Tankstelle); // Tankstelle Cewestr

        // === GEP Einkaufspassagen (y=41..78, x=36..68) ===
        push_b(&mut buildings, 36, 42, 13, 8, BuildingKind::Shop);     // GEP-Mall West
        push_b(&mut buildings, 52, 42, 14, 8, BuildingKind::Shop);     // GEP-Mall Ost
        add_row(&mut buildings, 36, 53, 3, 6, 6, 1, BuildingKind::Apartment);
        // Filiale 2 Block (Mitte): bleibt frei für IhleWall
        add_row(&mut buildings, 36, 72, 3, 6, 5, 1, BuildingKind::Apartment);

        // === Stadthalle (y=41..58, x=72..91) ===
        push_b(&mut buildings, 73, 42, 17, 7, BuildingKind::Rathaus);  // Stadthalle (großer Bau)
        add_row(&mut buildings, 73, 53, 3, 5, 6, 1, BuildingKind::House);

        // === Rathaus-Block (y=41..58, x=95..114) ===
        push_b(&mut buildings, 96, 52, 14, 7, BuildingKind::Rathaus);

        // === Schule (y=41..58, x=118..138) ===
        push_b(&mut buildings, 119, 52, 17, 8, BuildingKind::Schule);

        // === Stadtmuseum / Kirchen-Block (y=63..75, x=113..132) ===
        push_b(&mut buildings, 115, 63, 14, 8, BuildingKind::Shop); // Museum

        // === St. Jakobskirche (im Stadtpark) ===
        push_b(&mut buildings, 113, 31, 5, 5, BuildingKind::Kirche);

        // === Klinikum (y=51..68, x=158..178) ===
        push_b(&mut buildings, 159, 52, 17, 8, BuildingKind::Krankenhaus);
        push_b(&mut buildings, 159, 62, 18, 6, BuildingKind::Apartment); // Pflege-Wohnheim

        // === Freibad-Umgebung (y=41..58, x=131..158) ===
        add_row(&mut buildings, 131, 42, 3, 5, 6, 1, BuildingKind::House);

        // === Stadtmitte Wohnhäuser (y=62..78) — variiert ===
        add_row(&mut buildings, 80, 62, 3, 5, 6, 1, BuildingKind::Reihenhaus);
        add_row(&mut buildings, 80, 71, 3, 5, 6, 1, BuildingKind::Reihenhaus);
        // Filiale 1 ist in der Friedenstraße bei (100, 70)

        // === Friedenstr. Süd (y=82..98) ===
        add_row(&mut buildings, 78, 82, 4, 5, 6, 1, BuildingKind::House);
        add_row(&mut buildings, 78, 92, 4, 5, 6, 1, BuildingKind::House);
        add_row(&mut buildings, 108, 82, 2, 6, 6, 1, BuildingKind::Apartment);
        add_row(&mut buildings, 108, 92, 2, 6, 6, 1, BuildingKind::Apartment);

        // === Parsberg Siedlung (Waldrand) ===
        add_row(&mut buildings, 10, 67, 3, 4, 4, 1, BuildingKind::House);
        add_row(&mut buildings, 10, 73, 3, 4, 4, 1, BuildingKind::House);

        // === Polariom-Umgebung ===
        add_row(&mut buildings, 134, 82, 3, 6, 6, 1, BuildingKind::Apartment);
        add_row(&mut buildings, 161, 82, 3, 6, 6, 1, BuildingKind::Apartment);
        push_b(&mut buildings, 178, 82, 16, 7, BuildingKind::Shop); // Eisarena-Foyer

        // === Cordobar Ruine ===
        push_b(&mut buildings, 14, 92, 4, 4, BuildingKind::Ruin);
        push_b(&mut buildings, 21, 95, 3, 3, BuildingKind::Ruin);
        push_b(&mut buildings, 29, 92, 4, 5, BuildingKind::Ruin);
        push_b(&mut buildings, 15, 102, 4, 3, BuildingKind::Ruin);

        // === Reale Germeringer Landmarks (zusätzliche Wiedererkennbarkeit) ===
        // Feuerwehr Germering (an der Kerschensteinerstraße — wir nehmen Stadtmitte)
        push_b(&mut buildings, 132, 65, 5, 6, BuildingKind::Industrial);
        // Volksfestplatz-Bühne (kleine Halle nahe Stadthalle)
        push_b(&mut buildings, 73, 60, 6, 4, BuildingKind::Industrial);
        // Tennisclub Germering (südl. vom Freibad)
        push_b(&mut buildings, 144, 72, 8, 4, BuildingKind::Shop);
        // Polizeiwache Germering
        push_b(&mut buildings, 124, 65, 6, 5, BuildingKind::Apartment);
        // Sparkasse / Bank an der Münchner Straße
        push_b(&mut buildings, 53, 60, 5, 4, BuildingKind::Shop);
        // Aldi / REWE Discounter im Gewerbegebiet
        push_b(&mut buildings, 154, 24, 6, 6, BuildingKind::Industrial);
        // Bauhaus Baumarkt am Cewestr-Eingang
        push_b(&mut buildings, 188, 18, 10, 4, BuildingKind::Industrial);
        // Kindergarten am Stadtpark
        push_b(&mut buildings, 91, 32, 6, 5, BuildingKind::Schule);

        // === Ihle-Filiale-Gebäude — Filialen liegen IN diesen Shops ===
        // Maße passen exakt zum IhleWall-Stempel (5 breit × 3 hoch, dy -1..=1)
        // → die Wand-Tiles werden später drüber gestempelt, das Building-Detail
        //   rendert die hübsche Schaufenster-Fassade darüber.
        for f in &FILIALEN {
            push_b(
                &mut buildings,
                f.tile_x - 2, f.tile_y - 1,
                5, 3,
                BuildingKind::Shop,
            );
        }

        // Gebäude in die Tile-Map einbacken
        for b in &buildings {
            for y in b.y..(b.y + b.h).min(h) {
                for x in b.x..(b.x + b.w).min(w) {
                    if y < h && x < w {
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

        // 8) Brunnen / Landmarks
        // Germarbrunnen vor dem Rathaus
        tiles[idx(105, 50)] = Tile::Brunnen;
        // Cordobar-Easter-Egg
        tiles[idx(20, 105)] = Tile::Brunnen;
        // Jakobusbrunnen (Stadtpark)
        tiles[idx(115, 35)] = Tile::Brunnen;
        // Mariensäule (Augsburger Straße — wir nehmen Stadtmitte-Position)
        tiles[idx(85, 65)] = Tile::Brunnen;
        // Römischer Ziegelbrennofen
        tiles[idx(8, 54)] = Tile::Brunnen;

        // 9) Boss-Kaufhof Eingang
        for x in 95..105 {
            tiles[idx(x, 89)] = Tile::Sidewalk;
        }

        // 10) Zebrastreifen — auf Road-Tiles platzieren
        for &(zx, zy) in ZEBRA_TILES.iter() {
            if zx >= 0 && zy >= 0 && zx < w && zy < h {
                if matches!(tiles[idx(zx, zy)], Tile::Road | Tile::RedThread) {
                    tiles[idx(zx, zy)] = Tile::Zebra;
                }
                // Sidewalks links/rechts klar markieren (bleibt Sidewalk)
            }
        }

        World { tiles, w, h, areas, buildings }
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
