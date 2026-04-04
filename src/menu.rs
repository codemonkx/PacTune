use gtk::{glib, prelude::*};
use relm4::actions::*;
use relm4::prelude::*;

use crate::app::AppMsg;

#[derive(Debug)]
pub struct MenuModel {
    action_clear_playlist: RelmAction<ActionClearPlaylist>,
    action_reload_library: RelmAction<ActionReloadLibrary>,
    action_refresh_playlist: RelmAction<ActionRefreshPlaylist>,
}

#[derive(Debug)]
pub enum MenuMsg {
    SetActionsEnabled(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for MenuModel {
    type Init = adw::ApplicationWindow;
    type Input = MenuMsg;
    type Output = AppMsg;

    view! {
        gtk::MenuButton {
            set_icon_name: "open-menu-symbolic",
            set_halign: gtk::Align::Start,
            #[wrap(Some)]
            set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)),
        }
    }

    menu! {
        main_menu: {
            custom: "main_menu",
            "Add Folder" => ActionOpenRequest,
            "Add Song" => ActionOpenSong,
            "Clear" => ActionClearPlaylist,
            "Refresh Folder" => ActionRefreshPlaylist,
            "Reload library" => ActionReloadLibrary,
            section! {
                "About" => ActionAbout
            },
            section! {
                "Quit" => ActionQuit
            }
        }
    }

    fn init(
        window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let app = relm4::main_application();
        app.set_accelerators_for_action::<ActionOpenRequest>(&["<primary>F"]);
        app.set_accelerators_for_action::<ActionOpenSong>(&["<primary>S"]);
        app.set_accelerators_for_action::<ActionClearPlaylist>(&["<primary>L"]);
        app.set_accelerators_for_action::<ActionQuit>(&["<primary>Q"]);
        app.set_accelerators_for_action::<ActionReloadLibrary>(&["<primary>R"]);

        let action_reload_library: RelmAction<ActionReloadLibrary> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::Clear).unwrap();
                    sender.output(AppMsg::RestorePlaylist).unwrap();
                }
            ))
        };

        let action_open_request: RelmAction<ActionOpenRequest> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::OpenRequest).unwrap();
                }
            ))
        };

        let action_open_song: RelmAction<ActionOpenSong> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::OpenSong).unwrap();
                }
            ))
        };

        let action_clear_playlist: RelmAction<ActionClearPlaylist> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::ClearPlaylist).unwrap();
                }
            ))
        };

        let action_quit: RelmAction<ActionQuit> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::Quit).unwrap();
                }
            ))
        };

        let about_action: RelmAction<ActionAbout> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::ShowAboutDialog).unwrap();
                }
            ))
        };

        let action_refresh_playlist: RelmAction<ActionRefreshPlaylist> = {
            RelmAction::new_stateless(glib::clone!(
                #[strong]
                sender,
                move |_| {
                    sender.output(AppMsg::RefreshPlaylist).unwrap();
                }
            ))
        };

        let mut actions_group = RelmActionGroup::<MainMenuActionGroup>::new();
        actions_group.add_action(action_open_request);
        actions_group.add_action(action_open_song);
        actions_group.add_action((glib::clone!(
            #[strong]
            action_clear_playlist,
            move || action_clear_playlist
        ))());
        actions_group.add_action(action_quit);
        actions_group.add_action(about_action);
        actions_group.add_action((glib::clone!(
            #[strong]
            action_reload_library,
            move || action_reload_library
        ))());
        actions_group.add_action((glib::clone!(
            #[strong]
            action_refresh_playlist,
            move || action_refresh_playlist
        ))());
        actions_group.register_for_widget(&window);

        let model = MenuModel {
            action_clear_playlist,
            action_reload_library,
            action_refresh_playlist,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            MenuMsg::SetActionsEnabled(enabled) => {
                self.action_clear_playlist.set_enabled(enabled);
                self.action_reload_library.set_enabled(enabled);
                self.action_refresh_playlist.set_enabled(enabled);
            }
        }
    }
}

relm4::new_action_group!(MainMenuActionGroup, "main_menu_action_group");

relm4::new_stateless_action!(ActionOpenRequest, MainMenuActionGroup, "open_request");
relm4::new_stateless_action!(ActionOpenSong, MainMenuActionGroup, "open_song");
relm4::new_stateless_action!(ActionClearPlaylist, MainMenuActionGroup, "clear_playlist");
relm4::new_stateless_action!(ActionQuit, MainMenuActionGroup, "quit");
relm4::new_stateless_action!(ActionAbout, MainMenuActionGroup, "about");
relm4::new_stateless_action!(ActionReloadLibrary, MainMenuActionGroup, "reload_library");
relm4::new_stateless_action!(ActionRefreshPlaylist, MainMenuActionGroup, "refresh_playlist");
