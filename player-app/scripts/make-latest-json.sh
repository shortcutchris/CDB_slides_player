#!/usr/bin/env bash
# Erzeugt das Updater-Manifest latest.json aus den signierten Build-Artefakten.
# Reihenfolge:  ./scripts/build-macos.sh  →  ./scripts/make-latest-json.sh
# Dann latest.json + die .app.tar.gz (+ .dmg) ans GitHub-Release v<version> hängen;
# der Updater-Endpoint liefert .../releases/latest/download/latest.json.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
REPO="shortcutchris/CDB_slides_player"

VERSION="$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed -E 's/.*"version": *"([^"]+)".*/\1/')"
PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
BASE="https://github.com/$REPO/releases/download/v$VERSION"

MAC_SIG_FILE="src-tauri/target/release/bundle/macos/CDBrain Slide Player.app.tar.gz.sig"
[ -f "$MAC_SIG_FILE" ] || { echo "macOS-Signatur fehlt: $MAC_SIG_FILE (erst build-macos.sh mit Key)"; exit 1; }
MAC_SIG="$(cat "$MAC_SIG_FILE")"

# GitHub ersetzt Leerzeichen im Asset-Namen durch Punkte in der Download-URL.
cat > latest.json <<EOF
{
  "version": "$VERSION",
  "pub_date": "$PUB_DATE",
  "platforms": {
    "darwin-aarch64": {
      "signature": "$MAC_SIG",
      "url": "$BASE/CDBrain.Slide.Player_${VERSION}_aarch64.app.tar.gz"
    }
  }
}
EOF
echo "✓ latest.json für v$VERSION geschrieben"
