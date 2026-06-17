// CDB Slides Player — Tauri-Backend.
// Managed-Media-Modell: importierte Bilder/Videos werden in einen app-eigenen
// Medien-Ordner KOPIERT (App-Data). Playlists referenzieren nur diese Kopien →
// eine Show bleibt immer self-contained und beim nächsten Öffnen vollständig.
// Frontend = ../ui/index.html (System-Webview).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

const MEDIA_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "avif", "bmp", "mp4", "webm", "mov", "m4v", "ogv",
];
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

fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir
}

/// App-eigener Medien-Ordner (verwaltete Bibliothek).
fn media_dir(app: &tauri::AppHandle) -> PathBuf {
    let d = data_dir(app).join("media");
    let _ = fs::create_dir_all(&d);
    d
}

#[tauri::command]
fn get_media_dir(app: tauri::AppHandle) -> String {
    media_dir(&app).to_string_lossy().to_string()
}

/// Alle Bilder/Videos im verwalteten Medien-Ordner (rekursiv).
#[tauri::command]
fn scan_media(app: tauri::AppHandle) -> Vec<MediaItem> {
    let root = media_dir(&app);
    let mut out: Vec<MediaItem> = Vec::new();
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

// ── Datei-Dialoge (async, nicht-blockierend) ─────────────────────────────────
async fn pick_folder_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |p| {
        let _ = tx.send(p);
    });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
        .map(|p| PathBuf::from(p.to_string()))
}

async fn pick_files_paths(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Bilder & Videos", MEDIA_EXT)
        .pick_files(move |ps| {
            let _ = tx.send(ps);
        });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
        .map(|v| v.into_iter().map(|p| PathBuf::from(p.to_string())).collect())
        .unwrap_or_default()
}

/// Kopiert alle Medien aus `src_root` (rekursiv) nach `<media>/<sub>/…`.
fn copy_folder_media(src_root: &Path, managed: &Path, sub: &str) -> usize {
    let mut n = 0;
    let walker = walkdir::WalkDir::new(src_root).into_iter().filter_entry(|e| {
        if e.depth() == 0 {
            return true;
        }
        let nm = e.file_name().to_string_lossy();
        !(nm.starts_with('_') || nm.starts_with('.'))
    });
    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if classify(p).is_none() {
            continue;
        }
        if let Ok(rel) = p.strip_prefix(src_root) {
            let dst = managed.join(sub).join(rel);
            if let Some(parent) = dst.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::copy(p, &dst).is_ok() {
                n += 1;
            }
        }
    }
    n
}

/// Ordner wählen → dessen Medien in die verwaltete Bibliothek kopieren.
#[tauri::command]
async fn import_folder(app: tauri::AppHandle) -> usize {
    if let Some(folder) = pick_folder_path(&app).await {
        let base = folder
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Import".into());
        return copy_folder_media(&folder, &media_dir(&app), &base);
    }
    0
}

/// Einzelne Dateien wählen → in die verwaltete Bibliothek kopieren.
#[tauri::command]
async fn import_files(app: tauri::AppHandle) -> usize {
    let files = pick_files_paths(&app).await;
    let managed = media_dir(&app);
    let mut n = 0;
    for f in files {
        if classify(&f).is_none() {
            continue;
        }
        if let Some(name) = f.file_name() {
            if fs::copy(&f, managed.join(name)).is_ok() {
                n += 1;
            }
        }
    }
    n
}

/// Eine importierte Datei wieder aus der Bibliothek löschen (relativer Pfad).
#[tauri::command]
fn delete_media(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let target = media_dir(&app).join(&path);
    let managed = media_dir(&app);
    // Pfad-Sicherheit: nur innerhalb des verwalteten Ordners löschen.
    let canon = target.canonicalize().map_err(|e| e.to_string())?;
    if !canon.starts_with(&managed) {
        return Err("Pfad außerhalb der Bibliothek".into());
    }
    fs::remove_file(&canon).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_media,
            get_media_dir,
            load_playlists,
            save_playlists,
            import_folder,
            import_files,
            delete_media
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Anwendung");
}
