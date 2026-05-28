# Germering Delivery — Projektkontext für Claude Code

> Diese Datei wird von Claude Code automatisch bei jedem Sessionstart eingelesen.
> Keine Rückfragen stellen — alle Entscheidungen selbstständig treffen.

---

## Verhalten

- **AUTORUN: true** — Selbstständig arbeiten, kein Nachfragen
- Nur bei kritischen, nicht umkehrbaren Fehlern kurz nachfragen
- Inkompatible Library-Versionen eigenständig lösen
- `cargo run` muss nach jeder Änderung funktionieren
- `cargo clippy` soll ohne Warnings durchlaufen
- Alle UI-Texte auf **Deutsch** (Ausnahme: Jannicks Kölsch-Dialoge)

---

## Projekt

**Name:** Germering Delivery  
**Sprache:** Rust  
**Engine:** Bevy (aktuellste stabile Version)  
**Genre:** Pixelart 2D Top-Down Highscore-Spiel  
**Thema:** Lieferfahrer in Germering (Stadt bei München) fährt mit einem Ihle-Sprinter Waren aus

---

## Tech-Stack (Cargo.toml)

| Crate | Zweck |
|---|---|
| `bevy` | Game Engine, Rendering, ECS |
| `bevy_ecs_tilemap` | Tilemap-Rendering für die Spielwelt |
| `bevy_asset_loader` | Asset-Management |
| `rand` | Zufallsgeneratoren (NPC-Bewegung, Aufträge) |
| `serde` + `serde_json` | Highscore-Persistenz in `highscore.json` |

**Assets:** Alle Sprites als programmatisch generierte Bevy-Texturen oder hardcoded Pixel-Byte-Arrays. **Kein externes PNG oder Audio-File nötig.**  
Motorgeräusch via programmatisch erzeugter Sinus-Welle (Bevy Audio).

---

## Projektstruktur

```
src/
  main.rs            — Bevy App Setup, Plugin-Registrierung, GameState
  game/
    mod.rs
    player.rs        — Sprinter-Steuerung, Trägheit, Kollision, Speed-Boost
    map.rs           — Tilemap, Gebäude-Placement, Landmark-Definitionen
    delivery.rs      — Auftrags-System, Schwierigkeitsskalierung, Leben-System
    npc.rs           — NPC-Routen, State-Machine, Jannick-Dialoge
    shop.rs          — Jannick's Shop UI, Kauf-Logik, Speed-Boost-System
    navi.rs          — Navi-Overlay, Pfadberechnung, Richtungspfeil-Logik
    hud.rs           — HUD, Timer, Leben-Anzeige (3 Sprinter-Icons), Minimap
    highscore.rs     — Score-Berechnung, Top-5-Tabelle, JSON-Persistenz
    gamestate.rs     — GameState-Enum: MainMenu / Playing / GameOver
    assets.rs        — Alle Sprites, Texturen, Audio-Generierung
highscore.json       — Wird automatisch angelegt (Top 5 Einträge)
README.md            — Steuerung, Spielziel, Highscore-Erklärung
```

---

## Spielwelt: Germering (München)

Topdown-Karte als Pixelart-Tilemap (mind. 128×128 Tiles), orientiert an der echten Stadt Germering.

**Pflicht-Landmarks:**
- Rathaus Germering (Kirchenplatz)
- Bahnhof Germering (S-Bahn S8)
- Aldi Süd (Untere Bahnhofstr.)
- Rewe (Germering Zentrum)
- Hauptstraße / Untere Bahnhofstraße / Eichenauer Straße als Hauptachsen
- 4 Ihle-Filialen (Bahnhof, Zentrum, Nord, Süd)
- Wohngebiete (Wohnblöcke + Einfamilienhäuser)
- Grünflächen / Germering-Nord Park

---

## Fahrzeug: Ihle Sprinter

- Sprite: weißer Kastenwagen (16×16 oder 24×16 Pixel, Topdown)
  mit Ihle-Logo auf der Seite (blau/weiß Pixel-Schriftzug)
- 4 Richtungen, je 2 Frames (Räder drehen sich leicht)
- Leichte Fahr-Physik: Trägheit beim Beschleunigen und Bremsen (kein Drift)
- Kollision mit Gebäuden und Bordsteinen — Sprinter prallt ab
- NPCs weichen dem Sprinter mit größerem Radius aus

---

## Steuerung

| Taste | Aktion |
|---|---|
| WASD / Pfeiltasten | Bewegen |
| E | Interaktion (Abholen / Abliefern / Kaufen) |
| ESC | Pause-Menü |

---

## Gameplay-Ablauf

1. Auftrag erscheint: Ihle-Filiale (Abholpunkt) + Kundenadresse
2. Spieler fährt zur Ihle-Filiale → `[E]` → Ware geladen (Paket-Symbol am Sprinter)
3. Spieler fährt zur Kundenadresse → `[E]` → Lieferung abgeschlossen
4. Score + Zeitbonus gutgeschrieben, nächster Auftrag sofort
5. Countdown-Timer oben mittig, wird rot unter 10 Sek

---

## Navi-Overlay (in-game)

Erscheint nur bei aktiver Lieferung. Position: HUD unten-mitte, ca. 120×90 Pixel, leicht transparent.

- Zeigt vereinfachten Kartenausschnitt (~5×5 Tiles) zentriert auf Sprinter
- Grüner Richtungspfeil zum Ziel (dreht sich relativ zur Fahrtrichtung)
- Blinkendes rotes X als Ziel-Marker
- Entfernungsanzeige: Tile-Distanz × 10 = „Meter" (z. B. `→ GERADEAUS 320m`)
- Dunkler Pixel-Rahmen, abgerundete Ecken
- `BOOST AKTIV` Anzeige wenn Speed-Boost läuft

---

## Highscore & Schwierigkeitssystem

### Zeitlimit-Skalierung

| Lieferungen | Zeitlimit | Stimmung |
|---|---|---|
| 1–5 | 60 Sek | Kunden entspannt |
| 6–10 | 50 Sek | Kunden leicht ungeduldig |
| 11–20 | 40 Sek | Kunden nervig |
| 21–35 | 30 Sek | Kunden aggressiv |
| 36+ | 22 Sek | Kunden extrem |

Ab Lieferung 10: mehr NPCs als Hindernisse.  
Ab Lieferung 20: Ziel-Adressen weiter vom Abholpunkt entfernt.

### Leben-System

- 3 Leben = 3 Mini-Sprinter-Icons im HUD (oben rechts)
- Leben verloren wenn: Zeitlimit abläuft
- Einblendung: `"ZU SPÄT! Kunde wartet nicht mehr!"` (roter Flash, 2 Sek)
- **3 Leben weg → Game Over**

### Punkte-System

- Basis: 100 Punkte pro Lieferung
- Zeitbonus: verbleibende Sekunden × 5 Punkte
- Streak-Bonus: 3x hintereinander ohne Leben-Verlust → ×1.5 Multiplikator
- Speed-Boost bei Lieferung aktiv → +20 Bonus-Punkte

### Game-Over-Screen

- Pixelart-Schild: **„FEIERABEND!"**
- Anzeige: Deine Lieferungen + Dein Score
- Highscore-Tabelle (Top 5, aus `highscore.json`)
- Falls neuer Highscore: **„NEUER REKORD!"** blinkt golden
- Buttons: `[NOCHMAL SPIELEN]` und `[BEENDEN]`

---

## NPCs

### Allgemeine NPCs (mind. 8, skaliert ab Lieferung 10)

- Laufen auf Bürgersteigen, weichen Sprinter aus
- State-Machine: `walk → idle → walk`
- Sprites: Mann, Frau, Kind, Rentner (je 2 Richtungen)

### Sonder-NPC: Jannick

**Standort:** Pizzeria `"Jannicks Kölner Eck"` an der Hauptstraße  
**Erkennungszeichen:** Rot/weiß gestreiftes Markisen-Tile, Kölner Dom Pixel-Icon  
**Sprache:** Ausschließlich Kölsch

**Dialog-Pool (zufällig bei `[E]`):**
```
"Wat willste? Ne Pizza odder nen Kaffee? Hä?"
"Ich sach dir, Germering is nix. Kölle! DAS is ne Stadt!"
"Lecker Kölsch hät ich och, ävver dat verstehs du nit."
"Kauf dir wat, ich han Hunger un muss schließe!"
"Em Kölle am Rhing, da simmer doheem. Hier nit, ävver jut."
"Ey, dä Sprinter kütt vill ze schnell, pass op Minsch!"
```

**Shop-Menü** (öffnet sich bei `[E]`, Spiel pausiert):

| Item | Preis | Effekt |
|---|---|---|
| Margherita Pizza | 5 Münzen | +30 Sek Speed-Boost (+40% Bewegung) |
| „Geiler Kaffee" | 3 Münzen | +20 Sek Speed-Boost (+25% Bewegung) |

---

## HUD-Layout

```
┌─────────────────────────────────────────────────────┐
│  ⏱ 00:42   Score: 2.450   💰 12 Münzen   🚐 🚐 🚐  │
│  Auftrag: Ihle Bahnhof → Hauptstr. 14               │
└─────────────────────────────────────────────────────┘
                    [Spielwelt]
┌──────────────────┐              ┌──────────────┐
│  NAVI            │              │  [Minimap]   │
│  → GERADEAUS     │              │              │
│    320m          │              └──────────────┘
└──────────────────┘
```

---

## Visueller Stil

- 16×16 Pixel Tiles, SNES/GBC-Feeling
- Farbpalette: Erdtöne für Gebäude, Grün für Parks, Grau für Asphalt
- Ihle-Filialen: blau/weißes Pixel-Schild
- Jannicks Pizzeria: rot/weiß Markise, Kölner Dom Pixel-Icon
- Spieler-Sprinter: blinkt bei Speed-Boost aktiv

---

## Wichtige Hinweise

- Highscore wird in `highscore.json` im Projektordner gespeichert
- README.md muss Steuerung, Spielziel und Highscore-Erklärung enthalten
- Tag/Nacht-Zyklus optional (nice-to-have ab Lieferung 50)
