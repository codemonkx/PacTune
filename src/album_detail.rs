use crate::app::{AppModel, AppMsg};
use crate::backend::cover_cache::CoverCache;
use crate::backend::track_info::TrackInfo;

use gtk::glib::SignalHandlerId;
use gtk::prelude::*;
use relm4::typed_view::grid::TypedGridView;
use relm4::{ComponentParts, ComponentSender, adw, gtk::glib, prelude::*, typed_view::grid::RelmGridItem};
use std::sync::Arc;

// ── AlbumDetailTrackItem ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct AlbumDetailTrackItem {
    pub position: u32,
    pub disc_number: Option<u32>,
    pub track_number: Option<u32>,
    pub info: Arc<TrackInfo>,
    sender: ComponentSender<AppModel>,
    signal_handler: Option<SignalHandlerId>,
    button_widget: Option<gtk::Button>,
}

impl AlbumDetailTrackItem {
    pub fn new(
        sender: ComponentSender<AppModel>,
        info: Arc<TrackInfo>,
        position: u32,
    ) -> Self {
        AlbumDetailTrackItem {
            position,
            disc_number: info.disc_number(),
            track_number: info.track_number(),
            info,
            sender,
            signal_handler: None,
            button_widget: None,
        }
    }

    fn sort_key(&self) -> (u32, u32, u32) {
        (
            self.disc_number.unwrap_or(u32::MAX),
            self.track_number.unwrap_or(u32::MAX),
            self.position,
        )
    }
}

impl PartialEq for AlbumDetailTrackItem {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}
impl Eq for AlbumDetailTrackItem {}

impl PartialOrd for AlbumDetailTrackItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for AlbumDetailTrackItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

#[derive(Debug)]
pub struct AlbumDetailTrackWidgets {
    button: gtk::Button,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    image: gtk::Image,
}

impl RelmGridItem for AlbumDetailTrackItem {
    type Root = gtk::Box;
    type Widgets = AlbumDetailTrackWidgets;

    fn setup(item: &gtk::ListItem) -> (gtk::Box, AlbumDetailTrackWidgets) {
        relm4::view! {
            item.set_activatable(false),
            item.set_focusable(false),
            root = gtk::Box {
                #[name(button)]
                gtk::Button {
                    add_css_class: "flat",
                    set_margin_all: 6,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 6,
                        #[name(image)]
                        append = &gtk::Image {
                            add_css_class: "song-cover",
                            set_overflow: gtk::Overflow::Hidden,
                            set_icon_name: Some("PacTune"),
                            set_pixel_size: 120,
                            set_halign: gtk::Align::Center,
                        },
                        #[name(title_label)]
                        gtk::Label {
                            add_css_class: "large-title",
                            set_halign: gtk::Align::Center,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_width_chars: 12,
                            set_max_width_chars: 15,
                        },
                        #[name(artist_label)]
                        gtk::Label {
                            add_css_class: "small-title",
                            set_halign: gtk::Align::Center,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_width_chars: 12,
                            set_max_width_chars: 15,
                        },
                    },
                },
            }
        }

        let widgets = AlbumDetailTrackWidgets {
            button,
            title_label,
            artist_label,
            image,
        };
        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut AlbumDetailTrackWidgets, _root: &mut gtk::Box) {
        widgets.title_label.set_label(self.info.title());
        widgets.artist_label.set_label(self.info.artist());

        let index = self.position as usize;
        let handler_id = widgets.button.connect_clicked(glib::clone!(
            #[strong(rename_to = sender)]
            self.sender,
            move |_| sender.input(AppMsg::TrackSelected(index))
        ));
        self.signal_handler = Some(handler_id);

        if let Some(uuid) = self.info.cover_uuid() {
            let mut cache = CoverCache::global().lock().unwrap();
            if let Some(texture) = cache.get_texture(uuid) {
                widgets.image.set_paintable(Some(&texture));
                widgets.image.set_pixel_size(120);
            } else {
                widgets.image.set_icon_name(Some("PacTune"));
                widgets.image.set_pixel_size(120);
            }
        }

        glib::clone!(
            #[weak(rename_to = button)]
            widgets.button,
            move || { self.button_widget = Some(button); }
        )();
    }

    fn unbind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        if let Some(id) = self.signal_handler.take() {
            widgets.image.set_paintable(gtk::gdk::Paintable::NONE);
            widgets.button.disconnect(id);
        }
    }
}

// ── AlbumDetailModel ──────────────────────────────────────────────────────────

#[derive(Debug)]
#[tracker::track]
pub struct AlbumDetailModel {
    #[tracker::no_eq]
    pub tracks: TypedGridView<AlbumDetailTrackItem, gtk::NoSelection>,
    stack_visible_child: String,
    album_name: String,
    artist_name: String,
    cover_uuid: Option<String>,
}

#[derive(Debug)]
pub enum AlbumDetailMsg {
    Load {
        items: Vec<AlbumDetailTrackItem>,
        album_name: String,
        artist_name: String,
        cover_uuid: Option<String>,
    },
    Clear,
}

#[relm4::component(pub)]
impl Component for AlbumDetailModel {
    type Init = ();
    type Input = AlbumDetailMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_vexpand: true,
            // Back button row
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_top: 6,
                set_margin_bottom: 6,
                set_margin_start: 6,
                gtk::Button {
                    set_icon_name: "go-previous-symbolic",
                    add_css_class: "flat",
                    connect_clicked[sender] => move |_| {
                        sender.output(AppMsg::CloseAlbumDetail).unwrap();
                    },
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_margin_start: 8,
                    gtk::Label {
                        add_css_class: "large-title",
                        #[watch]
                        set_label: &model.album_name,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_halign: gtk::Align::Start,
                    },
                    gtk::Label {
                        add_css_class: "small-title",
                        #[watch]
                        set_label: &model.artist_name,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_halign: gtk::Align::Start,
                    },
                },
            },
            gtk::Stack {
                set_vexpand: true,
                set_transition_type: gtk::StackTransitionType::Crossfade,
                add_child = &adw::Clamp {
                    set_maximum_size: 250,
                    set_vexpand: true,
                    set_valign: gtk::Align::Fill,
                    adw::StatusPage {
                        set_icon_name: Some("audio-x-generic-symbolic"),
                        set_description: Some("No tracks found for this album."),
                    }
                } -> {
                    set_name: "empty",
                },
                add_child = &gtk::ScrolledWindow {
                    set_halign: gtk::Align::Fill,
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    #[local_ref]
                    tracks_view -> gtk::GridView {
                        add_css_class: "transparent-list",
                        set_single_click_activate: true,
                        set_max_columns: 10,
                        set_min_columns: 2,
                        set_enable_rubberband: false,
                    },
                } -> {
                    set_name: "tracks",
                },
                #[watch]
                set_visible_child_name: &model.stack_visible_child,
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let tracks: TypedGridView<AlbumDetailTrackItem, gtk::NoSelection> = TypedGridView::new();

        let model = AlbumDetailModel {
            tracker: 0,
            tracks,
            stack_visible_child: "empty".to_string(),
            album_name: String::new(),
            artist_name: String::new(),
            cover_uuid: None,
        };

        let tracks_view = &model.tracks.view;
        let widgets = view_output!();

        // Track accumulated horizontal and vertical scroll delta.
        // Require a strong rightward swipe that is clearly more horizontal than vertical.
        let accum = std::rc::Rc::new(std::cell::Cell::new((0.0f64, 0.0f64)));

        let scroll_ctrl = gtk::EventControllerScroll::new(
            gtk::EventControllerScrollFlags::BOTH_AXES
                | gtk::EventControllerScrollFlags::KINETIC,
        );
        scroll_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);

        let accum_begin = accum.clone();
        scroll_ctrl.connect_scroll_begin(move |_| {
            accum_begin.set((0.0, 0.0));
        });

        let accum_scroll = accum.clone();
        scroll_ctrl.connect_scroll(move |_, dx, dy| {
            let (ax, ay) = accum_scroll.get();
            accum_scroll.set((ax + dx, ay + dy.abs()));
            gtk::glib::Propagation::Proceed
        });

        let accum_end = accum.clone();
        let sender_scroll = sender.clone();
        scroll_ctrl.connect_scroll_end(move |_| {
            let (ax, ay) = accum_end.get();
            // Must swipe right (ax < -80), and horizontal must be 3x the vertical
            if ax < -80.0 && ax.abs() > ay * 3.0 {
                let _ = sender_scroll.output(AppMsg::CloseAlbumDetail);
            }
            accum_end.set((0.0, 0.0));
        });

        root.add_controller(scroll_ctrl);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();
        match message {
            AlbumDetailMsg::Load { mut items, album_name, artist_name, cover_uuid } => {
                self.tracks.clear();
                items.sort();
                let is_empty = items.is_empty();
                self.tracks.extend_from_iter(items);
                self.set_album_name(album_name);
                self.set_artist_name(artist_name);
                self.set_cover_uuid(cover_uuid);
                self.set_stack_visible_child(if is_empty {
                    "empty".to_string()
                } else {
                    "tracks".to_string()
                });
            }
            AlbumDetailMsg::Clear => {
                self.tracks.clear();
                self.set_stack_visible_child("empty".to_string());
                self.set_album_name(String::new());
                self.set_artist_name(String::new());
                self.set_cover_uuid(None);
            }
        }
        self.update_view(widgets, sender);
    }
}
