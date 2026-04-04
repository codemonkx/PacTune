use crate::AppModel;
use crate::app::AppMsg;

use gtk::glib;
use gtk::glib::SignalHandlerId;
use gtk::prelude::*;
use relm4::{prelude::*, typed_view::list::RelmListItem};

#[derive(Debug)]
pub struct LyricsItem {
    position: Option<i64>,
    line_lyric: String,
    sender: ComponentSender<AppModel>,
    signal_handler: Option<SignalHandlerId>,
    line_widget: Option<gtk::Label>,
}

impl PartialEq for LyricsItem {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for LyricsItem {}

impl PartialOrd for LyricsItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LyricsItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.into_iter().cmp(other.position)
    }
}

impl LyricsItem {
    pub fn new(
        sender: ComponentSender<AppModel>,
        line_lyric: String,
        position: Option<i64>,
    ) -> Self {
        LyricsItem {
            line_lyric,
            sender,
            position,
            signal_handler: Default::default(),
            line_widget: None,
        }
    }

    pub fn line_lyric(&self) -> &str {
        &self.line_lyric
    }

    pub fn position(&self) -> Option<i64> {
        self.position
    }

    pub fn accent(&self) {
        if let Some(line) = &self.line_widget {
            line.add_css_class("accent");
        }
    }

    pub fn unaccent(&self) {
        if let Some(line) = &self.line_widget {
            line.remove_css_class("accent");
        }
    }
}

#[derive(Debug)]
pub struct LyricsWidgets {
    line_label: gtk::Label,
    gesture: gtk::GestureClick,
}

impl RelmListItem for LyricsItem {
    type Root = gtk::Box;
    type Widgets = LyricsWidgets;

    fn setup(item: &gtk::ListItem) -> (gtk::Box, LyricsWidgets) {
        let gesture = gtk::GestureClick::new();
        relm4::view! {
            item.set_activatable(false),
            item.set_focusable(false),
            root = gtk::Box {
                    set_size_request: (325, 60),
                    set_margin_start: 10,
                    add_controller = gesture.clone(),
                    #[name(line_label)]
                    gtk::Label {
                        set_wrap: true,
                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                        set_align: gtk::Align::Center,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 60,
                        set_hexpand: true,
                    }
                },
        }

        let widgets = LyricsWidgets { line_label, gesture };

        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut LyricsWidgets, _root: &mut gtk::Box) {
        widgets.line_label.set_label(self.line_lyric());
        let handler_id = widgets.gesture.connect_pressed(glib::clone!(
            #[strong(rename_to=sender)]
            self.sender,
            #[strong(rename_to=position)]
            self.position,
            move |_gesture, _n_press, _x, _y| {
                if let Some(position) = position {
                    sender.input(AppMsg::SeekPositonLyrics(position))
                } else {
                    sender.input(AppMsg::AdwToastBuild("Positon not found".into()))
                }
            }
        ));
        self.signal_handler = Some(handler_id);

        glib::clone!(
            #[weak(rename_to=line)]
            widgets.line_label,
            move || {
                self.line_widget = Some(line);
            }
        )();
    }

    fn unbind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        if let Some(id) = self.signal_handler.take() {
            widgets.gesture.disconnect(id)
        }
    }
}
