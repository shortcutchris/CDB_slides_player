#!/usr/bin/env bash
# Release-Build der macOS-App (.app + .dmg) inkl. signierter Updater-Artefakte.
#
# Der Signing-Key liegt außerhalb des Repos (~/.tauri/cdb-slides-player.key).
# Ist er vorhanden, baut `cargo tauri build` zusätzlich ein signiertes
# .app.tar.gz (+ .sig) für den In-App-Updater.
#
# Nutzung:  ./scripts/build-macos.sh           # baut app + dmg + Updater-Artefakte
#           ./scripts/build-macos.sh --bundles app   # nur .app (schneller)
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/src-tauri"

KEY="${TAURI_SIGNING_PRIVATE_KEY_PATH:-$HOME/.tauri/cdb-slides-player.key}"
if [ -f "$KEY" ]; then
  export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY")"
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
  echo "✓ Updater-Signing-Key gefunden — Updater-Artefakte werden signiert"
else
  echo "⚠ Kein Signing-Key ($KEY) — Build ohne signierte Updater-Artefakte"
fi

cargo tauri build "$@"
echo "✓ Build fertig: src-tauri/target/release/bundle/{macos,dmg}"
