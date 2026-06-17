#!/usr/bin/env python3
"""CDBrain Slide Player — winziger lokaler Server für die Playlist-Verwaltung.

Nur Python-Standard-Library, keine Dependencies. Scannt einen Medien-Ordner
(Default: ../../sales), liefert die Editor-App + Medien aus und liest/schreibt
Playlists in playlists.json.

Start:  python3 server.py            (Browser öffnet automatisch)
        python3 server.py --media /pfad/zu/medien --port 8780
oder per Doppelklick auf start.command (macOS).
"""
import argparse
import json
import mimetypes
import os
import threading
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlparse

HERE = Path(__file__).resolve().parent
IMAGE_EXT = {".png", ".jpg", ".jpeg", ".webp", ".gif", ".avif", ".bmp"}
VIDEO_EXT = {".mp4", ".webm", ".mov", ".m4v", ".ogv"}
PLAYLISTS_FILE = HERE / "playlists.json"


def media_type(p: Path):
    ext = p.suffix.lower()
    if ext in VIDEO_EXT:
        return "video"
    if ext in IMAGE_EXT:
        return "image"
    return None


def scan_media(root: Path):
    """Alle Bilder/Videos unter root (rekursiv). Ordner mit führendem _ oder .
    werden übersprungen (Scratch/Kandidaten/versteckt)."""
    items = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if not d.startswith((".", "_"))]
        for fn in sorted(filenames):
            p = Path(dirpath) / fn
            t = media_type(p)
            if not t:
                continue
            rel = p.relative_to(root).as_posix()
            items.append({"path": rel, "name": fn, "type": t,
                          "dir": Path(rel).parent.as_posix() if "/" in rel else ""})
    items.sort(key=lambda x: (x["dir"], x["name"]))
    return items


def seed_playlists(media):
    """Erststart: leer beginnen (kein vorgefülltes Demo im Produkt-Repo)."""
    return {"playlists": []}


def load_playlists(media):
    if PLAYLISTS_FILE.exists():
        try:
            return json.loads(PLAYLISTS_FILE.read_text("utf-8"))
        except Exception:
            pass
    data = seed_playlists(media)
    save_playlists(data)
    return data


def save_playlists(data):
    PLAYLISTS_FILE.write_text(json.dumps(data, ensure_ascii=False, indent=2), "utf-8")


class Handler(BaseHTTPRequestHandler):
    media_root: Path = HERE

    def log_message(self, *a):  # ruhiger Konsolen-Output
        pass

    # ── helpers ──────────────────────────────────────────────
    def _json(self, obj, code=200):
        body = json.dumps(obj, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _safe_media_path(self, rel):
        target = (self.media_root / unquote(rel)).resolve()
        if self.media_root not in target.parents and target != self.media_root:
            return None
        return target if target.is_file() else None

    # ── routes ───────────────────────────────────────────────
    def do_GET(self):
        path = urlparse(self.path).path
        if path in ("/", "/editor.html"):
            return self._send_file(HERE / "editor.html", "text/html; charset=utf-8")
        if path == "/api/state":
            media = scan_media(self.media_root)
            return self._json({"media": media, "playlists": load_playlists(media)["playlists"],
                               "mediaRoot": str(self.media_root)})
        if path.startswith("/media/"):
            target = self._safe_media_path(path[len("/media/"):])
            if not target:
                return self._json({"error": "not found"}, 404)
            return self._send_file(target, mimetypes.guess_type(str(target))[0] or "application/octet-stream",
                                   ranged=True)
        return self._json({"error": "not found"}, 404)

    def do_PUT(self):
        path = urlparse(self.path).path
        if path == "/api/playlists":
            length = int(self.headers.get("Content-Length", 0))
            try:
                data = json.loads(self.rfile.read(length) or b"{}")
                assert isinstance(data.get("playlists"), list)
            except Exception as e:
                return self._json({"error": f"bad payload: {e}"}, 400)
            save_playlists({"playlists": data["playlists"]})
            return self._json({"ok": True})
        return self._json({"error": "not found"}, 404)

    # ── file serving (mit einfachem Range-Support für Video) ──
    def _send_file(self, fp: Path, ctype, ranged=False):
        if not fp.is_file():
            return self._json({"error": "not found"}, 404)
        size = fp.stat().st_size
        rng = self.headers.get("Range") if ranged else None
        # Verbindungsabbrüche des Browsers (Video-Abort, Seek, Re-Request mit
        # Range) sind normal — sauber abfangen statt Traceback-Spam.
        try:
            if rng and rng.startswith("bytes="):
                s, _, e = rng[6:].partition("-")
                start = int(s) if s else 0
                end = int(e) if e else size - 1
                end = min(end, size - 1)
                length = end - start + 1
                self.send_response(206)
                self.send_header("Content-Type", ctype)
                self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
                self.send_header("Accept-Ranges", "bytes")
                self.send_header("Content-Length", str(length))
                self.end_headers()
                with fp.open("rb") as f:
                    f.seek(start)
                    self.wfile.write(f.read(length))
                return
            self.send_response(200)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(size))
            self.send_header("Accept-Ranges", "bytes")
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(fp.read_bytes())
        except (BrokenPipeError, ConnectionResetError):
            pass


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--media", default=str(Path.cwd()),
                    help="Medien-Wurzelordner, der nach Bildern/Videos durchsucht wird (Default: aktuelles Verzeichnis)")
    ap.add_argument("--port", type=int, default=8780)
    ap.add_argument("--no-open", action="store_true", help="Browser nicht automatisch öffnen")
    args = ap.parse_args()

    media_root = Path(args.media).resolve()
    Handler.media_root = media_root

    port = args.port
    for attempt in range(20):
        try:
            httpd = ThreadingHTTPServer(("127.0.0.1", port), Handler)
            break
        except OSError:
            port += 1
    else:
        raise SystemExit("Kein freier Port gefunden.")

    url = f"http://localhost:{port}/"
    print(f"\n  CDBrain Slide Player\n  Medien: {media_root}\n  Läuft:  {url}\n  (Strg+C zum Beenden)\n")
    if not args.no_open:
        threading.Timer(0.6, lambda: webbrowser.open(url)).start()
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n  Beendet.")


if __name__ == "__main__":
    main()
