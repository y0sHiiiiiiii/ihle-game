//! NPCs mit echtem Germering-Bezug.

use crate::collision::Aabb;
use crate::consts::TILE_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcKind {
    MeisterIhle,
    Klaus,
    Gerhard,
    OmaLiesl,
    Franz,
}

pub struct Npc {
    pub kind: NpcKind,
    pub aabb: Aabb,
    pub name: &'static str,
    pub dialog: Vec<&'static str>,
    pub talked: bool,
}

impl Npc {
    pub fn meister_ihle() -> Self {
        Self {
            kind: NpcKind::MeisterIhle,
            aabb: Aabb::new(100.0 * TILE_SIZE - 8.0, 72.0 * TILE_SIZE, 14.0, 14.0),
            name: "Meister Ihle",
            dialog: vec![
                "Servus Max! Schee dass du da bist.",
                "Der Goldene Brezel-Schlüssel - weg!",
                "Er liegt im alten Kaufhof am Stadtplatz.",
                "Du musst den Schimmelmeister Modrý besiegen!",
                "Hol dir Kraft in unseren Filialen.",
                "Sammle Münzen - und vergiss den Genusspass nicht.",
                "Nach 10 Käufen kriegst du -15% auf alles.",
                "Pfiati und viel Erfolg, mei Bua.",
            ],
            talked: false,
        }
    }

    pub fn klaus() -> Self {
        Self {
            kind: NpcKind::Klaus,
            aabb: Aabb::new(105.0 * TILE_SIZE, 51.0 * TILE_SIZE, 14.0, 14.0),
            name: "Stadtführer Klaus",
            dialog: vec![
                "Grüß Gott! Klaus, Stadtführer.",
                "Germering - sechstgrößte Stadt Oberbayerns.",
                "Erstmals 859 als 'Kermaringon' erwähnt.",
                "Folge dem Roten Faden auf dem Boden!",
                "Er führt dich zu allen 4 Ihle-Filialen.",
                "TAB öffnet die Minimap - siehst alles.",
                "Servus & viel Erfolg gegen den Schimmel!",
            ],
            talked: false,
        }
    }

    pub fn gerhard() -> Self {
        Self {
            kind: NpcKind::Gerhard,
            aabb: Aabb::new(114.0 * TILE_SIZE, 18.0 * TILE_SIZE, 14.0, 14.0),
            name: "Gerhard, S8-Fahrer",
            dialog: vec![
                "Servus! Gerhard, S8 nach München.",
                "Pass auf den Münzregen am Bahnsteig auf!",
                "Wenn der Zug einfährt, Silbermünzen!",
                "Untere Bahnhofstr. 42 - Ihle hat schon ab 6 Uhr auf.",
                "Frühaufsteher: -20% vor 7 Uhr.",
            ],
            talked: false,
        }
    }

    pub fn oma_liesl() -> Self {
        Self {
            kind: NpcKind::OmaLiesl,
            aabb: Aabb::new(112.0 * TILE_SIZE, 38.0 * TILE_SIZE, 14.0, 14.0),
            name: "Oma Liesl",
            dialog: vec![
                "Pssscht! Hier, gute Kuchen, billig!",
                "Halber Preis! Aber sag's keinem.",
                "Sag halt nur nicht, wenn's grummelt im Bauch.",
                "(Schwarzmarkt - Vorsicht!)",
            ],
            talked: false,
        }
    }

    pub fn franz() -> Self {
        Self {
            kind: NpcKind::Franz,
            aabb: Aabb::new(22.0 * TILE_SIZE, 100.0 * TILE_SIZE, 14.0, 14.0),
            name: "Franz, Stiller-Fan",
            dialog: vec![
                "Heyyyy! Cordobar! Hier hat alles angefangen!",
                "Sportfreunde Stiller - die Jungs!",
                "'Wellenreiten' kennst du, oder?",
                "Hier, ich schalt dir den Chiptune frei!",
                "Schluck Bier auf die Cordobar. RIP.",
            ],
            talked: false,
        }
    }

    /// Trefferzone für Interaktion.
    pub fn interact_zone(&self) -> Aabb {
        Aabb::new(self.aabb.x - 8.0, self.aabb.y - 8.0, self.aabb.w + 16.0, self.aabb.h + 16.0)
    }
}

pub fn spawn_all_npcs() -> Vec<Npc> {
    vec![
        Npc::meister_ihle(),
        Npc::klaus(),
        Npc::gerhard(),
        Npc::oma_liesl(),
        Npc::franz(),
    ]
}

// ----------------------------------------------------------------
//  Dialog-State (Typewriter)
// ----------------------------------------------------------------

pub struct DialogState {
    pub npc_idx: usize,
    pub line: usize,
    pub char_count: f32,
    pub typing: bool,
}

impl DialogState {
    pub fn new(npc_idx: usize) -> Self {
        Self {
            npc_idx,
            line: 0,
            char_count: 0.0,
            typing: true,
        }
    }

    pub fn tick(&mut self, dt: f32, full_len: usize) {
        if self.typing {
            self.char_count += dt * 50.0; // ~50 Zeichen/Sek
            if self.char_count as usize >= full_len {
                self.char_count = full_len as f32;
                self.typing = false;
            }
        }
    }

    /// Bei Druck → entweder vollständig anzeigen oder nächste Zeile.
    pub fn advance(&mut self, total_lines: usize) -> bool {
        if self.typing {
            self.typing = false;
            self.char_count = 1_000_000.0;
            false
        } else {
            self.line += 1;
            if self.line >= total_lines {
                return true; // Dialog beenden
            }
            self.char_count = 0.0;
            self.typing = true;
            false
        }
    }
}
