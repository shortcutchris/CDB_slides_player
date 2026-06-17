// CDBrain Slide Player — Tauri-Backend.
// Scannt einen Medien-Ordner, hält Playlists in playlists.json (im App-Data-Dir),
// öffnet den nativen Ordner-Dialog. Frontend = ../ui/index.html (System-Webview).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

const IMAGE_EXT: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "avif", "bmp"];
const VIDEO_EXT: &[&str] = &["mp4", "webm", "mov", "m4v", "ogv"];

#[derive(Serialize)]
struct MediaItem {
    path: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    dir: String,
}

fn classify(p: &Path) -> Option<&'static str> {
    let ext = p.extension()?.to_str()?.to_lowercase();
    if VIDEO_EXT.contains(&ext.as_str()) {
        Some("video")
    } else if IMAGE_EXT.contains(&ext.as_str()) {
        Some("image")
    } else {
        None
    }
}

/// Alle Bilder/Videos unter `root` (rekursiv). Ordner mit führendem _ oder . werden übersprungen.
#[tauri::command]
fn scan_media(root: String) -> Vec<MediaItem> {
    let root = PathBuf::from(&root);
    let mut out: Vec<MediaItem> = Vec::new();
    if !root.is_dir() {
        return out;
    }
    let walker = walkdir::WalkDir::new(&root).into_iter().filter_entry(|e| {
        if e.depth() == 0 {
            return true;
        }
        let n = e.file_name().to_string_lossy();
        !(n.starts_with('_') || n.starts_with('.'))
    });
    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if let Some(kind) = classify(p) {
            if let Ok(rel) = p.strip_prefix(&root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                let name = p
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let dir = Path::new(&rel)
                    .parent()
                    .map(|d| d.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                out.push(MediaItem { path: rel, name, kind: kind.into(), dir });
            }
        }
    }
    out.sort_by(|a, b| (&a.dir, &a.name).cmp(&(&b.dir, &b.name)));
    out
}

fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir
}

#[tauri::command]
fn load_playlists(app: tauri::AppHandle) -> serde_json::Value {
    let f = data_dir(&app).join("playlists.json");
    if let Ok(s) = fs::read_to_string(&f) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            return v;
        }
    }
    serde_json::json!({ "playlists": [] })
}

#[tauri::command]
fn save_playlists(app: tauri::AppHandle, data: serde_json::Value) -> Result<(), String> {
    let f = data_dir(&app).join("playlists.json");
    let pretty = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(&f, pretty).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_media_root(app: tauri::AppHandle) -> String {
    let f = data_dir(&app).join("config.json");
    if let Ok(s) = fs::read_to_string(&f) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            if let Some(r) = v.get("media_root").and_then(|x| x.as_str()) {
                return r.to_string();
            }
        }
    }
    String::new()
}

#[tauri::command]
fn set_media_root(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let f = data_dir(&app).join("config.json");
    fs::write(&f, serde_json::json!({ "media_root": path }).to_string())
        .map_err(|e| e.to_string())
}

/// Nativer Ordner-Dialog. Async + nicht-blockierend: der Dialog läuft auf dem
/// Main-Thread, das Ergebnis kommt per Channel zurück. (Die blockierende
/// Variante würde den Main-Thread blockieren → Crash auf macOS.)
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |p| {
        let _ = tx.send(p);
    });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
        .map(|p| p.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_media,
            load_playlists,
            save_playlists,
            get_media_root,
            set_media_root,
            pick_folder
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Anwendung");
}
