/*Copyright (c) 2022 Diego da Silva Lima. All rights reserved.

This work is licensed under the terms of the GPL v3.0 License.
For a copy, see http://www.gnu.org/licenses.*/

use gtk4::*;
use gtk4::prelude::*;
use drafts::manager::*;
use drafts::typesetter::Typesetter;
use drafts::ui::*;
use drafts::analyzer::Analyzer;
use gtk4::gio;
use stateful::React;
use stateful::PersistentState;
use drafts::state::PapersState;
use drafts::typst_tools::Fonts;
use std::rc::Rc;

fn register_resource() -> gio::Resource {
    let bytes = glib::Bytes::from_static(include_bytes!(concat!(env!("OUT_DIR"), "/", "compiled.gresource")));
    let resource = gio::Resource::from_data(&bytes).unwrap();
    gio::resources_register(&resource);
    resource
}

fn main() {
    gtk4::init().unwrap();

    let application = Application::builder()
        .application_id(drafts::APP_ID)
        .build();

    systemd_journal_logger::init();
    log::set_max_level(log::LevelFilter::Info);

    let resource = register_resource();
    let fonts = Fonts::new(&resource);

    // let resource = gio::Resource::load("resources/compiled.gresource");

    // For non-flatpak builds, store at XDG_CACHE_HOME/my.add.id (usually ~/.local/share/my.add.id)
    // For flatpak build, store simply at XDG_CACHE_HOME, which will already point to the right location.
    // Perhaps check if there exists a dir my.app.id under XDG_CACHE_HOME. If there is, use it; or
    // create it otherwise.
    // let cache = std::env::var("XDG_CACHE_HOME");

    let style_manager = libadwaita::StyleManager::default();
    style_manager.set_color_scheme(libadwaita::ColorScheme::Default);

    /*match  {
        Some(style_manager) => {

        },
        None => {
            panic!()
        }
    }*/

    let user_state = if let Some(mut path) = filecase::get_datadir(drafts::APP_ID) {
        path.push(drafts::SETTINGS_FILE);
        PapersState::recover(&path.to_str().unwrap()).unwrap_or_default()
    } else {
        log::warn!("Unable to get datadir for state recovery");
        PapersState::default()
    };

    if let Some(display) = gdk::Display::default() {
        let theme = IconTheme::for_display(&display);
        theme.add_resource_path("/io/github/limads/drafts/icons");
    } else {
        panic!("No default display");
    }

    application.set_accels_for_action("win.save_file", &["<Ctrl>S"]);
    application.set_accels_for_action("win.open_file", &["<Ctrl>O"]);

    // Because "new file" is closing action to the user.
    application.set_accels_for_action("win.new_file", &["<Ctrl>Q"]);

    application.set_accels_for_action("win.save_as_file", &["<Ctrl><Shift>S"]);
    application.set_accels_for_action("win.typeset", &["F7"]);

    application.connect_activate({
        let user_state = user_state.clone();
        move |app| {
            let window = ApplicationWindow::builder()
                .application(app)
                .title("Drafts")
                .default_width(1024)
                .default_height(768)
                .build();

            let papers_win = PapersWindow::new(window, user_state.clone());
            user_state.update(&papers_win);
            papers_win.react(&papers_win.start_screen);

            let manager = FileManager::new();
            manager.react(&papers_win.titlebar.main_menu.open_dialog);
            manager.react(&papers_win.titlebar.main_menu.save_dialog);
            manager.react(&papers_win.titlebar.main_menu);
            manager.react(&papers_win);
            manager.react(&papers_win.editor);

            user_state.react(&papers_win);
            papers_win.titlebar.main_menu.save_dialog.react(&manager);
            papers_win.titlebar.main_menu.open_dialog.react(&manager);

            papers_win.start_screen.recent_list.react(&manager);

            let typesetter = Typesetter::new(fonts.clone());
            typesetter.react(&papers_win);
            typesetter.react(&manager);

            papers_win.titlebar.react(&typesetter);

            papers_win.editor.react(&typesetter);
            papers_win.editor.pdf_viewer.react(&papers_win.titlebar);
            papers_win.editor.react(&manager);
            papers_win.react(&manager);

            let analyzer = Analyzer::new();
            analyzer.react(&papers_win);
            analyzer.react(&papers_win.doc_tree);
            analyzer.react(&manager);

            papers_win.titlebar.react(&analyzer);
            papers_win.titlebar.react(&manager);
            papers_win.titlebar.bib_popover.react(&analyzer);
            papers_win.doc_tree.react(&analyzer);
            papers_win.editor.react(&analyzer);
            papers_win.react(&typesetter);

            papers_win.window.show();
        }
    });

    application.run();

    if let Some(mut path) = filecase::get_datadir(drafts::APP_ID) {
        path.push(drafts::SETTINGS_FILE);
        user_state.persist(&path.to_str().unwrap());
    } else {
        log::warn!("Unable to get datadir for state persistence");
    }
}


