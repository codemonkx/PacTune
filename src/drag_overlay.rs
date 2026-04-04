use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentParts, gtk::gdk};
use std::path::PathBuf;

use crate::app::AppMsg;

#[derive(Debug)]
pub struct DragOverlayModel;

#[relm4::component(pub)]
impl SimpleComponent for DragOverlayModel {
    type Init = ();
    type Input = ();
    type Output = AppMsg;

    view! {
        gtk::DropTarget {
            set_actions: gdk::DragAction::COPY,
            set_types: &[gdk::FileList::static_type()],
            connect_drop[sender] => move |_target, value, _x, _y| {
                if let Ok(file_list) = value.get::<gdk::FileList>() {
                    let files: Vec<PathBuf> = file_list
                    .files()
                    .into_iter()
                    .filter_map(|f| f.path())
                    .collect();
                    sender.output(AppMsg::OpenResponse(files, true)).unwrap();
                    return true;
                }
                false
            },
            connect_enter[sender] => move |_target, _x, _y| {
                sender.output(AppMsg::DragEnter).unwrap();
                gdk::DragAction::COPY
            },
            connect_leave[sender] => move |_target| {
                sender.output(AppMsg::DragLeave).unwrap();
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
