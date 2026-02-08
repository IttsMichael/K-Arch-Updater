use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use crate::update_manager::UpdateManager;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct UpdateRow {
        pub package: RefCell<String>,
        pub version: RefCell<String>,
        pub on_refresh: RefCell<Option<std::sync::mpsc::Sender<()>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UpdateRow {
        const NAME: &'static str = "UpdateRow";
        type Type = super::UpdateRow;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for UpdateRow {}
    impl WidgetImpl for UpdateRow {}
    impl BoxImpl for UpdateRow {}
}

glib::wrapper! {
    pub struct UpdateRow(ObjectSubclass<imp::UpdateRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Orientable;
}

impl UpdateRow {
    pub fn new(package: &str, version: &str, on_refresh: std::sync::mpsc::Sender<()>) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.set_data(package, version, on_refresh);
        obj.setup_ui();
        obj
    }

    fn set_data(&self, package: &str, version: &str, on_refresh: std::sync::mpsc::Sender<()>) {
        let imp = self.imp();
        imp.package.replace(package.to_string());
        imp.version.replace(version.to_string());
        imp.on_refresh.replace(Some(on_refresh));
    }

    fn setup_ui(&self) {
        let imp = self.imp();
        let package = imp.package.borrow().clone();
        let version = imp.version.borrow().clone();

        self.set_orientation(gtk::Orientation::Horizontal);
        self.set_spacing(12);
        self.set_margin_start(12);
        self.set_margin_end(12);
        self.set_margin_top(6);
        self.set_margin_bottom(6);

        let pkg_label = gtk::Label::builder()
            .label(&format!("{} - {}", package, version))
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();

        let install_button = gtk::Button::builder()
            .label("Update")
            .valign(gtk::Align::Center)
            .build();

        if let Some(sender) = imp.on_refresh.borrow().as_ref() {
            let sender = sender.clone();
            install_button.connect_clicked(glib::clone!(@weak install_button => move |_| {
                install_button.set_label("Updating...");
                UpdateManager::install_package(package.clone(), sender.clone());
            }));
        }

        self.append(&pkg_label);
        self.append(&install_button);

    
    }
}
