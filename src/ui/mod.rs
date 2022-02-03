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
use gio::prelude::*;
use glib::{types::Type, value::{Value, ToValue}};
use gdk_pixbuf::Pixbuf;

mod doctree;

mod titlebar;

mod editor;

pub use titlebar::*;

pub use doctree::*;

pub use editor::*;

#[derive(Debug, Clone)]
pub struct PapersWindow {
    pub window : ApplicationWindow,
    pub titlebar : Titlebar,
    pub editor : PapersEditor,
    pub doc_tree : DocTree,
    pub stack : Stack,
    pub start_screen : StartScreen
}

const ARTICLE_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}
\usepackage[utf8]{ulem}

\begin{document}

\end{document}"#;

const REPORT_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}

\begin{document}

\end{document}"#;

const PRESENTATION_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}

\begin{document}

\end{document}"#;

impl React<StartScreen> for PapersWindow {

    fn react(&self, start_screen : &StartScreen) {

        start_screen.empty_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.action_save.clone();
            let action_save_as = self.titlebar.main_menu.action_save_as.clone();
            move |_| {
                view.buffer().set_text("");
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.article_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.action_save.clone();
            let action_save_as = self.titlebar.main_menu.action_save_as.clone();
            move |_| {
                view.buffer().set_text(ARTICLE_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.report_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.action_save.clone();
            let action_save_as = self.titlebar.main_menu.action_save_as.clone();
            move |_| {
                view.buffer().set_text(REPORT_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.presentation_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.action_save.clone();
            let action_save_as = self.titlebar.main_menu.action_save_as.clone();
            move |_| {
                view.buffer().set_text(PRESENTATION_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
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

#[derive(Debug, Clone)]
pub struct StartScreen {
    bx : Box,
    empty_btn : Button,
    article_btn : Button,
    report_btn : Button,
    presentation_btn : Button
}

impl StartScreen {

    pub fn build() -> Self {
        let bx = Box::new(Orientation::Horizontal, 0);
        let empty_btn = Button::builder() /*.label("Empty")*/ .build();

        let img = Image::from_icon_name(Some("folder-documents-symbolic"));
        let lbl = Label::new(Some("Empty"));
        let bx_btn = Box::new(Orientation::Vertical, 0);
        bx_btn.append(&img);
        bx_btn.append(&lbl);
        empty_btn.set_child(Some(&bx_btn));
        let article_btn = Button::builder().label("Article").build();
        let report_btn = Button::builder().label("Report").build();
        let presentation_btn = Button::builder().label("Presentation").build();
        // letter
        // book
        for btn in [&empty_btn, &article_btn, &report_btn, &presentation_btn] {
            btn.style_context().add_class("flat");
            btn.set_vexpand(true);
            btn.set_valign(Align::Center);
        }
        bx.append(&empty_btn);
        bx.append(&article_btn);
        bx.append(&report_btn);
        Self { bx, empty_btn, article_btn, report_btn, presentation_btn }
    }

}

const GREEK_SMALL : [(&'static str, &'static str); 24] = [
    ("α", "\\alpha"),
    ("β", "\\beta"),
    ("γ", "\\gamma"),
    ("δ", "\\delta"),
    ("ε", "\\epsilon"),
    ("ζ", "\\zeta"),
    ("η", "\\eta"),
    ("θ", "\\theta"),
    ("ι", "\\iota"),
    ("κ", "\\kappa"),
    ("λ", "\\lambda"),
    ("μ", "\\mu"),
    ("ν", "\\nu"),
    ("ξ", "\\xi"),
    ("ο", "\\omicron"),
    ("π", "\\pi"),
    ("ρ", "\\rho"),
    ("σ", "\\sigma"),
    ("τ", "\\tau"),
    ("υ", "\\upsilon"),
    ("φ", "\\phi"),
    ("χ", "\\chi"),
    ("ψ", "\\psi"),
    ("ω", "\\omega")
];

const GREEK_CAPITAL : [(&'static str, &'static str); 24] = [
    ("Α", "\\Alpha"),
    ("Β", "\\Beta"),
    ("Γ", "\\Gamma"),
    ("Δ", "\\Delta"),
    ("Ε", "\\Epsilon"),
    ("Ζ", "\\Zeta"),
    ("Η", "\\Eta"),
    ("Θ", "\\Theta"),
    ("Ι", "\\Iota"),
    ("Κ", "\\Kappa"),
    ("Λ", "\\Lambda"),
    ("Μ", "\\Mu"),
    ("Ν", "\\Nu"),
    ("Ξ", "\\Xi"),
    ("Ο", "\\Omicron"),
    ("Π", "\\Pi"),
    ("Ρ", "\\Rho"),
    ("Σ", "\\Sigma"),
    ("Τ", "\\Tau"),
    ("Υ", "\\Upsilon"),
    ("Φ", "\\Phi"),
    ("Χ", "\\Chi"),
    ("Ψ", "\\Psi"),
    ("Ω", "\\Omega")
];

// https://en.wikipedia.org/wiki/Mathematical_operators_and_symbols_in_Unicode
const OPERATORS : [(&'static str, &'static str); 48] = [
    ("=", "\\eq"),
    ("⋜", "\\leq"),
    ("⋝", "\\geq"),
    ("≠", "\\neq"),
    ("√", "\\sqrt"),
    (">", ">"),
    ("<", "<"),
    ("×", "\\times"),
    ("÷", "\\div"),
    ("±",  "\\pm"),
    ("∫", "\\int"),
    ("∑", "\\sum"),
    ("⨅", "\\prod"),
    ("→", "\\to"),
    ("↦", "\\mapsto"),
    ("∂", "\\partial"),
    ("∇", "\\nabla"),
    ("∼", "\\tilde"),
    ("∣", "\\vert"),
    ("∘", "\\circ"),
    ("∗", "\\ast"),
    ("∠", "\\angle"),
    ("∀", "\\forall"),
    ("∃", "\\exists"),
    ("∄", "\\nexists"),
    ("∈", "\\in"),
    ("∈/", "\\notin"),
    ("∧", "\\land"),
    ("∨", "\\lor"),
    ("a^", "\\hat"),
    ("△", "\\triangle"),
    ("∴",  "\\therefore"),
    ("∵",  "\\because"),
    ("⋆", "\\star"),
    ("½", "\\frac{}{}"),
    ("∅", "\\emptyset"),
    ("∪", "\\cup"),
    ("∩", "\\cap"),
    ("⋃", "\\bigcup"),
    ("⋂ ", "\\bigcap"),
    ("∖", "\\setminus"),
    ("⊂", "\\sub"),
    ("⊆", "\\sube"),
    ("⊃", "\\supset"),
    ("⊇", "\\supe"),
    ("…",  "\\dots"),
    ("⋱", "\\ddots"),
    ("⋮", "\\vdots"),
];

#[derive(Debug, Clone)]
pub struct SymbolGrid {
    pub grid : Grid
}

impl SymbolGrid {

    pub fn new(symbols : &'static [(&'static str, &'static str)], view : View, dialog : Dialog, ncol : usize) -> Self {
        let grid = Grid::new();
        set_all_margins(&grid, 6);
        grid.set_margin_bottom(36);
        for row in 0..(symbols.len() / ncol) {
            for col in 0..ncol {
                if let Some(symbol) = symbols.get(row*ncol + col) {
                    let btn = Button::new();
                    btn.set_label(symbol.0);
                    btn.connect_clicked({
                        let view = view.clone();
                        let dialog = dialog.clone();
                        move|_| {
                            dialog.close();
                            view.buffer().insert_at_cursor(symbol.1);
                        }
                    });
                    btn.style_context().add_class("flat");
                    grid.attach(&btn, col as i32, row as i32, 1, 1);
                }
            }
        }
        Self { grid }
    }

}

// let min_driver = tectonic_bridge_core::MinimalDriver::new(tectonic_io_base::stdstreams::BufferedPrimaryIo::from_buffer(Vec::new()));
// let status = tectonic::status::plain::PlainStatusBackend::new(tectonic::status::ChatterLevel::Minimal);
// tectonic::engines::spx2html::SpxHtmlEngine::new(&mut min_driver, &mut status).process(hooks, status, spx_str);
impl PapersWindow {

    pub fn from(window : ApplicationWindow) -> Self {

        let titlebar = Titlebar::build();
        window.set_titlebar(Some(&titlebar.header));
        window.set_decorated(true);

        let doc_tree = DocTree::build();
        let editor = PapersEditor::build();
        let start_screen = StartScreen::build();

        titlebar.main_menu.save_dialog.dialog.set_transient_for(Some(&window));
        titlebar.main_menu.open_dialog.dialog.set_transient_for(Some(&window));

        // titlebar.main_menu.open_dialog.react(&titlebar.main_menu);
        titlebar.main_menu.save_dialog.react(&titlebar.main_menu);
        editor.react(&titlebar);
        editor.react(&titlebar.bib_popover);

        // source.set_halign(Align::Center);
        // source.set_margin_start(256);
        // source.set_margin_end(256);

        // let web = webkit2gtk5::WebView::new();
        // web.load_html("<html><head></head><body>Hello world</body></html>", None);
        // web.set_margin_start(18);

        // window.set_child(Some(&editor.overlay));

        // let ws = Rc::new(RefCell::new(Workspace::new()));

        window.add_action(&titlebar.main_menu.action_new);
        window.add_action(&titlebar.main_menu.action_open);
        window.add_action(&titlebar.main_menu.action_save);
        window.add_action(&titlebar.main_menu.action_save_as);

        window.add_action(&titlebar.sidebar_hide_action);
        window.add_action(&editor.ignore_file_save_action);

        let stack = Stack::new();
        stack.add_named(&start_screen.bx, Some("start"));
        stack.add_named(&editor.overlay, Some("editor"));

        editor.paned.set_start_child(&doc_tree.bx);
        editor.paned.set_end_child(&stack);
        editor.paned.set_position(0);

        window.set_child(Some(&editor.paned));

        let symbol_dialog = Dialog::new();
        symbol_dialog.set_title(Some("Symbols"));
        configure_dialog(&symbol_dialog);
        symbol_dialog.set_transient_for(Some(&window));
        let greek_small_grid = SymbolGrid::new(&GREEK_SMALL[..], editor.view.clone(), symbol_dialog.clone(), 12);
        let greek_capital_grid = SymbolGrid::new(&GREEK_CAPITAL[..], editor.view.clone(), symbol_dialog.clone(), 12);
        let operators_grid = SymbolGrid::new(&OPERATORS[..], editor.view.clone(), symbol_dialog.clone(), 12);
        let symbol_bx = Box::new(Orientation::Vertical, 0);
        let operators_lbl = Label::builder().label("Operators").halign(Align::Start).build();
        let greek_lbl = Label::builder().label("Greek alphabet").halign(Align::Start).build();
        let greek_capital_lbl = Label::builder().label("Greek alphabet (Capitalized)").halign(Align::Start).build();
        for lbl in [&operators_lbl, &greek_lbl, &greek_capital_lbl] {
            set_all_margins(lbl, 6);
        }
        symbol_bx.append(&operators_lbl);
        symbol_bx.append(&operators_grid.grid);
        symbol_bx.append(&greek_lbl);
        symbol_bx.append(&greek_small_grid.grid);
        symbol_bx.append(&greek_capital_lbl);
        symbol_bx.append(&greek_capital_grid.grid);

        symbol_dialog.set_child(Some(&symbol_bx));
        titlebar.math_actions.symbol.connect_activate(move |_, _| {
            symbol_dialog.show();
        });

        for action in titlebar.math_actions.iter().chain(titlebar.object_actions.iter().chain(titlebar.struct_actions.iter())) {
            window.add_action(&action);
        }

        Self { window, titlebar, editor, doc_tree, stack, start_screen }
    }

}

impl React<FileManager> for PapersWindow {

    fn react(&self, manager : &FileManager) {
        let win = self.window.clone();
        manager.connect_window_close(move |_| {
            win.destroy();
        });
        manager.connect_opened({
            let action_save = self.titlebar.main_menu.action_save.clone();
            let action_save_as = self.titlebar.main_menu.action_save_as.clone();
            let stack = self.stack.clone();
            let window = self.window.clone();
            move |(path, _)| {
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
                window.set_title(Some(&path));
                stack.set_visible_child_name("editor");
            }
        });
        manager.connect_new({
            let stack = self.stack.clone();
            let window = self.window.clone();
            let action_save = self.titlebar.main_menu.action_save.clone();
            let action_save_as = self.titlebar.main_menu.action_save_as.clone();
            let view = self.editor.view.clone();
            move |_| {
                stack.set_visible_child_name("start");
                window.set_title(Some("Papers"));
                // view.buffer().set_text("");
                action_save.set_enabled(false);
                action_save_as.set_enabled(false);
            }
        });
        manager.connect_open_request({
            let open_action = self.titlebar.main_menu.action_open.clone();
            let view = self.editor.view.clone();
            move |_| {
                open_action.activate(None);
            }
        });
        manager.connect_save({
            let window = self.window.clone();
            move |path| {
                window.set_title(Some(&path));
            }
        });
        manager.connect_file_changed({
            let window = self.window.clone();
            let view = self.editor.view.clone();
            move |opt_path| {
                if let Some(path) = opt_path {
                    window.set_title(Some(&format!("{}*", path)));
                } else {
                    window.set_title(Some("Untitled.tex*"));
                }
            }
        });
    }

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

impl React<FileManager> for SaveDialog {

    fn react(&self, manager : &FileManager) {
        let dialog = self.dialog.clone();
        manager.connect_save_unknown_path(move |path| {
            // let _ = dialog.set_file(&gio::File::for_path(path));
            dialog.show();
        });
        // let dialog = self.dialog.clone();
        /*scripts.connect_path_changed(move |opt_file| {
            if let Some(path) = opt_file.and_then(|f| f.path.clone() ) {
                let _ = dialog.set_file(&gio::File::for_path(&path));
            }
        });*/
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

impl React<FileManager> for OpenDialog {

    fn react(&self, manager : &FileManager) {
        let dialog = self.dialog.clone();
        manager.connect_show_open(move |_| {
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

#[derive(Debug, Clone)]
pub struct PackedImageLabel  {
    pub bx : Box,
    pub img : Image,
    pub lbl : Label
}

impl PackedImageLabel {

    pub fn build(icon_name : &str, label_name : &str) -> Self {
        let bx = Box::new(Orientation::Horizontal, 0);
        let img = Image::from_icon_name(Some(icon_name));
        let lbl = Label::new(Some(label_name));
        set_margins(&img, 6, 6);
        set_margins(&lbl, 6, 6);
        bx.append(&img);
        bx.append(&lbl);
        Self { bx, img, lbl }
    }

    pub fn extract(bx : &Box) -> Option<Self> {
        let img = get_child_by_index::<Image>(&bx, 0);
        let lbl = get_child_by_index::<Label>(&bx, 1);
        Some(Self { bx : bx.clone(), lbl, img })
    }

    pub fn change_label(&self, label_name : &str) {
        self.lbl.set_text(label_name);
    }

    pub fn change_icon(&self, icon_name : &str) {
        self.img.set_icon_name(Some(icon_name));
    }

}

fn set_border_to_title(bx : &Box) {
    let provider = CssProvider::new();
    provider.load_from_data("* { border-bottom : 1px solid #d9dada; } ".as_bytes());
    bx.style_context().add_provider(&provider, 800);
}

pub fn get_child_by_index<W>(w : &Box, pos : usize) -> W
where
    W : IsA<glib::Object>
{
    w.observe_children().item(pos as u32).unwrap().clone().downcast::<W>().unwrap()
}

pub fn set_margins<W : WidgetExt>(w : &W, horizontal : i32, vertical : i32) {
    w.set_margin_start(horizontal);
    w.set_margin_end(horizontal);
    w.set_margin_top(vertical);
    w.set_margin_bottom(vertical);
}

pub fn set_all_margins<W : WidgetExt>(w : &W, margin : i32) {
    w.set_margin_start(margin);
    w.set_margin_end(margin);
    w.set_margin_top(margin);
    w.set_margin_bottom(margin);
}
