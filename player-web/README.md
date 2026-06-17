# CDBrain Slide Player

Lokale Tools, um Slides (Bilder **und** animierte Videos) als Fullscreen-Show
abzuspielen — im CDBrain-Dark-Look, komplett offline. Zwei Varianten:

| Tool | Datei | Wofür |
|---|---|---|
| **Playlist-Verwaltung** ⭐ | `server.py` + `editor.html` | Mehrere Playlists anlegen/bearbeiten, in `playlists.json` gespeichert. Der „richtige" Weg. |
| **Schnell-Player** | `index.html` | Ad-hoc: Dateien per Drag-&-Drop reinziehen, loopen. Kein Server, kein Speichern. |

---

## Schnellstart (Verwaltung)

**macOS:** Doppelklick auf **`start.command`** → Server startet, Browser öffnet.

**oder Terminal:**
```bash
python3 server.py --media /pfad/zu/deinen/slides
# optional:  --port 8780   --no-open   (ohne --media wird das aktuelle Verzeichnis gescannt)
```

Beenden: im Terminal `Strg+C` (bzw. das Terminal-Fenster schließen).

> Nur Python 3 (Standardbibliothek) nötig — keine Installation, keine Dependencies.

### Bedienung
- **Links — Playlists:** `Neu` anlegen · Titel oben editieren = umbenennen ·
  Hover → Duplizieren / Löschen · Klick = auswählen.
- **Mitte — Reihenfolge:** Items mit **↑ ↓** nach vorne/hinten oder per
  **Drag-&-Drop** sortieren · **✕** entfernt. Pro Playlist: **Bilddauer**,
  **Show loopen**, **Videos loopen**.
- **Rechts — Medien-Bibliothek:** alle Bilder/Videos aus dem gewählten
  Medien-Ordner, nach Unterordner gruppiert, mit Suche. Klick = zur aktiven
  Playlist hinzufügen · **„Alle"** = alle angezeigten auf einmal.
- **Präsentieren** (oben rechts): Vollbild-Show. Tasten: `← →` blättern ·
  `Space` Pause · `F` Vollbild · `Esc` beenden. Maus/Leiste blenden aus.
- **Speichern passiert automatisch** (Autosave → `playlists.json`, Status oben rechts).

### Empfohlene Loop-Einstellung
- **Vortrag (du redest dazu):** „Videos loopen" **an** → jedes Animations-Slide
  loopt endlos, du blätterst mit `→` manuell weiter.
- **Unbeaufsichtigt (Messe/Empfang):** „Videos loopen" **aus** + „Show loopen"
  **an** → durchlaufender Showreel, jedes Video spielt einmal, dann weiter.

---

## Schnell-Player (`index.html`)
Doppelklick → Bilder/Videos ins Fenster ziehen → `Präsentieren`. Sortieren per
Drag, Bilddauer + Loop-Optionen unten. Kein Speichern (Session-only). Optional
oben im Script die `PRELOAD`-Liste mit Pfaden füllen, dann lädt er beim Öffnen
automatisch — so kann man eine fertige Show als vorbefüllten Player ausliefern
(HTML + Medien zusammen auf USB).

---

## Dateien
```
player-web/
  server.py        # lokaler HTTP-Server: scannt Medien, CRUD playlists.json, liefert App + Medien
  editor.html      # die Verwaltungs-Oberfläche (CRUD, Reorder, Bibliothek, Präsentieren)
  index.html       # eigenständiger Drag-&-Drop-Player (ohne Server)
  start.command    # Doppelklick-Starter (macOS)
  playlists.json   # deine Playlists — wird automatisch erzeugt, GITIGNORED (lokal)
```

`playlists.json`-Schema: `{ "playlists": [ { "id", "name", "settings": {imgDur,
loopShow, loopVid}, "items": ["<relpfad unter Medien-Ordner>", …] } ] }`.
Pfade sind relativ zum Medien-Ordner → portabel.

---

## Architektur — warum ein Server?
Eine reine HTML-Datei per `file://`-Doppelklick **darf nicht auf die Platte
schreiben** (Browser-Sandbox), und persistente Playlists brauchen stabile
Datei**pfade** (Drag-&-Drop liefert aus Sicherheitsgründen keine echten Pfade).
Der winzige lokale Server löst beides: er kennt die echten Pfade (scannt den
Medien-Ordner) und schreibt `playlists.json`. Deshalb Verwaltung = Server,
Schnellfall = `index.html`.

---

## Troubleshooting
- **Video bleibt schwarz / lädt nicht:** Der Browser braucht den **H.264-Codec**
  (Chrome, Safari, Edge haben ihn; ein nacktes Chromium ohne H.264 zeigt schwarz).
  Für garantiertes H.264 ohne Browser-Codec-Sorgen die **Desktop-App** (`../player-app/`) nehmen.
- **Port belegt:** Server sucht automatisch den nächsten freien Port (8780→…).
  Die tatsächliche URL steht in der Terminal-Ausgabe.
- **Anderer Medien-Ordner:** `python3 server.py --media /pfad`.
  Ordner mit führendem `_` oder `.` werden beim Scan übersprungen.
- **Playlists weg:** `playlists.json` ist lokal/gitignored. Fehlt sie, startet der
  Player mit leerer Liste.
