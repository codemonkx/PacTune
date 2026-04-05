mod about;
mod app;
mod backend;
mod drag_overlay;
mod lyrics_item;
mod menu;
mod track_item;
mod tracklist;
mod album_item;
mod albumlist;
mod album_detail;

use app::AppModel;
use gtk::prelude::*;
use relm4::{ComponentSender, RelmApp, adw};
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use crate::app::AppMsg;

pub static APP_ID: &str = "page.codeberg.M23Snezhok.PacTune";
pub static OPEN_FILE: Mutex<Option<Vec<PathBuf>>> = Mutex::new(None);
pub static APP_SENDER: OnceLock<ComponentSender<AppModel>> = OnceLock::new();

fn main() {
    gst::init().unwrap();

    let gtk_app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    gtk_app.connect_activate(|app| {
        // Bring existing window to front, or let relm4 create it
        if let Some(win) = app.active_window() {
            win.present();
        }
    });

    gtk_app.connect_open(|app, files, _hint| {
        let mut paths: Vec<PathBuf> = vec![];
        for file in files {
            if let Some(path) = file.path() {
                paths.push(path);
            }
        }
        if !paths.is_empty() {
            if let Some(sender) = APP_SENDER.get() {
                sender.input(AppMsg::OpenResponse(paths, true));
            } else {
                *OPEN_FILE.lock().unwrap() = Some(paths);
            }
        }
        app.activate();
    });

    gtk_app.connect_startup(|_app| {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
        set_global_css();
        initialize_custom_icons();
    });

    let relm4_app = RelmApp::from_app(gtk_app);
    relm4_app.run::<AppModel>(());
}

fn initialize_custom_icons() {
    gtk::gio::resources_register_include!("icons.gresource").unwrap();

    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/page/codeberg/M23Snezhok/PacTune/icons");
}

fn set_global_css() {
    // Load at APPLICATION priority (800) so it beats Adwaita's THEME (600)
    let provider = gtk::CssProvider::new();
    provider.load_from_string(
        r#"
    /* ═══════════════════════════════════════════════════════════
       PACTUNE — User Requested UI Theme
       ═══════════════════════════════════════════════════════════ */

    window, .background, .main-bg {
        background-color: #121212;
        color: #FFFFFF;
    }

    headerbar {
        background-color: transparent;
        border-bottom: none;
        box-shadow: none;
    }

    headerbar button.flat, .top-bar-btn, .ctrl-btn {
        color: rgba(255,255,255,0.65);
        border-radius: 8px;
        min-width: 32px;
        min-height: 32px;
        transition: all 150ms ease;
    }

    headerbar button.flat:hover, .top-bar-btn:hover, .ctrl-btn:hover {
        background-color: rgba(255,255,255,0.1);
        color: #fff;
    }

    /* ── Right panel / library area ──────────────────────────── */
    .view, scrolledwindow, viewport {
        background-color: transparent;
    }

    /* ── Scrollbars ──────────────────────────────────────────── */
    scrollbar, scrollbar.overlay-indicator {
        opacity: 0;
        background: transparent;
        border: none;
        box-shadow: none;
        min-width: 0;
        min-height: 0;
    }

    scrollbar slider {
        background-color: transparent;
        min-width: 0;
        min-height: 0;
    }

    scrollbar slider:hover, scrollbar slider:active {
        background-color: transparent;
    }

    scrollbar trough {
        background-color: transparent;
        border: none;
        min-width: 0;
        min-height: 0;
    }

    viewswitcher { margin-bottom: 8px; }

    /* ── StatusPage (Welcome Screen) Logo ────────────────────── */
    statuspage image {
        -gtk-icon-size: 256px;
    }

    /* ── Album grid cards ────────────────────────────────────── */
    .album-card-btn {
        border-radius: 8px;
        background: transparent;
        border: none;
        transition: all 150ms ease;
        padding: 12px;
        margin: 6px;
        min-width: 164px;
        min-height: 200px;
    }

    .album-card-btn:hover {
        background: rgba(255,255,255,0.04);
        border: none;
        transform: scale(1.02);
    }

    .album-grid .album-cover-container {
        width: 140px !important;
        height: 140px !important;
        min-width: 140px;
        min-height: 140px;
        max-width: 140px;
        max-height: 140px;
        border-radius: 4px;
        background: #282828;
        box-shadow: 0 4px 12px rgba(0,0,0,0.5);
    }
    
    .album-grid .album-cover-container picture {
        width: 140px !important;
        height: 140px !important;
    }

    .song-cover {
        border-radius: 4px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    }

    .large-title {
        font-weight: 500;
        font-size: 11pt;
        color: #f0f0f5;
        margin-top: 2px;
    }

    .small-title {
        font-size: 9pt;
        color: #B3B3B3;
    }

    /* ── Track list rows — Up Next style ────────────────────── */
    listview > row {
        border-radius: 12px;
        transition: background-color 150ms ease;
        padding: 4px;
    }

    listview > row:hover {
        background-color: rgba(255,255,255,0.08);
    }

    listview > row:selected {
        background: transparent;
    }

    .transparent-list > child > box > button.flat {
        border-radius: 8px;
        background: transparent;
        padding: 6px;
    }
    
    .track-background .large-title {
        color: #1DB954; /* Spotify Green */
        font-weight: 700;
    }

    .rounded-image {
        border-radius: 6px;
    }

    /* ── Progress slider ─────────────────────────────────────── */
    progressbar trough, trough {
        min-height: 4px;
        border-radius: 2px;
        background-color: rgba(255,255,255,0.15);
    }

    progressbar progress, trough highlight {
        background: #FFFFFF;
        border-radius: 2px;
        transition: background 150ms ease;
    }
    
    .sidebar-progress:hover trough highlight {
        background: #1DB954; /* Spotify green on hover */
    }

    slider {
        min-width: 12px;
        min-height: 12px;
        border-radius: 50%;
        background: #FFFFFF;
        box-shadow: 0 2px 6px rgba(0,0,0,0.3);
        transition: transform 150ms ease;
    }
    
    slider:hover {
        transform: scale(1.3);
    }

    .time-label {
        font-size: 10pt;
        font-weight: 500;
        color: #B3B3B3;
        min-width: 46px; /* Fixed width for digits to prevent jumps */
        text-align: center;
        font-feature-settings: "tnum"; /* Tabular (fixed-width) numbers */
    }

    /* ── Playback buttons ────────────────────────────────────── */
    .play-circle {
        min-width: 52px;
        min-height: 52px;
        border-radius: 50%;
        background: #FFFFFF;
        color: #000000;
        box-shadow: 0 4px 12px rgba(0,0,0,0.2);
        transition: transform 33ms ease;
        border: none;
    }

    .play-circle:hover {
        transform: scale(1.06); 
        box-shadow: 0 6px 16px rgba(0,0,0,0.3);
    }

    .play-circle image {
        -gtk-icon-size: 28px;
        color: #000000;
    }

    button.accent {
        color: #1DB954;
        background-color: transparent;
    }

    label.accent {
        color: #1DB954;
    }

    button.suggested-action {
        background: #FFFFFF;
        color: #000000;
        border-radius: 999px;
    }
    
    .ctrl-btn {
        transition: all 150ms ease;
        opacity: 0.7;
    }
    
    .ctrl-btn:hover {
        opacity: 1.0;
        transform: scale(1.1);
    }

    /* ── Custom Cards (Right Panel) ──────────────────────────── */
    .top-bar {
        min-width: 0;
        transition: none; /* No shifts allowed */
        width: 100%;
    }

    .right-panel, .right-panel > * {
        background: transparent;
        width: 300px !important;
        min-width: 300px !important;
        max-width: 300px !important;
        margin-right: 0 !important; /* Remove wiggle room */
        overflow: hidden;
        flex-basis: 300px;
        flex-grow: 0;
        flex-shrink: 0;
    }

    .player-card {
        background: transparent;
        border-radius: 8px;
        border: none;
        box-shadow: none;
        padding: 16px;
        margin-bottom: 12px;
        width: 300px;
        min-width: 300px;
        max-width: 300px;
    }

    .upnext-card {
        background: transparent;
        border-radius: 8px;
        border: none;
        padding: 8px;
        margin-bottom: 8px;
        max-width: 300px;
    }

    .player-cover {
        border-radius: 12px;
        box-shadow: 0 8px 24px rgba(0,0,0,0.5);
        margin-bottom: 24px;
        border: 1px solid rgba(255,255,255,0.08);
        width: 240px;
        height: 240px;
        min-width: 240px;
        max-width: 240px;
        margin-left: auto;
        margin-right: auto;
    }

    .player-title {
        font-size: 11.5pt;
        font-weight: 700;
        color: #fff;
        margin-bottom: 2px;
        min-width: 0;
        max-width: 276px;
        text-overflow: ellipsis;
    }

    .player-artist {
        font-size: 10pt;
        font-weight: 500;
        color: rgba(255,255,255,0.5);
        min-width: 0;
        max-width: 276px;
        text-overflow: ellipsis;
    }

    .upnext-header {
        font-size: 10pt;
        font-weight: 600;
        color: #fff;
    }

    .upnext-sub {
        font-size: 10pt;
        color: rgba(255,255,255,0.4);
    }
    
    .transparent-list {
        background-color: transparent;
        box-shadow: none;
    }
    
    .overlay-layer {
        opacity: 0;
        background: rgba(30, 30, 35, 0.9);
        transition: opacity 250ms ease-in-out;
        border-radius: 16px;
    }

    .overlay-layer.visible {
        opacity: 1;
    }
    "#);

    // Use USER priority (800) — highest available, beats Adwaita theme (600)
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}
