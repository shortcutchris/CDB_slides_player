# CDBrain Slide Player — Desktop-App (Tauri)

Native macOS-App (Rust + System-Webview/WKWebView) der Playlist-Verwaltung —
**kein Server, kein Terminal, kein Python**. Doppelklick auf die `.app`, fertig.
Weil die System-Webview genutzt wird, spielen **H.264-Videos nativ** (kein
Schwarzbild wie in einem nackten Chromium).

Sie ist die App-Variante der Web/Server-Edition (`../player-web/`); gleiche
Oberfläche (`ui/index.html` ≈ `editor.html`), nur statt `fetch('/api/...')` über
Tauri-`invoke()` ans Rust-Backend.

## Bauen

Einmalig CLI installieren (falls nicht vorhanden):
```bash
cargo install tauri-cli --version "^2.0" --locked
```

App bauen:
```bash
cd src-tauri
cargo tauri build                 # Release: .app + .dmg unter target/release/bundle/
cargo tauri build --debug --bundles app   # schneller, nur .app unter target/debug/bundle/macos/
cargo tauri dev                   # Entwicklung mit Hot-Reload-Fenster
```
Ergebnis (Release): `target/release/bundle/macos/CDBrain Slide Player.app`
und `target/release/bundle/dmg/*.dmg` (zum Weitergeben).

> Voraussetzungen: Rust, Xcode Command Line Tools. Windows-Build via Tauri
> möglich (auf einem Windows-Host bauen).

## Bedienung
Identisch zur Web-Edition (siehe `../player-web/README.md`): Playlists
anlegen/umbenennen/duplizieren/löschen, Items per ↑↓/Drag sortieren, Bilddauer +
Loop-Optionen, Präsentieren (`← →` · Space · F · Esc). **Unterschied:** oben den
**Medien-Ordner** über den nativen Dialog wählen (Button „Ordner wählen…"). Die
App merkt ihn sich.

## Wo liegen die Daten?
Im App-Data-Ordner des Nutzers (nicht im Repo):
`~/Library/Application Support/de.cdbrain.slideplayer/`
- `playlists.json` — deine Playlists
- `config.json` — gewählter Medien-Ordner

## Architektur / Dateien
```
src-tauri/
  Cargo.toml          # Rust-Deps (tauri, tauri-plugin-dialog, serde, walkdir)
  tauri.conf.json     # App-Config: Fenster, withGlobalTauri, assetProtocol, bundle/icons
  build.rs            # tauri-build
  capabilities/       # Berechtigungen (core + dialog)
  src/main.rs         # Commands: scan_media, load/save_playlists, get/set_media_root, pick_folder
  icons/              # App-Icons (aus icons/source.png via `cargo tauri icon`)
ui/
  index.html          # Frontend (= editor.html, Tauri-invoke statt fetch)
```
Medien werden über Tauris **Asset-Protokoll** (`convertFileSrc`) geladen — mit
Range-Support, daher läuft Video sauber. `target/` und `gen/` sind gitignored.

## Icon ändern
`icons/source.png` (1024×1024) ersetzen → `cargo tauri icon icons/source.png`
neu generieren → neu bauen.
