use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/Example/window.ui")]
    pub struct UpdaterWindow {
        // Template widgets
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub update_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub update_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub clear_button: TemplateChild<gtk::Button>,
        pub number: Cell<i32>,
    }

    impl Default for UpdaterWindow {
        fn default() -> Self {
            Self {
                label: TemplateChild::default(),
                update_button: TemplateChild::default(),
                refresh_button: TemplateChild::default(),
                clear_button: TemplateChild::default(),
                update_list: TemplateChild::default(),
                number: Cell::new(0),
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
            
            self.clear_button.connect_clicked(glib::clone!(@weak obj => move |_| {
                let imp = obj.imp();
                imp.label.set_text(" ");
            }));

            self.refresh_button.connect_clicked(glib::clone!(@weak obj => move |_| {
                let imp = obj.imp();
                imp.label.set_text("Checking...");
                
                // Clear the existing list before starting a new check
                while let Some(child) = imp.update_list.first_child() {
                    imp.update_list.remove(&child);
                }
                
                let (sender, receiver) = std::sync::mpsc::channel::<String>();
                
                std::thread::spawn(move || {
                    let output = std::process::Command::new("checkupdates").output();
                    let result = match output {
                        Ok(res) => String::from_utf8_lossy(&res.stdout).to_string(),
                        Err(_) => "Error".to_string(),
                    };
                    let _ = sender.send(result);
                });
                
                glib::timeout_add_local(std::time::Duration::from_millis(100), glib::clone!(@weak obj => @default-return glib::ControlFlow::Break, move || {
                    if let Ok(result_string) = receiver.try_recv() {
                        let imp = obj.imp();
                        
                        if result_string.trim().is_empty() {
                            imp.label.set_text("System up to date");
                        } else {
                            imp.label.set_text("Updates found!");
                            
                            // --- NEW LIST POPULATION LOGIC ---
                            for line in result_string.lines() {
                                if !line.trim().is_empty() {
                                    // Create a simple label for the row
                                    let label = gtk::Label::builder()
                                        .label(line)
                                        .halign(gtk::Align::Start)
                                        .margin_start(12)
                                        .margin_top(6)
                                        .margin_bottom(6)
                                        .build();

                                    // Add the label to the list
                                    imp.update_list.append(&label);
                                }
                            }
                        }
                        glib::ControlFlow::Break
                    } else {
                        glib::ControlFlow::Continue
                    }
                }));
            }));


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
}
