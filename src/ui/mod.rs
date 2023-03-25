use gtk4::*;
use gtk4::prelude::*;
use stateful::React;
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
use filecase::SingleArchiverImpl;
use filecase::{OpenDialog, SaveDialog};
use poppler::Document;

// TODO replace existing file is not working when saving it.
// (No current document to export)

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
    pub start_screen : StartScreen,
    pub export_pdf_dialog : SaveDialog,
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

fn start_document(view : &View, stack : &Stack, titlebar : &Titlebar, template : &str) {
    view.buffer().set_text(template);
    stack.set_visible_child_name("editor");
    titlebar.main_menu.actions.save.set_enabled(true);
    titlebar.main_menu.actions.save_as.set_enabled(true);
    titlebar.view_pdf_btn.set_active(false);
    titlebar.view_pdf_btn.set_sensitive(true);
}

impl React<StartScreen> for PapersWindow {

    fn react(&self, start_screen : &StartScreen) {

        start_screen.empty_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let titlebar = self.titlebar.clone();
            move |_| {
                start_document(&view, &stack, &titlebar, "");
            }
        });
        start_screen.article_btn.connect_clicked({
            let (view, stack)  = (self.editor.view.clone(), self.stack.clone());
            let titlebar = self.titlebar.clone();
            move |_| {
                start_document(&view, &stack, &titlebar, ARTICLE_TEMPLATE);
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

pub struct DocBtn {
    pub btn : Button
}

impl DocBtn {

    pub fn build(image : &str, title : &str, sub : &str) -> Self {
        let btn = Button::new();
        //let img = Picture::for_filename(image);
        let img = Picture::for_resource(&format!("io/github/limads/drafts/icons/scalable/actions/{}.svg", image));
        img.set_can_shrink(false);
        let lbl_bx = Box::new(Orientation::Vertical, 12);
        let lbl = Label::new(Some(title));
        lbl.set_justify(Justification::Left);
        lbl.set_halign(Align::Start);
        let sub_lbl = Label::builder().use_markup(true).label(&format!("<span font_weight='normal'>{}</span>", sub)).build();
        sub_lbl.set_halign(Align::Start);
        sub_lbl.set_justify(Justification::Fill);
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
Start a document without a template"#;

const MINIMAL_DESCRIPTION : &'static str = r#"
Useful for notes, drafts and other generic text"#;

const ARTICLE_DESCRIPTION : &'static str = r#"
Short document divided into sections and
subsections. Aimed at journal articles."#;

const REPORT_DESCRIPTION : &'static str = r#"
Longer document aimed at technical reports,
dissertations and thesis."#;

const BOOK_DESCRIPTION : &'static str = r#"
Long document divided into chapters
and with front, main and back matter."#;

const PRESENTATION_DESCRIPTION : &'static str = r#"
A document focusing on visual communication.
Separated into slides."#;

#[derive(Debug, Clone)]
pub struct StartScreen {
    bx : Box,
    empty_btn : Button,
    article_btn : Button,
    pub recent_list : RecentList
    //report_btn : Button,
    //presentation_btn : Button
}

impl StartScreen {

    pub fn build() -> Self {
        let doc_upper_bx = Box::new(Orientation::Horizontal, 0);
        let doc_middle_bx = Box::new(Orientation::Horizontal, 0);
        let doc_lower_bx = Box::new(Orientation::Horizontal, 0);
        let empty_btn = DocBtn::build("empty", "Empty", EMPTY_DESCRIPTION);
        let minimal_btn = DocBtn::build("minimal", "Minimal", MINIMAL_DESCRIPTION);
        let article_btn = DocBtn::build("article", "Article", ARTICLE_DESCRIPTION);
        let report_btn = DocBtn::build("report", "Report", REPORT_DESCRIPTION);
        let book_btn = DocBtn::build("book", "Book", BOOK_DESCRIPTION);
        let present_btn = DocBtn::build("presentation", "Presentation", PRESENTATION_DESCRIPTION);

        // let report_btn = Button::builder().label("Report").build();
        // let presentation_btn = Button::builder().label("Presentation").build();
        // letter
        // book

        let new_bx = Box::new(Orientation::Vertical, 16);
        doc_upper_bx.append(&empty_btn.btn);
        doc_upper_bx.append(&minimal_btn.btn);
        doc_middle_bx.append(&article_btn.btn);
        doc_middle_bx.append(&report_btn.btn);
        doc_lower_bx.append(&book_btn.btn);
        doc_lower_bx.append(&present_btn.btn);

        let bx = Box::new(Orientation::Horizontal, 0);
        let title = title_label("New");
        new_bx.append(&title);
        new_bx.append(&doc_upper_bx);
        new_bx.append(&doc_middle_bx);
        new_bx.append(&doc_lower_bx);
        new_bx.set_margin_end(128);
        //set_margins(&center_bx, 128, 0);

        new_bx.set_vexpand(true);
        new_bx.set_valign(Align::Center);
        new_bx.set_hexpand(true);
        new_bx.set_halign(Align::End);

        // bx.append(&report_btn);

        let recent_list = RecentList::build();
        bx.append(&recent_list.bx);
        bx.append(&new_bx);

        Self { bx, empty_btn : empty_btn.btn.clone(), article_btn : article_btn.btn.clone(), recent_list }
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

impl PapersWindow {

    pub fn from(window : ApplicationWindow) -> Self {

        let titlebar = Titlebar::build();
        window.set_titlebar(Some(&titlebar.header));
        window.set_decorated(true);
        let doc_tree = DocTree::build();
        let editor = PapersEditor::build(&titlebar.zoom_action);
        let start_screen = StartScreen::build();
        start_screen.recent_list.open_btn.connect_clicked({
            let open_action = titlebar.main_menu.actions.open.clone();
            move|_| {
                open_action.activate(None);
            }
        });

        let export_pdf_dialog = filecase::SaveDialog::build("*.pdf");
        export_pdf_dialog.dialog.set_transient_for(Some(&window));

        titlebar.main_menu.export_action.connect_activate({
            let export_pdf_dialog = export_pdf_dialog.clone();
            move |_,_| {
                export_pdf_dialog.dialog.show();
            }
        });

        export_pdf_dialog.dialog.connect_response({
            let doc = editor.pdf_viewer.doc.clone();
            move |dialog, resp| {
                match resp {
                    ResponseType::Accept => {
                        if let Some(path) = dialog.file().and_then(|f| f.path() ) {
                            if let Some(doc) = &*doc.borrow() {
                                if let Err(e) = doc.save(&format!("file://{}", path.to_str().unwrap())) {
                                    println!("Document save error: {}", e);
                                }
                            } else {
                                println!("No current document to export");
                            }
                        } else {
                            println!("No path available");
                        }
                    },
                    _ => { }
                }
            }
        });

        titlebar.main_menu.save_dialog.dialog.set_transient_for(Some(&window));
        titlebar.main_menu.open_dialog.dialog.set_transient_for(Some(&window));
        titlebar.react(&editor.pdf_viewer);

        // Keeps pdf paned hidden due to window changes. Maybe move to impl React<MainWindow> for Editor?
        /*window.connect_default_width_notify({
            let paned = editor.sub_paned.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |win| {
                if !pdf_btn.is_active() || !pdf_btn.is_sensitive() {
                    // paned.set_position(i32::MAX);
                }
            }
        });
        window.connect_default_height_notify({
            let paned = editor.sub_paned.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |win| {
                if !pdf_btn.is_active() || !pdf_btn.is_sensitive() {
                    // paned.set_position(i32::MAX);
                }
            }
        });*/

        /*window.connect_maximized_notify({
            let paned = editor.sub_paned.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |win| {
                if !pdf_btn.is_active() || !pdf_btn.is_sensitive() {
                    // paned.set_position(i32::MAX);
                }
            }
        });*/
        window.connect_fullscreened_notify({
            move |win| {
                println!("Fullscreened changed");
            }
        });

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
        window.add_action(&titlebar.main_menu.export_action);
        window.add_action(&titlebar.typeset_action);

        window.add_action(&titlebar.sidebar_hide_action);
        window.add_action(&titlebar.zoom_action);
        window.add_action(&editor.ignore_file_save_action);

        let stack = Stack::new();
        stack.add_named(&start_screen.bx, Some("start"));
        stack.add_named(&editor.overlay, Some("editor"));

        titlebar.view_pdf_btn.connect_toggled({
            let sub_paned = editor.sub_paned.clone();
            let stack = stack.clone();
            move|btn| {
                if let Some(nm) = stack.visible_child_name() {
                    if btn.is_sensitive() && &nm[..] == "editor" {
                        if btn.is_active() {
                            sub_paned.set_position(sub_paned.allocation().width() / 2);
                        } else {
                            sub_paned.set_position(i32::MAX);
                        }
                    }
                }
            }
        });

        // editor.popover.set_parent(&titlebar.explore_toggle);
        // editor.popover.set_pointing_to(Some(&titlebar.explore_toggle.allocation()));
        titlebar.explore_toggle.set_popover(Some(&editor.popover));

        editor.popover.set_child(Some(&doc_tree.bx));
        editor.popover.set_position(PositionType::Bottom);
        editor.popover.set_width_request(320);
        editor.popover.set_height_request(640);

        // editor.paned.set_start_child(Some(&doc_tree.bx));
        // editor.paned.set_end_child(Some(&stack));
        // editor.paned.set_position(0);

        // window.set_child(Some(&editor.paned));
        window.set_child(Some(&stack));

        let symbol_dialog = Dialog::new();
        symbol_dialog.set_title(Some("Symbols"));
        filecase::configure_dialog(&symbol_dialog);
        symbol_dialog.set_transient_for(Some(&window));

        let symbol_popover = SymbolPopover::build(&editor);
        titlebar.symbol_btn.set_popover(Some(&symbol_popover.popover));

        // symbol_dialog.set_child(Some(&symbol_bx));
        // titlebar.math_actions.symbol.connect_activate(move |_, _| {
        //    symbol_dialog.show();
        // });

        let titlebar_actions = titlebar.object_actions.iter()
            .chain(titlebar.layout_actions.iter())
            .chain(titlebar.sectioning_actions.iter())
            .chain(titlebar.block_actions.iter())
            .chain(titlebar.meta_actions.iter())
            .chain(titlebar.indexing_actions.iter());
        for action in titlebar_actions {
            window.add_action(&action);
        }

        Self { window, titlebar, editor, doc_tree, stack, start_screen, export_pdf_dialog }
    }

}

impl React<FileManager> for PapersWindow {

    fn react(&self, manager : &FileManager) {
        filecase::connect_manager_with_app_window_and_actions(manager, &self.window, &self.titlebar.main_menu.actions, "typ");
        manager.connect_new({
            let stack = self.stack.clone();
            let titlebar = self.titlebar.clone();
            let window = self.window.clone();
            let paned = self.editor.sub_paned.clone();
            let bib_list = self.titlebar.bib_popover.list.clone();
            move |_| {
                window.set_title(Some("Drafts"));
                paned.set_position(i32::MAX);
                stack.set_visible_child_name("start");
                titlebar.set_prepared(false);
                titlebar.clear_pages();
                titlebar::clear_list(&bib_list);
                titlebar::create_init_row(&bib_list);
            }
        });
        manager.connect_opened({
            let stack = self.stack.clone();
            // let paned = self.editor.sub_paned.clone();
            let titlebar = self.titlebar.clone();
            let export_pdf_dialog = self.export_pdf_dialog.clone();
            // let view_pdf_btn = self.titlebar.view_pdf_btn.clone();
            move |(path, _)| {
                stack.set_visible_child_name("editor");
                titlebar.set_prepared(true);
                titlebar.clear_pages();
                init_export_path(&export_pdf_dialog.dialog, path);
            }
        });
        manager.connect_save({
            let export_pdf_dialog = self.export_pdf_dialog.clone();
            move |path| {
                init_export_path(&export_pdf_dialog.dialog, path);
            }
        });
    }

}

fn init_export_path(export_pdf_dialog : &FileChooserDialog, source_path : String) {
    if export_pdf_dialog.file().is_none() {
        if let Some(parent) = Path::new(&source_path).parent() {
            if let Ok(_) = export_pdf_dialog.set_current_folder(Some(&gio::File::for_path(parent.to_str().unwrap()))) {
                if let Some(stem) = Path::new(&source_path).file_stem() {
                    export_pdf_dialog.set_current_name(&format!("{}.pdf", stem.to_str().unwrap()));
                }
            }
        }
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

#[derive(Debug, Clone)]
pub struct PackedImageLabel  {
    pub bx : Box,
    pub img : Image,
    pub lbl : Label
}

impl PackedImageLabel {

    pub fn build(icon_name : &str, label_name : &str) -> Self {
        let bx = Box::new(Orientation::Horizontal, 0);
        let img = Image::from_icon_name(icon_name);
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

pub fn try_get_child_by_index<W>(w : &Box, pos : usize) -> Option<W>
where
    W : IsA<glib::Object>
{
    w.observe_children().item(pos as u32)?.clone().downcast::<W>().ok()
}

pub fn get_child_by_index<W>(w : &Box, pos : usize) -> W
where
    W : IsA<glib::Object>
{
    // w.observe_children().item(pos as u32).unwrap().clone().downcast::<W>().unwrap()
    try_get_child_by_index::<W>(w, pos).unwrap()
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
    let (mut w, mut h) = (win.allocation().width(), win.allocation().height());

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

fn update_paned_from_allocation(win : &ApplicationWindow, paned : &Paned, ratio : &Rc<RefCell<f32>>) {
    let alloc = win.allocation();
    set_position_as_ratio(&win, &paned, *ratio.borrow());
}

fn preserve_ratio_on_resize(win : &ApplicationWindow, paned : &Paned, ratio : &Rc<RefCell<f32>>) {
    win.connect_default_width_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        move |win| {
            update_paned_from_allocation(&win, &paned, &ratio);
        }
    });
    win.connect_default_height_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        move |win| {
            update_paned_from_allocation(&win, &paned, &ratio);
        }
    });
    win.connect_maximized_notify({
        let paned = paned.clone();
        let ratio = ratio.clone();
        move |win| {
            update_paned_from_allocation(&win, &paned, &ratio);
        }
    });
    win.connect_resizable_notify({

        move |win| {
            println!("Resizable");
        }
    });
    let ratio = ratio.clone();
    let win = win.clone();
    /*paned.connect_accept_position(move |paned| {
        let dim = match paned.orientation() {
            Orientation::Horizontal => win.allocation().width() as f32,
            Orientation::Vertical => win.allocation().height() as f32,
            _ => { return true; }
        };
        let new_ratio = paned.position() as f32 / dim;
        *(ratio.borrow_mut()) = new_ratio;
        true
    });*/
}

const A4 : (f64, f64) = (210.0, 297.4);

const LETTER : (f64, f64) = (215.9, 279.4);

const LEGAL : (f64, f64) = (215.9, 355.6);

// const PX_PER_MM : f64 = 3.0;

fn update_titlebar(titlebar : &Titlebar, pdf_viewer : &PdfViewer) {
    // editor.sub_paned.set_position(editor.sub_paned.allocation().width() / 2);
    // editor.sub_paned.set_position(400);

    titlebar.set_typeset_mode(true);

    // If sidebar is open, use minimum zoom at PDF to minimize occlusion of content.

    /*if titlebar.explore_toggle.is_active() {
        while titlebar.zoom_out_btn.is_sensitive() {
            titlebar.zoom_out_btn.emit_clicked();
        }
    }*/

    let doc = pdf_viewer.doc();
    if let Some(doc) = &*doc.borrow() {
        let n = doc.n_pages();
        titlebar.page_button.set_label(&format!("of {}", n));
        titlebar.page_entry.set_text("1");
    }
}

impl React<Typesetter> for PapersWindow {

    fn react(&self, typesetter : &Typesetter) {
        let win = self.window.clone();
        let editor = self.editor.clone();
        let titlebar = self.titlebar.clone();
        typesetter.connect_done(move |target| {
            match target {
                TypesetterTarget::File(path) => {
                    let doc = poppler::Document::from_file(&format!("file://{}", path), None).unwrap();
                    editor.pdf_viewer.update(&doc, &titlebar.zoom_action);
                    println!("Showing with poppler");
                    update_titlebar(&titlebar, &editor.pdf_viewer);
                },
                TypesetterTarget::PDFContent(bytes) => {
                    use std::io::Write;
                    let mut f = std::fs::File::create("/home/diego/Downloads/out.pdf").unwrap();
                    f.write_all(&bytes).unwrap();
                    match poppler::Document::from_data(&bytes[..], None) {
                        Ok(doc) => {
                            editor.pdf_viewer.update(&doc, &titlebar.zoom_action);
                            update_titlebar(&titlebar, &editor.pdf_viewer);
                        },
                        Err(e) => {
                            println!("Poppler error: {}", e);
                        }
                    }
                },
                _ => {
                    println!("Unimplemented typesetting target");
                }
            }
        });

        typesetter.connect_error({
            let titlebar = self.titlebar.clone();
            move |_| {
                titlebar.page_button.set_label("of 0");
                titlebar.page_entry.set_text("0");
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

#[derive(Debug, Clone)]
pub struct PdfViewer {
    scroll : ScrolledWindow,
    pages_bx : Box,
    das : Rc<RefCell<Vec<DrawingArea>>>,
    doc : Rc<RefCell<Option<poppler::Document>>>,
    da1 : DrawingArea,
    da2 : DrawingArea,
    curr_page : Rc<RefCell<usize>>,
    stack : Stack,
    turn_action : gio::SimpleAction
}

impl React<Titlebar> for PdfViewer {
    fn react(&self, titlebar : &Titlebar) {
        titlebar.zoom_action.connect_activate({
            let das = self.das.clone();
            let (da1, da2) = (self.da1.clone(), self.da2.clone());
            move |_,_| {
                das.borrow().iter().for_each(|da| da.queue_draw() );
                da1.queue_draw();
                da2.queue_draw();
            }
        });
        /*titlebar.pdf_btn.connect_toggled({
            let viewer = self.clone();
            move |btn| {
                // if !btn.is_active() {
                //    viewer.clear_pages();
                // }
            }
        });*/

        // Called by event controller (instead of text changed event)
        // because the application changes the text at the entry too frequently without
        // user input (like button/arrow press or swipe).
        // Here we are sure this happened after some user input.
        let ev = EventControllerKey::new();
        titlebar.page_entry.add_controller(&ev);
        ev.connect_key_released({
            let doc = self.doc.clone();
            let da1 = self.da1.clone();
            let da2 = self.da2.clone();
            let stack = self.stack.clone();
            let turn_action = self.turn_action.clone();
            let page_entry = titlebar.page_entry.clone();
            let curr_page = self.curr_page.clone();
            move |_, _, _, _| {
                let txt = page_entry.text();
                if txt.is_empty() {
                    return;
                }

                // The user pages count from 1..n. The internal state count from
                // 0..n-1 (as poppler does).
                if let Ok(new_page) = txt.parse::<i32>() {
                    go_to_page(
                        &doc,
                        &da1,
                        &da2,
                        &curr_page,
                        &turn_action,
                        &stack,
                        new_page
                    );
                }
            }
        });

        titlebar.left_btn.connect_clicked({
            let doc = self.doc.clone();
            let da1 = self.da1.clone();
            let da2 = self.da2.clone();
            let stack = self.stack.clone();
            let turn_action = self.turn_action.clone();
            let page_entry = titlebar.page_entry.clone();
            let curr_page = self.curr_page.clone();
            move |_| {
                turn_page(&stack, &doc, &curr_page, &da1, &da2, &turn_action, true);
            }
        });
        titlebar.right_btn.connect_clicked({
            let doc = self.doc.clone();
            let da1 = self.da1.clone();
            let da2 = self.da2.clone();
            let stack = self.stack.clone();
            let turn_action = self.turn_action.clone();
            let page_entry = titlebar.page_entry.clone();
            let curr_page = self.curr_page.clone();
            move |_| {
                turn_page(&stack, &doc, &curr_page, &da1, &da2, &turn_action, false);
            }
        });

    }
}

fn go_to_page(
    doc : &Rc<RefCell<Option<poppler::Document>>>,
    da1 : &DrawingArea,
    da2 : &DrawingArea,
    curr_page : &Rc<RefCell<usize>>,
    turn_action : &gio::SimpleAction,
    stack : &Stack,
    new_page : i32
) {
    if new_page >= 1 {
        let doc = doc.borrow();
        if new_page <= doc.as_ref().map(|d| d.n_pages() ).unwrap_or(0) {
            let mut curr_page = curr_page.borrow_mut();
            if new_page == *curr_page as i32 {
                return;
            }
            if new_page > *curr_page as i32 {
                stack.set_transition_type(StackTransitionType::SlideLeft);
            } else {
                stack.set_transition_type(StackTransitionType::SlideRight);
            }
            *curr_page = new_page as usize - 1;
            turn_action.set_state(&(new_page - 1).to_variant());
            draw_at_even_or_odd(&stack, &da1, &da2, new_page as usize - 1);
        }
    }
}

// Equivalent to 0xdc
// const PAGE_BORDER_COLOR : f64 = 0.859375;

// Equivalent to 0xcf
pub const PAGE_BORDER_COLOR : f64 = 0.80859375;

pub const PAGE_BORDER_WIDTH : f64 = 0.5;

fn draw_at_even_or_odd(stack : &Stack, da1 : &DrawingArea, da2 : &DrawingArea, curr_page : usize) {
    if curr_page % 2 == 0 {
        stack.set_visible_child_name("left");
        da1.queue_draw();
    } else {
        stack.set_visible_child_name("right");
        da2.queue_draw();
    }
}

fn turn_page(
    stack : &Stack,
    doc : &Rc<RefCell<Option<Document>>>,
    curr_page : &Rc<RefCell<usize>>,
    da1 : &DrawingArea,
    da2 : &DrawingArea,
    turn_action : &gio::SimpleAction,
    left : bool
) {
    // da1.queue_draw();
    // da2.queue_draw();
    let mut cp = curr_page.borrow_mut();
    let n_pages = if let Ok(doc) = doc.try_borrow() {
        doc.as_ref().map(|d| d.n_pages() as usize ).unwrap_or(0)
    } else {
        return;
    };
    if n_pages == 0 {
        return;
    }
    if *cp == 0 && left {
        return;
    }
    if (*cp == n_pages-1 && !left) {
        return;
    }
    println!("curr page = {:?}; left = {:?}", *cp, left);
    if left {
        *cp -= 1;
        stack.set_transition_type(StackTransitionType::SlideRight);
    } else {
        *cp += 1;
        stack.set_transition_type(StackTransitionType::SlideLeft);
    }

    draw_at_even_or_odd(stack, da1, da2, *cp);

    turn_action.set_state(&(*cp as i32).to_variant());
    turn_action.activate(None);

    // da1.queue_draw();
    // da2.queue_draw();
    // stack.queue_draw();
    println!("Draw queued");
}

impl PdfViewer {

    pub fn doc(&self) -> &Rc<RefCell<Option<poppler::Document>>> {
        &self.doc
    }

    pub fn clear_pages(&self) {
        while let Some(child) = self.pages_bx.last_child() {
            self.pages_bx.remove(&child);
        }
        self.doc.replace(None);
        *(self.curr_page.borrow_mut()) = 0;
    }

    pub fn new(zoom_action : &gio::SimpleAction) -> Self {
        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
        let pages_bx = Box::new(Orientation::Vertical, 12);
        // scroll.set_child(Some(&pages_bx));
        let das = Rc::new(RefCell::new(Vec::new()));
        let da1 = DrawingArea::new();
        let da2 = DrawingArea::new();
        let stack = Stack::new();
        let click = GestureClick::new();
        let curr_page = Rc::new(RefCell::new(0));
        let doc = Rc::new(RefCell::new(None));

        click.connect_pressed({
            let stack = stack.clone();
            move|_, _, _, _| {
                stack.grab_focus();
                println!("Click");
            }
        });
        stack.add_controller(&click);
        let controller = EventControllerKey::new();
        stack.add_controller(&controller);
        controller.connect_key_pressed(|ev, key, code, modifier| {
            println!("key press {:?} {:?}", key, code);
            glib::signal::Inhibit(false)
        });
        let scroll_ev = EventControllerScroll::new(EventControllerScrollFlags::HORIZONTAL);
        let turn_action = gio::SimpleAction::new_stateful("sidebar_hide", None, &(0i32).to_variant());
        scroll_ev.connect_scroll({
            let stack = stack.clone();
            let sw = scroll.clone();
            let doc = doc.clone();
            let curr_page = curr_page.clone();
            let (da1, da2) = (da1.clone(), da2.clone());
            let turn_action = turn_action.clone();
            move|ev, a, b| {
                // println!("Scroll {:?}: {} {}", ev, a, b);
                // trajx.borrow_mut().push(a);

                // Automatically handled at edge_overshoot in this case. When we have
                // a horizontal bar, we should not move the page!
                let has_hbar = sw.allocation().width() != stack.allocation().width();
                if has_hbar {
                    return glib::signal::Inhibit(false);
                }

                if a < 0.0 {
                    turn_page(&stack, &doc, &curr_page, &da1, &da2, &turn_action, true);
                } else if a > 0.0 {
                    turn_page(&stack, &doc, &curr_page, &da1, &da2, &turn_action, false);
                }

                glib::signal::Inhibit(false)
            }
        });
        /*scroll.connect_edge_reached({
            let stack = stack.clone();
            move |s, pos| {
                println!("reached");
            }
        });
        scroll.connect_edge_overshot({
            let stack = stack.clone();
            move|s, pos| {
                println!("overshoot");
                /*match pos {
                    Position::Left => { turn_page(&stack, true); },
                    Position::Right => { turn_page(&stack, false); },
                    _ => { }
                }*/
            }
        });*/
        scroll.add_controller(&scroll_ev);

        // When passing a page, return zoom to best window fit, so the user does not
        // need to worry about moving to the edge of the screen again before moving
        // to the next page.

        // TODO only pass page at the second overshoot (never at the first).

        /*scroll.connect_scroll_end({
            // let trajx = trajx.clone();
            let curr_page = self.curr_page.clone();
            let doc = self.doc.clone();
            let stack = stack.clone();
            let sw = self.scroll.clone();
            move |ev| {

                println!("End {:?}", ev);
                let has_hbar = sw.allocation().width != stack.allocation().width;

                // Automatically handled at edge_overshoot
                if has_hbar {
                    return;
                }

                // let s = sw.hscrollbar().unwrap().downcast_ref::<Scrollbar>().unwrap();
                println!("Scroll end: {}", has_hbar );

                let mut trajx = trajx.borrow_mut();
                if let (Some(fst), Some(lst)) = (trajx.first(), trajx.last()) {
                    let dx = *lst - *fst;
                    let tl = trajx.len();
                    println!("traj len: {tl}, scroll: {dx}");

                    // Move to next page
                    if dx < 0.0 {
                        stack.set_transition_type(StackTransitionType::SlideLeft);
                        stack.set_visible_child_name("right");
                    } else {
                        // Move to prev page
                        stack.set_transition_type(StackTransitionType::SlideRight);
                        stack.set_visible_child_name("left");
                    }
                }
                trajx.clear();
            }
        });*/

        stack.add_named(&da1, Some("left"));
        stack.add_named(&da2, Some("right"));
        for (da_pos, da) in [(0, &da1), (1, &da2)] {
            da.set_draw_func({
                let zoom_action = zoom_action.clone();
                let doc = doc.clone();
                let curr_page = curr_page.clone();
                move |da, ctx, _, _| {
                    // println!("Drawing {}", da_pos);
                    let cp = curr_page.borrow();
                    //if *cp % 2 == da_pos {
                    let doc = doc.borrow();
                    if let Some(doc) = &*doc {
                        if let Some(page) = doc.page(*cp as i32) {
                            crate::adjust_dimension_for_page(da, zoom_action.clone(), &page);
                            crate::draw_page_content(da, ctx, &zoom_action.clone(), &page, true);
                            // println!("Just drawed {}", *cp);
                        } else {
                            println!("No page {} at draw", *cp);
                        }
                    } else {
                        println!("No doc at draw");
                    }
                    //}
                }
            });
        }

        crate::configure_da_for_doc(&da1);
        crate::configure_da_for_doc(&da2);
        scroll.set_child(Some(&stack));

        Self { scroll, das, pages_bx, doc, da1, da2, curr_page, stack, turn_action }
    }

    pub fn update_contiguous(&self, doc : &poppler::Document, zoom_action : &gio::SimpleAction) {
        self.turn_action.set_state(&(0i32).to_variant());
        self.turn_action.activate(None);
        {
            self.das.borrow_mut().clear();
        }
        self.clear_pages();
        for page_ix in 0..doc.n_pages() {
            let da = DrawingArea::new();
            // let zoom = zoom.clone();
            crate::draw_page_at_area(doc, page_ix, &da, zoom_action);
            self.pages_bx.append(&da);
            self.das.borrow_mut().push(da);
        }
        self.doc.replace(Some(doc.clone()));
    }

    pub fn update(&self, doc : &poppler::Document, zoom_action : &gio::SimpleAction) {
        self.turn_action.set_state(&(0i32).to_variant());
        self.turn_action.activate(None);
        // crate::draw_page_at_area(doc, 0, &self.da1, zoom_action);
        // if doc.n_pages() > 1 {
        //    crate::draw_page_at_area(doc, 1, &self.da2, zoom_action);
        // }
        self.doc.replace(Some(doc.clone()));
        {
            *(self.curr_page.borrow_mut()) = 0;
        }
        self.da1.queue_draw();
        self.da2.queue_draw();
        self.stack.set_visible_child_name("left");
    }

}

/*// #[cfg(feature="poppler")]
fn show_with_poppler(viewer : &PdfViewer, zoom_action : &gio::SimpleAction, win : &ApplicationWindow, path : &str) {

    // let dialog = Dialog::new();
    // dialog.set_default_width(1024);
    // dialog.set_default_height(768);
    // dialog.set_transient_for(Some(win));
    // let header = HeaderBar::new();
    // header.pack_start(&viewer.zoom_bx);
    // dialog.set_titlebar(Some(&header));
    // dialog.set_title(Some(&Path::new(&path).file_name().unwrap().to_str().unwrap()));
    // dialog.set_child(Some(&viewer.scroll));
    // set_margins(&bx, 32, 32);
    // dialog.show();
}*/

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

pub fn connect_toast_dismissed(t : &libadwaita::Toast, last : &Rc<RefCell<Option<libadwaita::Toast>>>) {
    let last = last.clone();
    t.connect_dismissed(move|_| {
        if let Ok(mut last) = last.try_borrow_mut() {
            *last = None;
        }
    });
}

pub const MARGIN_MAX : f64 = 5.0;

pub const MARGIN_MIN : f64 = 0.0;

pub struct PaperMargins {
    pub left : f64,
    pub top : f64,
    pub right : f64,
    pub bottom : f64
}

pub fn parse_int_or_float(txt : &str) -> Option<f64> {
    if let Ok(val) = txt.parse::<f64>() {
        Some(val)
    } else {
        if let Ok(val) = txt.parse::<i64>() {
            Some(val as f64)
        } else {
            None
        }
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
pub struct RecentList {
    pub open_btn : Button,
    pub list : ListBox,
    pub bx : Box
}

impl RecentList {

    pub fn build() -> Self {
        let open_btn = Button::from_icon_name("document-open-symbolic");
        open_btn.style_context().add_class("flat");
        let list = ListBox::new();
        set_margins(&list, 1, 1);
        list.style_context().add_class("boxed-list");
        let scroll = ScrolledWindow::new();
        scroll.set_child(Some(&list));
        scroll.set_width_request(560);
        scroll.set_height_request(442);
        scroll.set_has_frame(false);
        list.set_activate_on_single_click(true);
        list.set_show_separators(true);
        let bx = Box::new(Orientation::Vertical, 16);
        bx.set_halign(Align::Center);
        let title = title_label("Recent");
        let title_bx = Box::new(Orientation::Horizontal, 0);
        title_bx.append(&title);
        title_bx.append(&open_btn);
        title_bx.set_hexpand(true);
        title.set_halign(Align::Start);
        open_btn.set_halign(Align::End);
        open_btn.set_hexpand(true);
        bx.append(&title_bx);
        bx.append(&scroll);
        bx.set_valign(Align::Center);
        bx.set_hexpand(true);
        bx.set_margin_start(128);
        Self { open_btn, list, bx }
    }

    pub fn add_row(&self, path : &str, begin : bool) {
        let row = ListBoxRow::new();
        row.set_height_request(64);
        let lbl = PackedImageLabel::build("emblem-documents-symbolic", path);
        row.set_child(Some(&lbl.bx));
        row.set_selectable(false);
        row.set_activatable(true);
        if begin {
            self.list.prepend(&row);
        } else {
            self.list.append(&row);
        }
    }

}

impl React<FileManager> for RecentList {

    fn react(&self, manager : &FileManager) {
        let recent = self.clone();
        manager.connect_opened(move |(path, _)| {
            recent.add_row(&path, true);
        });
        let recent = self.clone();
        manager.connect_save(move |path| {
            recent.add_row(&path, true);
        });
    }
}

