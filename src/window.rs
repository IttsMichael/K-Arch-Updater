use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use crate::update_manager::UpdateManager;
use crate::update_row::UpdateRow;
use std::process::Command;
use std::thread;

mod imp {
    use super::*;
    use std::cell::Cell; // Moved here from top-level to keep Cell in scope

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/Example/window.ui")]
    pub struct UpdaterWindow {
        // Template widgets
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub update_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub updateall_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub clear_button: TemplateChild<gtk::Button>,
        pub number: Cell<i32>,
        pub refresh_sender: std::cell::OnceCell<std::sync::mpsc::Sender<()>>,
    }

    impl Default for UpdaterWindow {
        fn default() -> Self {
            Self {
                label: TemplateChild::default(),
                updateall_button: TemplateChild::default(),
                refresh_button: TemplateChild::default(),
                clear_button: TemplateChild::default(),
                update_list: TemplateChild::default(),
                number: Cell::new(0),
                refresh_sender: std::cell::OnceCell::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UpdaterWindow {
        const NAME: &'static str = "UpdaterWindow";
        type Type = super::UpdaterWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    impl ObjectImpl for UpdaterWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let (sender, receiver) = std::sync::mpsc::channel::<()>();
            self.refresh_sender.set(sender).expect("Sender already set");

            glib::timeout_add_local(std::time::Duration::from_millis(500), glib::clone!(@weak obj => @default-return glib::ControlFlow::Break, move || {
                if let Ok(_) = receiver.try_recv() {
                    obj.check_for_updates();
                }
                glib::ControlFlow::Continue
            }));

            // obj.check_sudo();

            obj.setup_css();
            obj.setup_callbacks();

            obj.check_for_updates();
        }
    }


    impl WidgetImpl for UpdaterWindow {}
    impl WindowImpl for UpdaterWindow {}
    impl ApplicationWindowImpl for UpdaterWindow {}
}

glib::wrapper! {
    pub struct UpdaterWindow(ObjectSubclass<imp::UpdaterWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl UpdaterWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(
            ".non-selectable-item {
                color: #e8e8e8;
                background-color: #1a1a1a;
                border-radius: 12px;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
                margin: 4px 12px;
                padding: 8px 10px;
            }"
        );
        
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn setup_callbacks(&self) {
        self.imp().clear_button.connect_clicked(glib::clone!(@weak self as obj => move |_| {
            obj.clear_list();
        }));

        self.imp().refresh_button.connect_clicked(glib::clone!(@weak self as obj => move |_| {
            obj.check_for_updates();
        }));

        self.imp().updateall_button.connect_clicked(glib::clone!(@weak self as obj => move |_| {
            obj.update_all();
        }));
    }

    fn clear_list(&self) {
        let imp = self.imp();
        while let Some(child) = imp.update_list.first_child() {
            imp.update_list.remove(&child);
        }
    }


    fn update_all(&self) {
        let imp = self.imp();
        imp.label.set_text("Updating All...");
        self.disable_all_row_buttons();
        imp.updateall_button.set_sensitive(false);
        println!("thread started for updating all");
        
        let (sender, receiver) = std::sync::mpsc::channel();
        
        thread::spawn(move || {
            match Command::new("pkexec")
                .args(["pacman", "-Syu", "--noconfirm"])
                .output()
            {
                Ok(output) => {
                    println!("all packages updating");
                    println!("Status: {}", output.status);
                    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                    sender.send(true).unwrap();
                }
                Err(e) => {
                    eprintln!("Command failed to execute: {}", e);
                    sender.send(false).unwrap();
                }
            }
        });
        
        glib::timeout_add_local(std::time::Duration::from_millis(100), 
            glib::clone!(@weak self as obj => @default-return glib::ControlFlow::Break, move || {
                if let Ok(success) = receiver.try_recv() {
                    if success {
                        obj.success_update();
                    } else {
                        obj.failed_update();
                    }
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            })
        );
    }

    fn success_update(&self) {
        let imp = self.imp();
        imp.label.set_text("Update Successful");
        imp.updateall_button.set_sensitive(true);
        self.clear_list();
    }

    fn failed_update(&self) {
        let imp = self.imp();
        imp.label.set_text("Update Failed");
        imp.updateall_button.set_sensitive(true);
        self.clear_list();
    }

    fn disable_all_row_buttons(&self) {
        let imp = self.imp();
        let mut child = imp.update_list.first_child();
        
        while let Some(row) = child {
            if let Some(box_widget) = row.first_child() {
                let mut button_child = box_widget.first_child();
                while let Some(widget) = button_child {
                    if let Ok(button) = widget.clone().downcast::<gtk::Button>() {
                        button.set_sensitive(false);
                    }
                    button_child = widget.next_sibling();
                }
            }
            child = row.next_sibling();
        }
    }

    fn check_sudo(&self) {
        match Command::new("sudo")
        .args(["echo", "hi"])
        .output()
        {
            Ok(_) => {
                println!("hihihih");
            }

            Err(e) => {
                self.close()
            }

        } 
    }

    fn check_for_updates(&self) {
        let imp = self.imp();
        imp.label.set_text("Checking...");
        
        // Clear the existing list before starting a new check
        self.clear_list();
        imp.label.set_text("Checking...");

        let (sender, receiver) = std::sync::mpsc::channel::<String>();
        UpdateManager::check_updates(sender);
        
        glib::timeout_add_local(std::time::Duration::from_millis(100),
            glib::clone!(@weak self as obj => @default-return glib::ControlFlow::Break, move || {
                if let Ok(result_string) = receiver.try_recv() {
                    obj.handle_update_result(result_string);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            })
        );
    }

    fn handle_update_result(&self, result_string: String) {
        let imp = self.imp();
        
        if result_string.trim().is_empty() {
            imp.label.set_text("System up to date");
        } else {
            imp.label.set_text("Updates found!");
            
            for line in result_string.lines() {
                if !line.trim().is_empty() {
                    let mut parts = line.split(' ');
                    let package = parts.next().unwrap_or("");
                    let version = parts.next_back().unwrap_or("");

                    if !package.is_empty() {
                        let sender = imp.refresh_sender.get().unwrap().clone();
                        let row = UpdateRow::new(package, version, sender);
                        imp.update_list.append(&row);

                        if let Some(row) = imp.update_list.last_child().and_downcast::<gtk::ListBoxRow>() {
                            row.set_activatable(false);
                            row.set_selectable(false);
                            row.add_css_class("non-selectable-item");
                        }
                    }
                }
            }
        }
    }
}
