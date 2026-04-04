use gtk::{ListScrollFlags, prelude::*};
use relm4::prelude::*;
use relm4::typed_view::grid::TypedGridView;
use relm4::{ComponentParts, ComponentSender, adw};

use crate::app::AppMsg;
use crate::album_item::AlbumItem;

#[derive(Debug)]
#[tracker::track]
pub struct AlbumListModel {
    #[tracker::no_eq]
    pub albums: TypedGridView<AlbumItem, gtk::NoSelection>,
    stack_visible_child: String,
}

#[derive(Debug)]
pub enum AlbumListMsg {
    OpenAlbums(Vec<AlbumItem>),
    ChangeStackChild(String),
    Clear,
    ScrollAlbumlist(String),
    SearchChanged(String),
    ToggleSearch,
}

#[relm4::component(pub)]
impl Component for AlbumListModel {
    type Init = ();
    type Input = AlbumListMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[name(main_list_stack)]
            gtk::Stack {
                add_child = &adw::Clamp {
                    set_maximum_size: 250,
                    set_vexpand: true,
                    set_valign: gtk::Align::Fill,
                    adw::StatusPage {
                        set_icon_name: Some("media-optical-cd-audio-symbolic"),
                        set_description: Some("Your albums will be listed here."),
                    }
                } -> {
                    set_name: "empty_albumlist",
                },
                add_child = &gtk::ScrolledWindow {
                    set_halign: gtk::Align::Fill,
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    set_propagate_natural_height: true,
                    #[local_ref]
                    albums_view -> gtk::GridView {
                        add_css_class: "album-grid",
                        set_single_click_activate: true,
                        set_max_columns: 10,
                        set_min_columns: 2,
                        set_enable_rubberband: false,
                    },
                } -> {
                    set_name: "full_albumlist",
                },
                #[watch]
                set_visible_child_name: &model.stack_visible_child
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let albums: TypedGridView<AlbumItem, gtk::NoSelection> = TypedGridView::new();
        
        let model = AlbumListModel {
            tracker: 0,
            albums,
            stack_visible_child: "empty_albumlist".to_string(),
        };
        let albums_view = &model.albums.view;
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
            AlbumListMsg::ChangeStackChild(child) => self.set_stack_visible_child(child),
            AlbumListMsg::Clear => {
                self.albums.clear();
            }
            AlbumListMsg::OpenAlbums(items) => {
                if !items.is_empty() {
                    self.albums.extend_from_iter(items);
                }
            }
            AlbumListMsg::ScrollAlbumlist(album_name) => {
                let albums_view = &widgets.albums_view;
                if let Some(idx) = self.albums.find(|c| c.info().album() == album_name) {
                    albums_view.scroll_to(idx, ListScrollFlags::NONE, None);
                }
            }
            AlbumListMsg::ToggleSearch => {}
            AlbumListMsg::SearchChanged(_) => {}
        }
        self.update_view(widgets, sender);
    }
}
