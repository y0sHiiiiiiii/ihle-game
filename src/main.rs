//! Germering Quest — Die Schimmel-Invasion
//!
//! Hauptmodul: Fullscreen-Konfiguration, Viewport-Skalierung (480×270
//! virtuell, letterboxed), Game-State-Machine, Hauptschleife.

mod audio;
mod boss;
mod collision;
mod consts;
mod entities;
mod npc;
mod player;
mod save;
mod shop;
mod sprites;
mod ui;
mod world;

use audio::Audio;
use boss::{Boss, BossPhase};
use collision::{move_aabb, Aabb};
use consts::*;
use entities::{
    spawn_world_entities, Coin, CoinKind, Enemy, EnemyKind, EnemyState, FloatingText, IcePatch,
    Projectile,
};
use macroquad::prelude::*;
use npc::{spawn_all_npcs, DialogState, Npc};
use player::{Facing, Player};
use save::SaveData;
use shop::{filiale_at, filiale_is_open, ShopState};
use ui::Textures;
use world::{time_of_day, GameCamera, TimeOfDay, World};

// ------------------------------------------------------------------------
//  Window-Konfiguration — Fullscreen 1920×1080
// ------------------------------------------------------------------------

fn window_conf() -> Conf {
    Conf {
        window_title: "Germering Quest".to_owned(),
        fullscreen: true,
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        sample_count: 0,
        ..Default::default()
    }
}

// ------------------------------------------------------------------------
//  Game-State
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    TitleMenu,
    Intro,
    Playing,
    Paused,
    Shopping,
    Dialog,
    GameOver,
    Victory,
    Credits,
}

struct Game {
    state: GameState,
    world: World,
    player: Player,
    coins: Vec<Coin>,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    ice_patches: Vec<IcePatch>,
    slow_tiles: Vec<IcePatch>, // wir nutzen IcePatch-Struct generisch für Slow-Spuren
    npcs: Vec<Npc>,
    boss: Option<Boss>,
    cam: GameCamera,
    audio: Audio,
    tex: Textures,
    game_seconds: f32,
    coins_collected: u32,
    purchases: u32,
    has_genusspass: bool,
    cordobar_unlocked: bool,
    roman_artifact: bool,
    boss_defeated: bool,
    checkpoint_filiale: usize,
    minimap_open: bool,
    area_name: String,
    area_fade: f32,
    last_area: String,
    shop: Option<ShopState>,
    dialog: Option<DialogState>,
    menu_cursor: usize,
    intro_scroll: f32,
    victory_scroll: f32,
    attack_anim_t: f32,
    interact_landmark_cooldown: f32,
    boss_intro_seen: bool,
    germarbrunnen_active: f32, // Restzeit Münze/Sek
    germarbrunnen_acc: f32,
    save_message_t: f32,
    save_message: String,
    // --- Dungeon-Kristalle (Bitmask: bit0..3 für die 4 Kristalle) ---
    crystals: u8,
    // --- S-Bahn ---
    train_t: f32,           // Sekunden bis zur nächsten Durchfahrt
    train_phase: TrainPhase,
    train_x: f32,           // Welt-X-Position der Lok
    train_dwell_t: f32,     // Restzeit Halt am Bahnhof
    train_dropped: bool,
    train_dir: TrainDir,
    train_next_stop: usize, // 0 = Harthaus, 1 = Germering
    train_remaining_stops: u8, // Bitmaske der noch anzufahrenden Stationen in dieser Tour
    // S-Bahn-Fahrt-Anim (kurzer Übergang beim "Reinhüpfen" in die Bahn)
    sbahn_ride_t: f32,
    sbahn_ride_from: usize,
    sbahn_ride_to: usize,
    // --- Busse ---
    buses: Vec<Bus>,
    bus_spawn_t: f32,
    bus_hit_cooldown: f32,
    bus_next_uid: u32,
    bus_uids: Vec<u32>,         // parallel zu self.buses (UID je Bus)
    riding_bus_uid: Option<u32>,// in welchem Bus sitzt der Spieler
    bus_ride_msg_t: f32,
    bus_ride_msg: String,
    // --- Pkw ---
    cars: Vec<Car>,
    car_spawn_t: f32,
    // --- Ampeln / Stau ---
    traffic_phase_t: f32,        // 0..LIGHT_CYCLE_S Sekunden
    traffic_jam_t: f32,          // Restzeit aktiver Stau
    traffic_jam_road_y: f32,     // welcher h_road (Welt-Y) ist gestaut
    traffic_jam_cooldown: f32,   // bis zum nächsten Stau
    // --- Wandernde NPCs (Bürger + Säufer) ---
    pedestrians: Vec<Pedestrian>,
    // --- Pause-Menü-Cursor ---
    pause_cursor: usize,
    // --- WWK-Sprung ---
    jump_t: f32,        // Restzeit aktiver Sprung
    // --- See-Schwimmstrecke ---
    lake_visited: u8,   // Bitmask der besuchten Checkpoints
    lake_swim_done: bool,
    // --- Quest-Stage ---
    quest_stage: u8,
    quest_hint_t: f32,
    quest_hint: String,
    // --- Klaus-Tour ---
    klaus_tour_done: bool,
    klaus_tour_t: f32,
    klaus_started_walking: bool,
    // --- Floating Damage- / Coin-Texte ---
    floating_texts: Vec<FloatingText>,
}

// ------------------------------------------------------------------------
//  S-Bahn-Phasen
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrainPhase {
    Idle,       // Wartet, nicht sichtbar
    Entering,   // Fährt von links Richtung Bahnhof
    Dwelling,   // Steht am Bahnsteig
    Leaving,    // Fährt vom Bahnhof nach rechts ab
}

// ------------------------------------------------------------------------
//  Busse — fahren auf horizontalen Straßen
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum BusDir { East, West }

#[derive(Clone)]
struct Bus {
    line_idx: usize,    // Index in BUS_LINES
    x: f32,
    y: f32,             // Welt-Y (auf einer h_road)
    dir: BusDir,
    speed: f32,
    swear_t: f32,       // Anzeigedauer der Schimpf-Sprechblase
    swear_text: String,
    honk_t: f32,        // Hupgeräusch-Indikator
}

impl Bus {
    fn aabb(&self) -> Aabb {
        Aabb::new(self.x + 1.0, self.y + 1.0, 28.0, 14.0)
    }
}

// ------------------------------------------------------------------------
//  Pkw — fahren auf horizontalen Straßen, halten an Ampeln & Zebras
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum CarDir { East, West }

#[derive(Clone)]
struct Car {
    x: f32,
    y: f32,
    dir: CarDir,
    speed: f32,         // Wunschgeschwindigkeit
    current_v: f32,     // tatsächliche Geschwindigkeit (für Stopp/Anfahren)
    body: Color,        // Karosseriefarbe
    honk_t: f32,
}

impl Car {
    fn aabb(&self) -> Aabb {
        Aabb::new(self.x + 1.0, self.y + 2.0, 18.0, 10.0)
    }
}

// ------------------------------------------------------------------------
//  S-Bahn — Richtung + aktueller Haltepunkt
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrainDir {
    East,
    West,
}

/// Liefert die Bahnsteig-Position eines Bahnhofs in Welt-X.
fn station_x(idx: usize) -> f32 {
    match idx {
        0 => TRAIN_STATION_HARTHAUS_X * TILE_SIZE,
        _ => TRAIN_STATION_GERMERING_X * TILE_SIZE,
    }
}

/// Stationsname.
fn station_name(idx: usize) -> &'static str {
    match idx {
        0 => "Harthaus",
        _ => "Germering",
    }
}

/// Bahnsteig-X-Bereich (Welt-Koordinaten).
fn station_platform(idx: usize) -> (f32, f32) {
    let (a, b) = match idx {
        0 => TRAIN_PLATFORM_HARTHAUS,
        _ => TRAIN_PLATFORM_GERMERING,
    };
    (a * TILE_SIZE, b * TILE_SIZE)
}

// ------------------------------------------------------------------------
//  Ampel-Logik (rot-für-Horizontalverkehr-Fenster im 14s-Zyklus)
// ------------------------------------------------------------------------

/// 0..6 grün, 6..7 gelb, 7..13 rot, 13..14 gelb (zurück nach grün).
fn light_is_red_for_horizontal(t: f32) -> bool {
    let p = t.rem_euclid(14.0);
    p >= 7.0 && p < 13.0
}

/// Hilfsfunktion für Fahrzeuge: sollte das Fahrzeug an Position (x,y) und
/// Fahrtrichtung in Kürze halten? (Ampel rot, aktiver Zebrastreifen).
///
/// `length` = Fahrzeuglänge (Vorderkante = x + length wenn East, x wenn West)
fn vehicle_should_stop(
    x: f32,
    y: f32,
    dir_east: bool,
    length: f32,
    traffic_t: f32,
    active_zebras: &[(f32, f32)],
) -> bool {
    let front_x = if dir_east { x + length } else { x };
    let look_ahead = 28.0;

    // 1) Ampeln
    if light_is_red_for_horizontal(traffic_t) {
        for &(lx, ly) in TRAFFIC_LIGHT_TILES.iter() {
            let lwy = ly as f32 * TILE_SIZE;
            if (lwy - y).abs() > 12.0 { continue; }
            let lwx = lx as f32 * TILE_SIZE + 8.0;
            let dist_ahead = if dir_east { lwx - front_x } else { front_x - lwx };
            if dist_ahead > -4.0 && dist_ahead < look_ahead {
                return true;
            }
        }
    }

    // 2) Zebrastreifen (Spieler bereits in der Nähe → aktiv)
    for &(zwx, zwy) in active_zebras {
        if (zwy - (y + 6.0)).abs() > 16.0 { continue; }
        let dist_ahead = if dir_east { zwx - front_x } else { front_x - zwx };
        if dist_ahead > -4.0 && dist_ahead < look_ahead {
            return true;
        }
    }

    false
}

// ------------------------------------------------------------------------
//  Wandernde Bürger + Säufer
// ------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PedKind {
    /// Freundlicher Bürger — wandert herum.
    Citizen,
    /// Tagsüber besoffener Bayer — versucht den Spieler zu hauen.
    Drunk,
}

#[derive(Clone)]
struct Pedestrian {
    kind: PedKind,
    aabb: Aabb,
    vx: f32,
    vy: f32,
    /// Aktueller Bewegungs-Timer; bei 0 wird neue Richtung gewählt.
    wander_t: f32,
    /// Cooldown für Berührungs-Schaden (nur Drunk).
    hit_cool: f32,
    /// Phase für Sprite-Wackeln.
    phase: f32,
    /// Anzeigte Schimpfwort-Restzeit.
    bubble_t: f32,
    bubble_text: String,
    /// Optionales Outfit-Tönung (für Vielfalt).
    tint: Color,
}

// ------------------------------------------------------------------------
//  Build helpers
// ------------------------------------------------------------------------

async fn build_textures() -> Textures {
    Textures {
        heart_full: sprites::make_texture(&sprites::HEART_FULL),
        heart_empty: sprites::make_texture(&sprites::HEART_EMPTY),
        coin_copper: sprites::make_texture(&sprites::COIN_COPPER),
        coin_silver: sprites::make_texture(&sprites::COIN_SILVER),
        coin_gold: sprites::make_texture(&sprites::COIN_GOLD),
        coin_brezel: sprites::make_texture(&sprites::COIN_BREZEL),
        player_down_a: sprites::make_texture(&sprites::PLAYER_DOWN_A),
        player_down_b: sprites::make_texture(&sprites::PLAYER_DOWN_B),
        player_up: sprites::make_texture(&sprites::PLAYER_UP_A),
        player_up_b: sprites::make_texture(&sprites::PLAYER_UP_B),
        player_left: sprites::make_texture(&sprites::PLAYER_LEFT_A),
        player_left_b: sprites::make_texture(&sprites::PLAYER_LEFT_B),
        player_right: sprites::make_texture(&sprites::PLAYER_RIGHT_A),
        player_right_b: sprites::make_texture(&sprites::PLAYER_RIGHT_B),
        player_swim: sprites::make_texture(&sprites::PLAYER_SWIM),
        enemy_mold: sprites::make_texture(&sprites::ENEMY_MOLD),
        enemy_blob: sprites::make_texture(&sprites::ENEMY_BLOB),
        enemy_rat: sprites::make_texture(&sprites::ENEMY_RAT),
        enemy_beat: sprites::make_texture(&sprites::ENEMY_BEAT),
        enemy_ice: sprites::make_texture(&sprites::ENEMY_ICE),
        boss: sprites::make_texture(&sprites::BOSS_MODRY),
        projectile: sprites::make_texture(&sprites::PROJECTILE_SPORE),
        npc_ihle: sprites::make_texture(&sprites::NPC_IHLE),
        npc_klaus: sprites::make_texture(&sprites::NPC_KLAUS),
        npc_gerhard: sprites::make_texture(&sprites::NPC_GERHARD),
        npc_oma: sprites::make_texture(&sprites::NPC_OMA),
        npc_franz: sprites::make_texture(&sprites::NPC_FRANZ),
        tile_grass: sprites::make_texture(&sprites::TILE_GRASS),
        tile_dirt: sprites::make_texture(&sprites::TILE_DIRT),
        tile_road: sprites::make_texture(&sprites::TILE_ROAD),
        tile_sidewalk: sprites::make_texture(&sprites::TILE_SIDEWALK),
        tile_redthread: sprites::make_texture(&sprites::TILE_REDTHREAD),
        tile_water: sprites::make_texture(&sprites::TILE_WATER),
        tile_sand: sprites::make_texture(&sprites::TILE_SAND),
        tile_ice: sprites::make_texture(&sprites::TILE_ICE),
        tile_forest: sprites::make_texture(&sprites::TILE_FOREST),
        tile_building: sprites::make_texture(&sprites::TILE_BUILDING),
        tile_ihle: sprites::make_texture(&sprites::TILE_IHLE_WALL),
        tile_bossfloor: sprites::make_texture(&sprites::TILE_BOSSFLOOR),
        tile_moldslow: sprites::make_texture(&sprites::TILE_MOLD_SLOW),
        tile_brunnen: sprites::make_texture(&sprites::TILE_BRUNNEN),
        tile_zebra: sprites::make_texture(&sprites::TILE_ZEBRA),
        crystal: sprites::make_texture(&sprites::CRYSTAL),
    }
}

impl Game {
    fn new(tex: Textures, audio: Audio) -> Self {
        let world = World::new();
        let (coins, enemies) = spawn_world_entities(&world);
        let player = Player::new(100.0 * TILE_SIZE, 72.0 * TILE_SIZE);
        Self {
            state: GameState::TitleMenu,
            world,
            player,
            coins,
            enemies,
            projectiles: Vec::new(),
            ice_patches: Vec::new(),
            slow_tiles: Vec::new(),
            npcs: spawn_all_npcs(),
            boss: None,
            cam: GameCamera::new(),
            audio,
            tex,
            game_seconds: 0.0,
            coins_collected: 0,
            purchases: 0,
            has_genusspass: false,
            cordobar_unlocked: false,
            roman_artifact: false,
            boss_defeated: false,
            checkpoint_filiale: 0,
            minimap_open: false,
            area_name: String::new(),
            area_fade: 0.0,
            last_area: String::new(),
            shop: None,
            dialog: None,
            menu_cursor: 0,
            intro_scroll: 0.0,
            victory_scroll: 0.0,
            attack_anim_t: 0.0,
            interact_landmark_cooldown: 0.0,
            boss_intro_seen: false,
            germarbrunnen_active: 0.0,
            germarbrunnen_acc: 0.0,
            save_message_t: 0.0,
            save_message: String::new(),
            crystals: 0,
            train_t: 8.0, // erste Durchfahrt nach 8s
            train_phase: TrainPhase::Idle,
            train_x: 0.0,
            train_dwell_t: 0.0,
            train_dropped: false,
            train_dir: TrainDir::East,
            train_next_stop: 0,
            train_remaining_stops: 0b11,
            sbahn_ride_t: 0.0,
            sbahn_ride_from: 0,
            sbahn_ride_to: 1,
            buses: Vec::new(),
            bus_spawn_t: 5.0,
            bus_hit_cooldown: 0.0,
            bus_next_uid: 1,
            bus_uids: Vec::new(),
            riding_bus_uid: None,
            bus_ride_msg_t: 0.0,
            bus_ride_msg: String::new(),
            cars: Vec::new(),
            car_spawn_t: 3.0,
            traffic_phase_t: 0.0,
            traffic_jam_t: 0.0,
            traffic_jam_road_y: 0.0,
            traffic_jam_cooldown: TRAFFIC_JAM_INTERVAL,
            pedestrians: spawn_pedestrians(),
            pause_cursor: 0,
            jump_t: 0.0,
            lake_visited: 0,
            lake_swim_done: false,
            quest_stage: 0,
            quest_hint_t: 0.0,
            quest_hint: String::new(),
            klaus_tour_done: false,
            klaus_tour_t: 0.0,
            klaus_started_walking: false,
            floating_texts: Vec::new(),
        }
    }

    fn reset_for_new_game(&mut self) {
        let (coins, enemies) = spawn_world_entities(&self.world);
        self.coins = coins;
        self.enemies = enemies;
        self.projectiles.clear();
        self.ice_patches.clear();
        self.slow_tiles.clear();
        self.npcs = spawn_all_npcs();
        self.pedestrians = spawn_pedestrians();
        self.buses.clear();
        self.bus_spawn_t = 5.0;
        self.bus_hit_cooldown = 0.0;
        self.pause_cursor = 0;
        self.boss = None;
        self.player = Player::new(100.0 * TILE_SIZE, 72.0 * TILE_SIZE);
        self.game_seconds = 0.0;
        self.purchases = 0;
        self.cordobar_unlocked = false;
        self.roman_artifact = false;
        self.boss_defeated = false;
        self.checkpoint_filiale = 0;
        self.intro_scroll = 0.0;
        self.victory_scroll = 0.0;
        self.boss_intro_seen = false;
        self.crystals = 0;
        self.train_t = 8.0;
        self.train_phase = TrainPhase::Idle;
        self.train_x = 0.0;
        self.train_dwell_t = 0.0;
        self.train_dropped = false;
        self.train_dir = TrainDir::East;
        self.train_next_stop = 0;
        self.train_remaining_stops = 0b11;
        self.sbahn_ride_t = 0.0;
        self.cars.clear();
        self.car_spawn_t = 3.0;
        self.bus_uids.clear();
        self.bus_next_uid = 1;
        self.riding_bus_uid = None;
        self.bus_ride_msg.clear();
        self.bus_ride_msg_t = 0.0;
        self.traffic_phase_t = 0.0;
        self.traffic_jam_t = 0.0;
        self.traffic_jam_cooldown = TRAFFIC_JAM_INTERVAL;
        self.jump_t = 0.0;
        self.lake_visited = 0;
        self.lake_swim_done = false;
        self.quest_stage = 0;
        self.quest_hint_t = 0.0;
        self.quest_hint.clear();
        self.klaus_tour_done = false;
        self.klaus_tour_t = 0.0;
        self.klaus_started_walking = false;
        self.floating_texts.clear();
    }

    fn load_from_save(&mut self, d: SaveData) {
        self.player.coins = d.coins;
        self.player.hearts = d.hearts.max(1);
        self.player.max_hearts = d.max_hearts;
        self.player.aabb.x = d.player_x;
        self.player.aabb.y = d.player_y;
        self.purchases = d.purchases;
        self.checkpoint_filiale = d.checkpoint_filiale.min(FILIALEN.len() - 1);
        self.roman_artifact = d.roman_artifact;
        self.cordobar_unlocked = d.cordobar_unlocked;
        self.boss_defeated = d.boss_defeated;
        self.game_seconds = d.game_seconds;
        self.crystals = d.crystals;
        self.lake_swim_done = d.lake_swim_done;
        self.quest_stage = d.quest_stage;
        self.klaus_tour_done = d.klaus_tour_done;
        if self.roman_artifact {
            self.player.powerups.attack_bonus = 1;
        }
    }

    fn to_save(&self) -> SaveData {
        SaveData {
            coins: self.player.coins,
            hearts: self.player.hearts,
            max_hearts: self.player.max_hearts,
            player_x: self.player.aabb.x,
            player_y: self.player.aabb.y,
            purchases: self.purchases,
            checkpoint_filiale: self.checkpoint_filiale,
            roman_artifact: self.roman_artifact,
            cordobar_unlocked: self.cordobar_unlocked,
            boss_defeated: self.boss_defeated,
            game_seconds: self.game_seconds,
            crystals: self.crystals,
            lake_swim_done: self.lake_swim_done,
            quest_stage: self.quest_stage,
            klaus_tour_done: self.klaus_tour_done,
        }
    }

    fn respawn_at_checkpoint(&mut self) {
        let f = &FILIALEN[self.checkpoint_filiale];
        self.player.aabb.x = f.tile_x as f32 * TILE_SIZE;
        self.player.aabb.y = (f.tile_y + 3) as f32 * TILE_SIZE;
        self.player.hearts = self.player.max_hearts;
        self.player.powerups = Default::default();
        self.player.swim_air = 10.0;
        self.player.vx = 0.0;
        self.player.vy = 0.0;
        self.state = GameState::Playing;
    }

    // --------------------------------------------------------------
    //  Update
    // --------------------------------------------------------------

    fn update_playing(&mut self, dt: f32) {
        self.game_seconds += dt;

        // Pause
        if is_key_pressed(KeyCode::P) {
            self.state = GameState::Paused;
            return;
        }
        // Tab → Minimap
        if is_key_pressed(KeyCode::Tab) {
            self.minimap_open = !self.minimap_open;
        }
        // Manual save
        if is_key_pressed(KeyCode::F5) {
            save::save(&self.to_save());
            self.save_message = "Gespeichert (save.dat)".to_string();
            self.save_message_t = 2.0;
        }

        self.save_message_t = (self.save_message_t - dt).max(0.0);
        self.interact_landmark_cooldown = (self.interact_landmark_cooldown - dt).max(0.0);
        self.attack_anim_t = (self.attack_anim_t - dt).max(0.0);
        self.quest_hint_t = (self.quest_hint_t - dt).max(0.0);
        // Klaus-Tour-Timer läuft nur, wenn er auch tatsächlich losgegangen ist.
        if self.klaus_started_walking {
            self.klaus_tour_t += dt;
        }

        // Ampeln + Stau-Events
        self.tick_traffic_lights(dt);
        self.tick_traffic_jam(dt);
        // S-Bahn-Zyklus
        self.tick_train(dt);
        // Aktive S-Bahn-Fahrt (Übergang/Teleport)
        self.tick_sbahn_ride(dt);
        // Busse
        self.tick_buses(dt);
        // NPC-Pkw
        self.tick_cars(dt);
        // Wandernde Bürger + Säufer
        self.tick_pedestrians(dt);
        // Toast für Bus-Steig-Meldung
        self.bus_ride_msg_t = (self.bus_ride_msg_t - dt).max(0.0);
        if self.bus_ride_msg_t == 0.0 { self.bus_ride_msg.clear(); }

        // Klaus wartet am Germarbrunnen, bis der Spieler nah genug ist.
        // Erst dann startet seine Stadtführungs-Tour. So findet man ihn IMMER.
        if !self.klaus_tour_done {
            let (kx, ky) = self.klaus_position();
            for n in self.npcs.iter_mut() {
                if n.kind == npc::NpcKind::Klaus {
                    n.aabb.x = kx;
                    n.aabb.y = ky;
                }
            }
        }

        // Germarbrunnen-Bonus
        if self.germarbrunnen_active > 0.0 {
            self.germarbrunnen_active -= dt;
            self.germarbrunnen_acc += dt;
            while self.germarbrunnen_acc >= 1.0 {
                self.germarbrunnen_acc -= 1.0;
                self.player.add_coins(1);
                self.audio.play_sfx(&self.audio.coin);
            }
        }

        // Music nach Tageszeit
        let tod = time_of_day(self.game_seconds);
        let in_boss = self.boss.as_ref().map(|b| b.alive()).unwrap_or(false);
        let in_cordobar = world::areas()
            .iter()
            .find(|a| a.name == AREA_CORDOBAR)
            .map(|a| a.contains_world(self.player.aabb.x, self.player.aabb.y))
            .unwrap_or(false);
        if in_boss {
            self.audio.play_music("boss", 0.45);
        } else if in_cordobar && self.cordobar_unlocked {
            self.audio.play_music("cordobar", 0.4);
        } else if tod == TimeOfDay::Nacht {
            self.audio.play_music("night", 0.3);
        } else {
            self.audio.play_music("day", 0.35);
        }

        // Player update — pausiert beim Mitfahren / während S-Bahn-Übergang
        if self.riding_bus_uid.is_none() && self.sbahn_ride_t <= 0.0 {
            self.player.update(&self.world, dt);
        } else {
            // Powerups laufen weiter, Input aber ignorieren
            self.player.powerups.tick(dt);
            self.player.attack_cooldown = (self.player.attack_cooldown - dt).max(0.0);
            self.player.damage_flash = (self.player.damage_flash - dt).max(0.0);
        }
        let (pcx, pcy) = self.player.center();
        self.cam.follow(pcx, pcy);
        self.cam.update_shake(dt, self.game_seconds);

        // Floating-Texts tick
        self.floating_texts.retain_mut(|f| f.update(dt));

        // Area-Erkennung
        let tx = (pcx / TILE_SIZE) as i32;
        let ty = (pcy / TILE_SIZE) as i32;
        if let Some(a) = self.world.area_at(tx, ty) {
            if a.name != self.last_area {
                self.area_name = a.name.to_string();
                self.area_fade = 2.5;
                self.last_area = a.name.to_string();

                // Boss spawnen — nur wenn alle 4 Kristalle gesammelt!
                if a.name == AREA_KAUFHOF && self.boss.is_none() && !self.boss_defeated {
                    let collected = self.crystals.count_ones();
                    if collected >= 4 {
                        self.boss = Some(Boss::new(
                            100.0 * TILE_SIZE - 22.0,
                            100.0 * TILE_SIZE - 22.0,
                        ));
                    } else {
                        self.area_name = format!(
                            "Verriegelt! Du brauchst 4 Kristalle ({}/4)",
                            collected
                        );
                        self.area_fade = 3.5;
                    }
                }
            }
        }
        self.area_fade = (self.area_fade - dt * 0.5).max(0.0);

        // Coins (mit Magnet + Floating-Pickup-Text)
        for c in self.coins.iter_mut() {
            c.update(dt);
            // Magnet: Münzen in 32px-Umkreis fliegen zum Spieler
            if c.alive {
                let dx = pcx - (c.x + 8.0);
                let dy = pcy - (c.y + 8.0);
                let d2 = dx * dx + dy * dy;
                if d2 < 32.0 * 32.0 && d2 > 8.0 * 8.0 {
                    let d = d2.sqrt().max(0.001);
                    let pull = 220.0;
                    c.x += (dx / d) * pull * dt;
                    c.y += (dy / d) * pull * dt;
                }
            }
            if c.alive && self.player.aabb.intersects(&c.aabb()) {
                let v = c.kind.value();
                let final_v = if self.player.powerups.double_coin > 0.0 { v * 2 } else { v };
                self.player.add_coins(v);
                self.coins_collected += v;
                c.alive = false;
                c.respawn_t = COIN_RESPAWN_SECONDS;
                self.audio.play_sfx(&self.audio.coin);
                self.floating_texts.push(FloatingText::coins(c.x + 4.0, c.y, final_v));
            }
        }

        // Enemies
        let mut new_ice_spawns: Vec<(f32, f32, f32)> = Vec::new();
        let mut enemy_drops: Vec<(f32, f32, CoinKind, i32)> = Vec::new();
        for e in self.enemies.iter_mut() {
            e.update(&self.world, &self.player, dt, &mut new_ice_spawns);
            if e.alive() && self.player.aabb.intersects(&e.aabb) {
                if self.player.powerups.invuln <= 0.0 && self.player.damage_flash <= 0.0 {
                    let dmg = e.damage();
                    self.player.take_damage(dmg);
                    self.audio.play_sfx(&self.audio.damage);
                    self.cam.add_shake(3.5);
                    self.floating_texts.push(FloatingText::damage(
                        self.player.aabb.x + 4.0,
                        self.player.aabb.y,
                        dmg,
                    ));
                }
            }
        }
        for (x, y, l) in new_ice_spawns {
            self.ice_patches.push(IcePatch::new(x, y, l));
        }

        // WWK-Sprung: in der WWK-Hochhaussiedlung wird Leertaste zum Sprung.
        let in_wwk = world::areas()
            .iter()
            .find(|a| a.name == AREA_WWK)
            .map(|a| a.contains_world(self.player.aabb.x, self.player.aabb.y))
            .unwrap_or(false);
        self.jump_t = (self.jump_t - dt).max(0.0);
        if in_wwk && is_key_pressed(KeyCode::Space) && self.jump_t <= 0.0 {
            self.jump_t = 0.35;
            // Kurzer Flugmodus überspringt Gebäude-Kollision während des Sprungs.
            self.player.powerups.fly = self.player.powerups.fly.max(0.35);
            let (vx, vy) = match self.player.facing {
                Facing::Down => (0.0, 240.0),
                Facing::Up => (0.0, -240.0),
                Facing::Left => (-240.0, 0.0),
                Facing::Right => (240.0, 0.0),
            };
            self.player.vx = vx;
            self.player.vy = vy;
            self.audio.play_sfx(&self.audio.menu_select);
        }

        // Player-Angriff (Leertaste, außer in WWK = Sprung)
        if !in_wwk && is_key_pressed(KeyCode::Space) && self.player.attack_cooldown <= 0.0 {
            self.player.attack_cooldown = 0.32;
            self.attack_anim_t = 0.22;
            // Forward-Lunge — kurzer Schub in Blickrichtung
            let lunge = 180.0;
            let (lx, ly) = match self.player.facing {
                Facing::Down => (0.0, lunge),
                Facing::Up => (0.0, -lunge),
                Facing::Left => (-lunge, 0.0),
                Facing::Right => (lunge, 0.0),
            };
            self.player.vx = lx;
            self.player.vy = ly;
            self.audio.play_sfx(&self.audio.menu_select);
            // Hitbox vor dem Spieler
            let (cx, cy) = self.player.center();
            let (hx, hy) = match self.player.facing {
                Facing::Down => (cx - 12.0, cy + 6.0),
                Facing::Up => (cx - 12.0, cy - 22.0),
                Facing::Left => (cx - 22.0, cy - 12.0),
                Facing::Right => (cx + 6.0, cy - 12.0),
            };
            let hit = Aabb::new(hx, hy, 24.0, 24.0);
            let damage = 1 + self.player.powerups.attack_bonus;
            // Treffer auf Gegner
            for e in self.enemies.iter_mut() {
                if e.alive() && e.aabb.intersects(&hit) {
                    e.hurt(damage);
                    e.apply_knockback(cx, cy, 220.0);
                    self.audio.play_sfx(&self.audio.enemy_kill);
                    self.floating_texts.push(FloatingText::damage(
                        e.aabb.x + 4.0,
                        e.aabb.y,
                        damage,
                    ));
                    self.cam.add_shake(2.0);
                    if !e.alive() {
                        // Drops queuen
                        for (k, n) in e.drop() {
                            for i in 0..n {
                                enemy_drops.push((e.aabb.x + i as f32 * 4.0, e.aabb.y, k, i));
                            }
                        }
                    }
                }
            }
            // Treffer auf Boss
            if let Some(b) = self.boss.as_mut() {
                if b.hurt(damage, hit) {
                    self.audio.play_sfx(&self.audio.boss_hit);
                    self.floating_texts.push(FloatingText::damage(
                        b.aabb.x + 22.0,
                        b.aabb.y,
                        damage,
                    ));
                    self.cam.add_shake(3.5);
                }
            }
            // Treffer auf Projektile (Sporen können zerstört werden)
            self.projectiles.retain(|p| !hit.intersects(&p.aabb()));
        }

        // Drops zu Münzen
        for (x, y, k, _) in enemy_drops {
            let mut c = Coin::new(x, y, k);
            c.respawn_t = 99999.0; // verschwinden nach Aufnahme — kein Respawn
            c.spawn_x = x;
            c.spawn_y = y;
            self.coins.push(c);
        }

        // Tote Gegner entfernen
        self.enemies.retain(|e| {
            // Wir entfernen sie verzögert nicht — sie respawnen einfach nicht.
            // Aus der Liste werfen, sobald Dead.
            e.state != EnemyState::Dead
        });

        // Projektile
        let mut player_hit = false;
        self.projectiles.retain_mut(|p| {
            let alive = p.update(dt);
            if alive && self.player.aabb.intersects(&p.aabb()) {
                player_hit = true;
                return false;
            }
            alive
        });
        if player_hit {
            self.player.take_damage(2);
            self.audio.play_sfx(&self.audio.damage);
            self.cam.add_shake(3.0);
            self.floating_texts.push(FloatingText::damage(
                self.player.aabb.x + 4.0,
                self.player.aabb.y,
                2,
            ));
        }

        // Boss
        let mut new_slow_spawns: Vec<(f32, f32, f32)> = Vec::new();
        let mut boss_died = false;
        if let Some(b) = self.boss.as_mut() {
            b.update(pcx, pcy, dt, &mut self.projectiles, &mut new_slow_spawns);
            // Berührungs-Schaden vom Boss
            if b.alive() && b.aabb.intersects(&self.player.aabb)
                && self.player.damage_flash <= 0.0
                && self.player.powerups.invuln <= 0.0
            {
                self.player.take_damage(3);
                self.audio.play_sfx(&self.audio.damage);
                self.cam.add_shake(5.0);
                self.floating_texts.push(FloatingText::damage(
                    self.player.aabb.x + 4.0,
                    self.player.aabb.y,
                    3,
                ));
            }
            if !b.alive() && b.phase == BossPhase::Dead {
                boss_died = true;
            }
        }
        for (x, y, l) in new_slow_spawns {
            self.slow_tiles.push(IcePatch::new(x, y, l));
        }

        if boss_died {
            self.boss_defeated = true;
            self.advance_quest_if(3, 4);
            self.state = GameState::Victory;
            self.victory_scroll = 0.0;
            save::save(&self.to_save());
        }

        // Eisflecken & Slow-Tiles
        self.ice_patches.retain_mut(|p| {
            let alive = p.update(dt);
            // Effekt auf Spieler: setze ihn auf Ice
            if alive && self.player.aabb.intersects(&p.aabb()) {
                self.player.on_ice = true;
            }
            alive
        });
        self.slow_tiles.retain_mut(|p| {
            let alive = p.update(dt);
            if alive && self.player.aabb.intersects(&p.aabb()) {
                self.player.on_slow = true;
            }
            alive
        });

        // === E-Aktionen mit Priorität: SBahn-Fahrt > Bus > NPCs > Filiale > Landmark ===
        let e_pressed = is_key_pressed(KeyCode::E);

        // Während aktiver S-Bahn-Fahrt: alle Interaktionen sperren
        if self.sbahn_ride_t > 0.0 {
            return;
        }

        // (1) Spieler steht auf Bahnsteig + Zug hält dort → E → Fahrt zum anderen Bahnhof
        let mut e_consumed = false;
        if e_pressed && self.train_phase == TrainPhase::Dwelling {
            if let Some(plat_idx) = self.player_on_station_platform() {
                if plat_idx == self.train_next_stop {
                    if self.player.coins >= SBAHN_RIDE_COST {
                        self.player.coins -= SBAHN_RIDE_COST;
                        let to = 1 - plat_idx;
                        self.start_sbahn_ride(plat_idx, to);
                        self.area_name = format!(
                            "S8 nach {} ({} M)",
                            station_name(to), SBAHN_RIDE_COST
                        );
                        self.area_fade = 2.5;
                        e_consumed = true;
                    } else {
                        self.area_name = format!(
                            "Brauchst {} Münzen für die S8",
                            SBAHN_RIDE_COST
                        );
                        self.area_fade = 2.0;
                        e_consumed = true;
                    }
                }
            }
        }

        // (2) Bus-Boarding / Aussteigen
        if !e_consumed && e_pressed {
            if let Some(_uid) = self.riding_bus_uid {
                // Aussteigen — Spieler 1 Tile südlich vom Bus absetzen
                if let Some(uid) = self.riding_bus_uid {
                    if let Some(idx) = self.bus_uids.iter().position(|u| *u == uid) {
                        let b = &self.buses[idx];
                        self.player.aabb.x = b.x + 6.0;
                        self.player.aabb.y = b.y + 18.0;
                    }
                }
                self.riding_bus_uid = None;
                self.bus_ride_msg = "Ausgestiegen.".to_string();
                self.bus_ride_msg_t = 2.0;
                self.audio.play_sfx(&self.audio.menu_select);
                e_consumed = true;
            } else {
                // Einsteigen: irgendein Bus in Reichweite?
                let mut best: Option<usize> = None;
                let mut best_d2 = 28.0 * 28.0;
                for (i, b) in self.buses.iter().enumerate() {
                    let bcx = b.x + 14.0;
                    let bcy = b.y + 8.0;
                    let dx = self.player.aabb.x + 6.0 - bcx;
                    let dy = self.player.aabb.y + 7.0 - bcy;
                    let d2 = dx * dx + dy * dy;
                    if d2 < best_d2 {
                        best_d2 = d2;
                        best = Some(i);
                    }
                }
                if let Some(i) = best {
                    if self.player.coins >= BUS_RIDE_COST {
                        self.player.coins -= BUS_RIDE_COST;
                        if let Some(&uid) = self.bus_uids.get(i) {
                            self.riding_bus_uid = Some(uid);
                            let line = BUS_LINES[self.buses[i].line_idx].number;
                            self.bus_ride_msg = format!(
                                "Linie {} - Mitfahrt ({} M)",
                                line, BUS_RIDE_COST
                            );
                            self.bus_ride_msg_t = 2.5;
                            self.audio.play_sfx(&self.audio.jingle);
                        }
                        e_consumed = true;
                    } else {
                        self.bus_ride_msg = format!(
                            "Brauchst {} Münzen für die Mitfahrt",
                            BUS_RIDE_COST
                        );
                        self.bus_ride_msg_t = 2.0;
                        e_consumed = true;
                    }
                }
            }
        }

        // Wenn der Spieler mitfährt → restliche Interaktionen sperren
        if self.riding_bus_uid.is_some() {
            return;
        }

        // NPCs — Interaktion
        let mut start_dialog: Option<usize> = None;
        if !e_consumed {
            for (i, npc) in self.npcs.iter().enumerate() {
                if self.player.aabb.intersects(&npc.interact_zone()) && e_pressed {
                    start_dialog = Some(i);
                    e_consumed = true;
                    break;
                }
            }
        }
        if let Some(i) = start_dialog {
            // Meister-Ihle-Dialog je nach Quest-Stage dynamisch setzen.
            if self.npcs[i].kind == npc::NpcKind::MeisterIhle {
                self.npcs[i].dialog = Self::meister_dialog_for_stage(self.quest_stage);
            }
            // Klaus zu sprechen bringt Quest 0 → 1.
            if self.npcs[i].kind == npc::NpcKind::Klaus {
                self.advance_quest_if(0, 1);
                self.klaus_tour_done = true; // Tour endet, sobald man mit ihm spricht
            }
            self.dialog = Some(DialogState::new(i));
            self.state = GameState::Dialog;
            return;
        }

        // Ihle-Filiale betreten (E) — Öffnungszeiten beachten
        if let Some(idx) = filiale_at(self.player.aabb.x, self.player.aabb.y) {
            if e_pressed && !e_consumed {
                if filiale_is_open(idx, self.game_seconds) {
                    self.checkpoint_filiale = idx;
                    self.shop = Some(ShopState::new(idx));
                    self.audio.play_sfx(&self.audio.jingle);
                    self.state = GameState::Shopping;
                    return;
                } else {
                    let (open, close) = FILIALE_OPEN_HOURS[idx];
                    self.area_name = format!(
                        "Geschlossen! Öffnet {:02}:00 - {:02}:00 Uhr",
                        open, close
                    );
                    self.area_fade = 2.5;
                }
            }
        }

        // Landmark-Interaktion (E auf Brunnen-Tile in Nähe)
        if self.interact_landmark_cooldown <= 0.0 && e_pressed && !e_consumed {
            let landmarks: &[((i32, i32), &str)] = &[
                ((105, 50), "germar"),
                ((115, 35), "jakobus"),
                ((85, 65), "marien"),
                ((8, 54), "ziegel"),
                ((20, 105), "cordobar"),
            ];
            for &((lx, ly), id) in landmarks {
                let dx = pcx - lx as f32 * TILE_SIZE - 8.0;
                let dy = pcy - ly as f32 * TILE_SIZE - 8.0;
                if dx * dx + dy * dy < 28.0 * 28.0 {
                    self.interact_landmark(id);
                    self.interact_landmark_cooldown = 1.0;
                    break;
                }
            }
        }

        // Museum betreten (Punktposition) — neue Position passend zum Rework
        let museum_dx = pcx - 122.0 * TILE_SIZE;
        let museum_dy = pcy - 67.0 * TILE_SIZE;
        if !self.roman_artifact
            && museum_dx * museum_dx + museum_dy * museum_dy < 28.0 * 28.0
            && e_pressed && !e_consumed
        {
            self.roman_artifact = true;
            self.player.powerups.attack_bonus = 1;
            self.area_name = "Römisches Artefakt gefunden! +1 Angriff dauerhaft".to_string();
            self.area_fade = 3.0;
            self.audio.play_sfx(&self.audio.jingle);
            self.advance_quest_if(1, 2);
        }

        // Dungeon-Kristalle einsammeln (Berührung — kein E nötig)
        for (i, &(cx_t, cy_t, name)) in CRYSTALS.iter().enumerate() {
            let bit = 1u8 << i;
            if (self.crystals & bit) != 0 {
                continue;
            }
            let cx_w = cx_t as f32 * TILE_SIZE + 8.0;
            let cy_w = cy_t as f32 * TILE_SIZE + 8.0;
            let dx = pcx - cx_w;
            let dy = pcy - cy_w;
            if dx * dx + dy * dy < 16.0 * 16.0 {
                self.crystals |= bit;
                self.player.add_coins(COIN_BREZEL); // jeder Kristall gibt eine Brezel-Münze
                self.area_name = format!("{} eingesammelt! ({}/4)", name, self.crystals.count_ones());
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
                if self.crystals.count_ones() >= 4 {
                    self.advance_quest_if(2, 3);
                }
            }
        }

        // See-Schwimmstrecke: Checkpoints im Wasser
        if self.player.on_water {
            for (i, &(cx_t, cy_t)) in LAKE_CHECKPOINTS.iter().enumerate() {
                let bit = 1u8 << i;
                if (self.lake_visited & bit) != 0 {
                    continue;
                }
                let cx_w = cx_t as f32 * TILE_SIZE + 8.0;
                let cy_w = cy_t as f32 * TILE_SIZE + 8.0;
                let dx = pcx - cx_w;
                let dy = pcy - cy_w;
                if dx * dx + dy * dy < 24.0 * 24.0 {
                    self.lake_visited |= bit;
                    self.area_name = format!(
                        "See-Checkpoint {}/4!",
                        self.lake_visited.count_ones()
                    );
                    self.area_fade = 2.0;
                    self.audio.play_sfx(&self.audio.coin);
                }
            }
            if !self.lake_swim_done && self.lake_visited == 0b1111 {
                self.lake_swim_done = true;
                self.player.add_coins(COIN_GOLD * 3);
                self.area_name = "See umrundet! +60 Goldmünzen!".to_string();
                self.area_fade = 3.5;
                self.audio.play_sfx(&self.audio.jingle);
            }
        }

        // Game Over
        if !self.player.alive() {
            self.state = GameState::GameOver;
            self.audio.play_sfx(&self.audio.damage);
        }
    }

    /// S-Bahn: fährt zwei Bahnhöfe nacheinander an (Harthaus + Germering),
    /// abwechselnd in Ost- und Westrichtung. Münzen droppen am Halt.
    fn tick_train(&mut self, dt: f32) {
        let track_y = TRAIN_TRACK_TILE_Y as f32 * TILE_SIZE;
        let enter_speed = 220.0;
        let leave_speed = 200.0;
        let train_len = 110.0;
        let map_right = MAP_W as f32 * TILE_SIZE;
        let track_left = (TRAIN_TRACK_X_MIN as f32) * TILE_SIZE;

        // Hilfsfunktion: weiter fahren in aktuelle Richtung
        let dir_sign = match self.train_dir { TrainDir::East => 1.0, TrainDir::West => -1.0 };

        // Nächste Stopposition (Welt-X) basierend auf train_next_stop
        let target_x = station_x(self.train_next_stop);

        match self.train_phase {
            TrainPhase::Idle => {
                self.train_t -= dt;
                if self.train_t <= 0.0 {
                    // Neue Tour beginnen — Richtung wechseln
                    self.train_dir = match self.train_dir {
                        TrainDir::East => TrainDir::West,
                        TrainDir::West => TrainDir::East,
                    };
                    // Alle Stationen wieder „offen"
                    self.train_remaining_stops = 0b11;
                    // Erstes Ziel: bei Ost-Fahrt zuerst Harthaus (0), dann Germering (1)
                    //              bei West-Fahrt umgekehrt
                    self.train_next_stop = match self.train_dir {
                        TrainDir::East => 0,
                        TrainDir::West => 1,
                    };
                    self.train_phase = TrainPhase::Entering;
                    self.train_x = match self.train_dir {
                        TrainDir::East => track_left - 16.0,
                        TrainDir::West => map_right + train_len + 16.0,
                    };
                    self.train_dropped = false;
                }
            }
            TrainPhase::Entering => {
                self.train_x += dir_sign * enter_speed * dt;
                let reached = match self.train_dir {
                    TrainDir::East => self.train_x >= target_x,
                    TrainDir::West => self.train_x <= target_x,
                };
                if reached {
                    self.train_x = target_x;
                    self.train_phase = TrainPhase::Dwelling;
                    self.train_dwell_t = TRAIN_STATION_DWELL;
                    // Münzen-Drop pro Halt
                    if !self.train_dropped {
                        self.train_dropped = true;
                        let platform_y = TRAIN_PLATFORM_TILE_Y as f32 * TILE_SIZE;
                        let (px_a, px_b) = station_platform(self.train_next_stop);
                        let n = 8;
                        for i in 0..n {
                            let frac = i as f32 / (n - 1) as f32;
                            let x = px_a + (px_b - px_a) * frac - 6.0;
                            let mut c = Coin::new(x, platform_y, CoinKind::Silver);
                            c.respawn_t = 99999.0;
                            self.coins.push(c);
                        }
                        self.audio.play_sfx(&self.audio.coin);
                    }
                }
            }
            TrainPhase::Dwelling => {
                self.train_dwell_t -= dt;
                if self.train_dwell_t <= 0.0 {
                    // Aktuelle Station als angefahren markieren
                    self.train_remaining_stops &= !(1u8 << self.train_next_stop);
                    self.train_dropped = false;
                    // Gibt es noch einen Halt in dieser Tour?
                    let next_candidate = match self.train_dir {
                        TrainDir::East => 1usize, // nach Harthaus → Germering
                        TrainDir::West => 0usize, // nach Germering → Harthaus
                    };
                    if (self.train_remaining_stops & (1u8 << next_candidate)) != 0 {
                        self.train_next_stop = next_candidate;
                        self.train_phase = TrainPhase::Entering;
                    } else {
                        self.train_phase = TrainPhase::Leaving;
                    }
                }
            }
            TrainPhase::Leaving => {
                self.train_x += dir_sign * leave_speed * dt;
                let off = match self.train_dir {
                    TrainDir::East => self.train_x > map_right + train_len + 16.0,
                    TrainDir::West => self.train_x < track_left - train_len - 16.0,
                };
                if off {
                    self.train_phase = TrainPhase::Idle;
                    self.train_t = TRAIN_INTERVAL_SECONDS;
                    self.train_dropped = false;
                }
            }
        }

        // Kollision mit Zug, wenn er rollt — ABER nicht während S-Bahn-Fahrt-Anim
        if matches!(self.train_phase, TrainPhase::Entering | TrainPhase::Leaving)
            && self.sbahn_ride_t <= 0.0
            && self.player.damage_flash <= 0.0
            && self.player.powerups.invuln <= 0.0
        {
            // Train-AABB hängt von Richtung ab: Lok ist vorne in Fahrtrichtung
            let train_aabb = match self.train_dir {
                TrainDir::East => Aabb::new(self.train_x - 84.0, track_y, 110.0, 14.0),
                TrainDir::West => Aabb::new(self.train_x, track_y, 110.0, 14.0),
            };
            if self.player.aabb.intersects(&train_aabb) {
                self.player.take_damage(3);
                self.audio.play_sfx(&self.audio.damage);
                self.cam.add_shake(4.5);
                self.floating_texts.push(FloatingText::damage(
                    self.player.aabb.x + 4.0, self.player.aabb.y, 3,
                ));
            }
        }
    }

    /// Welcher Bahnsteig steht direkt unter dem Spieler? (Index oder None)
    fn player_on_station_platform(&self) -> Option<usize> {
        let (pcx, pcy) = self.player.center();
        let platform_y = TRAIN_PLATFORM_TILE_Y as f32 * TILE_SIZE;
        if (pcy - (platform_y + 8.0)).abs() > 16.0 {
            return None;
        }
        for i in 0..2 {
            let (px_a, px_b) = station_platform(i);
            if pcx >= px_a && pcx <= px_b {
                return Some(i);
            }
        }
        None
    }

    /// Startet eine S-Bahn-Fahrt — kurzes Schwarzbild-„Übergang", danach
    /// teleportiert den Spieler zum anderen Bahnhof.
    fn start_sbahn_ride(&mut self, from: usize, to: usize) {
        self.sbahn_ride_from = from;
        self.sbahn_ride_to = to;
        self.sbahn_ride_t = 1.6;
        self.player.powerups.invuln = self.player.powerups.invuln.max(2.0);
        self.audio.play_sfx(&self.audio.jingle);
    }

    fn tick_sbahn_ride(&mut self, dt: f32) {
        if self.sbahn_ride_t <= 0.0 { return; }
        let prev = self.sbahn_ride_t;
        self.sbahn_ride_t = (self.sbahn_ride_t - dt).max(0.0);
        // Bei der Hälfte: Teleport
        if prev > 0.8 && self.sbahn_ride_t <= 0.8 {
            let to_x = station_x(self.sbahn_ride_to);
            let platform_y = TRAIN_PLATFORM_TILE_Y as f32 * TILE_SIZE;
            self.player.aabb.x = to_x;
            self.player.aabb.y = platform_y + 4.0;
            self.player.vx = 0.0;
            self.player.vy = 0.0;
            let (pcx, pcy) = self.player.center();
            self.cam.follow(pcx, pcy);
        }
    }

    /// Busse spawnen, fahren auf horizontalen Straßen, halten an Ampeln &
    /// Zebras. Bei Stau wird die Geschwindigkeit reduziert.
    fn tick_buses(&mut self, dt: f32) {
        self.bus_hit_cooldown = (self.bus_hit_cooldown - dt).max(0.0);
        // Spawn-Timer — viele Busse für lebendige Stadt
        self.bus_spawn_t -= dt;
        if self.bus_spawn_t <= 0.0 && self.buses.len() < 10 {
            self.bus_spawn_t = 3.5 + (self.game_seconds * 1.7).sin().abs() * 2.0;
            let line_idx = ((self.game_seconds * 0.7) as usize + self.buses.len())
                % BUS_LINES.len();
            let h_roads = [20i32, 40, 60, 80, 100];
            let road_idx = ((self.game_seconds * 0.5) as usize + self.buses.len() / 2)
                % h_roads.len();
            let road_y = h_roads[road_idx] as f32 * TILE_SIZE;
            let east = (line_idx + road_idx) % 2 == 0;
            let (x, dir) = if east {
                (-32.0, BusDir::East)
            } else {
                (MAP_W as f32 * TILE_SIZE + 4.0, BusDir::West)
            };
            self.buses.push(Bus {
                line_idx,
                x,
                y: road_y,
                dir,
                speed: 60.0 + (line_idx as f32) * 4.0,
                swear_t: 0.0,
                swear_text: String::new(),
                honk_t: 0.0,
            });
            self.bus_uids.push(self.bus_next_uid);
            self.bus_next_uid += 1;
        }

        // Vorbereitung
        let map_right = MAP_W as f32 * TILE_SIZE;
        let pcx_p = self.player.aabb.x;
        let pcy_p = self.player.aabb.y;
        let traffic_t = self.traffic_phase_t;
        let active_zebras = self.active_zebras_world();
        let jam_road_y = if self.traffic_jam_t > 0.0 { Some(self.traffic_jam_road_y) } else { None };

        let mut player_hit_by_bus = false;
        let mut swear_idx: Option<(usize, &'static str)> = None;
        for (i, b) in self.buses.iter_mut().enumerate() {
            b.swear_t = (b.swear_t - dt).max(0.0);
            b.honk_t = (b.honk_t - dt).max(0.0);

            // Stau?
            let in_jam = jam_road_y.map(|y| (b.y - y).abs() < 6.0).unwrap_or(false);
            let target_speed = if in_jam { b.speed * 0.25 } else { b.speed };

            // Soll der Bus halten?
            let dir_east = matches!(b.dir, BusDir::East);
            let stop_now = vehicle_should_stop(
                b.x, b.y, dir_east, 30.0, traffic_t, &active_zebras,
            );
            let v = if stop_now {
                0.0
            } else {
                match b.dir {
                    BusDir::East => target_speed,
                    BusDir::West => -target_speed,
                }
            };
            b.x += v * dt;

            // Hupen
            let dx = pcx_p - b.x;
            let dy = pcy_p - b.y;
            if dx.abs() < 80.0 && dy.abs() < 24.0 && b.honk_t <= 0.0 {
                let in_front = match b.dir {
                    BusDir::East => dx > 0.0 && dx < 80.0,
                    BusDir::West => dx < 0.0 && dx > -80.0,
                };
                if in_front && !stop_now {
                    b.honk_t = 2.0;
                }
            }

            // Kollision (nur wenn Spieler NICHT in diesem Bus mitfährt)
            let player_in_this_bus = self.riding_bus_uid
                .map(|uid| self.bus_uids.get(i).map(|u| *u == uid).unwrap_or(false))
                .unwrap_or(false);
            if !player_in_this_bus
                && b.aabb().intersects(&self.player.aabb)
                && self.bus_hit_cooldown <= 0.0
                && self.player.damage_flash <= 0.0
                && self.player.powerups.invuln <= 0.0
            {
                player_hit_by_bus = true;
                let swears = [
                    "Geh weida, du Depp!",
                    "Sakrament nochamoi!",
                    "Ja spinnst du!?",
                    "Heiliger Bimbam!",
                    "Gschissna Saupreiß!",
                    "Hirndoddl, schau hi!",
                ];
                let s = swears[(i + self.coins_collected as usize) % swears.len()];
                swear_idx = Some((i, s));
            }
        }
        if let Some((i, s)) = swear_idx {
            self.buses[i].swear_t = 2.5;
            self.buses[i].swear_text = s.to_string();
        }
        if player_hit_by_bus {
            self.player.take_damage(BUS_DAMAGE);
            self.bus_hit_cooldown = 1.5;
            self.cam.add_shake(4.0);
            self.audio.play_sfx(&self.audio.damage);
            self.floating_texts.push(FloatingText::damage(
                self.player.aabb.x + 4.0, self.player.aabb.y, BUS_DAMAGE,
            ));
        }

        // Wenn der Spieler mitfährt — Position an den Bus klemmen
        if let Some(uid) = self.riding_bus_uid {
            if let Some(idx) = self.bus_uids.iter().position(|u| *u == uid) {
                let b = &self.buses[idx];
                self.player.aabb.x = b.x + 6.0;
                self.player.aabb.y = b.y - 2.0;
                self.player.vx = 0.0;
                self.player.vy = 0.0;
                self.player.powerups.invuln = self.player.powerups.invuln.max(0.5);
            } else {
                // Bus wurde despawnt → automatisch absteigen
                self.riding_bus_uid = None;
                self.bus_ride_msg = "Endstation - aus dem Bus!".to_string();
                self.bus_ride_msg_t = 2.0;
            }
        }

        // Aus der Liste werfen, wenn weit außerhalb der Karte — parallel UIDs entfernen
        let mut keep: Vec<bool> = self.buses.iter()
            .map(|b| b.x > -120.0 && b.x < map_right + 120.0)
            .collect();
        // Falls der Spieler im Bus mitfährt und der Bus an der Karte verschwinden würde,
        // dismounten wir den Spieler an der letzten Bus-Position.
        if let Some(uid) = self.riding_bus_uid {
            if let Some(idx) = self.bus_uids.iter().position(|u| *u == uid) {
                if !keep[idx] {
                    // Spieler an Map-Rand-Position sicher absetzen
                    let b = &self.buses[idx];
                    let mid_road_y = b.y + 16.0; // Sidewalk unter der Straße
                    self.player.aabb.x = b.x.clamp(8.0, map_right - 8.0);
                    self.player.aabb.y = mid_road_y;
                    self.riding_bus_uid = None;
                    self.bus_ride_msg = "Endstation - aus dem Bus!".to_string();
                    self.bus_ride_msg_t = 2.0;
                    // Forciere keep für Sicherheit (kein partieller State)
                    let _ = &mut keep;
                }
            }
        }
        let mut i = 0;
        let mut j = 0;
        while i < self.buses.len() {
            if !keep[j] {
                self.buses.remove(i);
                self.bus_uids.remove(i);
                j += 1;
            } else {
                i += 1;
                j += 1;
            }
        }
    }

    /// NPC-Pkw: spawnen, fahren, an Ampeln + Zebras halten, Spielerkollision.
    fn tick_cars(&mut self, dt: f32) {
        self.car_spawn_t -= dt;
        if self.car_spawn_t <= 0.0 && self.cars.len() < CAR_MAX {
            self.car_spawn_t = 1.4 + (self.game_seconds * 2.1).cos().abs() * 1.4;
            let h_roads = [20i32, 40, 60, 80, 100];
            let road_idx = ((self.game_seconds * 1.3) as usize + self.cars.len())
                % h_roads.len();
            let road_y = h_roads[road_idx] as f32 * TILE_SIZE;
            let east = (self.cars.len() + road_idx + (self.game_seconds as usize)) % 2 == 0;
            let (x, dir) = if east {
                (-24.0, CarDir::East)
            } else {
                (MAP_W as f32 * TILE_SIZE + 4.0, CarDir::West)
            };
            let palette = [
                Color::new(0.85, 0.18, 0.18, 1.0), // rot
                Color::new(0.18, 0.30, 0.65, 1.0), // blau
                Color::new(0.10, 0.10, 0.12, 1.0), // schwarz
                Color::new(0.92, 0.92, 0.92, 1.0), // weiß
                Color::new(0.55, 0.55, 0.58, 1.0), // silber
                Color::new(0.95, 0.78, 0.20, 1.0), // gelb
                Color::new(0.20, 0.55, 0.30, 1.0), // grün
            ];
            let body = palette[self.cars.len() % palette.len()];
            self.cars.push(Car {
                x,
                y: road_y + 1.0,
                dir,
                speed: 70.0 + ((self.cars.len() * 13) % 30) as f32,
                current_v: 0.0,
                body,
                honk_t: 0.0,
            });
        }

        let map_right = MAP_W as f32 * TILE_SIZE;
        let traffic_t = self.traffic_phase_t;
        let active_zebras = self.active_zebras_world();
        let jam_road_y = if self.traffic_jam_t > 0.0 { Some(self.traffic_jam_road_y) } else { None };
        let pcx_p = self.player.aabb.x;
        let pcy_p = self.player.aabb.y;

        // Hinter-Auto-Stau: berechne pro Auto die nächste Auto-Position davor
        let snapshot: Vec<(f32, f32, bool)> = self.cars
            .iter()
            .map(|c| (c.x, c.y, matches!(c.dir, CarDir::East)))
            .collect();

        let mut player_hit = false;
        for (i, c) in self.cars.iter_mut().enumerate() {
            c.honk_t = (c.honk_t - dt).max(0.0);
            let dir_east = matches!(c.dir, CarDir::East);
            let in_jam = jam_road_y.map(|y| (c.y - y).abs() < 6.0).unwrap_or(false);
            let target_speed = if in_jam { c.speed * 0.18 } else { c.speed };

            // Halt für Ampeln / Zebras?
            let mut stop_now = vehicle_should_stop(
                c.x, c.y, dir_east, 20.0, traffic_t, &active_zebras,
            );

            // Stau dahinter: nicht in das vorherige Auto reinfahren
            if !stop_now {
                for (j, &(ox, oy, oe)) in snapshot.iter().enumerate() {
                    if j == i { continue; }
                    if (oy - c.y).abs() > 6.0 { continue; }
                    if oe != dir_east { continue; }
                    let ahead = match c.dir {
                        CarDir::East => ox - c.x,
                        CarDir::West => c.x - ox,
                    };
                    if ahead > 0.0 && ahead < 22.0 {
                        stop_now = true;
                        break;
                    }
                }
            }

            // Beschleunigung / Bremsung
            let want = if stop_now {
                0.0
            } else {
                match c.dir { CarDir::East => target_speed, CarDir::West => -target_speed }
            };
            let acc = 110.0 * dt;
            if c.current_v < want { c.current_v = (c.current_v + acc).min(want.max(0.0).max(c.current_v + acc)); }
            if c.current_v > want { c.current_v = (c.current_v - acc).max(want.min(0.0).min(c.current_v - acc)); }
            // Sicherheits-Clamp
            let max_v = c.speed.max(target_speed);
            c.current_v = c.current_v.clamp(-max_v, max_v);

            c.x += c.current_v * dt;

            // Hupen, wenn Spieler dicht vor dem Auto steht
            let dx = pcx_p - c.x;
            let dy = pcy_p - c.y;
            if dx.abs() < 50.0 && dy.abs() < 18.0 && c.honk_t <= 0.0 {
                let in_front = match c.dir {
                    CarDir::East => dx > 0.0 && dx < 50.0,
                    CarDir::West => dx < 0.0 && dx > -50.0,
                };
                if in_front && stop_now {
                    c.honk_t = 1.4;
                }
            }

            // Kollision
            if c.aabb().intersects(&self.player.aabb)
                && self.player.damage_flash <= 0.0
                && self.player.powerups.invuln <= 0.0
                && self.riding_bus_uid.is_none()
            {
                player_hit = true;
            }
        }

        if player_hit {
            self.player.take_damage(CAR_DAMAGE);
            self.cam.add_shake(3.0);
            self.audio.play_sfx(&self.audio.damage);
            self.floating_texts.push(FloatingText::damage(
                self.player.aabb.x + 4.0, self.player.aabb.y, CAR_DAMAGE,
            ));
        }

        self.cars.retain(|c| c.x > -120.0 && c.x < map_right + 120.0);
    }

    /// Liefert die Welt-Koordinaten aller Zebrastreifen, an denen der Spieler
    /// gerade nahe genug steht (Fahrzeuge halten dann).
    fn active_zebras_world(&self) -> Vec<(f32, f32)> {
        let (pcx, pcy) = self.player.center();
        let mut zs = Vec::new();
        for &(zx, zy) in ZEBRA_TILES.iter() {
            let zwx = zx as f32 * TILE_SIZE + 8.0;
            let zwy = zy as f32 * TILE_SIZE + 8.0;
            if (pcx - zwx).abs() < 28.0 && (pcy - zwy).abs() < 28.0 {
                zs.push((zwx, zwy));
            }
        }
        zs
    }

    /// Tickt den Ampel-Phasentimer.
    fn tick_traffic_lights(&mut self, dt: f32) {
        self.traffic_phase_t = (self.traffic_phase_t + dt) % 14.0;
    }

    /// Periodische Stau-Ereignisse — pickt eine h_road und reduziert Speeds.
    fn tick_traffic_jam(&mut self, dt: f32) {
        if self.traffic_jam_t > 0.0 {
            self.traffic_jam_t -= dt;
            return;
        }
        self.traffic_jam_cooldown -= dt;
        if self.traffic_jam_cooldown <= 0.0 {
            self.traffic_jam_cooldown = TRAFFIC_JAM_INTERVAL
                + (self.game_seconds * 0.7).sin().abs() * 30.0;
            let h_roads = [20i32, 40, 60, 80, 100];
            let idx = ((self.game_seconds * 0.13) as usize) % h_roads.len();
            self.traffic_jam_road_y = h_roads[idx] as f32 * TILE_SIZE;
            self.traffic_jam_t = TRAFFIC_JAM_DURATION;
            self.area_name = "STAU! Verkehr stockt auf dieser Strasse".to_string();
            self.area_fade = 3.5;
        }
    }

    /// Aktualisiert wandernde Bürger + Säufer.
    fn tick_pedestrians(&mut self, dt: f32) {
        let pcx_player = self.player.aabb.x + 6.0;
        let pcy_player = self.player.aabb.y + 7.0;
        let mut drunk_hit = false;
        for p in self.pedestrians.iter_mut() {
            p.phase += dt;
            p.wander_t -= dt;
            p.hit_cool = (p.hit_cool - dt).max(0.0);
            p.bubble_t = (p.bubble_t - dt).max(0.0);

            match p.kind {
                PedKind::Citizen => {
                    if p.wander_t <= 0.0 {
                        p.wander_t = 2.5 + (p.phase * 1.3).sin().abs() * 3.0;
                        let a = p.phase * 1.27;
                        p.vx = a.cos() * 22.0;
                        p.vy = a.sin() * 22.0;
                    }
                }
                PedKind::Drunk => {
                    let dx = pcx_player - (p.aabb.x + 6.0);
                    let dy = pcy_player - (p.aabb.y + 7.0);
                    let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                    if dist < 90.0 {
                        // Auf Spieler zu wanken (zickzack)
                        let wobble = (p.phase * 4.0).sin();
                        p.vx = (dx / dist) * 35.0 + wobble * 18.0;
                        p.vy = (dy / dist) * 35.0;
                        // Berührungs-Schaden
                        if p.aabb.intersects(&self.player.aabb)
                            && p.hit_cool <= 0.0
                            && self.player.damage_flash <= 0.0
                            && self.player.powerups.invuln <= 0.0
                        {
                            drunk_hit = true;
                            p.hit_cool = 1.2;
                            p.bubble_t = 2.0;
                            let taunts = [
                                "Gehst weida, Buaberl!",
                                "Prooooost!",
                                "I hau di um!",
                                "*hicks*",
                                "Du schauglst mi bled o!",
                            ];
                            p.bubble_text = taunts
                                [((p.phase * 17.0) as usize) % taunts.len()].to_string();
                        }
                    } else {
                        if p.wander_t <= 0.0 {
                            p.wander_t = 1.5;
                            let a = p.phase * 0.7;
                            p.vx = a.cos() * 14.0;
                            p.vy = a.sin() * 14.0;
                        }
                    }
                }
            }

            // Bewegung mit Tile-Kollision
            let _ = move_aabb(&self.world, &mut p.aabb, p.vx * dt, p.vy * dt);
        }
        if drunk_hit {
            self.player.take_damage(1);
            self.cam.add_shake(2.5);
            self.audio.play_sfx(&self.audio.damage);
            self.floating_texts.push(FloatingText::damage(
                self.player.aabb.x + 4.0, self.player.aabb.y, 1,
            ));
        }
    }

    /// Klaus wartet am Germarbrunnen, bis der Spieler nah ist.
    /// Erst dann läuft er los Richtung Filialen — so verliert man ihn nie.
    fn klaus_position(&mut self) -> (f32, f32) {
        // Statische Pose nach abgeschlossener Tour.
        if self.klaus_tour_done {
            return (105.0 * TILE_SIZE, 51.0 * TILE_SIZE);
        }
        // Wartet am Brunnen, bis Spieler näher als 6 Tiles ist.
        let home_tx = 105.0;
        let home_ty = 51.0;
        if !self.klaus_started_walking {
            let (pcx, pcy) = self.player.center();
            let dx = pcx - home_tx * TILE_SIZE;
            let dy = pcy - home_ty * TILE_SIZE;
            if dx * dx + dy * dy < (6.0 * TILE_SIZE).powi(2) {
                self.klaus_started_walking = true;
                self.klaus_tour_t = 0.0;
            }
            return (home_tx * TILE_SIZE, home_ty * TILE_SIZE);
        }
        // Pfad-Wegpunkte zur Filiale 2 (GEP, neue Position).
        let path: &[(f32, f32)] = &[
            (105.0, 51.0),
            (90.0, 60.0),
            (70.0, 60.0),
            (55.0, 60.0),
            (45.0, 67.0),
        ];
        let speed_seg = 12.0;
        let t = self.klaus_tour_t / speed_seg;
        let seg = (t as usize).min(path.len() - 2);
        let local = (t - seg as f32).clamp(0.0, 1.0);
        let (x0, y0) = path[seg];
        let (x1, y1) = path[seg + 1];
        let x = x0 + (x1 - x0) * local;
        let y = y0 + (y1 - y0) * local;
        (x * TILE_SIZE, y * TILE_SIZE)
    }

    /// Bringt die Quest weiter, wenn aktuell `from` aktiv ist.
    fn advance_quest_if(&mut self, from: u8, to: u8) {
        if self.quest_stage == from {
            self.quest_stage = to;
            self.quest_hint = match to {
                1 => "Quest: Hol das römische Artefakt aus dem Museum.".to_string(),
                2 => "Quest: Sammle die 4 Dungeon-Kristalle.".to_string(),
                3 => "Quest: Bezwinge Schimmelmeister Modrý im alten Kaufhof!".to_string(),
                4 => "Quest abgeschlossen - Germering ist gerettet!".to_string(),
                _ => String::new(),
            };
            self.quest_hint_t = 4.0;
        }
    }

    /// Liefert Meister Ihles Dialog je nach Quest-Stage.
    fn meister_dialog_for_stage(stage: u8) -> Vec<&'static str> {
        match stage {
            0 => vec![
                "Servus Max! Schee dass du da bist.",
                "Der Goldene Brezel-Schlüssel - weg!",
                "Schimmelmeister Modrý hat ihn geklaut.",
                "Rede zuerst mit Stadtführer Klaus.",
                "Er steht am Germarbrunnen, gleich nördlich.",
            ],
            1 => vec![
                "Klaus hat dir alles erklärt? Sehr gut.",
                "Geh ins Stadtmuseum ZEIT+RAUM.",
                "Dort liegt ein römisches Artefakt.",
                "Es macht dich stärker im Kampf!",
            ],
            2 => vec![
                "Mit dem Artefakt: +1 Angriff für immer.",
                "Jetzt brauchst du die 4 Kristalle.",
                "Polariom, Forst, Cordobar, Parsberg.",
                "Erst dann öffnet sich der alte Kaufhof.",
                "Kauf dir vorher Brot - du wirst Kraft brauchen!",
            ],
            3 => vec![
                "Alle 4 Kristalle! Du bist bereit.",
                "Der alte Kaufhof am Stadtplatz wartet.",
                "Modrý sitzt im Keller. Pass auf seine Phasen auf.",
                "Hol uns die Brezel zurück. Pfiati!",
            ],
            _ => vec![
                "Servus Max - du hast Germering gerettet!",
                "Die Brezel ist wieder bei uns.",
                "Komm immer wieder vorbei, ja?",
            ],
        }
    }

    fn interact_landmark(&mut self, id: &str) {
        match id {
            "germar" => {
                self.germarbrunnen_active = 30.0;
                self.area_name = "Germarbrunnen aktiv: +1 Münze/Sek (30s)".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.coin);
            }
            "jakobus" => {
                self.player.heal_full();
                self.area_name = "Jakobusbrunnen heilt dich vollständig!".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
            }
            "marien" => {
                self.player.powerups.invuln = self.player.powerups.invuln.max(20.0);
                self.area_name = "Mariensäule: 20s Unverwundbarkeit".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
            }
            "ziegel" => {
                self.player.add_coins(50);
                self.area_name = "Geheimgang offen: +50 Münzen!".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.coin);
            }
            "cordobar" => {
                self.cordobar_unlocked = true;
                self.area_name = "Cordobar: 'Wellenreiten'-Chiptune freigeschaltet!".to_string();
                self.area_fade = 3.0;
                self.audio.play_sfx(&self.audio.jingle);
            }
            _ => {}
        }
    }

    fn update_shopping(&mut self, dt: f32) {
        if let Some(shop) = self.shop.as_mut() {
            shop.tick(dt);
            let items = shop.visible_items();
            if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                if shop.cursor > 0 {
                    shop.cursor -= 1;
                }
                self.audio.play_sfx(&self.audio.menu_select);
            }
            if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                if shop.cursor + 1 < items.len() {
                    shop.cursor += 1;
                }
                self.audio.play_sfx(&self.audio.menu_select);
            }
            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) {
                let ok = shop.try_buy(&mut self.player, self.game_seconds, &mut self.purchases);
                if ok {
                    self.audio.play_sfx(&self.audio.buy);
                    save::save(&self.to_save());
                }
            }
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
                self.shop = None;
                self.state = GameState::Playing;
            }
        }
    }

    fn update_dialog(&mut self, dt: f32) {
        let total_lines;
        let line_len;
        if let Some(d) = self.dialog.as_ref() {
            let npc = &self.npcs[d.npc_idx];
            total_lines = npc.dialog.len();
            line_len = npc.dialog.get(d.line).map(|s| s.chars().count()).unwrap_or(0);
        } else {
            self.state = GameState::Playing;
            return;
        }
        if let Some(d) = self.dialog.as_mut() {
            d.tick(dt, line_len);
            if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Enter) {
                if d.advance(total_lines) {
                    self.dialog = None;
                    self.state = GameState::Playing;
                }
            }
            if is_key_pressed(KeyCode::Escape) {
                self.dialog = None;
                self.state = GameState::Playing;
            }
        }
    }

    fn update_paused(&mut self) {
        // 0=Weiter, 1=Respawn (Checkpoint), 2=Speichern, 3=Hauptmenü
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            if self.pause_cursor > 0 {
                self.pause_cursor -= 1;
            }
            self.audio.play_sfx(&self.audio.menu_select);
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            if self.pause_cursor < 3 {
                self.pause_cursor += 1;
            }
            self.audio.play_sfx(&self.audio.menu_select);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::E) {
            match self.pause_cursor {
                0 => { self.state = GameState::Playing; }
                1 => {
                    // Respawn am Checkpoint
                    self.respawn_at_checkpoint();
                    self.save_message = "Respawn am Checkpoint!".to_string();
                    self.save_message_t = 2.0;
                }
                2 => {
                    save::save(&self.to_save());
                    self.save_message = "Gespeichert (save.dat)".to_string();
                    self.save_message_t = 2.0;
                }
                3 => {
                    save::save(&self.to_save());
                    self.state = GameState::TitleMenu;
                    self.menu_cursor = 0;
                    self.pause_cursor = 0;
                }
                _ => {}
            }
        }
        if is_key_pressed(KeyCode::P) {
            self.state = GameState::Playing;
            self.pause_cursor = 0;
        }
    }

    fn update_gameover(&mut self) {
        if is_key_pressed(KeyCode::Enter) {
            self.respawn_at_checkpoint();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.state = GameState::TitleMenu;
        }
    }

    fn update_victory(&mut self, dt: f32) {
        self.victory_scroll += dt * 22.0;
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            self.state = GameState::TitleMenu;
        }
    }

    fn update_intro(&mut self, dt: f32) {
        self.intro_scroll += dt * 18.0;
        let mut start = false;
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            start = true;
        }
        // Auto-Übergang nach genug Scroll
        if self.intro_scroll > 240.0 + 22.0 * INTRO_TEXT.len() as f32 {
            start = true;
        }
        if start {
            self.state = GameState::Playing;
            // Initialer Quest-Hinweis sobald die Story losgeht.
            if self.quest_stage == 0 {
                self.quest_hint = "Quest: Sprich mit Stadtführer Klaus am Germarbrunnen.".to_string();
                self.quest_hint_t = 6.0;
            }
        }
    }

    fn update_title(&mut self, has_save: bool) {
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            if self.menu_cursor > 0 {
                self.menu_cursor -= 1;
            }
            self.audio.play_sfx(&self.audio.menu_select);
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            if self.menu_cursor < 3 {
                self.menu_cursor += 1;
            }
            self.audio.play_sfx(&self.audio.menu_select);
        }
        if is_key_pressed(KeyCode::Enter) {
            match self.menu_cursor {
                0 => {
                    self.reset_for_new_game();
                    self.state = GameState::Intro;
                }
                1 => {
                    if has_save {
                        if let Some(d) = save::load() {
                            self.load_from_save(d);
                            self.state = GameState::Playing;
                        }
                    }
                }
                2 => {
                    self.state = GameState::Credits;
                    self.victory_scroll = 0.0;
                }
                3 => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }

    fn update_credits(&mut self, dt: f32) {
        self.victory_scroll += dt * 22.0;
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            self.state = GameState::TitleMenu;
        }
    }

    // --------------------------------------------------------------
    //  Render
    // --------------------------------------------------------------

    fn draw_world(&self) {
        // Sichtbarer Tile-Bereich aus Kamera
        let cam_x = self.cam.x;
        let cam_y = self.cam.y;
        let tx0 = (cam_x / TILE_SIZE) as i32 - 1;
        let ty0 = (cam_y / TILE_SIZE) as i32 - 1;
        let tx1 = ((cam_x + VIRTUAL_W as f32) / TILE_SIZE) as i32 + 1;
        let ty1 = ((cam_y + VIRTUAL_H as f32) / TILE_SIZE) as i32 + 1;

        for y in ty0.max(0)..ty1.min(self.world.h) {
            for x in tx0.max(0)..tx1.min(self.world.w) {
                let t = self.world.get(x, y);
                let (sx, sy) = self.cam.world_to_screen(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                draw_texture_ex(
                    self.tex.tile_texture(t),
                    sx.floor(),
                    sy.floor(),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                );
            }
        }

        // Slow-Spuren (vom Boss)
        for s in &self.slow_tiles {
            let (sx, sy) = self.cam.world_to_screen(s.x, s.y);
            draw_rectangle(sx, sy, 16.0, 16.0, Color::new(0.2, 0.4, 0.2, 0.7));
        }
        // Eis-Flecken
        for s in &self.ice_patches {
            let (sx, sy) = self.cam.world_to_screen(s.x, s.y);
            draw_rectangle(sx, sy, 16.0, 16.0, Color::new(0.8, 0.95, 1.0, 0.6));
        }
    }

    fn draw_entities(&self) {
        // Dungeon-Kristalle (die noch nicht gesammelten)
        for (i, &(cx_t, cy_t, _)) in CRYSTALS.iter().enumerate() {
            if (self.crystals & (1u8 << i)) != 0 {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(
                cx_t as f32 * TILE_SIZE,
                cy_t as f32 * TILE_SIZE,
            );
            // sanftes Schweben & Glow
            let bob = (self.game_seconds * 2.5 + i as f32 * 1.7).sin() * 1.5;
            draw_circle(sx + 8.0, sy + 9.0, 9.0, Color::new(0.65, 0.30, 0.85, 0.25));
            draw_texture_ex(
                &self.tex.crystal,
                sx,
                sy + bob,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // S-Bahn (prozedural — 4 rot-weiße Wagen, Lok je nach Fahrtrichtung
        // vorne; Anhänger ziehen hinterher)
        if self.train_phase != TrainPhase::Idle {
            let train_world_y = TRAIN_TRACK_TILE_Y as f32 * TILE_SIZE;
            let (tsx, tsy) = self.cam.world_to_screen(self.train_x, train_world_y);
            let car_w = 26.0;
            let car_h = 12.0;
            let gap = 2.0;
            let east = matches!(self.train_dir, TrainDir::East);
            // Lok dunkelrot
            draw_rectangle(tsx, tsy + 1.0, car_w, car_h, Color::new(0.6, 0.10, 0.10, 1.0));
            draw_rectangle(tsx + 2.0, tsy + 3.0, car_w - 4.0, 3.0, Color::new(0.95, 0.95, 0.95, 1.0));
            draw_rectangle(tsx + 2.0, tsy + 8.0, 5.0, 3.0, Color::new(0.18, 0.18, 0.20, 1.0));
            // Scheinwerfer in Fahrtrichtung
            let head_x = if east { tsx + car_w - 1.0 } else { tsx + 1.0 };
            draw_circle(head_x, tsy + 5.0, 1.5, Color::new(1.0, 0.95, 0.55, 1.0));
            // 3 Anhängerwagen — bei Ostfahrt links der Lok, bei Westfahrt rechts der Lok
            for i in 1..=3 {
                let dir_off = if east { -1.0 } else { 1.0 };
                let cx = tsx + dir_off * i as f32 * (car_w + gap);
                draw_rectangle(cx, tsy + 1.0, car_w, car_h, Color::new(0.85, 0.18, 0.18, 1.0));
                draw_rectangle(cx + 2.0, tsy + 3.0, car_w - 4.0, 2.0, Color::new(0.98, 0.97, 0.94, 1.0));
                draw_rectangle(cx + 3.0, tsy + 6.0, 4.0, 3.0, Color::new(0.20, 0.30, 0.45, 1.0));
                draw_rectangle(cx + 10.0, tsy + 6.0, 4.0, 3.0, Color::new(0.20, 0.30, 0.45, 1.0));
                draw_rectangle(cx + 17.0, tsy + 6.0, 4.0, 3.0, Color::new(0.20, 0.30, 0.45, 1.0));
                if self.train_phase == TrainPhase::Dwelling {
                    draw_rectangle(
                        cx + car_w / 2.0 - 3.0,
                        tsy + 5.0,
                        6.0,
                        7.0,
                        Color::new(0.10, 0.10, 0.15, 1.0),
                    );
                }
            }
            // Räder
            for i in 0..4 {
                let dir_off = if east { -1.0 } else { 1.0 };
                let cx = tsx + dir_off * i as f32 * (car_w + gap);
                draw_circle(cx + 5.0, tsy + car_h + 1.0, 2.0, BLACK);
                draw_circle(cx + car_w - 5.0, tsy + car_h + 1.0, 2.0, BLACK);
            }
        }

        // S-Bahn-Übergangs-Overlay (vor allem darüber — kommt aber im UI-Pass)

        // --- NPC-Pkw ---
        let tod_now = time_of_day(self.game_seconds);
        let is_night_pkw = tod_now == TimeOfDay::Nacht || tod_now == TimeOfDay::Abend;
        for c in &self.cars {
            let (csx, csy) = self.cam.world_to_screen(c.x, c.y);
            // Schatten
            draw_rectangle(csx + 1.0, csy + 11.0, 18.0, 2.0, Color::new(0.0, 0.0, 0.0, 0.35));
            // Karosserie
            draw_rectangle(csx, csy, 20.0, 11.0, c.body);
            // Dachhaube (etwas dunkler)
            let mut roof = c.body;
            roof.r *= 0.75; roof.g *= 0.75; roof.b *= 0.75;
            draw_rectangle(csx + 4.0, csy + 2.0, 12.0, 5.0, roof);
            // Front-/Heckscheibe
            draw_rectangle(csx + 5.0, csy + 3.0, 5.0, 3.5, Color::new(0.30, 0.40, 0.55, 1.0));
            draw_rectangle(csx + 11.0, csy + 3.0, 4.0, 3.5, Color::new(0.30, 0.40, 0.55, 1.0));
            // Räder
            draw_circle(csx + 4.0, csy + 11.0, 1.8, BLACK);
            draw_circle(csx + 16.0, csy + 11.0, 1.8, BLACK);
            // Scheinwerfer bei Nacht
            if is_night_pkw {
                let (hx, cdx) = match c.dir {
                    CarDir::East => (csx + 19.0, 22.0_f32),
                    CarDir::West => (csx + 1.0, -22.0_f32),
                };
                draw_circle(hx, csy + 5.0, 1.5, Color::new(1.0, 0.95, 0.65, 1.0));
                draw_triangle(
                    macroquad::math::vec2(hx, csy + 2.0),
                    macroquad::math::vec2(hx, csy + 8.0),
                    macroquad::math::vec2(hx + cdx, csy + 5.0),
                    Color::new(1.0, 0.95, 0.60, 0.20),
                );
            }
            // Bremslicht beim Stoppen
            if c.current_v.abs() < 5.0 {
                let bx = match c.dir { CarDir::East => csx + 1.0, CarDir::West => csx + 18.0 };
                draw_rectangle(bx, csy + 4.0, 1.0, 3.0, Color::new(1.0, 0.20, 0.10, 1.0));
            }
        }

        // --- Busse ---
        let tod_now = time_of_day(self.game_seconds);
        let is_night = tod_now == TimeOfDay::Nacht || tod_now == TimeOfDay::Abend;
        for b in &self.buses {
            let (bsx, bsy) = self.cam.world_to_screen(b.x, b.y + 1.0);
            // Schatten
            draw_rectangle(bsx + 1.0, bsy + 14.0, 28.0, 2.0, Color::new(0.0, 0.0, 0.0, 0.35));
            // Bus-Körper (MVV-Grün-Weiß)
            draw_rectangle(bsx, bsy, 30.0, 14.0, Color::new(0.10, 0.45, 0.18, 1.0));
            // Weißer Streifen Mitte
            draw_rectangle(bsx, bsy + 4.0, 30.0, 4.0, Color::new(0.95, 0.95, 0.95, 1.0));
            // Fenster
            for fi in 0..3 {
                let fx = bsx + 4.0 + fi as f32 * 8.0;
                draw_rectangle(fx, bsy + 5.0, 6.0, 2.5, Color::new(0.25, 0.35, 0.50, 1.0));
            }
            // Front (heller, Windschutzscheibe)
            let (front_x, head_x) = match b.dir {
                BusDir::East => (bsx + 26.0, bsx + 29.0),
                BusDir::West => (bsx, bsx),
            };
            draw_rectangle(front_x, bsy + 2.0, 4.0, 9.0, Color::new(0.25, 0.35, 0.50, 1.0));
            // Räder
            draw_circle(bsx + 5.0, bsy + 14.0, 2.0, BLACK);
            draw_circle(bsx + 25.0, bsy + 14.0, 2.0, BLACK);
            // Liniennummer-Schild (Hintergrund — Text im UI-Pass crisp)
            let sign_x = match b.dir {
                BusDir::East => bsx + 16.0,
                BusDir::West => bsx + 2.0,
            };
            draw_rectangle(sign_x, bsy - 1.0, 12.0, 5.0, Color::new(0.18, 0.18, 0.22, 1.0));
            // Scheinwerfer bei Nacht
            if is_night {
                let light_dx = match b.dir { BusDir::East => 18.0, BusDir::West => -18.0 };
                let cone_dx = match b.dir { BusDir::East => 32.0, BusDir::West => -32.0 };
                draw_circle(head_x, bsy + 6.0, 2.5, Color::new(1.0, 0.95, 0.65, 1.0));
                // Lichtkegel
                draw_triangle(
                    macroquad::math::vec2(head_x, bsy + 2.0),
                    macroquad::math::vec2(head_x, bsy + 11.0),
                    macroquad::math::vec2(head_x + cone_dx, bsy + 6.0 + light_dx * 0.05),
                    Color::new(1.0, 0.95, 0.60, 0.22),
                );
            }
            // Hupe + Schimpfwort werden im UI-Pass crisp gerendert.
        }

        // --- Wandernde NPCs (Bürger + Säufer) ---
        for p in &self.pedestrians {
            let (sx, sy) = self.cam.world_to_screen(p.aabb.x - 2.0, p.aabb.y - 2.0);
            let bob = (p.phase * 2.4).sin() * 0.6;
            // Kopf
            draw_circle(sx + 8.0, sy + 4.0 + bob, 2.5,
                Color::new(0.95, 0.78, 0.55, 1.0));
            // Körper (mit Outfit-Tönung)
            draw_rectangle(sx + 5.0, sy + 6.0 + bob, 6.0, 7.0, p.tint);
            // Beine
            draw_rectangle(sx + 5.0, sy + 12.0 + bob, 2.0, 4.0,
                Color::new(0.25, 0.18, 0.10, 1.0));
            draw_rectangle(sx + 9.0, sy + 12.0 + bob, 2.0, 4.0,
                Color::new(0.25, 0.18, 0.10, 1.0));
            // Säufer: Bierkrug in der Hand + roter Kopf
            if p.kind == PedKind::Drunk {
                draw_circle(sx + 8.0, sy + 4.0 + bob, 2.5,
                    Color::new(1.0, 0.55, 0.50, 1.0));
                // Bierkrug
                draw_rectangle(sx + 12.0, sy + 7.0 + bob, 3.0, 4.0,
                    Color::new(0.95, 0.80, 0.30, 1.0));
                draw_rectangle(sx + 12.0, sy + 6.0 + bob, 3.0, 2.0,
                    Color::new(1.0, 1.0, 1.0, 0.9));
                // Wank-Schwanken
                let sway = (p.phase * 5.0).sin() * 0.5;
                let _ = sway;
            }
            // Sprechblase wird im UI-Pass gezeichnet (gestochen scharfer Text).
        }

        // Münzen
        for c in &self.coins {
            if !c.alive {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(c.x, c.y + c.bob_y());
            let tex = match c.kind {
                CoinKind::Copper => &self.tex.coin_copper,
                CoinKind::Silver => &self.tex.coin_silver,
                CoinKind::Gold => &self.tex.coin_gold,
                CoinKind::Brezel => &self.tex.coin_brezel,
            };
            // Doppelmünzen-Glow
            if self.player.powerups.double_coin > 0.0 {
                draw_circle(sx + 8.0, sy + 8.0, 7.0, Color::new(1.0, 0.85, 0.15, 0.3));
            }
            draw_texture_ex(
                tex,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // NPCs (mit Idle-Bobbing, jede Person mit eigener Phase)
        for npc in &self.npcs {
            // Phase aus der Spawn-Position ableiten → jeder NPC wippt anders.
            let phase = npc.aabb.x * 0.13 + npc.aabb.y * 0.07;
            let bob = (self.game_seconds * 2.6 + phase).sin() * 0.6;
            let (sx, sy) = self.cam.world_to_screen(npc.aabb.x - 1.0, npc.aabb.y - 1.0 + bob);
            let t = match npc.kind {
                npc::NpcKind::MeisterIhle => &self.tex.npc_ihle,
                npc::NpcKind::Klaus => &self.tex.npc_klaus,
                npc::NpcKind::Gerhard => &self.tex.npc_gerhard,
                npc::NpcKind::OmaLiesl => &self.tex.npc_oma,
                npc::NpcKind::Franz => &self.tex.npc_franz,
            };
            draw_texture_ex(
                t,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // Gegner
        for e in &self.enemies {
            if !e.alive() {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(e.aabb.x - 1.0, e.aabb.y - 1.0);
            let t = match e.kind {
                EnemyKind::Mold => &self.tex.enemy_mold,
                EnemyKind::Blob => &self.tex.enemy_blob,
                EnemyKind::Rat => &self.tex.enemy_rat,
                EnemyKind::Beat => &self.tex.enemy_beat,
                EnemyKind::Ice => &self.tex.enemy_ice,
            };
            let tint = if e.hit_flash > 0.0 {
                Color::new(1.0, 0.5, 0.5, 1.0)
            } else {
                WHITE
            };
            draw_texture_ex(
                t,
                sx,
                sy,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // Projektile
        for p in &self.projectiles {
            let (sx, sy) = self.cam.world_to_screen(p.x, p.y);
            draw_texture_ex(
                &self.tex.projectile,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..Default::default()
                },
            );
        }

        // Boss
        if let Some(b) = &self.boss {
            if b.alive() {
                let (sx, sy) = self.cam.world_to_screen(b.aabb.x, b.aabb.y);
                let mut tint = if b.hit_flash > 0.0 {
                    Color::new(1.0, 0.4, 0.4, 1.0)
                } else {
                    WHITE
                };
                // Phase 2: echter Boss blinkt schneller
                if b.phase == BossPhase::P2 {
                    if (b.anim_t * 8.0).sin() > 0.0 {
                        tint.a = 0.7;
                    }
                }
                draw_texture_ex(
                    &self.tex.boss,
                    sx,
                    sy,
                    tint,
                    DrawTextureParams {
                        dest_size: Some(vec2(48.0, 48.0)),
                        ..Default::default()
                    },
                );
                // Klone in Phase 2
                for c in &b.clones {
                    let (sx, sy) = self.cam.world_to_screen(c.aabb.x, c.aabb.y);
                    draw_texture_ex(
                        &self.tex.boss,
                        sx,
                        sy,
                        Color::new(0.6, 0.6, 0.7, 0.7),
                        DrawTextureParams {
                            dest_size: Some(vec2(48.0, 48.0)),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        // Player
        let (psx, psy) = self.cam.world_to_screen(self.player.aabb.x - 2.0, self.player.aabb.y - 2.0);
        let tex = if self.player.on_water && self.player.powerups.fly <= 0.0 {
            &self.tex.player_swim
        } else {
            // Animation nur wenn der Spieler sich tatsächlich bewegt.
            let moving = self.player.vx.abs() > 6.0 || self.player.vy.abs() > 6.0;
            let alt = moving && ((self.player.anim_t * 8.0) as i32 % 2 == 1);
            match self.player.facing {
                Facing::Down => if alt { &self.tex.player_down_b } else { &self.tex.player_down_a },
                Facing::Up => if alt { &self.tex.player_up_b } else { &self.tex.player_up },
                Facing::Left => if alt { &self.tex.player_left_b } else { &self.tex.player_left },
                Facing::Right => if alt { &self.tex.player_right_b } else { &self.tex.player_right },
            }
        };
        let mut tint = WHITE;
        if self.player.powerups.invuln > 0.0 {
            // golden blinkend
            let t = (self.player.anim_t * 12.0).sin();
            if t > 0.0 {
                tint = Color::new(1.0, 0.85, 0.15, 1.0);
            }
        }
        if self.player.damage_flash > 0.0 {
            tint = Color::new(1.0, 0.5, 0.5, 1.0);
        }
        draw_texture_ex(
            tex,
            psx,
            psy,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(16.0, 16.0)),
                ..Default::default()
            },
        );

        // Powerup-Effekte
        if self.player.powerups.speed > 0.0 {
            draw_circle(psx + 8.0, psy + 14.0, 4.0, Color::new(0.9, 0.55, 0.20, 0.4));
        }
        if self.player.powerups.fly > 0.0 {
            // Flügel
            draw_circle(psx + 2.0, psy + 10.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.8));
            draw_circle(psx + 14.0, psy + 10.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.8));
        }
        if self.player.powerups.shield {
            draw_circle_lines(psx + 8.0, psy + 8.0, 10.0, 1.0, Color::new(0.85, 0.65, 0.20, 0.8));
        }

        // Angriffs-Animation: expandierender Slash-Bogen + Inner Flash
        if self.attack_anim_t > 0.0 {
            const ATTACK_TOTAL: f32 = 0.22;
            let p = ((ATTACK_TOTAL - self.attack_anim_t) / ATTACK_TOTAL).clamp(0.0, 1.0);
            let (cx, cy) = self.player.center();
            let (sx, sy) = self.cam.world_to_screen(cx, cy);
            let dir_angle = match self.player.facing {
                Facing::Down => std::f32::consts::FRAC_PI_2,
                Facing::Up => -std::f32::consts::FRAC_PI_2,
                Facing::Left => std::f32::consts::PI,
                Facing::Right => 0.0,
            };
            // 3 expanding Bögen mit Versatz (sieht aus wie eine wirbelnde Klinge)
            for ring in 0..3 {
                let phase = (p * 1.4 - ring as f32 * 0.18).clamp(0.0, 1.0);
                if phase <= 0.0 || phase >= 1.0 {
                    continue;
                }
                let radius = 6.0 + phase * 22.0;
                let alpha = (1.0 - phase) * 0.85;
                let col = Color::new(1.0, 0.95 - phase * 0.35, 0.40, alpha);
                let span = 1.6 - ring as f32 * 0.25;
                let start = dir_angle - span / 2.0;
                let steps = 16;
                for s in 0..steps {
                    let a0 = start + (s as f32 / steps as f32) * span;
                    let a1 = start + ((s + 1) as f32 / steps as f32) * span;
                    let x0 = sx + a0.cos() * radius;
                    let y0 = sy + a0.sin() * radius;
                    let x1 = sx + a1.cos() * radius;
                    let y1 = sy + a1.sin() * radius;
                    draw_line(x0, y0, x1, y1, 2.0, col);
                }
            }
            // Inner-Flash am Anfang
            if self.attack_anim_t > ATTACK_TOTAL * 0.55 {
                let flash_a = (self.attack_anim_t / ATTACK_TOTAL) * 0.8;
                draw_circle(sx, sy, 6.0, Color::new(1.0, 1.0, 0.85, flash_a));
            }
            // Speed-Lines hinter dem Spieler in Sprungrichtung (Lunge-Feel)
            if p < 0.5 {
                let trail_alpha = (1.0 - p * 2.0) * 0.6;
                let trail_len = 14.0;
                let (lx, ly) = match self.player.facing {
                    Facing::Down => (0.0, -trail_len),
                    Facing::Up => (0.0, trail_len),
                    Facing::Left => (trail_len, 0.0),
                    Facing::Right => (-trail_len, 0.0),
                };
                for i in 0..3 {
                    let off = (i as f32 - 1.0) * 3.0;
                    let (ox, oy) = if matches!(self.player.facing, Facing::Left | Facing::Right) {
                        (0.0, off)
                    } else {
                        (off, 0.0)
                    };
                    draw_line(
                        sx + ox,
                        sy + oy,
                        sx + ox + lx,
                        sy + oy + ly,
                        1.0,
                        Color::new(1.0, 0.95, 0.55, trail_alpha),
                    );
                }
            }
        }

        // Floating Damage / Coin Texte
        for f in &self.floating_texts {
            let (sx, sy) = self.cam.world_to_screen(f.x, f.y);
            let mut col = f.color;
            col.a *= f.alpha();
            // dunkler Schatten für Lesbarkeit
            let mut shadow = Color::new(0.0, 0.0, 0.0, 0.7 * f.alpha());
            shadow.a = shadow.a.min(0.7);
            draw_text(&f.text, sx + 1.0, sy + 1.0, 12.0, shadow);
            draw_text(&f.text, sx, sy, 12.0, col);
        }
    }

    fn draw_tint(&self) {
        let tod = time_of_day(self.game_seconds);
        let tint = tod.tint();
        if tint.a > 0.0 {
            draw_rectangle(0.0, 0.0, VIRTUAL_W as f32, VIRTUAL_H as f32, tint);
        }

        // Fenster-Licht bei Nacht
        if tod == TimeOfDay::Nacht {
            // Filiale-Schaufenster leuchten (geschlossen aber Licht an)
            for f in &FILIALEN {
                let (sx, sy) = self.cam.world_to_screen(
                    f.tile_x as f32 * TILE_SIZE - 16.0,
                    f.tile_y as f32 * TILE_SIZE - 4.0,
                );
                if sx > -32.0 && sx < VIRTUAL_W as f32 && sy > -32.0 && sy < VIRTUAL_H as f32 {
                    draw_rectangle(sx + 4.0, sy + 4.0, 24.0, 10.0, Color::new(1.0, 0.95, 0.55, 0.4));
                }
            }
        }
    }

    fn draw_interact_hints(&self, ui: &ui::UiCtx) {
        // Priorität: NPC > Filiale > Landmark > Museum — nur EIN Hint gleichzeitig.
        for npc in &self.npcs {
            if self.player.aabb.intersects(&npc.interact_zone()) {
                ui::draw_hint(ui, &format!("[E] mit {} sprechen", npc.name));
                return;
            }
        }
        if let Some(idx) = filiale_at(self.player.aabb.x, self.player.aabb.y) {
            let f = &FILIALEN[idx];
            ui::draw_hint(ui, &format!("[E] {} - {}", f.name, f.adresse));
            return;
        }
        let (pcx, pcy) = self.player.center();
        let landmarks: &[((i32, i32), &str)] = &[
            ((105, 50), "Germarbrunnen"),
            ((115, 35), "Jakobusbrunnen"),
            ((85, 65), "Mariensäule"),
            ((8, 54), "Römischer Ziegelbrennofen"),
            ((20, 105), "Cordobar-Ruine"),
        ];
        for &((lx, ly), name) in landmarks {
            let dx = pcx - lx as f32 * TILE_SIZE - 8.0;
            let dy = pcy - ly as f32 * TILE_SIZE - 8.0;
            if dx * dx + dy * dy < 28.0 * 28.0 {
                ui::draw_hint(ui, &format!("[E] {}", name));
                return;
            }
        }
        if !self.roman_artifact {
            let mdx = pcx - 122.0 * TILE_SIZE;
            let mdy = pcy - 67.0 * TILE_SIZE;
            if mdx * mdx + mdy * mdy < 28.0 * 28.0 {
                ui::draw_hint(ui, "[E] Stadtmuseum ZEIT+RAUM betreten");
                return;
            }
        }
        // Bus-Boarding-Hinweis
        if self.riding_bus_uid.is_none() {
            for b in &self.buses {
                let bcx = b.x + 14.0;
                let bcy = b.y + 8.0;
                let dx = self.player.aabb.x + 6.0 - bcx;
                let dy = self.player.aabb.y + 7.0 - bcy;
                if dx * dx + dy * dy < 26.0 * 26.0 {
                    ui::draw_hint(
                        ui,
                        &format!(
                            "[E] Linie {} mitfahren ({} M)",
                            BUS_LINES[b.line_idx].number, BUS_RIDE_COST
                        ),
                    );
                    return;
                }
            }
        } else {
            ui::draw_hint(ui, "[E] aus dem Bus aussteigen");
            return;
        }
        // S-Bahn-Boarding-Hinweis
        if self.train_phase == TrainPhase::Dwelling {
            if let Some(idx) = self.player_on_station_platform() {
                if idx == self.train_next_stop {
                    let to = 1 - idx;
                    ui::draw_hint(
                        ui,
                        &format!(
                            "[E] S8 → {} ({} M)",
                            station_name(to), SBAHN_RIDE_COST
                        ),
                    );
                }
            }
        }
    }

    /// Sind wir in einem State, der die Pixel-Welt rendert?
    fn has_world(&self) -> bool {
        !matches!(
            self.state,
            GameState::TitleMenu | GameState::Intro | GameState::Victory | GameState::Credits
        )
    }

    /// Render-Pass auf die virtuelle 480×270-Canvas — nur Pixel-Welt.
    fn draw_world_pass(&self) {
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));
        if self.has_world() {
            self.draw_world();
            self.draw_decorations();
            self.draw_entities();
            self.draw_tint();
        }
    }

    /// Welt-Dekorationen: Gebäude-Details, Schilder-Hintergründe.
    /// Texte werden im UI-Pass gerendert (siehe `draw_world_labels`) — dort
    /// nativ in Bildschirmauflösung für gestochen scharfe Lesbarkeit.
    fn draw_decorations(&self) {
        // --- S-Bahn-Gleise (nur östlich vom See, ab TRAIN_TRACK_X_MIN) ---
        let track_world_y = TRAIN_TRACK_TILE_Y as f32 * TILE_SIZE;
        let cam_tx0 = (self.cam.x / TILE_SIZE) as i32 - 2;
        let cam_tx1 = ((self.cam.x + VIRTUAL_W as f32) / TILE_SIZE) as i32 + 2;
        let tx0 = cam_tx0.max(TRAIN_TRACK_X_MIN);
        let tx1 = cam_tx1.min(MAP_W);
        if tx1 > tx0 {
            let (_, gsy) = self.cam.world_to_screen(0.0, track_world_y - 2.0);
            let (gsx_l, _) = self.cam.world_to_screen(tx0 as f32 * TILE_SIZE, 0.0);
            let (gsx_r, _) = self.cam.world_to_screen(tx1 as f32 * TILE_SIZE, 0.0);
            // Schotter-Streifen
            draw_rectangle(gsx_l, gsy, gsx_r - gsx_l, 18.0,
                Color::new(0.45, 0.40, 0.32, 1.0));
            // Schwellen
            for tx in tx0..tx1 {
                let (sx, sy) = self.cam.world_to_screen(tx as f32 * TILE_SIZE, track_world_y);
                for s_off in 0..4 {
                    let sxx = sx + s_off as f32 * 4.0;
                    draw_rectangle(sxx, sy + 1.0, 2.5, 14.0,
                        Color::new(0.25, 0.18, 0.10, 1.0));
                }
            }
            // Zwei Schienen
            let (_, rsy_top) = self.cam.world_to_screen(0.0, track_world_y + 3.0);
            let (_, rsy_bot) = self.cam.world_to_screen(0.0, track_world_y + 11.0);
            draw_rectangle(gsx_l, rsy_top, gsx_r - gsx_l, 1.5,
                Color::new(0.75, 0.75, 0.78, 1.0));
            draw_rectangle(gsx_l, rsy_bot, gsx_r - gsx_l, 1.5,
                Color::new(0.75, 0.75, 0.78, 1.0));
        }

        // --- Bahnsteige für beide Stationen (Harthaus + Germering) ---
        let (_, psy) = self.cam.world_to_screen(0.0, TRAIN_PLATFORM_TILE_Y as f32 * TILE_SIZE);
        for station_idx in 0..2usize {
            let (bahn_x0, bahn_x1) = station_platform(station_idx);
            let (psx_l, _) = self.cam.world_to_screen(bahn_x0, 0.0);
            let (psx_r, _) = self.cam.world_to_screen(bahn_x1, 0.0);
            if psx_r < 0.0 || psx_l > VIRTUAL_W as f32 { continue; }
            draw_rectangle(psx_l, psy, psx_r - psx_l, 16.0,
                Color::new(0.72, 0.72, 0.74, 1.0));
            // Gelbe Sicherheitslinie am Bahnsteig
            draw_rectangle(psx_l, psy + 1.0, psx_r - psx_l, 2.0,
                Color::new(1.0, 0.85, 0.15, 1.0));
            // Bank am Bahnsteig
            draw_rectangle(psx_l + 6.0, psy + 8.0, 12.0, 2.0,
                Color::new(0.45, 0.30, 0.18, 1.0));
        }

        // --- Ampelmasten an allen Lichtkreuzungen ---
        let red = light_is_red_for_horizontal(self.traffic_phase_t);
        let yellow = {
            let p = self.traffic_phase_t.rem_euclid(14.0);
            (6.0..7.0).contains(&p) || (13.0..14.0).contains(&p)
        };
        for &(lx, ly) in TRAFFIC_LIGHT_TILES.iter() {
            // Mast steht neben der Kreuzung — rechts vom Sidewalk
            let wx = lx as f32 * TILE_SIZE + 16.0;
            let wy = ly as f32 * TILE_SIZE - 16.0;
            let (sx, sy) = self.cam.world_to_screen(wx, wy);
            if sx < -16.0 || sx > VIRTUAL_W as f32 + 16.0
                || sy < -32.0 || sy > VIRTUAL_H as f32 + 16.0
            {
                continue;
            }
            // Mast (vertikal)
            draw_rectangle(sx + 1.0, sy + 4.0, 1.5, 14.0,
                Color::new(0.18, 0.18, 0.22, 1.0));
            // Lichterkasten
            draw_rectangle(sx, sy, 4.0, 9.0,
                Color::new(0.10, 0.10, 0.12, 1.0));
            // Rotes Licht (oben)
            let r_on = red;
            let y_on = yellow;
            let g_on = !red && !yellow;
            draw_circle(sx + 2.0, sy + 1.5, 0.9,
                if r_on { Color::new(1.0, 0.20, 0.10, 1.0) } else { Color::new(0.30, 0.05, 0.05, 1.0) });
            draw_circle(sx + 2.0, sy + 4.0, 0.9,
                if y_on { Color::new(1.0, 0.85, 0.15, 1.0) } else { Color::new(0.35, 0.30, 0.05, 1.0) });
            draw_circle(sx + 2.0, sy + 6.5, 0.9,
                if g_on { Color::new(0.30, 0.95, 0.30, 1.0) } else { Color::new(0.05, 0.25, 0.05, 1.0) });
        }

        // --- Stau-Markierung über betroffener Strasse ---
        if self.traffic_jam_t > 0.0 {
            let y = self.traffic_jam_road_y;
            let (sx0, sy) = self.cam.world_to_screen(self.cam.x, y - 4.0);
            // schimmernder Streifen
            let alpha = (self.game_seconds * 6.0).sin().abs() * 0.18 + 0.10;
            let _ = sx0;
            draw_rectangle(
                0.0, sy, VIRTUAL_W as f32, 24.0,
                Color::new(1.0, 0.20, 0.10, alpha),
            );
        }

        // --- Echte Germering-Gebäude-Details auf der Building-Schicht ---
        self.draw_building_details();

        // --- Ihle-Filialen: Logo-Schild-Hintergrund + Café-Sitzbereich ---
        for f in FILIALEN.iter() {
            let bx = f.tile_x as f32 * TILE_SIZE;
            let by = f.tile_y as f32 * TILE_SIZE;

            // Logo-Schild über dem Gebäude (Hintergrund — Text kommt im UI-Pass)
            let sign_w = 44.0;
            let sign_h = 13.0;
            let sign_world_x = bx - sign_w / 2.0 + 8.0;
            let sign_world_y = by - 23.0;
            let (sx, sy) = self.cam.world_to_screen(sign_world_x, sign_world_y);
            draw_rectangle(sx + 4.0, sy + sign_h, 1.5, 7.0,
                Color::new(0.30, 0.30, 0.30, 1.0));
            draw_rectangle(sx + sign_w - 5.5, sy + sign_h, 1.5, 7.0,
                Color::new(0.30, 0.30, 0.30, 1.0));
            draw_rectangle(sx, sy, sign_w, sign_h, Color::new(0.55, 0.12, 0.12, 1.0));
            draw_rectangle(sx + 1.0, sy + 1.0, sign_w - 2.0, sign_h - 2.0,
                Color::new(1.0, 0.85, 0.15, 1.0));
            draw_rectangle(sx + 1.0, sy + 1.0, sign_w - 2.0, 2.0,
                Color::new(1.0, 0.95, 0.55, 1.0));

            // Café-Sitzbereich südlich der Tür
            let cafe_y = by + 56.0;
            let table_color = Color::new(0.55, 0.36, 0.20, 1.0);
            let table_top = Color::new(0.78, 0.55, 0.30, 1.0);
            let chair_color = Color::new(0.40, 0.26, 0.14, 1.0);
            let cloth_color = Color::new(0.95, 0.92, 0.85, 0.85);

            for &dx_tile in &[-22.0_f32, 20.0_f32] {
                let (tx, ty) = self.cam.world_to_screen(bx + dx_tile, cafe_y);
                draw_circle(tx - 8.0, ty - 1.0, 2.5, chair_color);
                draw_circle(tx + 8.0, ty - 1.0, 2.5, chair_color);
                draw_circle(tx - 1.0, ty - 8.0, 2.5, chair_color);
                draw_circle(tx - 1.0, ty + 6.0, 2.5, chair_color);
                draw_circle(tx, ty + 2.0, 5.5, Color::new(0.0, 0.0, 0.0, 0.3));
                draw_circle(tx, ty, 5.0, table_color);
                draw_circle(tx, ty - 0.5, 4.0, table_top);
                draw_circle(tx, ty - 1.0, 3.0, cloth_color);
                if f.nummer <= 2 {
                    draw_circle(tx, ty - 3.0, 6.0, Color::new(0.85, 0.18, 0.18, 0.55));
                    draw_rectangle(tx - 0.5, ty - 3.0, 1.0, 4.0,
                        Color::new(0.3, 0.2, 0.1, 0.8));
                }
            }
        }

        // --- St. Jakobskirche (Turm-Detail, Text im UI-Pass) ---
        let (kx, ky) = self.cam.world_to_screen(115.0 * TILE_SIZE, 30.0 * TILE_SIZE);
        draw_rectangle(kx - 6.0, ky - 24.0, 12.0, 24.0, Color::new(0.78, 0.74, 0.66, 1.0));
        draw_rectangle_lines(kx - 6.0, ky - 24.0, 12.0, 24.0, 1.0,
            Color::new(0.40, 0.36, 0.28, 1.0));
        draw_triangle(
            macroquad::math::vec2(kx - 7.0, ky - 24.0),
            macroquad::math::vec2(kx + 7.0, ky - 24.0),
            macroquad::math::vec2(kx, ky - 38.0),
            Color::new(0.35, 0.18, 0.10, 1.0),
        );
        // Kreuz
        draw_rectangle(kx - 0.5, ky - 46.0, 1.0, 9.0, Color::new(1.0, 0.85, 0.15, 1.0));
        draw_rectangle(kx - 2.5, ky - 42.0, 5.0, 1.0, Color::new(1.0, 0.85, 0.15, 1.0));
        // Uhr im Turm
        draw_circle(kx, ky - 14.0, 2.0, Color::new(0.95, 0.92, 0.78, 1.0));
        draw_rectangle(kx - 0.3, ky - 15.0, 0.6, 1.5, Color::new(0.0, 0.0, 0.0, 1.0));
        draw_rectangle(kx - 0.3, ky - 14.3, 1.0, 0.6, Color::new(0.0, 0.0, 0.0, 1.0));
        // Glockenfenster
        draw_rectangle(kx - 4.0, ky - 18.0, 3.0, 5.0, Color::new(0.18, 0.18, 0.30, 1.0));
        draw_rectangle(kx + 1.0, ky - 18.0, 3.0, 5.0, Color::new(0.18, 0.18, 0.30, 1.0));
        // Buntglas
        draw_rectangle(kx - 3.0, ky - 8.0, 2.0, 5.0, Color::new(0.40, 0.20, 0.55, 1.0));
        draw_rectangle(kx + 1.0, ky - 8.0, 2.0, 5.0, Color::new(0.40, 0.20, 0.55, 1.0));

        // --- Stadthalle Banner-Hintergrund ---
        let (sx, sy) = self.cam.world_to_screen(77.0 * TILE_SIZE, 49.0 * TILE_SIZE);
        draw_rectangle(sx, sy, 90.0, 14.0, Color::new(0.16, 0.24, 0.50, 1.0));
        draw_rectangle(sx + 1.0, sy + 1.0, 88.0, 12.0,
            Color::new(0.22, 0.32, 0.60, 1.0));

        // --- Stadtmuseum ZEIT+RAUM Banner-Hintergrund ---
        let (mx, my) = self.cam.world_to_screen(108.0 * TILE_SIZE, 49.0 * TILE_SIZE);
        draw_rectangle(mx, my, 76.0, 14.0, Color::new(0.40, 0.22, 0.10, 1.0));
        draw_rectangle(mx + 1.0, my + 1.0, 74.0, 12.0,
            Color::new(0.55, 0.32, 0.16, 1.0));

        // --- Polariom Eishalle Banner-Hintergrund ---
        let (px, py) = self.cam.world_to_screen(160.0 * TILE_SIZE, 75.0 * TILE_SIZE);
        draw_rectangle(px, py, 96.0, 16.0, Color::new(0.20, 0.45, 0.70, 1.0));
        draw_rectangle(px + 1.0, py + 1.0, 94.0, 14.0,
            Color::new(0.45, 0.75, 0.92, 1.0));

        // --- Bahnhof S8-Schild-Hintergrund ---
        let (bsx, bsy) = self.cam.world_to_screen(96.0 * TILE_SIZE, 4.0 * TILE_SIZE);
        draw_rectangle(bsx, bsy, 38.0, 14.0, Color::new(0.06, 0.30, 0.10, 1.0));
        draw_rectangle(bsx + 1.0, bsy + 1.0, 36.0, 12.0,
            Color::new(0.12, 0.55, 0.20, 1.0));

        // --- St. Jakobskirche Schild-Hintergrund ---
        draw_rectangle(kx - 22.0, ky + 8.0, 44.0, 9.0, Color::new(0.18, 0.18, 0.22, 0.9));

        // --- Stadtpark-Bäume + Bänke ---
        self.draw_park();
    }

    /// Zeichnet jedes Gebäude einmal als zusammenhängendes Ganzes — Dachform,
    /// Fenstermuster und Farbpalette hängen von BuildingKind ab. So sieht
    /// jede Häuserzeile anders aus.
    fn draw_building_details(&self) {
        let cam_x = self.cam.x;
        let cam_y = self.cam.y;

        for b in &self.world.buildings {
            // Welt-Pixel-Bereich
            let bx = b.x as f32 * TILE_SIZE;
            let by = b.y as f32 * TILE_SIZE;
            let bw = b.w as f32 * TILE_SIZE;
            let bh = b.h as f32 * TILE_SIZE;
            // Off-Screen-Check (großzügig)
            if bx + bw < cam_x - 16.0 || bx > cam_x + VIRTUAL_W as f32 + 16.0
                || by + bh < cam_y - 32.0 || by > cam_y + VIRTUAL_H as f32 + 16.0
            {
                continue;
            }
            let (sx, sy) = self.cam.world_to_screen(bx, by);
            self.draw_one_building(b, sx, sy, bw, bh);
        }
    }

    /// Rendert ein einzelnes Gebäude. Die Wand-Tile darunter ist bereits in
    /// `draw_world` gefüllt (TILE_BUILDING); hier kommt die obere Schicht
    /// (Dach, Fassade, Fenster, Tür).
    fn draw_one_building(
        &self,
        b: &crate::world::Building,
        sx: f32, sy: f32,
        bw: f32, bh: f32,
    ) {
        use crate::world::BuildingKind as BK;
        let seed = b.seed;

        // Hilfsfunktion: deterministischer Pick aus einer Liste anhand seed
        let pick = |bucket: u32, n: usize| -> usize {
            ((seed.wrapping_mul(31).wrapping_add(bucket)) as usize) % n.max(1)
        };

        // Farbpalette pro Gebäudeart
        let (wall_col, roof_col, accent) = match b.kind {
            BK::House => {
                let walls = [
                    Color::new(0.92, 0.88, 0.78, 1.0),
                    Color::new(0.85, 0.78, 0.65, 1.0),
                    Color::new(0.78, 0.66, 0.50, 1.0),
                    Color::new(0.95, 0.90, 0.82, 1.0),
                ];
                let roofs = [
                    Color::new(0.55, 0.18, 0.12, 1.0),
                    Color::new(0.45, 0.22, 0.12, 1.0),
                    Color::new(0.70, 0.30, 0.20, 1.0),
                ];
                (walls[pick(1, walls.len())], roofs[pick(2, roofs.len())],
                 Color::new(0.30, 0.20, 0.10, 1.0))
            }
            BK::Reihenhaus => (
                [
                    Color::new(0.72, 0.60, 0.46, 1.0),
                    Color::new(0.82, 0.72, 0.58, 1.0),
                    Color::new(0.66, 0.54, 0.40, 1.0),
                ][pick(1, 3)],
                Color::new(0.45, 0.22, 0.12, 1.0),
                Color::new(0.30, 0.20, 0.10, 1.0),
            ),
            BK::Apartment => (
                Color::new(0.80, 0.78, 0.72, 1.0),
                Color::new(0.36, 0.30, 0.22, 1.0),
                Color::new(0.20, 0.20, 0.20, 1.0),
            ),
            BK::Highrise => (
                [
                    Color::new(0.75, 0.78, 0.82, 1.0),
                    Color::new(0.68, 0.72, 0.78, 1.0),
                    Color::new(0.82, 0.82, 0.86, 1.0),
                ][pick(1, 3)],
                Color::new(0.40, 0.40, 0.44, 1.0),
                Color::new(0.18, 0.18, 0.20, 1.0),
            ),
            BK::Shop => (
                [
                    Color::new(0.95, 0.92, 0.85, 1.0),
                    Color::new(0.90, 0.86, 0.80, 1.0),
                ][pick(1, 2)],
                Color::new(0.30, 0.30, 0.34, 1.0),
                Color::new(0.85, 0.18, 0.18, 1.0),
            ),
            BK::Industrial => (
                Color::new(0.65, 0.65, 0.68, 1.0),
                Color::new(0.50, 0.50, 0.55, 1.0),
                Color::new(0.95, 0.55, 0.20, 1.0),
            ),
            BK::Rathaus => (
                Color::new(0.95, 0.88, 0.72, 1.0),
                Color::new(0.45, 0.20, 0.18, 1.0),
                Color::new(1.0, 0.85, 0.15, 1.0),
            ),
            BK::Schule => (
                Color::new(0.95, 0.92, 0.85, 1.0),
                Color::new(0.55, 0.22, 0.15, 1.0),
                Color::new(0.85, 0.18, 0.18, 1.0),
            ),
            BK::Krankenhaus => (
                Color::new(0.98, 0.98, 0.98, 1.0),
                Color::new(0.85, 0.85, 0.88, 1.0),
                Color::new(0.85, 0.18, 0.18, 1.0),
            ),
            BK::Tankstelle => (
                Color::new(0.95, 0.92, 0.85, 1.0),
                Color::new(0.20, 0.50, 0.30, 1.0),
                Color::new(1.0, 0.85, 0.15, 1.0),
            ),
            BK::Ruin => (
                Color::new(0.55, 0.50, 0.40, 1.0),
                Color::new(0.35, 0.30, 0.25, 1.0),
                Color::new(0.25, 0.20, 0.15, 1.0),
            ),
            BK::Kirche => (
                Color::new(0.85, 0.80, 0.70, 1.0),
                Color::new(0.40, 0.20, 0.12, 1.0),
                Color::new(1.0, 0.85, 0.15, 1.0),
            ),
        };

        // 1) Wand-Fassade
        draw_rectangle(sx, sy + 4.0, bw, bh - 4.0, wall_col);
        // Schatten am Fuß
        draw_rectangle(sx, sy + bh - 1.0, bw, 1.0, Color::new(0.0, 0.0, 0.0, 0.35));

        // 2) Dach — Form je nach Art
        match b.kind {
            BK::Highrise | BK::Apartment | BK::Industrial | BK::Krankenhaus | BK::Tankstelle => {
                // Flachdach
                draw_rectangle(sx, sy, bw, 4.5, roof_col);
                draw_rectangle(sx, sy + 3.5, bw, 1.0, Color::new(0.10, 0.10, 0.12, 1.0));
            }
            BK::Shop => {
                draw_rectangle(sx, sy, bw, 5.0, roof_col);
                // Markise (Akzentfarbe)
                draw_rectangle(sx, sy + 5.0, bw, 2.0, accent);
            }
            BK::Rathaus | BK::Schule | BK::Kirche => {
                // Satteldach mit „Mittelturm"
                draw_rectangle(sx, sy + 1.0, bw, 5.0, roof_col);
                let tower_w = 6.0;
                let tower_x = sx + bw * 0.5 - tower_w * 0.5;
                draw_rectangle(tower_x, sy - 10.0, tower_w, 14.0, wall_col);
                draw_triangle(
                    macroquad::math::vec2(tower_x - 1.0, sy - 10.0),
                    macroquad::math::vec2(tower_x + tower_w + 1.0, sy - 10.0),
                    macroquad::math::vec2(tower_x + tower_w * 0.5, sy - 18.0),
                    roof_col,
                );
                // Akzent (Uhr / Glocke / Wappen)
                draw_rectangle(tower_x + 1.5, sy - 7.0, tower_w - 3.0, 2.5, accent);
            }
            BK::House | BK::Reihenhaus => {
                // Satteldach — Dreieck oben drauf
                draw_rectangle(sx, sy + 2.0, bw, 4.0, roof_col);
                draw_triangle(
                    macroquad::math::vec2(sx, sy + 2.5),
                    macroquad::math::vec2(sx + bw, sy + 2.5),
                    macroquad::math::vec2(sx + bw * 0.5, sy - 3.0),
                    roof_col,
                );
                // Schornstein an einem Rand (deterministisch)
                if pick(7, 2) == 0 {
                    draw_rectangle(sx + bw * 0.75, sy - 5.0, 2.2, 4.0,
                        Color::new(0.32, 0.18, 0.12, 1.0));
                    draw_rectangle(sx + bw * 0.75, sy - 6.0, 2.2, 1.0,
                        Color::new(0.20, 0.20, 0.20, 1.0));
                }
            }
            BK::Ruin => {
                // gezackte Wand-Oberkante (zerfallenes Dach)
                let segs = (bw / 4.0) as i32;
                for s in 0..segs {
                    let h = if (s + (seed as i32) % 7) % 3 == 0 { 2.0 } else { 5.0 };
                    let x = sx + s as f32 * 4.0;
                    draw_rectangle(x, sy + 4.0 - h, 4.0, h, wall_col);
                }
            }
        }

        // 3) Fenster — pro Art unterschiedliches Raster
        match b.kind {
            BK::Ruin => {} // keine Fenster in Ruine
            BK::Tankstelle => {
                // Großes Vordach + zwei Säulen
                draw_rectangle(sx + bw * 0.20, sy + bh - 12.0, bw * 0.60, 3.0, roof_col);
                draw_rectangle(sx + bw * 0.22, sy + bh - 9.0, 1.5, 9.0, Color::new(0.5, 0.5, 0.55, 1.0));
                draw_rectangle(sx + bw * 0.76, sy + bh - 9.0, 1.5, 9.0, Color::new(0.5, 0.5, 0.55, 1.0));
                // Zapfsäulen
                draw_rectangle(sx + bw * 0.30, sy + bh - 7.0, 3.0, 6.0, Color::new(0.10, 0.10, 0.12, 1.0));
                draw_rectangle(sx + bw * 0.62, sy + bh - 7.0, 3.0, 6.0, Color::new(0.10, 0.10, 0.12, 1.0));
                // großes Schaufenster
                draw_rectangle(sx + 2.0, sy + 6.0, bw - 4.0, 5.0,
                    Color::new(0.40, 0.55, 0.70, 1.0));
            }
            BK::Shop => {
                // Großes Schaufenster über die ganze Fassade
                let win_y = sy + 8.0;
                let win_h = (bh - 14.0).max(3.0);
                draw_rectangle(sx + 1.5, win_y, bw - 3.0, win_h,
                    Color::new(0.45, 0.60, 0.78, 1.0));
                // Sprossen alle 6px
                let mut x = sx + 1.5;
                while x < sx + bw - 1.5 {
                    draw_rectangle(x, win_y, 0.4, win_h, wall_col);
                    x += 6.0;
                }
            }
            BK::Highrise => {
                // Raster aus vielen kleinen Fenstern
                let rows = (bh / 6.0) as i32;
                let cols = (bw / 5.0) as i32;
                for r in 1..rows {
                    for c in 0..cols {
                        let fx = sx + 1.0 + c as f32 * 5.0;
                        let fy = sy + r as f32 * 5.0 + 1.0;
                        let lit = ((r * 31 + c * 17 + seed as i32) % 7) == 0
                            && time_of_day(self.game_seconds) == TimeOfDay::Nacht;
                        let col = if lit {
                            Color::new(1.0, 0.85, 0.45, 1.0)
                        } else {
                            Color::new(0.25, 0.35, 0.50, 1.0)
                        };
                        draw_rectangle(fx, fy, 3.0, 3.0, col);
                    }
                }
            }
            BK::Krankenhaus => {
                // Rotes Kreuz prominent + Fensterreihen
                let cx = sx + bw * 0.5;
                let cy = sy + bh * 0.5;
                draw_rectangle(cx - 1.5, cy - 5.0, 3.0, 10.0, accent);
                draw_rectangle(cx - 5.0, cy - 1.5, 10.0, 3.0, accent);
                let rows = (bh / 7.0) as i32;
                let cols = (bw / 8.0) as i32;
                for r in 1..rows {
                    for c in 0..cols {
                        let fx = sx + 2.0 + c as f32 * 8.0;
                        let fy = sy + r as f32 * 7.0;
                        if (fx - cx).abs() < 7.0 && (fy - cy).abs() < 7.0 { continue; }
                        draw_rectangle(fx, fy, 5.0, 3.5,
                            Color::new(0.45, 0.65, 0.85, 1.0));
                    }
                }
            }
            BK::Industrial => {
                // Wenige große Tore + flache Fensterreihe
                let tile_w = bw / 3.0;
                for t in 0..3 {
                    let x = sx + t as f32 * tile_w + 2.0;
                    draw_rectangle(x, sy + bh - 9.0, tile_w - 4.0, 8.0,
                        Color::new(0.40, 0.40, 0.45, 1.0));
                    // Tor-Streifen
                    for k in 0..4 {
                        draw_rectangle(x, sy + bh - 9.0 + k as f32 * 2.0, tile_w - 4.0, 0.5,
                            Color::new(0.25, 0.25, 0.28, 1.0));
                    }
                }
                // Liniennummer-Schild oben
                draw_rectangle(sx + 2.0, sy + 5.0, bw - 4.0, 2.0, accent);
            }
            BK::Apartment => {
                // 3 Stockwerke à 3-4 Fenster
                let cols = ((bw / 5.0) as i32).max(2);
                let rows = ((bh / 6.0) as i32).max(2);
                for r in 1..rows {
                    for c in 0..cols {
                        let fx = sx + 1.5 + c as f32 * (bw / cols as f32);
                        let fy = sy + r as f32 * 5.0 + 1.0;
                        let lit = ((r * 7 + c * 13 + seed as i32) % 5) == 0
                            && time_of_day(self.game_seconds) == TimeOfDay::Nacht;
                        let col = if lit {
                            Color::new(1.0, 0.85, 0.45, 1.0)
                        } else {
                            Color::new(0.30, 0.40, 0.55, 1.0)
                        };
                        draw_rectangle(fx, fy, 3.0, 3.0, col);
                        // Balkon-Andeutung
                        if r == 2 && c < cols - 1 {
                            draw_rectangle(fx - 1.0, fy + 3.5, 4.5, 0.6,
                                Color::new(0.30, 0.20, 0.10, 1.0));
                        }
                    }
                }
            }
            BK::Rathaus | BK::Schule | BK::Kirche => {
                // Klassiches Fensterraster
                let cols = ((bw / 5.0) as i32).max(2);
                for c in 0..cols {
                    let fx = sx + 2.0 + c as f32 * (bw - 4.0) / cols as f32;
                    let fy = sy + bh - 9.0;
                    draw_rectangle(fx, fy, 3.0, 5.0,
                        Color::new(0.40, 0.55, 0.78, 1.0));
                }
            }
            BK::House | BK::Reihenhaus => {
                // 2 Fenster + Mitteltür
                let win_y = sy + bh - 10.0;
                draw_rectangle(sx + 2.0, win_y, 4.0, 4.0,
                    Color::new(0.40, 0.55, 0.78, 1.0));
                draw_rectangle(sx + bw - 6.0, win_y, 4.0, 4.0,
                    Color::new(0.40, 0.55, 0.78, 1.0));
            }
        }

        // 4) Tür (außer Ruin) — mittig unten
        if !matches!(b.kind, BK::Ruin) {
            let dsx = sx + bw * 0.5 - 2.0;
            let dsy = sy + bh - 6.0;
            draw_rectangle(dsx, dsy, 4.0, 6.0,
                Color::new(0.32, 0.18, 0.10, 1.0));
            draw_rectangle(dsx + 0.5, dsy + 0.5, 3.0, 5.0,
                Color::new(0.45, 0.25, 0.14, 1.0));
            draw_circle(dsx + 3.2, dsy + 3.0, 0.5,
                Color::new(1.0, 0.85, 0.15, 1.0));
        }
    }

    /// Stadtpark-Detail: Bäume + Bänke + Springbrunnen-Mosaik.
    fn draw_park(&self) {
        // Bäume im Stadtpark (Area: 90..130, 30..50)
        let trees: &[(f32, f32)] = &[
            (94.0, 33.0), (98.0, 35.0), (105.0, 32.0), (110.0, 38.0),
            (120.0, 33.0), (125.0, 38.0), (98.0, 45.0), (103.0, 47.0),
            (108.0, 46.0), (122.0, 45.0),
        ];
        for &(tx, ty) in trees {
            let (sx, sy) = self.cam.world_to_screen(tx * TILE_SIZE, ty * TILE_SIZE);
            // Stamm
            draw_rectangle(sx + 6.0, sy + 8.0, 3.0, 6.0,
                Color::new(0.32, 0.20, 0.12, 1.0));
            // Krone
            draw_circle(sx + 8.0, sy + 6.0, 6.0, Color::new(0.18, 0.45, 0.20, 1.0));
            draw_circle(sx + 5.0, sy + 7.0, 4.0, Color::new(0.22, 0.55, 0.25, 1.0));
            draw_circle(sx + 11.0, sy + 7.0, 4.0, Color::new(0.20, 0.50, 0.22, 1.0));
        }
        // Park-Bänke
        let benches: &[(f32, f32)] = &[
            (102.0, 42.0), (115.0, 42.0), (108.0, 48.0),
        ];
        for &(bx, by) in benches {
            let (sx, sy) = self.cam.world_to_screen(bx * TILE_SIZE, by * TILE_SIZE);
            draw_rectangle(sx + 2.0, sy + 7.0, 12.0, 2.0, Color::new(0.45, 0.30, 0.18, 1.0));
            draw_rectangle(sx + 2.0, sy + 4.0, 12.0, 1.0, Color::new(0.50, 0.32, 0.20, 1.0));
            draw_rectangle(sx + 2.0, sy + 9.0, 1.0, 3.0, Color::new(0.25, 0.18, 0.10, 1.0));
            draw_rectangle(sx + 13.0, sy + 9.0, 1.0, 3.0, Color::new(0.25, 0.18, 0.10, 1.0));
        }
        // Springbrunnen-Mosaik um den Jakobusbrunnen
        let (fsx, fsy) = self.cam.world_to_screen(115.0 * TILE_SIZE, 35.0 * TILE_SIZE);
        draw_circle(fsx + 8.0, fsy + 8.0, 11.0, Color::new(0.75, 0.75, 0.78, 0.5));
        draw_circle(fsx + 8.0, fsy + 8.0, 9.0, Color::new(0.55, 0.55, 0.58, 0.6));
    }

    /// Zeichnet alle Welt-bezogenen Beschriftungen (Stadthalle, Polariom, …)
    /// im UI-Pass mit nativer Bildschirmauflösung — kein Aliasing.
    fn draw_world_labels(&self, ui: &ui::UiCtx) {
        let w2s = |wx: f32, wy: f32| self.cam.world_to_screen(wx, wy);

        // Schwebende Landmark-Schilder. Format: (world_center_x, world_y_unten, txt, fg, bg)
        let signs: [(f32, f32, &str, Color, Color); 12] = [
            (
                81.0 * TILE_SIZE, 41.0 * TILE_SIZE,
                "Stadthalle",
                Color::new(1.0, 0.95, 0.85, 1.0),
                Color::new(0.16, 0.24, 0.50, 1.0),
            ),
            (
                103.0 * TILE_SIZE, 49.0 * TILE_SIZE,
                "Rathaus Germering",
                Color::new(1.0, 0.88, 0.30, 1.0),
                Color::new(0.55, 0.18, 0.12, 1.0),
            ),
            (
                127.0 * TILE_SIZE, 49.0 * TILE_SIZE,
                "Schule",
                Color::new(1.0, 1.0, 1.0, 1.0),
                Color::new(0.55, 0.18, 0.12, 1.0),
            ),
            (
                122.0 * TILE_SIZE, 62.0 * TILE_SIZE,
                "Stadtmuseum ZEIT+RAUM",
                Color::new(1.0, 0.88, 0.30, 1.0),
                Color::new(0.40, 0.22, 0.10, 1.0),
            ),
            (
                168.0 * TILE_SIZE, 51.0 * TILE_SIZE,
                "Klinikum",
                Color::new(0.85, 0.18, 0.18, 1.0),
                Color::new(0.98, 0.98, 0.98, 1.0),
            ),
            (
                178.0 * TILE_SIZE, 76.0 * TILE_SIZE,
                "Polariom Eishalle",
                Color::new(0.95, 0.98, 1.0, 1.0),
                Color::new(0.20, 0.45, 0.70, 1.0),
            ),
            (
                120.0 * TILE_SIZE, 4.0 * TILE_SIZE,
                "Bahnhof Germering S8",
                Color::new(0.95, 0.98, 0.85, 1.0),
                Color::new(0.06, 0.30, 0.10, 1.0),
            ),
            (
                68.0 * TILE_SIZE, 5.0 * TILE_SIZE,
                "Bahnhof Harthaus S8",
                Color::new(0.95, 0.98, 0.85, 1.0),
                Color::new(0.06, 0.30, 0.10, 1.0),
            ),
            (
                115.0 * TILE_SIZE, 30.0 * TILE_SIZE,
                "St. Jakobskirche",
                Color::new(0.95, 0.95, 0.95, 1.0),
                Color::new(0.18, 0.18, 0.22, 1.0),
            ),
            (
                50.0 * TILE_SIZE, 41.0 * TILE_SIZE,
                "GEP Einkaufspassagen",
                Color::new(0.20, 0.18, 0.10, 1.0),
                Color::new(1.0, 0.85, 0.15, 1.0),
            ),
            (
                146.0 * TILE_SIZE, 49.0 * TILE_SIZE,
                "Freibad Germering",
                Color::new(0.95, 0.98, 1.0, 1.0),
                Color::new(0.20, 0.55, 0.80, 1.0),
            ),
            (
                192.0 * TILE_SIZE, 30.0 * TILE_SIZE,
                "Tankstelle Cewestr.",
                Color::new(0.20, 0.18, 0.10, 1.0),
                Color::new(1.0, 0.85, 0.15, 1.0),
            ),
        ];

        let size = 9.0; // konstante, gut lesbare Größe

        for &(cx, by, txt, fg, bg) in &signs {
            let tw = ui.text_w(txt, size);
            let pad = 4.0;
            let sign_w = tw + pad * 2.0;
            let sign_h = size + 4.0;
            let sign_world_x = cx - sign_w / 2.0;
            let sign_world_y = by - sign_h;
            let (sx, sy) = w2s(sign_world_x, sign_world_y);

            // Off-Screen-Check (am Schild-Anker, nicht am Text)
            if sx < -sign_w - 20.0 || sx > VIRTUAL_W as f32 + 20.0
                || sy < -sign_h - 20.0 || sy > VIRTUAL_H as f32 + 20.0
            {
                continue;
            }

            // Holz-Stange unten — verankert das Schild visuell am Gebäude
            ui.rect(sx + sign_w / 2.0 - 1.0, sy + sign_h, 2.0, 5.0,
                Color::new(0.30, 0.20, 0.12, 1.0));
            // Schild-Schatten
            ui.rect(sx + 1.0, sy + 1.0, sign_w, sign_h,
                Color::new(0.0, 0.0, 0.0, 0.4));
            // Schild-Rahmen (dunkel)
            ui.rect(sx - 1.0, sy - 1.0, sign_w + 2.0, sign_h + 2.0,
                Color::new(0.10, 0.08, 0.06, 1.0));
            // Schild-Fläche
            ui.rect(sx, sy, sign_w, sign_h, bg);
            // Glanzlinie oben
            ui.rect(sx, sy, sign_w, 1.5, Color::new(1.0, 1.0, 1.0, 0.18));
            // Text
            ui.text(txt, sx + pad, sy + size + 1.0, size, fg);
        }

        // IHLE-Logo auf den 4 Filialen (über dem Gebäude, kompakter Stil)
        for f in FILIALEN.iter() {
            let bx = f.tile_x as f32 * TILE_SIZE;
            let by = f.tile_y as f32 * TILE_SIZE;
            // Position über dem Gebäude (Tile-Mitte, ca. 10px höher)
            let sign_world_y = by - 12.0;
            let logo_w = 32.0;
            let logo_h = 11.0;
            let sign_world_x = bx + 8.0 - logo_w / 2.0;
            let (sx, sy) = w2s(sign_world_x, sign_world_y);
            if sx < -logo_w - 20.0 || sx > VIRTUAL_W as f32 + 20.0
                || sy < -30.0 || sy > VIRTUAL_H as f32 + 20.0
            {
                continue;
            }
            // Halterung
            ui.rect(sx + 2.0, sy + logo_h, 1.5, 5.0,
                Color::new(0.25, 0.25, 0.25, 1.0));
            ui.rect(sx + logo_w - 3.5, sy + logo_h, 1.5, 5.0,
                Color::new(0.25, 0.25, 0.25, 1.0));
            // Schild
            ui.rect(sx - 0.5, sy - 0.5, logo_w + 1.0, logo_h + 1.0,
                Color::new(0.10, 0.05, 0.04, 1.0));
            ui.rect(sx, sy, logo_w, logo_h, Color::new(0.55, 0.12, 0.12, 1.0));
            ui.rect(sx + 1.0, sy + 1.0, logo_w - 2.0, logo_h - 2.0,
                Color::new(1.0, 0.85, 0.15, 1.0));
            ui.rect(sx + 1.0, sy + 1.0, logo_w - 2.0, 2.0,
                Color::new(1.0, 0.95, 0.55, 1.0));
            ui.text("IHLE", sx + 7.0, sy + 9.0, 10.0,
                Color::new(0.55, 0.12, 0.12, 1.0));
        }

        // Quest-Marker über Klaus, solange Quest 0 läuft
        if self.quest_stage == 0 && !self.klaus_tour_done {
            for n in &self.npcs {
                if n.kind == npc::NpcKind::Klaus {
                    let (sx, sy) = w2s(n.aabb.x + 4.0, n.aabb.y - 16.0);
                    let bob = (self.game_seconds * 4.0).sin() * 1.5;
                    let alpha = 0.9;
                    // Sprechblase mit "!"
                    ui.rect(sx - 4.0, sy + bob - 1.0, 14.0, 14.0,
                        Color::new(0.0, 0.0, 0.0, 0.7));
                    ui.text("!", sx + 1.0, sy + 11.0 + bob, 13.0,
                        Color::new(1.0, 0.85, 0.15, alpha));
                    break;
                }
            }
            self.draw_offscreen_klaus_arrow(ui);
        }

        // --- Bus-Linien-Nummern (auf dem Schild vorne am Bus) ---
        for b in &self.buses {
            let sign_x_off = match b.dir { BusDir::East => 16.0, BusDir::West => 2.0 };
            let (sx, sy) = w2s(b.x + sign_x_off, b.y);
            if sx < -20.0 || sx > VIRTUAL_W as f32 + 20.0
                || sy < -20.0 || sy > VIRTUAL_H as f32 + 20.0
            {
                continue;
            }
            ui.text(BUS_LINES[b.line_idx].number, sx + 1.0, sy + 4.0, 7.0,
                Color::new(1.0, 0.85, 0.15, 1.0));
        }

        // --- Bus HUP! + Schimpfwort-Sprechblasen ---
        for b in &self.buses {
            let (sx, sy) = w2s(b.x, b.y);
            if sx < -120.0 || sx > VIRTUAL_W as f32 + 20.0 {
                continue;
            }
            if b.honk_t > 0.0 {
                let alpha = (b.honk_t / 2.0).clamp(0.0, 1.0);
                let bob = (self.game_seconds * 18.0).sin() * 0.8;
                ui.rect(sx + 2.0, sy - 12.0 + bob, 24.0, 10.0,
                    Color::new(0.0, 0.0, 0.0, 0.65 * alpha));
                ui.text("HUP!", sx + 4.0, sy - 4.0 + bob, 9.0,
                    Color::new(1.0, 0.85, 0.15, alpha));
            }
            if b.swear_t > 0.0 {
                let alpha = (b.swear_t / 2.5).clamp(0.0, 1.0).powf(0.6);
                let txt = b.swear_text.as_str();
                let size = 9.0;
                let tw = ui.text_w(txt, size);
                let pad = 4.0;
                let box_w = tw + pad * 2.0;
                let box_h = size + 4.0;
                // Blase oberhalb des Busses
                ui.rect(sx + 1.0, sy - box_h - 6.0, box_w, box_h,
                    Color::new(0.0, 0.0, 0.0, 0.0)); // shadow placeholder
                // Schatten + Hintergrund
                ui.rect(sx + 2.0, sy - box_h - 5.0, box_w, box_h,
                    Color::new(0.0, 0.0, 0.0, 0.6 * alpha));
                ui.rect(sx, sy - box_h - 7.0, box_w, box_h,
                    Color::new(0.95, 0.92, 0.88, alpha));
                ui.rect(sx, sy - box_h - 7.0, box_w, 1.5,
                    Color::new(1.0, 1.0, 1.0, alpha));
                // Schwänzchen
                ui.rect(sx + 6.0, sy - 7.0, 2.0, 2.0,
                    Color::new(0.95, 0.92, 0.88, alpha));
                ui.text(txt, sx + pad, sy - box_h + size - 4.0, size,
                    Color::new(0.55, 0.10, 0.10, alpha));
            }
        }

        // --- Pedestrian-Sprechblasen (Säufer-Schimpfworte) ---
        for p in &self.pedestrians {
            if p.bubble_t <= 0.0 || p.bubble_text.is_empty() {
                continue;
            }
            let (sx, sy) = w2s(p.aabb.x, p.aabb.y);
            if sx < -100.0 || sx > VIRTUAL_W as f32 + 20.0 {
                continue;
            }
            let alpha = (p.bubble_t / 2.0).clamp(0.0, 1.0).powf(0.6);
            let txt = p.bubble_text.as_str();
            let size = 9.0;
            let tw = ui.text_w(txt, size);
            let pad = 4.0;
            let box_w = tw + pad * 2.0;
            let box_h = size + 4.0;
            // Schatten
            ui.rect(sx + 2.0, sy - box_h - 4.0, box_w, box_h,
                Color::new(0.0, 0.0, 0.0, 0.5 * alpha));
            // Sprechblasen-Hintergrund (weiß)
            ui.rect(sx, sy - box_h - 6.0, box_w, box_h,
                Color::new(0.98, 0.95, 0.92, alpha));
            ui.rect(sx, sy - box_h - 6.0, box_w, 1.5,
                Color::new(1.0, 1.0, 1.0, alpha));
            // Schwänzchen nach unten
            ui.rect(sx + 4.0, sy - 6.0, 2.5, 2.5,
                Color::new(0.98, 0.95, 0.92, alpha));
            // Text (dunkelrot für Säufer)
            ui.text(txt, sx + pad, sy - box_h + size - 5.0, size,
                Color::new(0.55, 0.10, 0.10, alpha));
        }

        // --- "S8" auf der Lok ---
        if self.train_phase != TrainPhase::Idle {
            let train_world_y = TRAIN_TRACK_TILE_Y as f32 * TILE_SIZE;
            let (sx, sy) = w2s(self.train_x + 6.0, train_world_y + 2.0);
            if sx > -20.0 && sx < VIRTUAL_W as f32 + 20.0 {
                ui.text("S8", sx, sy + 9.0, 9.0,
                    Color::new(1.0, 0.95, 0.55, 1.0));
            }
        }
    }

    /// Zeigt einen Pfeil am Bildschirmrand Richtung Klaus, falls offscreen.
    fn draw_offscreen_klaus_arrow(&self, ui: &ui::UiCtx) {
        let mut klaus_pos: Option<(f32, f32)> = None;
        for n in &self.npcs {
            if n.kind == npc::NpcKind::Klaus {
                klaus_pos = Some((n.aabb.x + 6.0, n.aabb.y + 7.0));
            }
        }
        let Some((kx, ky)) = klaus_pos else { return };
        let (sx, sy) = self.cam.world_to_screen(kx, ky);
        // Bereits sichtbar?
        if sx > 0.0 && sx < VIRTUAL_W as f32 && sy > 0.0 && sy < VIRTUAL_H as f32 {
            return;
        }
        let cx = VIRTUAL_W as f32 / 2.0;
        let cy = VIRTUAL_H as f32 / 2.0;
        let dx = sx - cx;
        let dy = sy - cy;
        // Clamp auf einen Kreis am Rand
        let max_d = 120.0;
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        let scale = max_d / d;
        let ax = cx + dx * scale;
        let ay = cy + dy * scale;
        ui.rect(ax - 8.0, ay - 7.0, 60.0, 14.0,
            Color::new(0.0, 0.0, 0.0, 0.75));
        ui.text("Klaus →", ax - 6.0, ay + 4.0, 11.0,
            Color::new(1.0, 0.85, 0.15, 1.0));
    }

    /// Render-Pass auf den echten Bildschirm — gestochen scharfer UI-Text.
    fn draw_ui_pass(&self, ui: &ui::UiCtx) {
        match self.state {
            GameState::TitleMenu => {
                ui::draw_title_menu(ui, self.menu_cursor, save::exists());
            }
            GameState::Intro => {
                ui::draw_intro(ui, self.intro_scroll);
            }
            GameState::Credits | GameState::Victory => {
                ui::draw_victory(ui, self.victory_scroll);
            }
            _ => {
                // Welt-Labels (Stadthalle, Polariom, IHLE…) — gestochen scharf
                self.draw_world_labels(ui);
                if let Some(b) = &self.boss {
                    if b.alive() {
                        ui::draw_boss_bar(ui, b.hp, BOSS_HP_MAX, b.current_taunt, b.taunt_t);
                    }
                }
                ui::draw_hud(
                    ui,
                    &self.tex,
                    &self.player,
                    self.game_seconds,
                    &self.area_name,
                    self.area_fade,
                    self.cordobar_unlocked,
                    self.purchases,
                    self.crystals,
                    &self.quest_hint,
                    self.quest_hint_t,
                );
                if matches!(self.state, GameState::Playing) {
                    self.draw_interact_hints(ui);
                    if !self.save_message.is_empty() && self.save_message_t > 0.0 {
                        ui::draw_toast(ui, &self.save_message, Color::new(0.5, 1.0, 0.5, 1.0));
                    }
                    if !self.bus_ride_msg.is_empty() && self.bus_ride_msg_t > 0.0 {
                        ui::draw_toast(ui, &self.bus_ride_msg, Color::new(1.0, 0.85, 0.30, 1.0));
                    }
                    // S-Bahn-Fahrt-Übergang
                    if self.sbahn_ride_t > 0.0 {
                        let pct = (self.sbahn_ride_t / 1.6).clamp(0.0, 1.0);
                        // Fade-In dann Fade-Out
                        let fade = if pct > 0.5 { (1.0 - pct) * 2.0 } else { 1.0 - pct * 2.0 + 1.0 };
                        let alpha = fade.clamp(0.0, 1.0);
                        ui.fullscreen_rect(Color::new(0.0, 0.0, 0.0, alpha * 0.92));
                        let to = station_name(self.sbahn_ride_to);
                        let s1 = format!("S8 → {}", to);
                        let s2 = "Tür schließt automatisch.";
                        let tw = ui.text_w(&s1, 22.0);
                        ui.text(
                            &s1,
                            VIRTUAL_W as f32 / 2.0 - tw / 2.0,
                            VIRTUAL_H as f32 / 2.0,
                            22.0,
                            Color::new(1.0, 0.85, 0.15, alpha),
                        );
                        let tw2 = ui.text_w(s2, 10.0);
                        ui.text(
                            s2,
                            VIRTUAL_W as f32 / 2.0 - tw2 / 2.0,
                            VIRTUAL_H as f32 / 2.0 + 18.0,
                            10.0,
                            Color::new(0.95, 0.95, 0.95, alpha),
                        );
                    }
                }
                if self.minimap_open {
                    ui::draw_minimap(ui, &self.world, &self.player);
                }
                if matches!(self.state, GameState::Paused) {
                    ui::draw_paused(ui, self.pause_cursor);
                }
                if matches!(self.state, GameState::Shopping) {
                    if let Some(s) = &self.shop {
                        ui::draw_shop(ui, &self.tex, s, &self.player, self.game_seconds, self.purchases);
                    }
                }
                if matches!(self.state, GameState::Dialog) {
                    if let Some(d) = &self.dialog {
                        ui::draw_dialog(ui, &self.tex, &self.npcs[d.npc_idx], d);
                    }
                }
                if matches!(self.state, GameState::GameOver) {
                    ui::draw_gameover(ui);
                }
            }
        }
    }
}

/// Spawnt ~20 wandernde NPCs verteilt über die Stadt.
/// 13 freundliche Bürger + 7 Säufer in Cordobar / Stadtmitte / Stadtpark.
fn spawn_pedestrians() -> Vec<Pedestrian> {
    let outfits: [Color; 6] = [
        Color::new(0.85, 0.20, 0.20, 1.0),
        Color::new(0.20, 0.45, 0.85, 1.0),
        Color::new(0.30, 0.70, 0.30, 1.0),
        Color::new(0.85, 0.60, 0.20, 1.0),
        Color::new(0.65, 0.40, 0.75, 1.0),
        Color::new(0.40, 0.30, 0.20, 1.0),
    ];

    // (tx, ty, kind) — mehr Bürger, deutlich weniger Säufer (3, nur an Cordobar)
    let pos: &[(f32, f32, PedKind)] = &[
        // Friedenstr. / Stadtmitte
        (88.0, 64.0, PedKind::Citizen),
        (97.0, 67.0, PedKind::Citizen),
        (104.0, 73.0, PedKind::Citizen),
        (112.0, 65.0, PedKind::Citizen),
        (95.0, 81.0, PedKind::Citizen),
        (88.0, 84.0, PedKind::Citizen),
        // GEP
        (40.0, 56.0, PedKind::Citizen),
        (47.0, 70.0, PedKind::Citizen),
        (35.0, 64.0, PedKind::Citizen),
        // Bahnhof
        (100.0, 22.0, PedKind::Citizen),
        (118.0, 22.0, PedKind::Citizen),
        (90.0, 25.0, PedKind::Citizen),
        (112.0, 30.0, PedKind::Citizen),
        // Stadtpark
        (115.0, 40.0, PedKind::Citizen),
        (122.0, 45.0, PedKind::Citizen),
        (105.0, 38.0, PedKind::Citizen),
        // Cewestr
        (170.0, 35.0, PedKind::Citizen),
        (180.0, 30.0, PedKind::Citizen),
        // Polariom-Umgebung
        (148.0, 85.0, PedKind::Citizen),
        // Parsberg
        (15.0, 70.0, PedKind::Citizen),
        // Säufer: NUR an der Cordobar-Ruine (klassischer Treffpunkt)
        (22.0, 100.0, PedKind::Drunk),
        (28.0, 104.0, PedKind::Drunk),
        (16.0, 107.0, PedKind::Drunk),
    ];

    let mut peds = Vec::new();
    for (i, &(tx, ty, kind)) in pos.iter().enumerate() {
        peds.push(Pedestrian {
            kind,
            aabb: Aabb::new(tx * TILE_SIZE, ty * TILE_SIZE, 12.0, 14.0),
            vx: 0.0,
            vy: 0.0,
            wander_t: 0.0,
            hit_cool: 0.0,
            phase: i as f32 * 0.73,
            bubble_t: 0.0,
            bubble_text: String::new(),
            tint: outfits[i % outfits.len()],
        });
    }
    peds
}

// ------------------------------------------------------------------------
//  Hauptschleife
// ------------------------------------------------------------------------

#[macroquad::main(window_conf)]
async fn main() {
    // Texturen
    let tex = build_textures().await;
    // Audio
    let audio = Audio::load().await;

    // Render-Target für virtuelle 480×270-Canvas
    let target = render_target(VIRTUAL_W as u32, VIRTUAL_H as u32);
    target.texture.set_filter(FilterMode::Nearest);

    // Kamera, die in den Render-Target zeichnet
    let mut virt_camera = Camera2D::from_display_rect(Rect::new(
        0.0,
        0.0,
        VIRTUAL_W as f32,
        VIRTUAL_H as f32,
    ));
    virt_camera.render_target = Some(target.clone());

    let mut game = Game::new(tex, audio);

    // 60-FPS-Cap: schlafen bis das Ziel-Frame-Zeit-Budget aufgebraucht ist.
    let target_frame_secs: f32 = 1.0 / 60.0;

    loop {
        let frame_start = std::time::Instant::now();
        let dt = get_frame_time().min(0.05);

        // ESC: Hauptmenü = beenden, Spielen = pausieren, Pause = weiterspielen
        // update_paused fängt ESC selbst NICHT mehr ab — sonst Race-Condition.
        if is_key_pressed(KeyCode::Escape) {
            match game.state {
                GameState::TitleMenu => {
                    std::process::exit(0);
                }
                GameState::Playing => {
                    game.state = GameState::Paused;
                    game.pause_cursor = 0;
                }
                GameState::Paused => {
                    game.state = GameState::Playing;
                    game.pause_cursor = 0;
                }
                _ => {}
            }
        }

        // State-Update
        match game.state {
            GameState::TitleMenu => game.update_title(save::exists()),
            GameState::Intro => game.update_intro(dt),
            GameState::Playing => game.update_playing(dt),
            GameState::Paused => game.update_paused(),
            GameState::Shopping => game.update_shopping(dt),
            GameState::Dialog => game.update_dialog(dt),
            GameState::GameOver => game.update_gameover(),
            GameState::Victory => game.update_victory(dt),
            GameState::Credits => game.update_credits(dt),
        }

        // === Pass 1: Pixel-Welt auf virtuelle 480×270-Canvas ===
        set_camera(&virt_camera);
        game.draw_world_pass();

        // === Pass 2: Bildschirm-Space — UI direkt in nativer Auflösung ===
        set_default_camera();
        clear_background(BLACK);

        let aspect = VIRTUAL_W as f32 / VIRTUAL_H as f32;
        let sw = screen_width();
        let sh = screen_height();
        let (w, h) = if sw / sh > aspect {
            (sh * aspect, sh)
        } else {
            (sw, sw / aspect)
        };
        let dx = (sw - w) / 2.0;
        let dy = (sh - h) / 2.0;

        // Pixel-Welt skaliert hochziehen (Nearest-Filter sorgt für scharfe Pixel)
        if game.has_world() {
            draw_texture_ex(
                &target.texture,
                dx,
                dy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    flip_y: true,
                    ..Default::default()
                },
            );
        }

        // UI in nativer Auflösung darüber — Text wird hier scharf gerastert
        let ui_ctx = ui::UiCtx::new();
        game.draw_ui_pass(&ui_ctx);

        // FPS-Cap: warte bis das 60-Hz-Slot vorbei ist (CPU/GPU-schonend).
        let elapsed = frame_start.elapsed().as_secs_f32();
        if elapsed < target_frame_secs {
            std::thread::sleep(std::time::Duration::from_secs_f32(target_frame_secs - elapsed));
        }

        next_frame().await
    }
}
