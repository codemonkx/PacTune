use gtk::{ListScrollFlags, prelude::*};
use relm4::prelude::*;
use relm4::typed_view::grid::TypedGridView;
use relm4::{ComponentParts, ComponentSender, adw};

use crate::app::AppMsg;
use crate::track_item::TrackItem;

#[derive(Debug)]
#[tracker::track]
pub struct PlaylistModel {
    #[tracker::no_eq]
    pub tracks: TypedGridView<TrackItem, gtk::NoSelection>,
    stack_visible_child: String,
    visible_search: bool,
}

#[derive(Debug)]
pub enum PlaylistMsg {
    OpenFiles(Vec<TrackItem>),
    ChangeStackChild(String),
    Clear,
    AccentTrack(u32),
    ScrollTracklist(u32),
    SearchChanged(String),
    ToggleSearch,
    FilterByAlbum(Option<String>),
}

#[relm4::component(pub)]
impl Component for PlaylistModel {
    type Init = ();
    type Input = PlaylistMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[name(search_bar)]
            gtk::SearchBar {
                #[watch]
                set_search_mode: model.visible_search,
                #[wrap(Some)]
                set_child = &gtk::SearchEntry {
                    set_hexpand: true,
                    set_placeholder_text: Some("Search tracks..."),
                    set_width_request: 300,

                    connect_search_changed[sender] => move |entry| {
                        let text = entry.text().to_string();
                        sender.input(PlaylistMsg::SearchChanged(text));
                    },

                    connect_stop_search[sender] => move |_| {
                        sender.input(PlaylistMsg::ToggleSearch);
                    },
                },
            },
            #[name(main_list_stack)]
            gtk::Stack {
                add_child = &adw::Clamp {
                    set_maximum_size: 250,
                    set_vexpand: true,
                    set_valign: gtk::Align::Fill,
                    adw::StatusPage {
                        set_icon_name: Some("audio-x-generic-symbolic"),
                        set_description: Some("Your tracks will be listed here."),
                    }
                } -> {
                    set_name: "empty_playlist",
                },
                add_child = &gtk::ScrolledWindow {
                    set_halign: gtk::Align::Fill,
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    set_propagate_natural_height: true,
                    #[local_ref]
                    tracks_view -> gtk::GridView {
                        add_css_class: "transparent-list",
                        set_single_click_activate: true,
                        set_max_columns: 1,
                        set_min_columns: 1,
                    },
                } -> {
                    set_name: "full_playlist",
                },
                #[watch]
                set_visible_child_name: &model.stack_visible_child
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let tracks: TypedGridView<TrackItem, gtk::NoSelection> = TypedGridView::new();

        let model = PlaylistModel {
            tracker: 0,
            tracks,
            stack_visible_child: "empty_playlist".to_string(),
            visible_search: false,
        };
        let tracks_view = &model.tracks.view;
        let widgets = view_output!();

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
            PlaylistMsg::ChangeStackChild(child) => self.set_stack_visible_child(child),
            PlaylistMsg::Clear => {
                self.tracks.clear();
            }
            PlaylistMsg::OpenFiles(items) => {
                if items.is_empty() {
                    sender.output(AppMsg::AdwToastBuild("There are no tracks available".into())).unwrap();
                } else {
                    self.tracks.extend_from_iter(items);
                }
            }
            PlaylistMsg::AccentTrack(position) => {
                if let Some(idx) = self.tracks.find(|c| c.position() == position) {
                    self.tracks.iter().for_each(|c| c.borrow().unaccent());
                    let cell = self.tracks.get(idx).unwrap();
                    cell.borrow().accent();
                }
            }
            PlaylistMsg::ScrollTracklist(position) => {
                let tracks_view = &widgets.tracks_view;
                if let Some(idx) = self.tracks.find(|c| c.position() == position) {
                    tracks_view.scroll_to(idx, ListScrollFlags::NONE, None);
                }
            }
            PlaylistMsg::SearchChanged(query) => {
                self.tracks.clear_filters();
                self.tracks.add_filter(move |item| {
                    item.info()
                        .title()
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                        || item
                            .info()
                            .artist()
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                        || item
                            .info()
                            .album()
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                });
            }
            PlaylistMsg::ToggleSearch => {
                self.set_visible_search(!self.visible_search);
            }
            PlaylistMsg::FilterByAlbum(album) => {
                self.tracks.clear_filters();
                if let Some(album_name) = album {
                    self.tracks.add_filter(move |item| item.info().album() == album_name.as_str());
                }
                widgets.tracks_view.scroll_to(0, ListScrollFlags::NONE, None);
            }
        }
        self.update_view(widgets, sender);
    }
}
