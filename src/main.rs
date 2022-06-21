use gtk4::*;
use gtk4::prelude::*;
use papers::manager::*;
use papers::typesetter::Typesetter;
use papers::ui::*;
use papers::analyzer::Analyzer;
use gtk4::gio;
use stateful::React;
use stateful::PersistentState;
use papers::state::PapersState;

// flatpak remote-add --if-not-exists gnome-nightly https://nightly.gnome.org/gnome-nightly.flatpakrepo
// flatpak install gnome-nightly org.gnome.Sdk master
// flatpak install gnome-nightly org.gnome.Platform master
// flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable
// To install locally, pass the --install flag without any arguments.
// flatpak-builder --force-clean --install /home/diego/Downloads/papers-build com.github.limads.Papers.json

/*
At flatpak builds, the application can always read/write from (which will be resolved to the corresponding subdir of ~/.var/<appid>:
XDG_DATA_HOME
XDG_CONFIG_HOME
XDG_CACHE_HOME
XDG_STATE_HOME
*/

fn main() {
    gtk4::init().unwrap();
    let application = Application::builder()
        .application_id(papers::APP_ID)
        .build();

    systemd_journal_logger::init();
    log::set_max_level(log::LevelFilter::Info);

    let bytes = glib::Bytes::from_static(include_bytes!(concat!(env!("OUT_DIR"), "/", "compiled.gresource")));
    let resource = gio::Resource::from_data(&bytes).unwrap();
    gio::resources_register(&resource);

    // let resource = gio::Resource::load("resources/compiled.gresource");

    // For non-flatpak builds, store at XDG_CACHE_HOME/my.add.id (usually ~/.local/share/my.add.id)
    // For flatpak build, store simply at XDG_CACHE_HOME, which will already point to the right location.
    // Perhaps check if there exists a dir my.app.id under XDG_CACHE_HOME. If there is, use it; or
    // create it otherwise.
    // let cache = std::env::var("XDG_CACHE_HOME");

    match libadwaita::StyleManager::default() {
        Some(style_manager) => {
            style_manager.set_color_scheme(libadwaita::ColorScheme::Default);
        },
        None => {
            panic!()
        }
    }

    let user_state = if let Some(mut path) = archiver::get_datadir(papers::APP_ID) {
        path.push(papers::SETTINGS_FILE);
        PapersState::recover(&path.to_str().unwrap()).unwrap_or_default()
    } else {
        log::warn!("Unable to get datadir for state recovery");
        PapersState::default()
    };

    if let Some(display) = gdk::Display::default() {
        if let Some(theme) = IconTheme::for_display(&display) {
            // Useful for local builds
            // theme.add_search_path("/home/diego/Software/papers/assets/icons");
            // theme.add_search_path("/home/diego/Software/papers/data/icons");

            // theme.add_resource_path("/com/github/limads/papers");

            // This is the valid path according to docs.
            theme.add_resource_path("/com/github/limads/papers/icons");
            // theme.add_resource_path("/com/github/limads/papers/icons/scalable");

            // theme.add_resource_path("/com/github/limads/papers/icons/symbolic");


            // theme.add_resource_path("/com/github/limads/papers/icons/hicolor");

            // theme.add_search_path("/home/diego/Software/papers/data/icons/hicolor/symbolic");
            // theme.add_search_path("/home/diego/Software/papers/data/icons/hicolor/scalable");
            // println!("Theme search path={:?}", theme.search_path());
            // println!("Icon names = {:?}", theme.icon_names());
            // let icon = theme.lookup_icon("break-point-symbolic", &[], 16, 1, TextDirection::Ltr, IconLookupFlags::empty());
            // println!("Icon = {:?}", icon);
            // println!("Icon file = {:?}", icon.and_then(|icon| icon.file().and_then(|f| f.path() )));
            // Then Pixbuf::from_file_at_scale("assets/icons/break-point-symbolic.svg", 16, 16, true) with the desired path.
        } else {
            panic!("No icon theme");
        }
    } else {
        panic!("No default display");
    }

    application.connect_activate({
        let user_state = user_state.clone();
        move |app| {
            let window = ApplicationWindow::builder()
                .application(app)
                .title("Papers")
                .default_width(1024)
                .default_height(768)
                .build();

            // println!("{:?}", app.resource_base_path());
            // let icon = gio::resources_lookup_data("/com/github/limads/papers/icons/symbolic/actions/equation-symbolic.svg", gio::ResourceLookupFlags::empty()).unwrap();
            // println!("{:?}", String::from_utf8(icon.as_ref().to_owned()).unwrap());

            let papers_win = PapersWindow::from(window);
            let s = { user_state.borrow().window.width };
            papers_win.editor.sub_paned.set_position(s);

            papers_win.react(&papers_win.start_screen);
            user_state.react(&papers_win);

            let manager = FileManager::new();
            manager.react(&papers_win.titlebar.main_menu.open_dialog);
            manager.react(&papers_win.titlebar.main_menu.save_dialog);
            manager.react(&papers_win.titlebar.main_menu);
            manager.react(&papers_win);
            manager.react(&papers_win.editor);

            papers_win.titlebar.main_menu.save_dialog.react(&manager);
            papers_win.titlebar.main_menu.open_dialog.react(&manager);

            let typesetter = Typesetter::new();
            typesetter.react(&papers_win);
            typesetter.react(&manager);

            papers_win.titlebar.react(&typesetter);
            papers_win.editor.react(&typesetter);
            papers_win.editor.viewer.react(&papers_win.titlebar);
            papers_win.editor.react(&manager);
            papers_win.react(&manager);

            let analyzer = Analyzer::new();
            analyzer.react(&papers_win.editor);
            analyzer.react(&papers_win.doc_tree);
            analyzer.react(&manager);

            papers_win.titlebar.bib_popover.react(&analyzer);
            papers_win.doc_tree.react(&analyzer);
            papers_win.editor.react(&analyzer);
            papers_win.react(&typesetter);

            papers_win.window.show();
        }
    });

    // application.connect_window_added()
    // application.connect_window_removed()

    application.run();

    if let Some(mut path) = archiver::get_datadir(papers::APP_ID) {
        path.push(papers::SETTINGS_FILE);
        user_state.persist(&path.to_str().unwrap());
    } else {
        log::warn!("Unable to get datadir for state persistence");
    }
}


