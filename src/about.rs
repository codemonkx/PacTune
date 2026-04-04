use relm4::gtk::License;
use relm4::{ComponentParts, SimpleComponent, adw};

pub struct About;

#[relm4::component(pub)]
impl SimpleComponent for About {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[name(dialog)]
        adw::AboutDialog {
            set_application_name: "PacTune",
            set_application_icon: "PacTune",
            set_developer_name: "Mikhail Kostin",
            set_version: env!("CARGO_PKG_VERSION"),
            set_website: "https://codeberg.org/M23Snezhok/Vinyl",
            set_issue_url: "https://codeberg.org/M23Snezhok/Vinyl/issues",

            set_developers: &["Mikhail Kostin <m23snezhok@protonmail.com>"],

            set_copyright: "@ 2026 Mikhail Kostin",
            set_license_type: License::Gpl30,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
