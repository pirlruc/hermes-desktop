use std::path::PathBuf;
use std::sync::Mutex;

use hermes_core::{upload_allowed, LibraryWork, QiqqaReader, ResultCache, ZoteroReader};
use serde::Serialize;
use tauri::State;

struct AppState {
    library: Mutex<Vec<LibraryWork>>,
    consented: Mutex<Vec<String>>,
    cache_path: PathBuf,
}

#[derive(Serialize)]
struct WorkView {
    key: String,
    title: String,
    doi: Option<String>,
    queued: bool,
}

#[tauri::command]
fn library(state: State<'_, AppState>) -> Vec<WorkView> {
    state
        .library
        .lock()
        .expect("library")
        .iter()
        .map(|work| WorkView {
            key: work.key.clone(),
            title: work.title.clone(),
            doi: work.doi.clone(),
            queued: work.pdf_path.is_some(),
        })
        .collect()
}

#[tauri::command]
fn sync_status(state: State<'_, AppState>) -> Result<usize, String> {
    let cache = ResultCache::open(&state.cache_path).map_err(|err| err.to_string())?;
    let works = state.library.lock().expect("library");
    cache
        .put(
            "sync",
            "status",
            &serde_json::json!({ "count": works.len() }),
        )
        .map_err(|err| err.to_string())?;
    Ok(works.len())
}

#[tauri::command]
fn grant_consent(state: State<'_, AppState>, key: String) -> Result<(), String> {
    state.consented.lock().expect("consent").push(key.clone());
    let granted = state
        .consented
        .lock()
        .expect("consent")
        .iter()
        .any(|item| item == &key);
    upload_allowed(granted, &key).map_err(|err| err.to_string())
}

pub fn load_library(
    zotero: Option<&std::path::Path>,
    qiqqa: Option<&std::path::Path>,
    directory: &std::path::Path,
) -> Vec<LibraryWork> {
    let mut works = Vec::new();
    if let Some(path) = zotero {
        if let Ok(rows) = ZoteroReader::read(path, directory) {
            works.extend(rows);
        }
    }
    if let Some(path) = qiqqa {
        if let Ok(rows) = QiqqaReader::read(path, directory) {
            works.extend(rows);
        }
    }
    works
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let directory = std::env::temp_dir().join("hermes-desktop");
    let _ = std::fs::create_dir_all(&directory);
    tauri::Builder::default()
        .manage(AppState {
            library: Mutex::new(load_library(None, None, &directory)),
            consented: Mutex::new(Vec::new()),
            cache_path: directory.join("cache.sqlite"),
        })
        .invoke_handler(tauri::generate_handler![
            library,
            sync_status,
            grant_consent
        ])
        .run(tauri::generate_context!())
        .expect("error while running hermes");
}
