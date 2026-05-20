//! Alle textbasierten Konstanten für Germering Quest:
//! - Ortsnamen, Adressen
//! - Lore-Texte
//! - Shop-Produktdaten
//! - Game-Balance-Werte
//!
//! Alle Adressen entsprechen echten Standorten in Germering, Lkr. Fürstenfeldbruck.

// --------------------------------------------------------------------
//  Auflösung & Render
// --------------------------------------------------------------------

/// Virtuelle Render-Auflösung - wird letterboxed auf den echten Bildschirm
/// skaliert, damit das Spiel auf 16:9, 16:10, 21:9 und Ultrawide gleich aussieht.
pub const VIRTUAL_W: i32 = 480;
pub const VIRTUAL_H: i32 = 270;

pub const TILE_SIZE: f32 = 16.0;
pub const MAP_W: i32 = 200;
pub const MAP_H: i32 = 120;

// --------------------------------------------------------------------
//  Welt / Day-Night
// --------------------------------------------------------------------

/// Ein voller Spieltag = 8 Minuten Echtzeit (480 s).
pub const DAY_LENGTH_SECONDS: f32 = 480.0;

// --------------------------------------------------------------------
//  Player-Defaults
// --------------------------------------------------------------------

pub const PLAYER_START_HEARTS: i32 = 3;
pub const PLAYER_MAX_HEARTS_INIT: i32 = 10;
pub const PLAYER_MAX_HEARTS_CAP: i32 = 12;

pub const PLAYER_SPEED: f32 = 80.0;
pub const PLAYER_SWIM_SPEED: f32 = 40.0;
pub const PLAYER_ICE_FRICTION: f32 = 0.03;
pub const PLAYER_NORMAL_FRICTION: f32 = 0.8;

// --------------------------------------------------------------------
//  Ortsnamen - die 16 echten Germeringer Gebiete
// --------------------------------------------------------------------

pub const AREA_STADTMITTE: &str = "Neugermering / Stadtmitte";
pub const AREA_FRIEDENSTR: &str = "Friedenstraße";
pub const AREA_BAHNHOF: &str = "Bahnhof Germering (S8)";
pub const AREA_HARTHAUS: &str = "Bahnhof Harthaus (S8)";
pub const AREA_RATHAUS: &str = "Rathausplatz";
pub const AREA_SCHULE: &str = "Schule am Kirchenplatz";
pub const AREA_KRANKENHAUS: &str = "Klinikum Germering";
pub const AREA_GEP: &str = "GEP - Germering Einkaufspassagen";
pub const AREA_CEWESTR: &str = "Cewestraße / Gewerbegebiet";
pub const AREA_GERMSEE: &str = "Germeringer See (Baggersee)";
pub const AREA_PARSBERG: &str = "Parsberg";
pub const AREA_FORST: &str = "Kreuzlinger Forst";
pub const AREA_POLARIOM: &str = "Polariom Eissporthalle";
pub const AREA_STADTHALLE: &str = "Stadthalle Germering";
pub const AREA_MUSEUM: &str = "Stadtmuseum ZEIT+RAUM";
pub const AREA_FREIBAD: &str = "Freibad Germering";
pub const AREA_CORDOBAR: &str = "Cordobar-Ruine";
pub const AREA_WWK: &str = "WWK-Hochhaussiedlung";
pub const AREA_STADTPARK: &str = "Stadtpark";
pub const AREA_KAUFHOF: &str = "Alter Kaufhof - Endboss-Dungeon";

// --------------------------------------------------------------------
//  Die 4 echten Ihle-Filialen in Germering
// --------------------------------------------------------------------

pub struct Filiale {
    pub nummer: u8,
    pub name: &'static str,
    pub adresse: &'static str,
    pub besonderheit: &'static str,
    /// Position in Tile-Koordinaten auf der Map (Mitte des Gebäudes).
    pub tile_x: i32,
    pub tile_y: i32,
    /// Discount (0.0-1.0) der zusätzlich zum Genusspass gilt.
    pub discount_morgen: f32,
}

pub const FILIALEN: [Filiale; 4] = [
    Filiale {
        nummer: 1,
        name: "Ihle Hauptfiliale",
        adresse: "Friedenstr. 19a",
        besonderheit: "Vollsortiment - Meister Ihle gibt hier Quests",
        tile_x: 100,
        tile_y: 70,
        discount_morgen: 0.0,
    },
    Filiale {
        nummer: 2,
        name: "Ihle im GEP",
        adresse: "Münchner Straße 1",
        besonderheit: "Café-Atmosphäre - Sitzplätze heilen 1 Herz",
        tile_x: 45,
        tile_y: 67,
        discount_morgen: 0.0,
    },
    Filiale {
        nummer: 3,
        name: "Ihle am S-Bahnhof",
        adresse: "Untere Bahnhofstr. 42",
        besonderheit: "Frühaufsteher: -20% vor 7 Uhr",
        tile_x: 118,
        tile_y: 26,
        discount_morgen: 0.20,
    },
    Filiale {
        nummer: 4,
        name: "Ihle Gewerbegebiet",
        adresse: "Cewestraße 1",
        besonderheit: "Versteckte Menüpunkte - billigste Preise",
        tile_x: 168,
        tile_y: 28,
        discount_morgen: 0.0,
    },
];

// --------------------------------------------------------------------
//  Ihle Shop-Menü - echte Produkte
// --------------------------------------------------------------------

pub struct ShopItem {
    pub name: &'static str,
    pub preis: u32,
    pub effekt: &'static str,
    pub lore: &'static str,
    /// Nur in Filiale 4 (Cewestr.) verfügbar.
    pub legendary: bool,
}

pub const SHOP_ITEMS: [ShopItem; 8] = [
    ShopItem {
        name: "Ihle-Breze",
        preis: 8,
        effekt: "+1 Herz",
        lore: "Mit Reichenhaller Bergsalz - DER bayerische Snack.",
        legendary: false,
    },
    ShopItem {
        name: "Buttercroissant",
        preis: 15,
        effekt: "+1 Herz, 8s Speedboost",
        lore: "Französisch-bayrische Fusion.",
        legendary: false,
    },
    ShopItem {
        name: "Berliner",
        preis: 22,
        effekt: "+2 Herzen",
        lore: "Prall gefüllt mit Marmelade.",
        legendary: false,
    },
    ShopItem {
        name: "Bio-Bauernbrot",
        preis: 35,
        effekt: "+3 Herzen",
        lore: "Steinofen - aus regionalem Biogetreide.",
        legendary: false,
    },
    ShopItem {
        name: "Lechfelder",
        preis: 40,
        effekt: "Max-Herz +1 dauerhaft",
        lore: "Regionales Ihle-Original.",
        legendary: false,
    },
    ShopItem {
        name: "Schwabenkorn",
        preis: 55,
        effekt: "Max-Herz +1 + Doppelmünzen 30s",
        lore: "Brotspezialität aus dem Süden.",
        legendary: false,
    },
    ShopItem {
        name: "Bio-Kornwunder",
        preis: 70,
        effekt: "Voll heilen + 15s Unverwundbarkeit",
        lore: "Das pure Korn-Erlebnis.",
        legendary: false,
    },
    ShopItem {
        name: "MEGA-IHLE-BREZE",
        preis: 120,
        effekt: "+5 Herzen, 25s Fliegen, Doppelmünzen",
        lore: "Nur in Filiale 4 - Cewestraße.",
        legendary: true,
    },
];

// --------------------------------------------------------------------
//  Lore-Texte für Landmarks
// --------------------------------------------------------------------

pub const LORE_GERMARBRUNNEN: &str =
    "Germarbrunnen: Germar, der Krieger - er würde Ihle-Breze über alles stellen.\n\
     +1 Münze/Sek für 30 Sekunden.";

pub const LORE_JAKOBUSBRUNNEN: &str =
    "Jakobusbrunnen an der St. Jakobskirche: Du fühlst dich gesegnet. Volle Heilung!";

pub const LORE_MARIENSAEULE: &str =
    "Mariensäule (Augsburger Straße): 20 Sekunden Unverwundbarkeit.";

pub const LORE_ZIEGELOFEN: &str =
    "Römischer Ziegelbrennofen Richtung Alling - ein Geheimgang öffnet sich.";

pub const LORE_MUSEUM: &str =
    "Stadtmuseum ZEIT+RAUM: Du findest ein römisches Artefakt. +1 Angriff dauerhaft.";

pub const LORE_CORDOBAR: &str =
    "Hier haben die Sportfreunde Stiller angefangen - RIP Cordobar.\n\
     'Wellenreiten'-Chiptune freigeschaltet!";

pub const LORE_KIRCHE: &str =
    "St. Jakobskirche - Safe Zone. Orgelmusik wabert durch das Kirchenschiff.";

// --------------------------------------------------------------------
//  Story-Texte
// --------------------------------------------------------------------

pub const INTRO_TEXT: [&str; 8] = [
    "Germering, 2025. Früh morgens, 5:47 Uhr.",
    "Du bist Max Huber, 17, Bäckerlehrling bei Ihle.",
    "Friedenstr. 19a. Du sperrst auf.",
    "Die Backstube ist verwüstet. Schimmelklumpen.",
    "Der Goldene Brezel-Schlüssel ist weg.",
    "Auf der Notiz: 'Mir ghört Germering! - Die Schimmel-Gang'",
    "Schimmelmeister Modrý hat sich im alten Kaufhof verbarrikadiert.",
    "Servus, Max. Du musst Germering retten.",
];

pub const VICTORY_TEXT: [&str; 5] = [
    "Max hat den Goldenen Brezel-Schlüssel zurückgebracht!",
    "Schimmelmeister Modrý wurde besiegt.",
    "Germering ist wieder frei.",
    "Meister Ihle backt wieder - und zwar EXTRA ofenwarm.",
    "DANKE, dass du Germering gerettet hast.",
];

pub const CREDITS: [&str; 14] = [
    "GERMERING QUEST",
    "Die Schimmel-Invasion",
    "",
    "Echte Orte:",
    "Friedenstr. 19a - Ihle Hauptfiliale",
    "Münchner Str. 1 - GEP",
    "Untere Bahnhofstr. 42 - S-Bahnhof",
    "Cewestraße 1 - Gewerbegebiet",
    "Germeringer See, Parsberg, Kreuzlinger Forst",
    "Polariom, Stadthalle, Stadtmuseum ZEIT+RAUM",
    "",
    "Sechstgrößte Stadt Oberbayerns.",
    "Seit 859 n. Chr. als 'Kermaringon' bekannt.",
    "Servus & Pfiati!",
];

// --------------------------------------------------------------------
//  Boss
// --------------------------------------------------------------------

pub const BOSS_NAME: &str = "Schimmelmeister Modrý";
pub const BOSS_HP_MAX: i32 = 300;
pub const BOSS_PHASE2_HP: i32 = 180;
pub const BOSS_PHASE3_HP: i32 = 80;

pub const BOSS_TAUNT_P1: &str = "Mei, des is mei Germering!";
pub const BOSS_TAUNT_P2: &str = "Da rauf! Find den echten Modrý!";
pub const BOSS_TAUNT_P3: &str = "Schimmel über alles!";

// --------------------------------------------------------------------
//  Münzen-Werte
// --------------------------------------------------------------------

pub const COIN_COPPER: u32 = 1;
pub const COIN_SILVER: u32 = 5;
pub const COIN_GOLD: u32 = 20;
pub const COIN_BREZEL: u32 = 50;

pub const COIN_RESPAWN_SECONDS: f32 = 45.0;

// --------------------------------------------------------------------
//  Dungeon-Kristalle (Boss-Gate)
// --------------------------------------------------------------------

/// 4 Kristalle in 4 Dungeon-Areas. Tile-Position, Area-Name (für Bit-Index 0..3).
pub const CRYSTALS: [(i32, i32, &str); 4] = [
    (175, 87, "Polariom-Kristall"),
    (75,  100, "Forst-Kristall"),
    (25,  105, "Cordobar-Kristall"),
    (15,  60,  "Parsberg-Kristall"),
];

// --------------------------------------------------------------------
//  Öffnungszeiten der Ihle-Filialen (in Spiel-Stunden, 5–28 Uhr-Skala)
// --------------------------------------------------------------------

/// Filiale Nr. → (open_hour, close_hour). Filiale 3 (Bahnhof) öffnet früh,
/// alle anderen 8-20 Uhr. Wenn open_hour < close_hour: einfacher Bereich.
pub const FILIALE_OPEN_HOURS: [(u8, u8); 4] = [
    (8, 20),  // F1 Hauptfiliale
    (8, 20),  // F2 GEP
    (6, 22),  // F3 Bahnhof (Frühaufsteher)
    (8, 20),  // F4 Cewestraße
];

// --------------------------------------------------------------------
//  See-Schwimm-Loop — 4 Checkpoints im/am Wasser
// --------------------------------------------------------------------

/// Tile-Koordinaten der 4 Eckpunkte des Sees (im Wasser).
pub const LAKE_CHECKPOINTS: [(i32, i32); 4] = [
    (45, 25),  // Nordost-Ecke
    (45, 52),  // Südost-Ecke
    (10, 52),  // Südwest-Ecke
    (10, 25),  // Nordwest-Ecke
];

// --------------------------------------------------------------------
//  S-Bahn — zwei echte Germering-Stationen: Harthaus (West) + Germering (Ost)
// --------------------------------------------------------------------

/// Sekunden zwischen Zugdurchfahrten (Spielzeit).
pub const TRAIN_INTERVAL_SECONDS: f32 = 90.0;
/// Y-Tile-Koordinate der S8-Gleise.
pub const TRAIN_TRACK_TILE_Y: i32 = 14;
/// Y-Tile-Koordinate des Bahnsteigs (hier landen die Münzen).
pub const TRAIN_PLATFORM_TILE_Y: i32 = 17;
/// Westlicher Streckenanfang — vor dem Bahnhof Harthaus.
pub const TRAIN_TRACK_X_MIN: i32 = 55;
/// Östliches Ende.
pub const TRAIN_TRACK_X_MAX: i32 = 200;
/// Bahnhof Harthaus (West) Halt-Position in Tile-X.
pub const TRAIN_STATION_HARTHAUS_X: f32 = 68.0;
/// Bahnhof Germering (Ost) Halt-Position in Tile-X.
pub const TRAIN_STATION_GERMERING_X: f32 = 120.0;
/// Bahnsteig-X-Bereich für Harthaus (für Bahnsteig-Rendering und Ride-Erkennung).
pub const TRAIN_PLATFORM_HARTHAUS: (f32, f32) = (60.0, 78.0);
/// Bahnsteig-X-Bereich für Germering Bahnhof.
pub const TRAIN_PLATFORM_GERMERING: (f32, f32) = (112.0, 132.0);
/// Sekunden Halt am Bahnhof.
pub const TRAIN_STATION_DWELL: f32 = 5.0;
/// Kosten in Münzen für eine S-Bahn-Fahrt zwischen den 2 Stationen.
pub const SBAHN_RIDE_COST: u32 = 5;

// --------------------------------------------------------------------
//  Busse — echte Germering-Linien (851 + lokale Linien)
// --------------------------------------------------------------------

pub struct BusLine {
    pub number: &'static str,
    pub name: &'static str,
}

/// Echte Buslinien rund um Germering (MVV + lokale).
pub const BUS_LINES: [BusLine; 5] = [
    BusLine { number: "851", name: "Germering Bf - Puchheim" },
    BusLine { number: "843", name: "Germering - Gilching" },
    BusLine { number: "844", name: "Germering - Olching" },
    BusLine { number: "X910", name: "Express Stadtmitte" },
    BusLine { number: "732", name: "Schulbus Germering" },
];

/// Schaden, wenn ein Bus den Spieler überfährt.
pub const BUS_DAMAGE: i32 = 1;

/// Kosten in Münzen für eine Mitfahrt im Bus (Einstieg per [E]).
pub const BUS_RIDE_COST: u32 = 10;

// --------------------------------------------------------------------
//  Auto-Verkehr (NPC-Pkw)
// --------------------------------------------------------------------

/// Schaden, wenn ein Auto den Spieler trifft.
pub const CAR_DAMAGE: i32 = 1;
/// Maximale gleichzeitig auf der Karte herumfahrende Autos.
pub const CAR_MAX: usize = 18;

// --------------------------------------------------------------------
//  Ampeln & Zebrastreifen — Innerortsverkehr
// --------------------------------------------------------------------

/// Tile-Koordinaten der Ampel-Kreuzungen (X, Y) — Mitte der Kreuzung.
/// Fahrzeuge auf der zugehörigen h_road halten bei Rot ca. 1 Tile vor dem Light.
pub const TRAFFIC_LIGHT_TILES: [(i32, i32); 6] = [
    (75, 40),   // GEP-Kreuzung
    (105, 40),  // Stadtmitte Nord
    (130, 40),  // Stadthalle/Stadtmuseum
    (75, 60),   // Stadtmitte West
    (105, 60),  // Stadtmitte Ost
    (160, 60),  // Cewestr Übergang
];

/// Tile-Koordinaten der Zebrastreifen (X, Y) auf horizontalen Straßen.
/// Fahrzeuge stoppen, wenn der Spieler innerhalb von ~2 Tiles ist.
pub const ZEBRA_TILES: [(i32, i32); 12] = [
    (50, 20),   // Nord Wohngebiet
    (90, 20),   // Bahnhof Süd
    (140, 20),  // Bahnhof Ost
    (40, 40),   // GEP
    (90, 40),   // Stadtmitte Nord
    (120, 40),  // Museum
    (165, 40),  // Cewestr
    (50, 60),   // Parsberg
    (90, 60),   // Stadtmitte West
    (120, 60),  // Stadtmitte Ost
    (90, 80),   // Friedenstr Süd
    (140, 80),  // Polariom Nord
];

/// Sekunden zwischen zufälligen Stau-Ereignissen.
pub const TRAFFIC_JAM_INTERVAL: f32 = 75.0;
/// Sekunden, die ein Stau anhält.
pub const TRAFFIC_JAM_DURATION: f32 = 18.0;
