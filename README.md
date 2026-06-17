# CDB Slides Player

Ein kleiner, lokaler **Slideshow-/Kiosk-Player** für Bilder **und** Videos —
mit Playlist-Verwaltung, Vollbild-Präsentation und Endlos-Loop. Komplett offline,
im dunklen CDBrain-Look. Du fütterst einen Medien-Ordner, baust daraus Playlists
(Reihenfolge frei sortierbar), und spielst sie als Show ab.

> Ideal für Messe-/Empfangs-Screens, Pitch-Loops und Vortrags-Decks aus
> animierten Slides.

## Zwei Editionen

| Edition | Ordner | Wofür |
|---|---|---|
| **Desktop-App** ⭐ | [`player-app/`](player-app/) | Native `.app` (Rust + Tauri, System-Webview). Doppelklick, **kein Server/Terminal**, **H.264 läuft nativ**. Der empfohlene Weg. |
| **Web/Server** | [`player-web/`](player-web/) | Eine Python-Datei (Standard-Library) + Browser-UI. Ohne Build, plattformneutral. Gut zum schnellen Ausprobieren. |

Beide haben dieselbe Oberfläche: links Playlists (anlegen/umbenennen/duplizieren/
löschen), Mitte die Reihenfolge (↑↓ / Drag, „Alle"-Button), rechts die
Medien-Bibliothek mit Suche. Präsentieren im Vollbild: `← →` blättern · `Space`
Pause · `F` Vollbild · `Esc` beenden.

## Schnellstart

### Desktop-App (macOS)
```bash
cd player-app/src-tauri
cargo install tauri-cli --version "^2.0" --locked   # einmalig
cargo tauri build            # → .app + .dmg unter target/release/bundle/
# oder zum Entwickeln:
cargo tauri dev
```
Beim ersten Start oben **„Ordner wählen…"** → deinen Medien-Ordner. Voraussetzung:
Rust + (macOS) Xcode Command Line Tools. Details: [`player-app/README.md`](player-app/README.md).

### Web/Server
```bash
cd player-web
python3 server.py --media /pfad/zu/deinen/slides
# Browser öffnet automatisch; ohne --media wird das aktuelle Verzeichnis gescannt.
```
macOS: Doppelklick auf `start.command`. Details: [`player-web/README.md`](player-web/README.md).

## Wie es funktioniert
- **Medien-Ordner** wird rekursiv nach Bildern (png/jpg/webp/gif/…) und Videos
  (mp4/webm/mov/…) durchsucht; Ordner mit führendem `_` oder `.` werden übersprungen.
- **Playlists** speichern nur **Pfade** (relativ zum Medien-Ordner) → portabel.
- **Persistenz:**
  - App: `~/Library/Application Support/de.cdbrain.slideplayer/playlists.json`
  - Web: `player-web/playlists.json` (gitignored)
- Pro Playlist einstellbar: **Bilddauer**, **Show loopen**, **Videos loopen**.

## Warum zwei Editionen?
Eine reine HTML-Datei (`file://`) darf nicht auf die Platte schreiben und kennt
keine echten Dateipfade. Beide Editionen lösen das mit etwas „Backend": die App
über Rust/Tauri (nativer Datei-Dialog, Asset-Protokoll mit Range-Support → Video
läuft), die Web-Variante über einen winzigen lokalen HTTP-Server. Die App ist die
runde Produktform; die Web-Variante der dependency-arme Fallback.

## Lizenz / Status
Interndes CDBrain-Tool. Markenfarben/Look gehören zu CDBrain.
