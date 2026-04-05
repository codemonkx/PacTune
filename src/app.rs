use gtk::glib::GString;
use gtk::prelude::WidgetExt;
use gtk::{ListScrollFlags, prelude::*};
use rand::prelude::*;
use relm4::abstractions::Toaster;
use relm4::adw::prelude::*;
use relm4::prelude::*;
use relm4::typed_view::list::TypedListView;
use relm4::{ComponentParts, ComponentSender, adw, gtk::gdk, gtk::glib};
use relm4_components::open_dialog::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::about::About;
use crate::backend::cover_cache::CoverCache;
use crate::backend::player_state::PlayerState;
use crate::backend::track_info::TrackInfo;
use crate::backend::utils::{open_paths, read_json, write_json, save_watch_folder, load_watch_folder};
use crate::drag_overlay::DragOverlayModel;
use crate::lyrics_item::LyricsItem;
use crate::menu::{MenuModel, MenuMsg};
use crate::track_item::TrackItem;
use crate::tracklist::{PlaylistModel, PlaylistMsg};
use crate::albumlist::{AlbumListModel, AlbumListMsg};
use crate::album_detail::{AlbumDetailModel, AlbumDetailMsg, AlbumDetailTrackItem};

#[tracker::track]
#[derive(Debug)]
pub struct AppModel {
    #[tracker::no_eq]
    open_dialog_folder: Controller<OpenDialogMulti>,
    #[tracker::no_eq]
    open_dialog_song: Controller<OpenDialogMulti>,
    #[tracker::no_eq]
    open_dialog_watch: Controller<OpenDialogMulti>,
    #[tracker::no_eq]
    playlist: Controller<PlaylistModel>,
    #[tracker::no_eq]
    albumlist: Controller<AlbumListModel>,
    #[tracker::no_eq]
    album_detail: Controller<AlbumDetailModel>,
    #[tracker::no_eq]
    menu: Controller<MenuModel>,
    #[tracker::no_eq]
    drag_overlay: Controller<DragOverlayModel>,
    #[tracker::no_eq]
    lyrics: TypedListView<LyricsItem, gtk::NoSelection>,
    #[tracker::no_eq]
    toaster: Toaster,
    #[tracker::no_eq]
    vec_for_restore: Vec<PathBuf>,
    #[tracker::no_eq]
    opened_folders: Vec<PathBuf>,
    #[tracker::no_eq]
    player_state: PlayerState,
    play_pause: &'static str,
    position: f64,
    duration: f64,
    getting_position: bool,
    repeat_stage: RepeatStage,
    repeat_icon: &'static str,
    is_shuffle: bool,
    show_lyrics: bool,
    volume_value: f64,
    volume_icon: &'static str,
    getting_volume: bool,
    is_drag: bool,
    visible_sidebar: bool,
    raw_lyrics: Vec<(Option<f64>, String)>,
    #[tracker::no_eq]
    watch_folder: Option<PathBuf>,
    current_list_title: String,
}

#[derive(Debug, PartialEq)]
pub enum RepeatStage {
    RepeatTrack,
    RepeatPlaylist,
    NotRepeat,
}

#[derive(Debug)]
pub enum AppMsg {
    OpenRequest,
    OpenResponse(Vec<PathBuf>, bool),
    OpenSong,
    Play,
    Stop,
    Pause,
    Ignore,
    Quit,
    Clear,
    ClearPlaylist,
    ShowAboutDialog,
    TrackSelected(usize),
    TrackPlayPause,
    TrackNext,
    TrackPrevious,
    StartGetPosition,
    EndGetPosition(f64),
    GetPosition(f64),
    SeekPositonLyrics(i64),
    RepeatTrack,
    ChangeVolume(f64),
    ChangeVolumeScale(f64),
    StartChangeVolume,
    EndChangeVolume,
    ChangeVolumeIcon,
    NotifyTrackEnd,
    UpdateMusicBox,
    UpdateUiInitialTrack,
    UpdateUiSimpleTrack,
    UpdatePosition(u64),
    AdwToastBuild(GString),
    AdwToastTrackBuild(GString),
    ShufflePlaylist,
    ToggleLyrics,
    RestorePlaylist,
    AddFilesToPlaylist(Vec<Arc<TrackInfo>>),
    Progress(f64),
    ProgressFinished,
    DragEnter,
    DragLeave,
    ShowHideSidebar,
    SetSidebarVisible(bool),
    ScrollLyrics(f64),
    ToggleSearch,
    SetRepeatStage(RepeatStage),
    OpenAlbumDetail(String),
    CloseAlbumDetail,
    LyricsReady(Vec<(Option<f64>, String)>),
    RefreshPlaylist,
    SetWatchFolder,
    WatchFolderResponse(Vec<PathBuf>),
    ShowFileInfo,
}

#[derive(Debug)]
pub enum CommandMsg {
    AddFiles(Vec<Arc<TrackInfo>>),
    RestoreFiles(Vec<Arc<TrackInfo>>),
}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = CommandMsg;

    view! {
        #[name(main_window)]
        adw::ApplicationWindow {
            set_default_size: (1000, 700),
            set_size_request: (700, 500),
            set_maximized: true,
            set_decorated: false, // For tiling support
            #[name(main_overlay)]
            gtk::Overlay {
                #[local_ref]
                toast_overlay -> adw::ToastOverlay {
                    #[name(main_stack)]
                    gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::Crossfade,

                        // Welcome screen
                        add_child = &adw::ToolbarView {
                            add_top_bar = &adw::HeaderBar { set_show_title: false },
                            adw::Clamp {
                                set_maximum_size: 480,
                                set_vexpand: true,
                                set_valign: gtk::Align::Fill,
                                adw::StatusPage {
                                    set_icon_name: Some("PacTune"),
                                    set_title: "Vinyl",
                                    set_description: Some("Drop a folder here, or pick one below."),
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_halign: gtk::Align::Center,
                                        set_spacing: 12,
                                        gtk::Button {
                                            set_label: "Restore",
                                            set_css_classes: &["pill", "suggested-action"],
                                            connect_clicked => AppMsg::RestorePlaylist,
                                        },
                                        gtk::Button {
                                            set_label: "Open Folder",
                                            add_css_class: "pill",
                                            connect_clicked => AppMsg::OpenRequest,
                                        },
                                        gtk::Button {
                                            set_label: "Add Song",
                                            add_css_class: "pill",
                                            connect_clicked => AppMsg::OpenSong,
                                        },
                                    }
                                }
                            }
                        } -> { set_name: "initial_view" },

                        // Main view: New design — album grid + right panel player
                        add_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "main-bg",

                            // Top bar: rigid 3-column anchor
                            gtk::Grid {
                                add_css_class: "top-bar",
                                set_margin_start: 8,
                                set_margin_end: 8,
                                set_margin_top: 4,
                                set_margin_bottom: 4,
                                set_column_spacing: 0,

                                // Column 0: Left Anchor (60px)
                                attach[0, 0, 1, 1] = &gtk::Box {
                                    set_halign: gtk::Align::Start,
                                    set_width_request: 60,
                                    #[name(pack_start_main_header)]
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 4,
                                    },
                                },

                                // Column 1: Middle Spacer (flexible)
                                attach[1, 0, 1, 1] = &gtk::Box {
                                    set_hexpand: true,
                                },

                                // Column 2: Right Anchor (60px)
                                attach[2, 0, 1, 1] = &gtk::Box {
                                    set_halign: gtk::Align::End,
                                    set_width_request: 60,
                                    gtk::WindowControls {
                                        set_side: gtk::PackType::End,
                                    }
                                },
                            },

                            // Body: album grid + right panel
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_vexpand: true,
                                set_margin_start: 16,
                                set_margin_end: 16,
                                set_margin_top: 0,
                                set_margin_bottom: 16,
                                set_spacing: 16,

                                // ALBUM GRID — takes all remaining space
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_hexpand: true,
                                    set_halign: gtk::Align::Fill,

                                    #[name(progress_bar)]
                                    gtk::ProgressBar {
                                        set_visible: false,
                                        add_css_class: "osd",
                                    },

                                    #[name(content_nav_view)]
                                    adw::NavigationView {
                                        set_vexpand: true,
                                        add = &adw::NavigationPage {
                                            set_tag: Some("albums_view"),
                                            set_can_pop: false,
                                            #[wrap(Some)]
                                            set_child = &gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_vexpand: true,
                                                #[name(main_content_stack)]
                                                append = &adw::ViewStack {
                                                    set_vexpand: true,
                                                    add_titled[Some("Albums"), "Albums"] = &gtk::Box {
                                                        set_orientation: gtk::Orientation::Vertical,
                                                        #[name(albumlist_box)]
                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Vertical,
                                                            set_vexpand: true,
                                                        }
                                                    },
                                                    add_titled[Some("Tracks"), "Tracks"] = &gtk::Box {
                                                        set_orientation: gtk::Orientation::Vertical,
                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Vertical,
                                                            set_vexpand: true,
                                                        }
                                                    },
                                                },
                                            },
                                        },
                                        #[name(album_detail_page)]
                                        add = &adw::NavigationPage {
                                            set_tag: Some("album_detail"),
                                            #[wrap(Some)]
                                            set_child = &gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_vexpand: true,
                                                #[name(album_detail_box)]
                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Vertical,
                                                    set_vexpand: true,
                                                },
                                            },
                                        },
                                    },


                                },

                                // RIGHT PANEL — fixed 300px, never expands
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_hexpand: false,
                                    set_vexpand: true,
                                    set_halign: gtk::Align::End,
                                    set_valign: gtk::Align::Fill,
                                    set_width_request: 300,
                                    add_css_class: "right-panel",

                                    adw::Clamp {
                                        set_maximum_size: 300,
                                        set_halign: gtk::Align::Fill,
                                        set_vexpand: true,
                                        
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_vexpand: true,

                                            // Player card
                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_hexpand: false,
                                                set_margin_bottom: 12,
                                                set_margin_start: 12,
                                                set_margin_end: 12,
                                                add_css_class: "player-card",

                                                // Cover art — fixed size, never drives panel width
                                                #[name(cover)]
                                                gtk::Image {
                                                    add_css_class: "player-cover",
                                                    set_overflow: gtk::Overflow::Hidden,
                                                    set_icon_name: Some("PacTune"),
                                                    set_pixel_size: 300,
                                                    set_width_request: 300,
                                                    set_height_request: 300,
                                                    set_hexpand: false,
                                                    set_halign: gtk::Align::Center,
                                                    set_margin_bottom: 16,
                                                },

                                                // Track info
                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Vertical,
                                                    set_margin_top: 8,
                                                    set_margin_start: 12,
                                                    set_margin_end: 12,
                                                    set_hexpand: false,
                                                    set_vexpand: false,
                                                    set_halign: gtk::Align::Fill,
                                                    set_width_request: 276,
                                                    adw::Clamp {
                                                        set_maximum_size: 276,
                                                        set_halign: gtk::Align::Fill,
                                                        #[name(title)]
                                                        gtk::Label {
                                                            add_css_class: "player-title",
                                                            set_label: "No track playing",
                                                            set_halign: gtk::Align::Fill,
                                                            set_xalign: 0.5,
                                                            set_hexpand: false,
                                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                                            set_lines: 1,
                                                            set_max_width_chars: 30,
                                                        },
                                                    },
                                                    adw::Clamp {
                                                        set_maximum_size: 276,
                                                        set_halign: gtk::Align::Fill,
                                                        #[name(artist)]
                                                        gtk::Label {
                                                            add_css_class: "player-artist",
                                                            set_halign: gtk::Align::Fill,
                                                            set_xalign: 0.5,
                                                            set_hexpand: false,
                                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                                            set_lines: 1,
                                                            set_max_width_chars: 30,
                                                        },
                                                    },
                                                    #[name(album)]
                                                    gtk::Label {
                                                        set_visible: false,
                                                    },
                                                },

                                                // Progress + time (Spotify-style inline)
                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_valign: gtk::Align::Center,
                                                    set_margin_top: 24,
                                                    set_margin_bottom: 8,
                                                    set_spacing: 12,
                                                    add_css_class: "progress-box",
                                                    
                                                    gtk::Label {
                                                        add_css_class: "time-label",
                                                        set_width_request: 46,
                                                        set_halign: gtk::Align::Start,
                                                        #[watch]
                                                        set_class_active: ("accent", model.getting_position),
                                                        #[watch]
                                                        set_label: &format!("{:02}:{:02}", model.position as u32 / 60, model.position as u32 % 60),
                                                    },
                                                    gtk::Scale {
                                                        add_css_class: "sidebar-progress",
                                                        set_orientation: gtk::Orientation::Horizontal,
                                                        set_hexpand: true,
                                                        set_draw_value: false,
                                                        set_valign: gtk::Align::Center,
                                                        #[watch]
                                                        set_range: (0.0, model.duration),
                                                        #[watch]
                                                        set_value: model.position,
                                                        connect_change_value[sender] => move |_, _, value| {
                                                            sender.input(AppMsg::GetPosition(value));
                                                            gtk::glib::Propagation::Stop
                                                        },
                                                        add_controller = gtk::EventControllerLegacy {
                                                            set_propagation_phase: gtk::PropagationPhase::Capture,
                                                            connect_event[sender] => move |controller, event| {
                                                                match event.event_type() {
                                                                    gdk::EventType::ButtonPress => sender.input(AppMsg::StartGetPosition),
                                                                    gdk::EventType::ButtonRelease => {
                                                                        let scale = controller.widget().unwrap().downcast::<gtk::Scale>().unwrap();
                                                                        sender.input(AppMsg::EndGetPosition(scale.value()));
                                                                    },
                                                                    _ => {}
                                                                }
                                                                gtk::glib::Propagation::Proceed
                                                            },
                                                        },
                                                    },
                                                    gtk::Label {
                                                        add_css_class: "time-label",
                                                        set_width_request: 46,
                                                        set_halign: gtk::Align::End,
                                                        #[watch]
                                                        set_label: &{
                                                            let rem = (model.duration as i64 - model.position as i64).max(0);
                                                            format!("-{:02}:{:02}", rem as u32 / 60, rem as u32 % 60)
                                                        },
                                                    },
                                                },

                                                // Playback controls
                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_halign: gtk::Align::Center,
                                                    set_spacing: 16,
                                                    set_margin_top: 12,
                                                    gtk::Button {
                                                        add_css_class: "flat",
                                                        add_css_class: "ctrl-btn",
                                                        set_icon_name: "view-continuous-symbolic",
                                                        #[watch]
                                                        set_class_active: ("accent", model.show_lyrics),
                                                        connect_clicked => AppMsg::ToggleLyrics,
                                                    },
                                                    gtk::Button {
                                                        add_css_class: "flat",
                                                        add_css_class: "ctrl-btn",
                                                        set_icon_name: "media-skip-backward-symbolic",
                                                        connect_clicked => AppMsg::TrackPrevious,
                                                    },
                                                    gtk::Button {
                                                        set_css_classes: &["suggested-action", "play-circle"],
                                                        #[watch]
                                                        set_icon_name: &model.play_pause,
                                                        connect_clicked => AppMsg::TrackPlayPause,
                                                    },
                                                    gtk::Button {
                                                        add_css_class: "flat",
                                                        add_css_class: "ctrl-btn",
                                                        set_icon_name: "media-skip-forward-symbolic",
                                                        connect_clicked => AppMsg::TrackNext,
                                                    },
                                                    gtk::Button {
                                                        add_css_class: "flat",
                                                        add_css_class: "ctrl-btn",
                                                        #[watch]
                                                        set_icon_name: &model.repeat_icon,
                                                        connect_clicked => AppMsg::RepeatTrack,
                                                    },
                                                },

                                                // View switcher / Now Playing (replaces volume row to match mock bottom tabs)

                                            },

                                            // "Up Next" tracklist
                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_vexpand: true,
                                                set_margin_start: 12,
                                                set_margin_end: 12,
                                                set_margin_bottom: 12,
                                                add_css_class: "upnext-card",

                                                #[name(right_panel_stack)]
                                                gtk::Stack {
                                                    set_transition_type: gtk::StackTransitionType::Crossfade,
                                                    set_vexpand: true,
                                                    #[watch]
                                                    set_visible_child_name: if model.show_lyrics { "lyrics" } else { "tracklist" },
                                                    
                                                    add_child = &gtk::ScrolledWindow {
                                                        set_vexpand: true,
                                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                                        set_vscrollbar_policy: gtk::PolicyType::Automatic,
                                                        #[name(tracklist_box)]
                                                        gtk::Box {
                                                            set_orientation: gtk::Orientation::Vertical,
                                                            set_vexpand: true,
                                                        },
                                                    } -> { set_name: "tracklist" },
                                                    
                                                    add_child = &gtk::ScrolledWindow {
                                                        set_vexpand: true,
                                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                                        set_vscrollbar_policy: gtk::PolicyType::Automatic,
                                                        set_propagate_natural_height: true,
                                                        #[local_ref]
                                                        lyrics_view -> gtk::ListView {
                                                            set_align: gtk::Align::Center,
                                                            add_css_class: "track-list",
                                                            set_single_click_activate: true,
                                                        },
                                                    } -> { set_name: "lyrics" }
                                                }
                                            },
                                        }
                                    },
                                },
                            },
                        } -> { set_name: "main_view" },
                    },
                },
                // Drag overlay
                add_overlay = &adw::Clamp {
                    set_can_target: false,
                    add_css_class: "overlay-layer",
                    #[watch]
                    set_class_active: ("visible", model.is_drag),
                    set_maximum_size: 700,
                    set_vexpand: true,
                    set_align: gtk::Align::Fill,
                    adw::StatusPage {
                        set_align: gtk::Align::Center,
                        set_icon_name: Some("audio-x-generic-symbolic"),
                        set_title: "Drop here",
                        set_width_request: 300,
                        set_hexpand: true,
                    }
                },
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let lyrics: TypedListView<LyricsItem, gtk::NoSelection> = TypedListView::new();

        let open_dialog_folder = OpenDialogMulti::builder()
            .transient_for_native(&root)
            .launch(OpenDialogSettings { folder_mode: true, ..Default::default() })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(paths) => AppMsg::OpenResponse(paths, true),
                OpenDialogResponse::Cancel => AppMsg::Ignore,
            });

        let open_dialog_song = {
            let audio_filter = gtk::FileFilter::new();
            audio_filter.set_name(Some("Audio Files"));
            ["audio/*"].into_iter().for_each(|mime| audio_filter.add_mime_type(mime));
            OpenDialogMulti::builder()
                .transient_for_native(&root)
                .launch(OpenDialogSettings { filters: vec![audio_filter], ..Default::default() })
                .forward(sender.input_sender(), |response| match response {
                    OpenDialogResponse::Accept(paths) => AppMsg::OpenResponse(paths, true),
                    OpenDialogResponse::Cancel => AppMsg::Ignore,
                })
        };

        open_dialog_song.widget().set_title("Select song");
        open_dialog_folder.widget().set_title("Select folder");

        let open_dialog_watch = OpenDialogMulti::builder()
            .transient_for_native(&root)
            .launch(OpenDialogSettings { folder_mode: true, ..Default::default() })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(paths) => AppMsg::WatchFolderResponse(paths),
                OpenDialogResponse::Cancel => AppMsg::Ignore,
            });
        open_dialog_watch.widget().set_title("Select watch folder");

        let player_state = glib::clone!(
            #[strong] sender,
            move || PlayerState::new(Vec::new(), sender)
        )();

        let playlist = PlaylistModel::builder().launch(()).forward(sender.input_sender(), |msg| msg);
        let albumlist = AlbumListModel::builder().launch(()).forward(sender.input_sender(), |msg| msg);
        let drag_overlay = DragOverlayModel::builder().launch(()).forward(sender.input_sender(), |msg| msg);

        let menu = glib::clone!(
            #[strong] sender,
            #[strong(rename_to=main_window)] root,
            move || MenuModel::builder().launch(main_window).forward(sender.input_sender(), |msg| msg)
        )();

        let album_detail = AlbumDetailModel::builder().launch(()).forward(sender.input_sender(), |msg| msg);

        let model = AppModel {
            open_dialog_folder,
            open_dialog_song,
            open_dialog_watch,
            player_state,
            vec_for_restore: Vec::new(),
            opened_folders: Vec::new(),
            lyrics,
            drag_overlay,
            toaster: Toaster::default(),
            tracker: 0,
            playlist,
            albumlist,
            album_detail,
            menu,
            play_pause: "media-playback-start-symbolic",
            position: 0.0,
            duration: 100.0,
            getting_position: false,
            getting_volume: false,
            repeat_stage: RepeatStage::NotRepeat,
            repeat_icon: "media-playlist-consecutive-symbolic",
            is_shuffle: false,
            show_lyrics: false,
            volume_value: 1.0,
            volume_icon: "audio-volume-high-symbolic",
            is_drag: false,
            visible_sidebar: true,
            raw_lyrics: vec![],
            watch_folder: load_watch_folder(),
            current_list_title: "Up Next".to_string(),
        };

        let lyrics_view = &model.lyrics.view;
        let toast_overlay = model.toaster.overlay_widget();
        let widgets = view_output!();

        glib::clone!(
            #[weak(rename_to=main_overlay)] widgets.main_overlay,
            #[weak(rename_to=drag_overlay)] model.drag_overlay.widget(),
            move || { main_overlay.add_controller(drag_overlay); }
        )();

        widgets.albumlist_box.append(model.albumlist.widget());
        widgets.album_detail_box.append(model.album_detail.widget());
        widgets.tracklist_box.append(model.playlist.widget());
        widgets.pack_start_main_header.append(model.menu.widget());

        glib::clone!(
            #[strong] sender,
            move || { let _ = crate::APP_SENDER.set(sender); }
        )();

        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let sender_key = sender.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::space {
                sender_key.input(AppMsg::TrackPlayPause);
                return gtk::glib::Propagation::Stop;
            }
            gtk::glib::Propagation::Proceed
        });
        widgets.main_window.add_controller(key_controller);

        let has_saved = { let (vec, _, _) = read_json(); vec.is_some() };
        if has_saved {
            widgets.main_stack.set_visible_child_name("main_view");
        }

        gtk::glib::idle_add_local_once({
            move || {
                if let Some(path) = crate::OPEN_FILE.lock().unwrap().take() {
                    sender.input(AppMsg::OpenResponse(path, true));
                } else if has_saved {
                    sender.input(AppMsg::RestorePlaylist);
                    if let Some(wf) = load_watch_folder() {
                        if wf.exists() { sender.input(AppMsg::OpenResponse(vec![wf], true)); }
                    }
                } else if let Some(wf) = load_watch_folder() {
                    if wf.exists() { sender.input(AppMsg::OpenResponse(vec![wf], true)); }
                }
            }
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();
        let mut rng = rand::rng();
        match message {
            AppMsg::ToggleSearch => {
                self.playlist.emit(PlaylistMsg::ToggleSearch);
            },
            AppMsg::OpenRequest => self.open_dialog_folder.emit(OpenDialogMsg::Open),
            AppMsg::OpenSong => self.open_dialog_song.emit(OpenDialogMsg::Open),
            AppMsg::ShufflePlaylist => self.set_is_shuffle(!self.is_shuffle),
            AppMsg::ToggleLyrics => self.set_show_lyrics(!self.show_lyrics),
            AppMsg::Stop => {
                self.player_state.stop();
                self.set_play_pause("media-playback-start-symbolic");
                self.set_position(0.0);
            }
            AppMsg::DragEnter => self.set_is_drag(true),
            AppMsg::DragLeave => self.set_is_drag(false),
            AppMsg::ShowHideSidebar => self.set_visible_sidebar(!self.visible_sidebar),
            AppMsg::SetSidebarVisible(visible) => self.set_visible_sidebar(visible),
            AppMsg::ClearPlaylist => {
                widgets.main_stack.set_visible_child_name("initial_view");
                sender.input(AppMsg::Clear);
            }
            AppMsg::ShowAboutDialog => {
                let about = About::builder().launch(()).detach();
                glib::clone!(
                    #[weak] root,
                    move || about.widgets().dialog.present(Some(&root))
                )()
            }
            AppMsg::Clear => {
                self.playlist.emit(PlaylistMsg::ChangeStackChild("empty_playlist".to_string()));
                self.albumlist.emit(AlbumListMsg::ChangeStackChild("empty_albumlist".to_string()));
                self.playlist.emit(PlaylistMsg::Clear);
                self.albumlist.emit(AlbumListMsg::Clear);
                let mut cache = CoverCache::global().lock().unwrap();
                cache.clear();
                self.player_state.clear();
                widgets.title.set_label("No track playing");
                widgets.artist.set_label("");
                widgets.album.set_label("");
                self.set_duration(100.0);
                widgets.cover.set_icon_name(Some("PacTune"));
                self.album_detail.emit(AlbumDetailMsg::Clear);
                while widgets.content_nav_view.navigation_stack().n_items() > 1 {
                    widgets.content_nav_view.pop();
                }

                self.set_current_list_title("Up Next".to_string());
                if !self.vec_for_restore.is_empty() {
                    write_json(&self.vec_for_restore, self.player_state.current_track(), self.position);
                    self.vec_for_restore.clear();
                }
                self.opened_folders.clear();
            }
            AppMsg::Play => {
                self.player_state.play();
                self.set_play_pause("media-playback-pause-symbolic");
            }
            AppMsg::Pause => {
                self.player_state.pause();
                self.set_play_pause("media-playback-start-symbolic");
            }
            AppMsg::Quit => { relm4::main_application().quit(); }
            AppMsg::Progress(process) => {
                widgets.main_stack.set_visible_child_name("main_view");
                widgets.progress_bar.set_visible(true);
                widgets.progress_bar.set_fraction(process);
                self.menu.emit(MenuMsg::SetActionsEnabled(false));
            }
            AppMsg::ProgressFinished => {
                widgets.progress_bar.set_visible(false);
                self.playlist.emit(PlaylistMsg::ChangeStackChild("full_playlist".to_string()));
                self.albumlist.emit(AlbumListMsg::ChangeStackChild("full_albumlist".to_string()));
                self.menu.emit(MenuMsg::SetActionsEnabled(true));
            }
            AppMsg::TrackNext => {
                if self.is_shuffle {
                    let idx = rng.random_range(0..self.player_state.len());
                    self.player_state.set_current_track(idx);
                    self.playlist.emit(PlaylistMsg::ScrollTracklist(idx as u32));
                    sender.input(AppMsg::UpdateUiSimpleTrack);
                    self.player_state.play_track();
                } else {
                    self.player_state.track_next(&self.repeat_stage);
                }
                self.player_state.seek_position(0.0);
            }
            AppMsg::TrackPrevious => {
                self.player_state.track_previous(&self.repeat_stage);
                self.player_state.seek_position(0.0);
            }
            AppMsg::UpdateMusicBox => {
                if let Some(track) = self.player_state.get_track() {
                    widgets.title.set_label(track.title());
                    widgets.artist.set_label(track.artist());
                    widgets.album.set_label(track.album());
                    self.playlist.emit(PlaylistMsg::FilterByAlbum(Some(track.album().to_string())));
                    self.playlist.emit(PlaylistMsg::AccentTrack(self.player_state.current_track() as u32));
                    self.lyrics.clear();

                    let artist_name = track.artist().to_string();
                    let title_name = track.title().to_string();
                    let lrc_path = track.path().with_extension("lrc");

                    if !lrc_path.exists() {
                        let sender_clone = sender.clone();
                        tokio::spawn(async move {
                            if let Some(downloaded_lrc) = crate::backend::lyrics_fetcher::fetch_lrc(&artist_name, &title_name).await {
                                let _ = std::fs::write(&lrc_path, downloaded_lrc.as_bytes());
                                use lyrx::Lyrics;
                                if let Ok(parsed) = Lyrics::from_str(&downloaded_lrc) {
                                    sender_clone.input(AppMsg::LyricsReady(parsed.to_vec()));
                                }
                            }
                        });
                    }

                    let lyrics = track.lyrics().clone();
                    let duration = track.duration();
                    let cover_uuid = track.cover_uuid().map(|s| s.to_string());

                    if !lyrics.is_empty() {
                        for (position, text) in lyrics.iter() {
                            if text.is_empty() { continue; }
                            let lyric_item = glib::clone!(
                                #[strong] sender,
                                move || LyricsItem::new(sender, text.to_string(), position.map(|v| v as i64))
                            )();
                            self.lyrics.append(lyric_item);
                        }
                    }

                    if let Some(uuid) = cover_uuid {
                        let mut cache = CoverCache::global().lock().unwrap();
                        if let Some(texture) = cache.get_texture(&uuid) {
                            widgets.cover.set_paintable(Some(&texture));
                        } else {
                            widgets.cover.set_icon_name(Some("Vinyl"));
                        }
                    }

                    self.set_duration(duration as f64);
                    self.set_raw_lyrics(lyrics);
                } else {
                    sender.input(AppMsg::AdwToastBuild("Error updating music box".into()));
                }
            }
            AppMsg::UpdateUiInitialTrack => {
                sender.input(AppMsg::UpdateMusicBox);
                self.set_play_pause("media-playback-start-symbolic");
            }
            AppMsg::UpdateUiSimpleTrack => {
                sender.input(AppMsg::UpdateMusicBox);
                self.set_position(0.0);
                self.set_play_pause("media-playback-pause-symbolic");
            }
            AppMsg::AdwToastBuild(message) => {
                let toast = adw::Toast::builder().title(message).timeout(2).build();
                self.toaster.add_toast(toast);
            }
            AppMsg::AdwToastTrackBuild(message) => {
                let toast = adw::Toast::builder().title(message).timeout(4).button_label("Play").build();
                let index = self.player_state.len() - 1;
                glib::clone!(
                    #[strong] sender,
                    #[weak] toast,
                    move || {
                        toast.connect_button_clicked(move |this| {
                            this.dismiss();
                            sender.input(AppMsg::TrackSelected(index));
                        });
                    }
                )();
                self.toaster.add_toast(toast);
            }
            AppMsg::NotifyTrackEnd => {
                match self.repeat_stage {
                    RepeatStage::NotRepeat => {
                        if self.player_state.current_track() == self.player_state.len() - 1 {
                            self.player_state.set_current_track(0);
                            self.player_state.stop_track();
                            self.set_play_pause("media-playback-start-symbolic");
                            self.set_position(0.0);
                            return sender.input(AppMsg::UpdateUiInitialTrack);
                        }
                    }
                    RepeatStage::RepeatTrack => return self.player_state.play_track(),
                    RepeatStage::RepeatPlaylist => {
                        if self.player_state.current_track() == self.player_state.len() - 1 {
                            self.player_state.set_current_track(0);
                            self.player_state.play_track();
                            return sender.input(AppMsg::UpdateMusicBox);
                        }
                    }
                }
                return sender.input(AppMsg::TrackNext);
            }
            AppMsg::UpdatePosition(position) => {
                if self.player_state.is_playing() && !self.getting_position {
                    self.set_position(position as f64 / 1000.0);
                    if !self.raw_lyrics.is_empty() {
                        sender.input(AppMsg::ScrollLyrics(position as f64 / 1000.0));
                    }
                }
            }
            AppMsg::GetPosition(position) => { self.set_position(position); }
            AppMsg::StartGetPosition => { self.set_getting_position(true); }
            AppMsg::EndGetPosition(value) => {
                self.set_position(value);
                self.player_state.seek_position(self.position);
                self.set_getting_position(false);
            }
            AppMsg::SeekPositonLyrics(value) => {
                if value > self.duration as i64 {
                    sender.input(AppMsg::AdwToastBuild("Position not found".into()));
                } else {
                    self.set_position(value as f64);
                    self.player_state.seek_position(self.position);
                }
            }
            AppMsg::ScrollLyrics(current_time) => {
                let mut best_idx = None;
                for (idx, choice) in self.lyrics.iter().enumerate() {
                    if let Some(lyric_time) = choice.borrow().position() {
                        if lyric_time as f64 <= current_time {
                            best_idx = Some(idx as u32);
                        } else {
                            break;
                        }
                    }
                }

                if let Some(idx) = best_idx {
                    self.lyrics.iter().for_each(|choice| choice.borrow().unaccent());
                    if let Some(cell) = self.lyrics.get(idx) {
                        cell.borrow().accent();
                        widgets.lyrics_view.scroll_to(idx, gtk::ListScrollFlags::FOCUS, None);
                    }
                }
            }
            AppMsg::OpenResponse(paths, sort) => {
                if self.playlist.model().tracks.is_empty() {
                    self.player_state.set_current_track(0);
                    self.player_state.seek_position(0.0);
                    self.set_position(0.0);
                }
                // Track opened folders for refresh
                for p in &paths {
                    if p.is_dir() && !self.opened_folders.contains(p) {
                        self.opened_folders.push(p.clone());
                    }
                }
                sender.oneshot_command(glib::clone!(
                    #[strong] sender,
                    async move {
                        let items_backend = open_paths(paths, sender, sort).await;
                        CommandMsg::AddFiles(items_backend)
                    }
                ));
            }
            AppMsg::TrackSelected(index) => {
                if self.player_state.current_track() != index {
                    self.player_state.set_current_track(index);
                    sender.input(AppMsg::UpdateUiSimpleTrack);
                    self.player_state.seek_position(0.0);
                    self.player_state.play_track();
                } else {
                    sender.input(AppMsg::TrackPlayPause);
                }
            }
            AppMsg::TrackPlayPause => {
                if self.player_state.is_playing() { sender.input(AppMsg::Pause); }
                else { sender.input(AppMsg::Play); }
            }
            AppMsg::RepeatTrack => match self.repeat_stage {
                RepeatStage::NotRepeat => sender.input(AppMsg::SetRepeatStage(RepeatStage::RepeatPlaylist)),
                RepeatStage::RepeatPlaylist => sender.input(AppMsg::SetRepeatStage(RepeatStage::RepeatTrack)),
                RepeatStage::RepeatTrack => sender.input(AppMsg::SetRepeatStage(RepeatStage::NotRepeat)),
            },
            AppMsg::SetRepeatStage(stage) => {
                self.set_repeat_stage(stage);
                self.set_repeat_icon(match self.repeat_stage {
                    RepeatStage::RepeatTrack => "media-playlist-repeat-song-symbolic",
                    RepeatStage::RepeatPlaylist => "media-playlist-repeat-symbolic",
                    RepeatStage::NotRepeat => "media-playlist-consecutive-symbolic",
                });
            }
            AppMsg::OpenAlbumDetail(album_name) => {
                self.playlist.emit(PlaylistMsg::FilterByAlbum(Some(album_name)));
                self.set_current_list_title("Up Next".to_string());
                self.set_show_lyrics(false);
            }
            AppMsg::CloseAlbumDetail => {
                self.playlist.emit(PlaylistMsg::FilterByAlbum(None));
                self.set_current_list_title("Up Next".to_string());
            }
            AppMsg::LyricsReady(lines) => {
                if self.raw_lyrics.is_empty() && !lines.is_empty() {
                    self.lyrics.clear();
                    for (position, text) in lines.iter() {
                        if text.is_empty() { continue; }
                        let lyric_item = glib::clone!(
                            #[strong] sender,
                            move || LyricsItem::new(sender, text.to_string(), position.map(|v| v as i64))
                        )();
                        self.lyrics.append(lyric_item);
                    }
                    self.set_raw_lyrics(lines);
                }
            }
            AppMsg::RefreshPlaylist => {
                // Prefer opened folders (re-scans for new files), fall back to watch folder, then file list
                let paths_to_reload: Option<Vec<PathBuf>> = if !self.opened_folders.is_empty() {
                    Some(self.opened_folders.clone())
                } else if let Some(wf) = load_watch_folder() {
                    if wf.exists() { Some(vec![wf]) } else { None }
                } else if !self.vec_for_restore.is_empty() {
                    Some(self.vec_for_restore.clone())
                } else {
                    None
                };

                if let Some(paths) = paths_to_reload {
                    // Clear UI and state but keep paths safe
                    self.playlist.emit(PlaylistMsg::ChangeStackChild("empty_playlist".to_string()));
                    self.albumlist.emit(AlbumListMsg::ChangeStackChild("empty_albumlist".to_string()));
                    self.playlist.emit(PlaylistMsg::Clear);
                    self.albumlist.emit(AlbumListMsg::Clear);
                    let mut cache = CoverCache::global().lock().unwrap();
                    cache.clear();
                    self.player_state.clear();
                    widgets.title.set_label("No track playing");
                    widgets.artist.set_label("");
                    widgets.album.set_label("");
                    self.set_duration(100.0);
                    widgets.cover.set_icon_name(Some("Vinyl"));
                    self.album_detail.emit(AlbumDetailMsg::Clear);
                    while widgets.content_nav_view.navigation_stack().n_items() > 1 {
                        widgets.content_nav_view.pop();
                    }
                    self.vec_for_restore.clear();
                    // Keep opened_folders so refresh works again after
                    self.player_state.set_current_track(0);
                    self.set_position(0.0);
                    sender.input(AppMsg::OpenResponse(paths, true));
                } else {
                    sender.input(AppMsg::AdwToastBuild("Nothing to refresh".into()));
                }
            }
            AppMsg::SetWatchFolder => { self.open_dialog_watch.emit(OpenDialogMsg::Open); }
            AppMsg::WatchFolderResponse(paths) => {
                if let Some(folder) = paths.into_iter().next() {
                    save_watch_folder(&folder);
                    self.set_watch_folder(Some(folder.clone()));
                    sender.input(AppMsg::AdwToastBuild(
                        format!("Watch folder: {}", folder.file_name().and_then(|n| n.to_str()).unwrap_or("folder")).into()
                    ));
                }
            }
            AppMsg::ShowFileInfo => {
                if let Some(track) = self.player_state.get_track() {
                    let path = track.path().to_string_lossy().to_string();
                    let ext = track.path().extension().and_then(|e| e.to_str()).unwrap_or("unknown").to_uppercase();
                    let bitrate = track.bitrate.map(|b| format!("{} kbps", b)).unwrap_or_else(|| "Unknown".to_string());
                    let sample_rate = track.sample_rate.map(|s| format!("{} Hz", s)).unwrap_or_else(|| "Unknown".to_string());
                    let channels = track.channels.map(|c| if c == 1 { "Mono".to_string() } else { "Stereo".to_string() }).unwrap_or_else(|| "Unknown".to_string());
                    let file_size = std::fs::metadata(track.path()).map(|m| {
                        let b = m.len();
                        if b > 1_000_000 { format!("{:.1} MB", b as f64 / 1_000_000.0) } else { format!("{} KB", b / 1000) }
                    }).unwrap_or_else(|_| "Unknown".to_string());
                    let dialog = adw::AlertDialog::new(
                        Some(track.title()),
                        Some(&format!("Format: {}\nBitrate: {}\nSample Rate: {}\nChannels: {}\nSize: {}\n\nPath: {}", ext, bitrate, sample_rate, channels, file_size, path)),
                    );
                    dialog.add_response("ok", "OK");
                    dialog.present(Some(root));
                } else {
                    sender.input(AppMsg::AdwToastBuild("No track playing".into()));
                }
            }
            AppMsg::ChangeVolume(value) => {
                if !self.getting_volume {
                    self.set_volume_value(value);
                    sender.input(AppMsg::ChangeVolumeIcon);
                }
            }
            AppMsg::ChangeVolumeScale(value) => {
                self.set_volume_value(value);
                self.player_state.set_volume(value);
                sender.input(AppMsg::ChangeVolumeIcon);
            }
            AppMsg::StartChangeVolume => { self.set_getting_volume(true); }
            AppMsg::EndChangeVolume => { self.set_getting_volume(false); }
            AppMsg::ChangeVolumeIcon => {
                self.set_volume_icon(if self.volume_value >= 0.8 { "audio-volume-high-symbolic" }
                    else if self.volume_value >= 0.5 { "audio-volume-medium-symbolic" }
                    else if self.volume_value > 0.0 { "audio-volume-low-symbolic" }
                    else { "audio-volume-muted-symbolic" });
            }
            AppMsg::RestorePlaylist => {
                if self.vec_for_restore.is_empty() {
                    let (vec, current_track, position) = read_json();
                    if let Some(vec) = vec {
                        self.set_vec_for_restore(vec);
                        self.player_state.set_current_track(current_track);
                        self.set_position(position);
                        self.player_state.seek_position(position);
                        sender.input(AppMsg::RestorePlaylist);
                    }
                } else {
                    sender.oneshot_command(glib::clone!(
                        #[strong] sender,
                        #[strong(rename_to=vec_for_restore)] self.vec_for_restore,
                        async move {
                            let items_backend = open_paths(vec_for_restore, sender, false).await;
                            CommandMsg::RestoreFiles(items_backend)
                        }
                    ));
                }
            }
            AppMsg::AddFilesToPlaylist(items_backend) => {
                let mut items_front = vec![];
                let mut album_items = vec![];
                let mut seen_albums = std::collections::HashSet::new();
                let len = self.player_state.len();
                let start_position = len - items_backend.len();
                for (i, item) in items_backend.iter().enumerate() {
                    let position = start_position + i;
                    let track_item = glib::clone!(
                        #[strong] sender, #[strong] item,
                        move || TrackItem::new(sender, item, position as u32)
                    )();
                    items_front.push(track_item);
                    let album_name = item.album().to_string();
                    if !seen_albums.contains(&album_name) {
                        seen_albums.insert(album_name.clone());
                        let album_item = glib::clone!(
                            #[strong] sender, #[strong] item,
                            move || crate::album_item::AlbumItem::new(sender, item, position as u32, album_name)
                        )();
                        album_items.push(album_item);
                    }
                }
                self.playlist.emit(PlaylistMsg::OpenFiles(items_front));
                self.albumlist.emit(AlbumListMsg::OpenAlbums(album_items));
                if self.playlist.model().tracks.is_empty() {
                    self.player_state.initial_track();
                }
            }
            AppMsg::Ignore => {}
        }
        self.update_view(widgets, sender);
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            CommandMsg::AddFiles(items_backend) => {
                for item in items_backend.iter() {
                    self.vec_for_restore.push(item.path().to_path_buf());
                }
                self.player_state.extend(items_backend.clone());
                sender.input(AppMsg::AddFilesToPlaylist(items_backend));
            }
            CommandMsg::RestoreFiles(items_backend) => {
                self.player_state.extend(items_backend.clone());
                sender.input(AppMsg::AddFilesToPlaylist(items_backend));
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if !self.vec_for_restore.is_empty() {
            write_json(&self.vec_for_restore, self.player_state.current_track(), self.position);
        }
    }
}
