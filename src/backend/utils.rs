use gtk::glib;
use relm4::prelude::*;
use relm4::tokio;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use walkdir::WalkDir;

use crate::APP_ID;
use crate::AppModel;
use crate::app::AppMsg;
use crate::backend::track_info::TrackInfo;

static USER_STATE_PATH: LazyLock<PathBuf> = LazyLock::new(|| glib::user_state_dir().join(APP_ID));

pub fn write_json(vec: &Vec<PathBuf>, current_track: usize, position: f64) {
    let restore_playlist = USER_STATE_PATH.join("files_for_restore.json");
    let save_state = USER_STATE_PATH.join("saved_state.json");
    fs::create_dir_all(&*USER_STATE_PATH).ok();
    if let Ok(json) = serde_json::to_string(vec) {
        fs::write(restore_playlist, json).ok();
    }
    if let Ok(json) = serde_json::to_string(&(current_track, position)) {
        fs::write(save_state, json).ok();
    }
}

pub fn read_json() -> (Option<Vec<PathBuf>>, usize, f64) {
    let restore_playlist = USER_STATE_PATH.join("files_for_restore.json");
    let save_state = USER_STATE_PATH.join("saved_state.json");
    let vec: Option<Vec<PathBuf>> = fs::read_to_string(restore_playlist)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    let (current_track, position): (usize, f64) = fs::read_to_string(save_state)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or((0, 0.0));
    (vec, current_track, position)
}

pub fn save_watch_folder(path: &PathBuf) {
    let watch_folder_path = USER_STATE_PATH.join("watch_folder.json");
    fs::create_dir_all(&*USER_STATE_PATH).ok();
    if let Ok(json) = serde_json::to_string(path) {
        fs::write(watch_folder_path, json).ok();
    }
}

pub fn load_watch_folder() -> Option<PathBuf> {
    let watch_folder_path = USER_STATE_PATH.join("watch_folder.json");
    fs::read_to_string(watch_folder_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub async fn open_paths(
    paths: Vec<PathBuf>,
    sender: ComponentSender<AppModel>,
    sort: bool,
) -> Vec<Arc<TrackInfo>> {
    let audio_extensions = ["mp3", "flac", "wav", "ogg", "m4a", "opus"];

    sender.input(AppMsg::Progress(0.0));

    let received_paths = tokio::task::spawn_blocking(move || {
        paths
            .into_iter()
            .flat_map(|paths| {
                WalkDir::new(&paths)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.path().to_path_buf())
                    .filter(|p| {
                        p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| audio_extensions.contains(&ext.to_lowercase().as_str()))
                            .unwrap_or(false)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    let total = received_paths.len();
    let mut items_backend = Vec::new();

    for (process, path) in received_paths.into_iter().enumerate() {
        let info = Arc::new(TrackInfo::new(&path));
        items_backend.push(info);
        let progress = (process + 1) as f64 / total as f64;
        sender.input(AppMsg::Progress(progress));
    }

    // Sort tracks by tags
    if sort {
        items_backend.sort_by(|a, b| {
            a.album()
                .cmp(b.album())
                .then(a.disc_number().cmp(&b.disc_number()))
                .then(a.track_number().cmp(&b.track_number()))
                .then(a.path().cmp(b.path()))
        });
        sender.input(AppMsg::ProgressFinished);
        return items_backend;
    }
    sender.input(AppMsg::ProgressFinished);

    items_backend
}
