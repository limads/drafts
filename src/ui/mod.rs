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
use crate::typesetter::{Typesetter, TypesetterTarget};
use gio::prelude::*;
use glib::{types::Type, value::{Value, ToValue}};
use gdk_pixbuf::Pixbuf;
use std::path::Path;

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

// \usepackage[utf8]{ulem}

const ARTICLE_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}

\begin{document}
(Article)
\end{document}"#;

const REPORT_TEMPLATE : &'static str = r#"
\documentclass[a4,11pt]{article}

\usepackage[utf8]{inputenc}
(Article)
\begin{document}

\end{document}"#;

const BOOK_TEMPLATE : &'static str = r#"
\begin{document}
\frontmatter

\maketitle

\chapter{Preface}

\mainmatter
\chapter{First chapter}

\appendix
\chapter{First Appendix}

\backmatter
\chapter{Last note}
"#;

const LETTER_TEMPLATE : &'static str = r#"
\documentclass{letter}
\usepackage{hyperref}
\signature{Joe Bloggs}
\address{21 Bridge Street \\ Smallville \\ Dunwich DU3 4WE}
\begin{document}

\begin{letter}{Director \\ Doe \& Co \\ 35 Anthony Road
\\ Newport \\ Ipswich IP3 5RT}
\opening{Dear Sir or Madam:}

I am writing to you on behalf of the Wikipedia project (http://www.wikipedia.org/),
an endeavour to build a fully-fledged multilingual encyclopaedia in an entirely
open manner, to ask for permission to use your copyrighted material.

% The \ldots command produces dots in a way that will not upset
% the typesetting of the document.
\ldots

That said, allow me to reiterate that your material will be used to the noble end of
providing a free collection of knowledge for everyone; naturally enough, only if you
agree. If that is the case, could you kindly fill in the attached form and post it
back to me? We shall greatly appreciate it.

Thank you for your time and consideration.

I look forward to your reply.

\closing{Yours Faithfully,}

\ps

P.S. You can find the full text of GFDL license at
\url{http://www.gnu.org/copyleft/fdl.html}.

\encl{Copyright permission form}

\end{letter}
\end{document}
"#;

const PRESENTATION_TEMPLATE : &'static str = r#"\documentclass{beamer}
\begin{document}
  \begin{frame}
    \frametitle{This is the first slide}
    %Content goes here
  \end{frame}
  \begin{frame}
    \frametitle{This is the second slide}
    \framesubtitle{A bit more information about this}
    %More content goes here
  \end{frame}
% etc
\end{document}
"#;

impl React<StartScreen> for PapersWindow {

    fn react(&self, start_screen : &StartScreen) {

        start_screen.empty_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text("");
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.article_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text(ARTICLE_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        /*start_screen.report_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text(REPORT_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });
        start_screen.presentation_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                view.buffer().set_text(PRESENTATION_TEMPLATE);
                stack.set_visible_child_name("editor");
                action_save.set_enabled(true);
                action_save_as.set_enabled(true);
            }
        });*/
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
    //report_btn : Button,
    //presentation_btn : Button
}

pub struct DocBtn {
    pub btn : Button
}

impl DocBtn {

    pub fn build(image : &str, title : &str, sub : &str) -> Self {
        let btn = Button::new();
        // let pxb = Pixbuf::from_file(image).unwrap();
        let img = Picture::for_filename(image);
        img.set_can_shrink(false);
        // let img = Picture::from_pixbuf(Some(&pxb));
        // img.set_icon_size(IconSize::Large);
        //img.set_pixel_size(2);
        let lbl_bx = Box::new(Orientation::Vertical, 12);
        let lbl = Label::new(Some(title));
        lbl.set_justify(Justification::Left);
        lbl.set_halign(Align::Start);
        let sub_lbl = Label::builder().use_markup(true).label(&format!("<span font_weight='normal'>{}</span>", sub)).build();
        sub_lbl.set_halign(Align::Start);
        sub_lbl.set_justify(Justification::Left);
        lbl_bx.append(&lbl);
        lbl_bx.append(&sub_lbl);

        let bx_btn = Box::new(Orientation::Horizontal, 12);
        bx_btn.append(&img);
        bx_btn.append(&lbl_bx);

        btn.set_child(Some(&bx_btn));
        btn.style_context().add_class("flat");
        btn.set_vexpand(true);
        btn.set_valign(Align::Center);
        btn.set_width_request(480);
        Self { btn }
    }

}

// \url{http://www.uni.edu/~myname/best-website-ever.html}
/*
\begin{comment}
rather stupid,
but helpful
\end{comment}

\textcolor[rgb]{0,1,0}{This text will appear green-colored}

\setmainfont{Georgia}
\setsansfont{Arial}

*/
const PAPERS_PRELUDE : &'static str = r"
    \usepackage{url}
    \usepackage{comment}
    \usepackage{xcolor}
    \usepackage{fontspec}
    \usepackage{multicols}
    \usepackage{amsmath}
";

const EMPTY_DESCRIPTION : &'static str = r#"
Start a document from scratch
without a predefined class"#;

const MINIMAL_DESCRIPTION : &'static str = r#"
Minimal document. Useful for notes, drafts
or any kind of document which do not
require sectioning or metadata."#;

const ARTICLE_DESCRIPTION : &'static str = r#"
Short document divided into sections and
subsections. Aimed at journal articles."#;

const REPORT_DESCRIPTION : &'static str = r#"
Longer document divided into chapters.
Usually aimed at technical reports or
academic documents such as dissertations
and thesis."#;

const BOOK_DESCRIPTION : &'static str = r#"
Long document divided into chapters.
Structured into front matter, main matter
and back matter."#;

const PRESENTATION_DESCRIPTION : &'static str = r#"A slide presentation."#;

impl StartScreen {

    pub fn build() -> Self {
        let doc_upper_bx = Box::new(Orientation::Horizontal, 0);
        let doc_lower_bx = Box::new(Orientation::Horizontal, 0);
        let empty_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/empty.svg", "Empty", EMPTY_DESCRIPTION);
        let minimal_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/minimal.svg", "Minimal", MINIMAL_DESCRIPTION);
        let article_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/article.svg", "Article", ARTICLE_DESCRIPTION);
        let report_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/report.svg", "Report", REPORT_DESCRIPTION);
        let book_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/book.svg", "Book", BOOK_DESCRIPTION);
        let present_btn = DocBtn::build("/home/diego/Software/papers/assets/pictures/presentation.svg", "Presentation", PRESENTATION_DESCRIPTION);
        // let report_btn = Button::builder().label("Report").build();
        // let presentation_btn = Button::builder().label("Presentation").build();
        // letter
        // book

        let center_bx = Box::new(Orientation::Vertical, 32);
        doc_upper_bx.append(&empty_btn.btn);
        doc_upper_bx.append(&minimal_btn.btn);
        doc_upper_bx.append(&article_btn.btn);
        doc_lower_bx.append(&report_btn.btn);
        doc_lower_bx.append(&book_btn.btn);
        doc_lower_bx.append(&present_btn.btn);

        let bx = Box::new(Orientation::Vertical, 0);
        let title = title_label("New document");
        center_bx.append(&title);
        center_bx.append(&doc_upper_bx);
        center_bx.append(&doc_lower_bx);
        //set_margins(&center_bx, 128, 0);
        bx.append(&center_bx);
        center_bx.set_vexpand(true);
        center_bx.set_valign(Align::Center);
        center_bx.set_hexpand(true);
        center_bx.set_halign(Align::Center);

        // bx.append(&report_btn);
        Self { bx, empty_btn : empty_btn.btn.clone(), article_btn : article_btn.btn.clone(), /*report_btn, presentation_btn*/ }
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

    pub fn new(symbols : &'static [(&'static str, &'static str)], view : View, popover : Popover, ncol : usize) -> Self {
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
                        let popover = popover.clone();
                        move|_| {
                            popover.popdown();
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

#[derive(Debug, Clone)]
pub struct SymbolPopover {
    pub popover : Popover
}

impl SymbolPopover {

    pub fn build(editor : &PapersEditor) -> Self {
        let popover = Popover::new();
        let greek_small_grid = SymbolGrid::new(&GREEK_SMALL[..], editor.view.clone(), popover.clone(), 12);
        let greek_capital_grid = SymbolGrid::new(&GREEK_CAPITAL[..], editor.view.clone(), popover.clone(), 12);
        let operators_grid = SymbolGrid::new(&OPERATORS[..], editor.view.clone(), popover.clone(), 12);
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
        popover.set_child(Some(&symbol_bx));
        Self { popover }
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

        window.add_action(&titlebar.main_menu.actions.new);
        window.add_action(&titlebar.main_menu.actions.open);
        window.add_action(&titlebar.main_menu.actions.save);
        window.add_action(&titlebar.main_menu.actions.save_as);

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

        let symbol_popover = SymbolPopover::build(&editor);
        titlebar.symbol_btn.set_popover(Some(&symbol_popover.popover));

        // symbol_dialog.set_child(Some(&symbol_bx));
        titlebar.math_actions.symbol.connect_activate(move |_, _| {
            symbol_dialog.show();
        });

        for action in titlebar.math_actions.iter().chain(titlebar.object_actions.iter().chain(titlebar.struct_actions.iter())) {
            window.add_action(&action);
        }

        Self { window, titlebar, editor, doc_tree, stack, start_screen }
    }

}

pub fn connect_manager_with_window(
    manager : &FileManager,
    window : &ApplicationWindow,
    actions : &FileActions,
    extension : &'static str
) {
    let win = window.clone();
    manager.connect_window_close(move |_| {
        win.destroy();
    });
    manager.connect_opened({
        let action_save = actions.save.clone();
        let action_save_as = actions.save_as.clone();
        let window = window.clone();
        move |(path, _)| {
            action_save.set_enabled(true);
            action_save_as.set_enabled(true);
            window.set_title(Some(&path));
        }
    });
    manager.connect_open_request({
        let open_action = actions.open.clone();
        move |_| {
            open_action.activate(None);
        }
    });
    manager.connect_save({
        let window = window.clone();
        move |path| {
            window.set_title(Some(&path));
        }
    });
    manager.connect_file_changed({
        let window = window.clone();
        move |opt_path| {
            if let Some(path) = opt_path {
                window.set_title(Some(&format!("{}*", path)));
            } else {
                window.set_title(Some(&format!("Untitled.{}*", extension)));
            }
        }
    });
}


impl React<FileManager> for PapersWindow {

    fn react(&self, manager : &FileManager) {
        connect_manager_with_window(manager, &self.window, &self.titlebar.main_menu.actions, "tex");
        manager.connect_new({
            let window = self.window.clone();
            let action_save = self.titlebar.main_menu.actions.save.clone();
            let action_save_as = self.titlebar.main_menu.actions.save_as.clone();
            move |_| {
                window.set_title(Some("Papers"));
                action_save.set_enabled(false);
                action_save_as.set_enabled(false);
            }
        });
        manager.connect_opened({
            let stack = self.stack.clone();
            move |(path, _)| {
                stack.set_visible_child_name("editor");
            }
        });
        manager.connect_new({
            let stack = self.stack.clone();
            move |_| {
                stack.set_visible_child_name("start");
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
        menu.actions.save_as.connect_activate(move |_,_| {
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

pub fn set_border_to_title(bx : &Box) {
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

pub fn setup_position_as_ratio(win : &ApplicationWindow, paned : &Paned, ratio : f32) {
    let ratio = start_position_as_ratio(win, paned, ratio);
    preserve_ratio_on_resize(win, paned, &ratio);
}

fn start_position_as_ratio(win : &ApplicationWindow, paned : &Paned, ratio : f32) -> Rc<RefCell<f32>> {
    let paned = paned.clone();
    win.connect_show(move |win| {
        set_position_as_ratio(win, &paned, ratio);
    });
    Rc::new(RefCell::new(ratio))
}

fn set_position_as_ratio(win : &ApplicationWindow, paned : &Paned, ratio : f32) {
    let (mut w, mut h) = (win.allocation().width, win.allocation().height);

    // Allocation will be zero at the first time window is shown.
    if w == 0 {
        w = win.default_width();
    }
    if h == 0 {
        h = win.default_height();
    }
    let dim = match paned.orientation() {
        Orientation::Horizontal => w as f32,
        Orientation::Vertical => h as f32,
        _ => { return; }
    };
    println!("({:?})", dim);
    paned.set_position((dim * ratio) as i32);
}

fn preserve_ratio_on_resize(win : &ApplicationWindow, paned : &Paned, ratio : &Rc<RefCell<f32>>) {
    win.connect_default_width_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        println!("Size changed");
        move |win| {
            let alloc = win.allocation();
            set_position_as_ratio(&win,&paned, *ratio.borrow());
        }
    });
    win.connect_default_height_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        println!("Size changed");
        move |win| {
            let alloc = win.allocation();
            set_position_as_ratio(&win, &paned, *ratio.borrow());
        }
    });
    let ratio = ratio.clone();
    let win = win.clone();
    paned.connect_accept_position(move |paned| {
        let dim = match paned.orientation() {
            Orientation::Horizontal => win.allocation().width as f32,
            Orientation::Vertical => win.allocation().height as f32,
            _ => { return true; }
        };
        let new_ratio = paned.position() as f32 / dim;
        *(ratio.borrow_mut()) = new_ratio;
        true
    });
}

const A4 : (f64, f64) = (210.0, 297.4);

const LEGAL : (f64, f64) = (215.9, 355.6);

const LETTER : (f64, f64) = (215.9, 279.4);

const PX_PER_MM : f64 = 3.0;

const DEFAULT_SCALE : f64 = 1.5;

const SCALE_INCREMENT : f64 = 0.5;

impl React<Typesetter> for PapersWindow {

    fn react(&self, typesetter : &Typesetter) {
        let win = self.window.clone();
        typesetter.connect_done(move |target| {
            match target {
                TypesetterTarget::File(path) => {

                    #[cfg(feature="poppler")]
                    {
                        show_with_poppler(&win, &path[..]);
                    }

                    show_with_evince(&path);
                },
                _ => {

                }
            }
        });
    }

}

fn show_with_evince(path : &str) {

    use std::process::Command;

    let out = Command::new("evince")
        .args(&[&path])
        .spawn()
        .unwrap();
}

#[cfg(feature="poppler")]
fn show_with_poppler(win : &ApplicationWindow, path : &str) {

    let doc = poppler::Document::from_file(&path, None).unwrap();

    let dialog = Dialog::new();
    dialog.set_default_width(1024);
    dialog.set_default_height(768);
    dialog.set_transient_for(Some(&win));

    let scroll = ScrolledWindow::new();
    let bx = Box::new(Orientation::Vertical, 12);
    scroll.set_child(Some(&bx));
    dialog.set_child(Some(&scroll));
    set_margins(&bx, 32, 32);

    let header = HeaderBar::new();
    let zoom_bx = Box::new(Orientation::Horizontal, 0);
    let zoom_in_btn = Button::new();
    let zoom_out_btn = Button::new();
    zoom_out_btn.set_sensitive(false);
    zoom_in_btn.set_icon_name("zoom-in-symbolic");
    zoom_out_btn.set_icon_name("zoom-out-symbolic");
    zoom_bx.style_context().add_class("linked");
    zoom_bx.append(&zoom_in_btn);
    zoom_bx.append(&zoom_out_btn);
    header.pack_start(&zoom_bx);
    dialog.set_titlebar(Some(&header));
    dialog.set_title(Some(&Path::new(&path).file_name().unwrap().to_str().unwrap()));

    let mut zoom = Rc::new(RefCell::new(DEFAULT_SCALE));
    let mut das : Rc<RefCell<Vec<DrawingArea>>> = Rc::new(RefCell::new(Vec::new()));
    zoom_in_btn.connect_clicked({
        let zoom = zoom.clone();
        let das = das.clone();
        let zoom_out_btn = zoom_out_btn.clone();
        let bx = bx.clone();
        move |btn| {
            let mut z = zoom.borrow_mut();
            if *z <= 5.0 {
                *z += SCALE_INCREMENT;
                if *z == 5.0 {
                    btn.set_sensitive(false);
                }
                if *z > 1.0 {
                    zoom_out_btn.set_sensitive(true);
                }
            } else {
                btn.set_sensitive(false);
            }
            das.borrow().iter().for_each(|da| da.queue_draw() );
        }
    });
    zoom_out_btn.connect_clicked({
        let zoom = zoom.clone();
        let das = das.clone();
        let zoom_in_btn = zoom_in_btn.clone();
        let bx = bx.clone();
        move |btn| {
            let mut z = zoom.borrow_mut();
            if *z > 1.0 {
                *z -= DEFAULT_SCALE;
                if *z == 1.0 {
                    btn.set_sensitive(false);
                }
                if *z > 1.0 {
                    btn.set_sensitive(true);
                }
                if *z < 5.0 {
                    zoom_in_btn.set_sensitive(true);
                }
            } else {
                btn.set_sensitive(false);
            }
            das.borrow().iter().for_each(|da| da.queue_draw() );
        }
    });

    for page_ix in 0..doc.n_pages() {

        let da = DrawingArea::new();

        let zoom = zoom.clone();
        let page = doc.page(page_ix).unwrap();
        da.set_vexpand(false);
        da.set_hexpand(false);
        da.set_halign(Align::Center);
        da.set_valign(Align::Center);
        //da.set_width_request((A4.0 * PX_PER_MM) as i32);
        //da.set_height_request((A4.1 * PX_PER_MM) as i32);

        da.set_draw_func(move |da, ctx, _, _| {
            ctx.save();

            let z = *zoom.borrow();
            let (w, h) = page.size();
            da.set_width_request((w * z) as i32);
            da.set_height_request((h * z) as i32);

            let (w, h) = (da.allocation().width as f64, da.allocation().height as f64);

            // Draw white background of page
            ctx.set_source_rgb(1., 1., 1.);
            ctx.rectangle(1., 1., w, h);

            // Draw page borders
            let color = 0.5843;
            ctx.set_line_width(0.5);
            let grad = cairo::LinearGradient::new(0.0, 0.0, w, h);
            grad.add_color_stop_rgba(0.0, color, color, color, 0.5);
            grad.add_color_stop_rgba(0.5, color, color, color, 1.0);
            ctx.move_to(1., 1.);
            ctx.line_to(w - 1., 1.);
            ctx.line_to(w - 1., h);
            ctx.line_to(1., h - 1.);
            ctx.line_to(1., 1.);

            // Linear gradient derefs into pattern.
            ctx.set_source(&*grad);

            ctx.stroke();

            // Poppler always render with the same dpi from the physical page resolution. We must
            // apply a scale to the context if we want the content to be scaled.
            ctx.scale(z, z);

            // TODO remove the transmute when GTK/cairo version match.
            page.render(unsafe { std::mem::transmute::<_, _>(ctx) });

            ctx.restore();
        });
        bx.append(&da);
        das.borrow_mut().push(da);
    }
    dialog.show();
}

pub fn title_label(txt : &str) -> Label {
    let lbl = Label::builder()
        .label(&format!("<span font_weight=\"600\" font_size=\"large\" fgalpha=\"60%\">{}</span>", txt))
        .use_markup(true)
        .justify(Justification::Left)
        .halign(Align::Start)
        .build();
    set_margins(&lbl, 0, 12);
    lbl
}

