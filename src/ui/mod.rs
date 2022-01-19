use gtk4::*;
use gtk4::prelude::*;
use super::React;
use sourceview5::*;
use sourceview5::prelude::ViewExt;
use sourceview5::prelude::BufferExt;
use sourceview5::prelude::CompletionWordsExt;
use tempfile;
use std::rc::Rc;
use std::cell::RefCell;
use crate::manager::FileManager;
use crate::typesetter::Typesetter;

#[derive(Debug, Clone)]
pub struct MainMenu {
    pub popover : PopoverMenu,
    pub action_new : gio::SimpleAction,
    pub action_open : gio::SimpleAction,
    pub action_save : gio::SimpleAction,
    pub action_save_as : gio::SimpleAction,
}

impl MainMenu {

    fn build() -> Self {
        let menu = gio::Menu::new();
        menu.append(Some("New"), Some("win.new_file"));
        menu.append(Some("Open"), Some("win.open_file"));
        menu.append(Some("Save"), Some("win.save_file"));
        menu.append(Some("Save as"), Some("win.save_as_file"));
        let popover = PopoverMenu::from_model(Some(&menu));
        let action_new = gio::SimpleAction::new("new_file", None);
        let action_open = gio::SimpleAction::new("open_file", None);
        let action_save = gio::SimpleAction::new("save_file", None);
        let action_save_as = gio::SimpleAction::new("save_as_file", None);
        Self { popover, action_new, action_open, action_save, action_save_as }
    }

}

#[derive(Debug, Clone)]
pub struct Titlebar {
    pub header : HeaderBar,
    pub menu_button : MenuButton,
    pub main_menu : MainMenu,
    pub pdf_btn : Button
}

impl Titlebar {

    fn build() -> Self {
        let header = HeaderBar::new();
        let menu_button = MenuButton::builder().icon_name("open-menu-symbolic").build();

        let pdf_btn = Button::builder().icon_name("evince-symbolic").build();

        // TODO make this another option at a SpinButton.
        // let web_btn = ToggleButton::builder().icon_name("globe-symbolic").build();

        header.pack_end(&menu_button);
        header.pack_end(&pdf_btn);
        // header.pack_end(&web_btn);

        let main_menu = MainMenu::build();
        menu_button.set_popover(Some(&main_menu.popover));
        Self { main_menu, header, menu_button, pdf_btn }
    }
}

impl React<Typesetter> for Titlebar {

    fn react(&self, typesetter : &Typesetter) {
        let btn = self.pdf_btn.clone();
        typesetter.connect_done({
            move |_| {
                btn.set_icon_name("evince-symbolic");
                btn.set_sensitive(true);
            }
        });
        typesetter.connect_error({
            let btn = self.pdf_btn.clone();
            move |_| {
                btn.set_icon_name("evince-symbolic");
                btn.set_sensitive(true);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct PapersWindow {
    pub window : ApplicationWindow,
    pub titlebar : Titlebar,
    pub editor : PapersEditor,
    pub open_dialog : OpenDialog,
    pub save_dialog : SaveDialog
}

#[derive(Debug, Clone)]
pub struct PapersEditor {
    pub view : View,
    pub scroll : ScrolledWindow
}

impl PapersEditor {

    pub fn build() -> Self {
        let view = View::new();
        view.set_hexpand(true);
        configure_view(&view);
        view.set_width_request(800);
        view.set_halign(Align::Center);
        view.set_hexpand(true);
        view.set_margin_top(98);
        view.set_margin_bottom(98);

        let scroll = ScrolledWindow::new();
        let provider = CssProvider::new();
        provider.load_from_data("* { background-color : #ffffff; } ".as_bytes());

        // scroll.set_kinetic_scrolling(false);

        scroll.style_context().add_provider(&provider, 800);
        scroll.set_child(Some(&view));

        Self { scroll, view }
    }
}

impl React<FileManager> for PapersEditor {

    fn react(&self, manager : &FileManager) {
        manager.connect_opened({
            let view = self.view.clone();
            move |(path, content)| {
                view.buffer().set_text(&content);
            }
        });
    }

}

// Create tectonic workspace
// tectonic -X new myfirstdoc

// Compile document in on-off fashion
// tectonic -X compile myfile.tex

// Compile workspace
// tectonic -X build

// tectonic -X compile geometrical-notes.odt --outfmt html

/* Add at the preamble to support Unicode/HTML
\usepackage{fontspec}
\setmainfont{texgyrepagella}[
  Extension = .otf,
  UprightFont = *-regular,
  BoldFont = *-bold,
  ItalicFont = *-italic,
  BoldItalicFont = *-bolditalic,
]
*/

// let min_driver = tectonic_bridge_core::MinimalDriver::new(tectonic_io_base::stdstreams::BufferedPrimaryIo::from_buffer(Vec::new()));
// let status = tectonic::status::plain::PlainStatusBackend::new(tectonic::status::ChatterLevel::Minimal);
// tectonic::engines::spx2html::SpxHtmlEngine::new(&mut min_driver, &mut status).process(hooks, status, spx_str);
impl PapersWindow {

    pub fn from(window : ApplicationWindow) -> Self {

        let titlebar = Titlebar::build();
        window.set_titlebar(Some(&titlebar.header));
        window.set_decorated(true);

        let editor = PapersEditor::build();
        let open_dialog = OpenDialog::build();
        let save_dialog = SaveDialog::build();

        open_dialog.react(&titlebar.main_menu);
        save_dialog.react(&titlebar.main_menu);

        // source.set_halign(Align::Center);
        // source.set_margin_start(256);
        // source.set_margin_end(256);

        // let paned = Paned::new(Orientation::Horizontal);

        // let web = webkit2gtk5::WebView::new();
        // web.load_html("<html><head></head><body>Hello world</body></html>", None);
        // web.set_margin_start(18);

        // paned.set_start_child(&scroll);
        // paned.set_end_child(&web);
        // window.set_child(Some(&paned));
        window.set_child(Some(&editor.scroll));

        // let ws = Rc::new(RefCell::new(Workspace::new()));

        window.add_action(&titlebar.main_menu.action_new);
        window.add_action(&titlebar.main_menu.action_open);
        window.add_action(&titlebar.main_menu.action_save);
        window.add_action(&titlebar.main_menu.action_save_as);

        Self { window, titlebar, editor, open_dialog, save_dialog }
    }

}

fn configure_view(view : &View) {
    let buffer = view.buffer()
        .downcast::<sourceview5::Buffer>().unwrap();
    let manager = sourceview5::StyleSchemeManager::new();
    let scheme = manager.scheme("Adwaita").unwrap();
    buffer.set_style_scheme(Some(&scheme));
    buffer.set_highlight_syntax(true);
    let provider = CssProvider::new();
    provider.load_from_data(b"textview { font-family: \"Source Code Pro\"; font-size: 16pt; }");
    let ctx = view.style_context();
    ctx.add_provider(&provider, 800);
    let lang_manager = sourceview5::LanguageManager::default().unwrap();
    let lang = lang_manager.language("latex").unwrap();
    buffer.set_language(Some(&lang));
    view.set_tab_width(4);
    view.set_indent_width(4);
    view.set_auto_indent(true);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_highlight_current_line(false);
    view.set_indent_on_tab(true);
    view.set_show_line_marks(true);
    view.set_enable_snippets(true);
    view.set_wrap_mode(WrapMode::Word);

    // Seems to be working, but only when you click on the the word
    // and **then** press CTRL+Space (simply pressing CTRL+space does not work).
    let completion = view.completion().unwrap();
    let words = sourceview5::CompletionWords::new(Some("main"));
    words.register(&view.buffer());
    completion.add_provider(&words);
    view.set_show_line_numbers(false);
}

#[derive(Debug, Clone)]
pub struct SaveDialog {
    pub dialog : FileChooserDialog
}

impl SaveDialog {

    pub fn build() -> Self {
        let dialog = FileChooserDialog::new(
            Some("Save file"),
            None::<&Window>,
            FileChooserAction::Save,
            &[("Cancel", ResponseType::None), ("Save", ResponseType::Accept)]
        );
        dialog.connect_response(move |dialog, resp| {
            match resp {
                ResponseType::Close | ResponseType::Reject | ResponseType::Accept | ResponseType::Yes |
                ResponseType::No | ResponseType::None | ResponseType::DeleteEvent => {
                    dialog.close();
                },
                _ => { }
            }
        });
        configure_dialog(&dialog);
        let filter = FileFilter::new();
        filter.add_pattern("*.tex");
        dialog.set_filter(&filter);
        Self { dialog }
    }

}


impl React<MainMenu> for SaveDialog {

    fn react(&self, menu : &MainMenu) {
        let dialog = self.dialog.clone();
        menu.action_save_as.connect_activate(move |_,_| {
            dialog.show();
        });
    }

}


#[derive(Debug, Clone)]
pub struct OpenDialog {
    pub dialog : FileChooserDialog
}

impl OpenDialog {

    pub fn build() -> Self {
        let dialog = FileChooserDialog::new(
            Some("Open file"),
            None::<&Window>,
            FileChooserAction::Open,
            &[("Cancel", ResponseType::None), ("Open", ResponseType::Accept)]
        );
        dialog.connect_response(move |dialog, resp| {
            match resp {
                ResponseType::Reject | ResponseType::Accept | ResponseType::Yes | ResponseType::No |
                ResponseType::None | ResponseType::DeleteEvent => {
                    dialog.close();
                },
                _ => { }
            }
        });
        configure_dialog(&dialog);
        let filter = FileFilter::new();
        filter.add_pattern("*.tex");
        dialog.set_filter(&filter);
        Self { dialog }
    }

}

impl React<MainMenu> for OpenDialog {

    fn react(&self, menu : &MainMenu) {
        let dialog = self.dialog.clone();
        menu.action_open.connect_activate(move |_,_| {
            dialog.show();
        });
    }

}

pub fn configure_dialog(dialog : &impl GtkWindowExt) {
    dialog.set_modal(true);
    dialog.set_deletable(true);
    dialog.set_destroy_with_parent(true);
    dialog.set_hide_on_close(true);
}

