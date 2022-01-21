use gtk4::*;
use gtk4::prelude::*;
use papers::manager::*;
use papers::React;
use papers::typesetter::Typesetter;
use papers::ui::*;
use papers::analyzer::Analyzer;

const APP_ID : &'static str = "com.github.limads.papers";

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
            manager.react(&papers_win.open_dialog);
            manager.react(&papers_win.save_dialog);
            manager.react(&papers_win.titlebar.main_menu);
            manager.react(&papers_win);
            manager.react(&papers_win.editor);

            papers_win.save_dialog.react(&manager);

            let typesetter = Typesetter::new();
            typesetter.react(&(papers_win.titlebar.clone(), papers_win.editor.clone()));
            papers_win.titlebar.react(&typesetter);
            papers_win.editor.react(&typesetter);

            papers_win.editor.react(&manager);
            papers_win.react(&manager);

            let analyzer = Analyzer::new();
            analyzer.react(&papers_win.editor);

            papers_win.window.show();

        }
    });
    application.run();
}


