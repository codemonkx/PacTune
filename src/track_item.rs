use crate::app::AppMsg;
use crate::backend::track_info::TrackInfo;
use crate::{AppModel, backend::cover_cache::CoverCache};

use gtk::glib::SignalHandlerId;
use gtk::prelude::*;
use relm4::{gtk::glib, prelude::*, typed_view::grid::RelmGridItem};
use std::sync::Arc;

#[derive(Debug)]
pub struct TrackItem {
    position: u32,
    info: Arc<TrackInfo>,
    sender: ComponentSender<AppModel>,
    signal_handler: Option<SignalHandlerId>,
    button_widget: Option<gtk::Button>,
}

impl PartialEq for TrackItem {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for TrackItem {}

impl PartialOrd for TrackItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TrackItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.cmp(&other.position)
    }
}

impl TrackItem {
    pub fn new(sender: ComponentSender<AppModel>, info: Arc<TrackInfo>, position: u32) -> Self {
        TrackItem {
            position,
            info,
            sender,
            signal_handler: Default::default(),
            button_widget: None,
        }
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn accent(&self) {
        if let Some(button) = &self.button_widget {
            button.add_css_class("track-background");
        }
    }

    pub fn unaccent(&self) {
        if let Some(button) = &self.button_widget {
            button.remove_css_class("track-background");
        }
    }

    pub fn info(&self) -> &Arc<TrackInfo> {
        &self.info
    }
}

#[derive(Debug)]
pub struct TrackWidgets {
    button: gtk::Button,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    duration_label: gtk::Label,
    image: gtk::Image,
}

impl RelmGridItem for TrackItem {
    type Root = gtk::Box;
    type Widgets = TrackWidgets;

    fn setup(item: &gtk::ListItem) -> (gtk::Box, TrackWidgets) {
        relm4::view! {
            item.set_activatable(false),
            item.set_focusable(false),
            root = gtk::Box {
                #[name(button)]
                gtk::Button {
                    add_css_class: "flat",
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        #[name(image)]
                        append = &gtk::Image {
                            add_css_class: "rounded-image",
                            set_overflow: gtk::Overflow::Hidden,
                            set_icon_name: Some("Vinyl"),
                            set_pixel_size: 32,
                            set_margin_end: 8,
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,
                            set_width_request: 75,
                            set_hexpand: true,

                            #[name(title_label)]
                            gtk::Label {
                                add_css_class: "large-title",
                                set_halign: gtk::Align::Start,
                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                                set_lines: 1,
                                set_max_width_chars: 10,
                            },

                            #[name(artist_label)]
                            gtk::Label {
                                add_css_class: "small-title",
                                set_halign: gtk::Align::Start,
                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                                set_lines: 1,
                                set_max_width_chars: 10,
                            },
                        },

                        #[name(duration_label)]
                        gtk::Label {
                            add_css_class: "numeric",
                            set_halign: gtk::Align::End,
                            set_margin_end: 7,
                        }
                    },
                },
            }
        }

        let widgets = TrackWidgets {
            button,
            title_label,
            artist_label,
            duration_label,
            image,
        };

        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut TrackWidgets, _root: &mut gtk::Box) {
        widgets.title_label.set_label(self.info.title());
        widgets.artist_label.set_label(self.info.artist());
        let duration = format!(
            "{:02}:{:02}",
            &self.info.duration() / 60,
            &self.info.duration() % 60
        );
        widgets.duration_label.set_label(&duration);

        let index = self.position as usize;
        let handler_id = widgets.button.connect_clicked(glib::clone!(
            #[strong(rename_to = sender)]
            self.sender,
            move |_btn| {
                // Use the cloned sender to send a message
                sender.input(AppMsg::TrackSelected(index))
            }
        ));
        self.signal_handler = Some(handler_id);

        if let Some(uuid) = self.info.cover_uuid() {
            let mut cache = CoverCache::global().lock().unwrap();
            if let Some(texture) = cache.get_texture(uuid) {
                widgets.image.set_paintable(Some(&texture));
                widgets.image.set_pixel_size(32);
            } else {
                widgets.image.set_icon_name(Some("Vinyl"));
                widgets.image.set_pixel_size(32);
            }
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
