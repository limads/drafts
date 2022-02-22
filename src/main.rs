use gtk4::*;
use gtk4::prelude::*;
use papers::manager::*;
use papers::React;
use papers::typesetter::Typesetter;
use papers::ui::*;
use papers::analyzer::Analyzer;

const APP_ID : &'static str = "com.github.limads.papers";

// flatpak remote-add --if-not-exists gnome-nightly https://nightly.gnome.org/gnome-nightly.flatpakrepo
// flatpak install gnome-nightly org.gnome.Sdk master
// flatpak install gnome-nightly org.gnome.Platform master
// flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable
// To install locally, pass the --install flag without any arguments.
// flatpak-builder --force-clean --install /home/diego/Downloads/papers-build com.github.limads.Papers.json

fn main() {
    gtk4::init().unwrap();
    let application = Application::builder()
        .application_id(APP_ID)
        .build();

    match libadwaita::StyleManager::default() {
        Some(style_manager) => {
            style_manager.set_color_scheme(libadwaita::ColorScheme::Default);
        },
        None => {
            panic!()
        }
    }

    if let Some(display) = gdk::Display::default() {
        if let Some(theme) = IconTheme::for_display(&display) {
            theme.add_search_path("/home/diego/Software/papers/assets/icons");
        }
    }

    application.connect_activate({
        move |app| {
            let window = ApplicationWindow::builder()
                .application(app)
                .title("Papers")
                .default_width(1024)
                .default_height(768)
                .build();
            let papers_win = PapersWindow::from(window);

            papers_win.react(&papers_win.start_screen);

            let manager = FileManager::new();
            manager.react(&papers_win.titlebar.main_menu.open_dialog);
            manager.react(&papers_win.titlebar.main_menu.save_dialog);
            manager.react(&papers_win.titlebar.main_menu);
            manager.react(&papers_win);
            manager.react(&papers_win.editor);

            papers_win.titlebar.main_menu.save_dialog.react(&manager);
            papers_win.titlebar.main_menu.open_dialog.react(&manager);

            let typesetter = Typesetter::new();
            typesetter.react(&(papers_win.titlebar.clone(), papers_win.editor.clone()));
            papers_win.titlebar.react(&typesetter);
            papers_win.editor.react(&typesetter);

            papers_win.editor.react(&manager);
            papers_win.react(&manager);

            let analyzer = Analyzer::new();
            analyzer.react(&papers_win.editor);
            analyzer.react(&papers_win.doc_tree);

            papers_win.titlebar.bib_popover.react(&analyzer);
            papers_win.doc_tree.react(&analyzer);
            papers_win.editor.react(&analyzer);
            papers_win.react(&typesetter);

            papers_win.window.show();
        }
    });
    application.run();
}


