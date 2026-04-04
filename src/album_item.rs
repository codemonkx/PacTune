use crate::app::AppMsg;
use crate::backend::track_info::TrackInfo;
use crate::{AppModel, backend::cover_cache::CoverCache};

use gtk::glib::SignalHandlerId;
use gtk::prelude::*;
use relm4::{gtk::glib, prelude::*, typed_view::grid::RelmGridItem};
use std::sync::Arc;

#[derive(Debug)]
pub struct AlbumItem {
    first_track_position: u32,
    album_name: String,
    info: Arc<TrackInfo>,
    sender: ComponentSender<AppModel>,
    signal_handler: Option<SignalHandlerId>,
    button_widget: Option<gtk::Button>,
}

impl PartialEq for AlbumItem {
    fn eq(&self, other: &Self) -> bool {
        self.album_name == other.album_name
    }
}

impl Eq for AlbumItem {}

impl PartialOrd for AlbumItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AlbumItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.album_name.cmp(&other.album_name)
    }
}

impl AlbumItem {
    pub fn new(sender: ComponentSender<AppModel>, info: Arc<TrackInfo>, first_track_position: u32, album_name: String) -> Self {
        AlbumItem {
            first_track_position,
            album_name,
            info,
            sender,
            signal_handler: Default::default(),
            button_widget: None,
        }
    }

    pub fn first_track_position(&self) -> u32 {
        self.first_track_position
    }

    pub fn info(&self) -> &Arc<TrackInfo> {
        &self.info
    }
}

pub struct AlbumWidgets {
    button: gtk::Button,
    title_label: gtk::Label,
    image: gtk::Picture,
}

impl RelmGridItem for AlbumItem {
    type Root = gtk::Box;
    type Widgets = AlbumWidgets;

    fn setup(item: &gtk::ListItem) -> (gtk::Box, AlbumWidgets) {
        relm4::view! {
            item.set_activatable(false),
            item.set_focusable(false),
            root = gtk::Box {
                #[name(button)]
                gtk::Button {
                    add_css_class: "album-card-btn",
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,
                                gtk::Box {
                                    add_css_class: "album-cover-container",
                                    #[name(image)]
                                    gtk::Picture {
                                        add_css_class: "song-cover",
                                        set_content_fit: gtk::ContentFit::Cover,
                                        set_overflow: gtk::Overflow::Hidden,
                                        set_halign: gtk::Align::Fill,
                                        set_valign: gtk::Align::Fill,
                                        set_hexpand: true,
                                        set_vexpand: true,
                                    },
                                },
                        #[name(title_label)]
                        gtk::Label {
                            add_css_class: "large-title",
                            set_halign: gtk::Align::Center,
                            set_justify: gtk::Justification::Center,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_lines: 1,
                        },
                    },
                },
            }
        }

        let widgets = AlbumWidgets {
            button,
            title_label,
            image,
        };

        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut AlbumWidgets, _root: &mut gtk::Box) {
        widgets.title_label.set_label(&self.album_name);

        let album = self.album_name.clone();
        let handler_id = widgets.button.connect_clicked(glib::clone!(
            #[strong(rename_to = sender)]
            self.sender,
            move |_btn| {
                sender.input(AppMsg::OpenAlbumDetail(album.clone()))
            }
        ));
        self.signal_handler = Some(handler_id);

        if let Some(uuid) = self.info.cover_uuid() {
            let mut cache = CoverCache::global().lock().unwrap();
            if let Some(texture) = cache.get_texture(uuid) {
                widgets.image.set_paintable(Some(&texture));
            } else {
                widgets.image.set_paintable(gtk::gdk::Paintable::NONE);
            }
        } else {
            widgets.image.set_paintable(gtk::gdk::Paintable::NONE);
        }

        glib::clone!(
            #[weak(rename_to=button)]
            widgets.button,
            move || {
                self.button_widget = Some(button);
            }
        )();
    }

    fn unbind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        if let Some(id) = self.signal_handler.take() {
            widgets.image.set_paintable(gtk::gdk::Paintable::NONE);
            widgets.button.disconnect(id)
        }
    }
}
