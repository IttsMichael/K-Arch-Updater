/* main.rs
 *
 * Copyright 2026 Unknown
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod application;
mod config;
mod window;
mod update_manager;
mod update_row;

use self::application::UpdaterApplication;
use config::{GETTEXT_PACKAGE, LOCALEDIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::{gio, glib};
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
    
    // --- PERMANENT RESOURCE FIX ---
    // include_bytes! bakes the file into your binary.
    // Ensure src/updater.gresource exists before running meson/ninja.
    let resource_data = include_bytes!("updater.gresource");
    let resource_bytes = glib::Bytes::from_static(resource_data);
    let resources = gio::Resource::from_data(&resource_bytes)
        .expect("Could not load embedded resources");
    gio::resources_register(&resources);
    
    // Create app
    let app = UpdaterApplication::new("org.gnome.Example", &gio::ApplicationFlags::empty());
    
    // Load CSS after app is created
    app.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("../data/style.css"));
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
            
    app.run()
}