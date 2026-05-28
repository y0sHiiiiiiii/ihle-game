# Germering Delivery

Pixelart 2D Top-Down Highscore-Spiel in Rust + Bevy.
Du fährst einen weißen Ihle-Sprinter durch Germering bei München und lieferst
Pakete von einer von vier Ihle-Filialen an Kundenadressen aus.

## Spielziel

Liefere so viele Pakete wie möglich aus, bevor drei Kunden zu lange auf
ihre Bestellung warten mussten. Je weniger Zeit du brauchst, desto mehr
Punkte und Münzen gibt es. Fahre sauber und schnell, um deinen **Nitro**
aufzuladen, und halte deine **Streak** am Leben, damit der Multiplikator
explodiert. Münzen kannst du bei Jannicks Kölnem Eck in Pizza oder Kaffee
umtauschen — beides gibt einen zusätzlichen Speed-Boost.

## Steuerung

| Taste | Aktion |
|---|---|
| W / A / S / D oder Pfeiltasten | Sprinter fahren (mit Trägheit) |
| **LEERTASTE** (im Spiel) | **Nitro-Schub zünden** (wenn die Leiste voll ist) |
| E | Interagieren (Paket abholen, abliefern, Shop öffnen, Pizza/Kaffee kaufen) |
| 1 / 2 (im Shop) | Pizza / Kaffee kaufen |
| ESC | Pause (im Spiel) bzw. Shop verlassen |
| LEERTASTE / ENTER | Spiel starten bzw. nach Game Over neu starten |
| Q | Spiel beenden |
| R (im Game-Over) | Sofort neu starten |

## Nitro

Unten links siehst du die **Nitro-Leiste**. Sie lädt sich, während du fährst —
auf der Straße schneller als querfeldein —, und jede saubere Lieferung gibt
einen kräftigen Schub dazu. Ist sie voll (`NITRO BEREIT`), zündest du mit
`LEERTASTE` einen 3-Sekunden-Schub (+60 % Tempo), perfekt um eine knappe
Lieferung doch noch rechtzeitig zu schaffen. Der Sprinter blinkt und zieht
eine Abgasfahne, solange ein Boost aktiv ist.

## Spielablauf

1. Im Hauptmenü mit `LEERTASTE` ins Spiel starten.
2. Ein **blauer Pickup-Marker** erscheint über einer der vier Ihle-Filialen
   (Bahnhof, Zentrum, Nord, Süd). Fahre dort hin und drücke `E`, um das
   Paket zu laden.
3. Ein **rotes blinkendes X** erscheint an der Kundenadresse. Fahre dort
   hin und drücke `E`, um zu liefern.
4. Score- und Münz-Belohnungen werden gutgeschrieben, der nächste Auftrag
   startet sofort.
5. Wenn der Timer abläuft, verlierst du ein Leben. Nach drei verlorenen
   Leben ist Feierabend.

## Schwierigkeitsskalierung

| Lieferungen | Zeitlimit | Stimmung |
|---|---|---|
| 1–5 | 60 Sek | Kunden entspannt |
| 6–10 | 50 Sek | Kunden leicht ungeduldig |
| 11–20 | 40 Sek | Kunden nervig |
| 21–35 | 30 Sek | Kunden aggressiv |
| 36+ | 22 Sek | Kunden extrem |

Ab Lieferung 20 liegen Ziel-Adressen deutlich weiter vom Abholpunkt entfernt.

## Punkte-System

- **Basis:** 100 Punkte pro Lieferung
- **Zeitbonus:** verbleibende Sekunden × 5 Punkte
- **Perfekt-Bonus:** noch >60 % der Zeit übrig → +50 Punkte
- **Boost aktiv bei Lieferung:** +20 Bonus-Punkte
- **Streak-Multiplikator** (Lieferungen am Stück ohne Leben-Verlust):
  - 3–4 → ×1.5
  - 5–7 → ×2.0
  - 8+ → ×2.5
- **Münzen:** ~2–10 pro Lieferung (Zeitbonus + Streak-Bonus)

Ein verpasster Kunde setzt die Streak zurück — Tempo lohnt sich also doppelt.

## Jannicks Kölner Eck

In der Hauptstraße steht Jannicks Pizzeria (rot/weiße Markise + Kölner Dom).
Wenn du mit dem Sprinter nahe vorbeikommst, blendet das Spiel einen Hinweis ein.
Mit `E` öffnest du Jannicks Shop. Pause, Kölsch-Sprüche und das Menü:

| Item | Preis | Effekt |
|---|---|---|
| Margherita Pizza | 5 Münzen | +30 Sek Speed-Boost (+40 % Bewegung) |
| Geiler Kaffee | 3 Münzen | +20 Sek Speed-Boost (+25 % Bewegung) |

`1` kauft Pizza, `2` kauft Kaffee, `ESC` verlässt den Shop.

## Highscore

Die Top 5 Highscores werden in `highscore.json` im Projektordner gespeichert
(`{ "entries": [...] }`). Beim ersten Spielstart wird die Datei automatisch
angelegt. Schlägst du den bisherigen Spitzenreiter, blinkt `NEUER REKORD!`
im Game-Over-Screen golden.

## Sound & Optik

Es werden **keine externen Dateien** benötigt — alle Sprites *und* der
komplette Sound werden zur Laufzeit prozedural erzeugt:

- **Motorgeräusch**, dessen Tonhöhe und Lautstärke mit dem Tempo mitgehen
- **Liefer-Jingle**, Münz-Klimpern, Crash-Rumms, Nitro-Whoosh und ein
  unfreundlicher Buzzer, wenn ein Kunde aufgibt
- **Game Feel:** Screen-Shake bei Crashes und Lieferungen, Münz- und
  Funken-Partikel, Abgaswolken bei Nitro und eine leicht vorausschauende Kamera
- Der Sprinter nutzt **echte Richtungs-Sprites** (kein verzerrtes Rotieren)
  mit animierten Rädern und weichem Schatten

## Bauen und Starten

```bash
cargo run --release
```

Beim ersten Build kann es einige Minuten dauern, weil Bevy kompiliert wird.

## Stadt

Die 128×128-Pixelart-Karte ist an Germering orientiert und enthält:

- **Rathaus Germering** (Kirchenplatz)
- **Bahnhof Germering** mit S-Bahn-Gleisen (S8)
- **Aldi Süd** in der Unteren Bahnhofstraße
- **Rewe** im Zentrum
- **Vier Ihle-Filialen** (Bahnhof, Zentrum, Nord, Süd)
- **Jannicks Kölner Eck** an der Hauptstraße
- **Hauptstraße / Untere Bahnhofstraße / Eichenauer Straße** als Hauptachsen
- **Stadtpark Nord** und Grünflächen
- Wohnblöcke mit gemischten Einfamilien-/Mehrfamilienhäusern

Viel Spaß beim Liefern!
