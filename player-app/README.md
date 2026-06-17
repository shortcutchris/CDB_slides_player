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

## In-App-Update (Tauri-Updater)
Die App prüft beim Start einen Update-Endpoint und zeigt bei neuer Version ein
Banner „Update vX verfügbar → Jetzt installieren" (lädt + installiert + Neustart).
Mechanik wie bei `cdb_desktop_pdf_compressor`:
- Plugin `tauri-plugin-updater`; `pubkey` + `endpoints` in `tauri.conf.json`
  (Endpoint: `…/releases/latest/download/latest.json`), `createUpdaterArtifacts: true`.
- Signing-Keypair (einmalig erzeugt): privat `~/.tauri/cdb-slides-player.key`
  (**geheim halten, NICHT committen**), public in der Config. Neu erzeugen:
  `cargo tauri signer generate -w ~/.tauri/cdb-slides-player.key -p ""`.

**Neue Version veröffentlichen:**
1. `version` in `tauri.conf.json` erhöhen (z.B. 0.1.0 → 0.1.1).
2. `./scripts/build-macos.sh` → baut `.dmg` **und** signiertes `.app.tar.gz` (+ `.sig`).
3. `./scripts/make-latest-json.sh` → erzeugt `latest.json` aus der Signatur.
4. GitHub-Release `v<version>` anlegen und Assets anhängen:
   ```bash
   gh release create v0.1.1 \
     "src-tauri/target/release/bundle/dmg/CDBrain Slide Player_0.1.1_aarch64.dmg" \
     "src-tauri/target/release/bundle/macos/CDBrain Slide Player.app.tar.gz" \
     latest.json --repo shortcutchris/CDB_slides_player --title v0.1.1 --notes "…"
   ```
   Laufende Apps sehen das Update beim nächsten Start.

> **Privat-Repo-Hinweis:** Der Updater lädt `latest.json` + Artefakte von den
> Release-Assets. Bei einem **privaten** Repo sind diese URLs nicht ohne Token
> erreichbar → Auto-Update greift erst, wenn das Repo (bzw. die Releases) **public**
> ist oder ein authentifizierter Endpoint genutzt wird. Der `.dmg`-Download zum
> manuellen Installieren funktioniert unabhängig davon.
