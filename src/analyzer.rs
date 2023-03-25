use gtk4::*;
use gtk4::prelude::*;
use crate::ui::*;
use crate::tex::*;
use stateful::React;
use std::ops::Range;
use std::boxed;
use crate::Callbacks;
use std::sync::mpsc;
use std::fs::File;
use crate::manager::FileManager;
use filecase::SingleArchiverImpl;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub enum AnalyzerAction {

    TextChanged(String),

    TextInit(String),

    ChangeBaseDir(Option<String>),

    BibChanged(String),

    BibError(String),

    // Item selected from the left sidebar. Calculate char position from byte offset at current
    // document model. Then calculate line from char offset. Propagate line to editor, so the
    // mark can be positioned there.
    ItemSelected(Vec<usize>)

}

pub struct Analyzer {

    send : glib::Sender<AnalyzerAction>,

    on_reference_changed : Callbacks<Difference>,

    on_section_changed : Callbacks<Difference>,

    on_doc_changed : Callbacks<Document>,

    on_doc_cleared : Callbacks<()>,

    on_refs_cleared : Callbacks<()>,

    on_refs_validated : Callbacks<()>,

    on_doc_error : Callbacks<TexError>,

    on_ref_file_changed : Callbacks<String>,

    on_line_selection : Callbacks<usize>

}

#[derive(Debug, Clone)]
pub struct BibFile {
    filename : Option<String>,
    base_dir : Option<String>
}

impl Analyzer {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<AnalyzerAction>(glib::PRIORITY_DEFAULT);
        let on_reference_changed : Callbacks<Difference> = Default::default();
        let on_section_changed : Callbacks<Difference> = Default::default();
        let on_doc_changed : Callbacks<Document> = Default::default();
        let on_line_selection : Callbacks<usize> = Default::default();
        let on_doc_error : Callbacks<TexError> = Default::default();
        let on_doc_cleared : Callbacks<()> = Default::default();
        let on_refs_cleared : Callbacks<()> = Default::default();
        let on_refs_validated : Callbacks<()> = Default::default();
        let on_ref_file_changed : Callbacks<String> = Default::default();
        // TODO keep an thread watching an external bib file (if any). The user can simply use
        // the embedded bibliography instead.

        let mut ix = 0;
        recv.attach(None, {
            let mut tk_info = TokenInfo::default();
            let mut doc = Document::default();
            let mut last_err : Option<TexError> = None;
            let on_reference_changed = on_reference_changed.clone();
            let on_section_changed = on_section_changed.clone();
            let on_doc_changed = on_doc_changed.clone();
            let on_line_selection = on_line_selection.clone();
            let on_doc_cleared = on_doc_cleared.clone();
            let on_doc_error = on_doc_error.clone();
            let on_refs_cleared = on_refs_cleared.clone();
            let on_refs_validated = on_refs_validated.clone();
            let on_ref_file_changed = on_ref_file_changed.clone();

            let mut bib_file : Option<BibFile> = None;
            let (bib_send, bib_recv) = mpsc::channel::<Option<BibFile>>();
            std::thread::spawn({
                let send = send.clone();
                move || {
                    loop {
                        match bib_recv.recv() {
                            Ok(Some(bib)) => {
                                // println!("received {:?}", bib);
                                if let (Some(fname), Some(base_path)) = (bib.filename, bib.base_dir) {

                                    // let path = format!("{}/{}.bib", base_path, fname);
                                    let path = format!("{}/{}", base_path, fname);

                                    if Path::new(&path).exists() {
                                        if let Ok(mut f) = File::open(&path) {
                                            let mut content = String::new();
                                            if let Ok(_) = f.read_to_string(&mut content) {
                                                println!("Bib file read");
                                                send.send(AnalyzerAction::BibChanged(content));
                                            } else {
                                                println!("could not read file");
                                            }
                                        } else {
                                            println!("could not open file");
                                        }
                                    } else {
                                        send.send(AnalyzerAction::BibError(format!("Path {} does not exist", path)));
                                    }
                                } else {
                                    println!("Bibliography: No filename or base path available");
                                }
                            },
                            Ok(None) => { },
                            Err(_) => {
                                return;
                            }
                        }
                    }
                }
            });

            move |action| {

                // println!("{}: {:?}", ix, action);
                // ix += 1;

                match action {
                    AnalyzerAction::ChangeBaseDir(opt_path) => {
                        if let Some(path) = opt_path {
                            if let Some(parent) = Path::new(&path).parent() {
                                let parent_path = parent.to_str().unwrap().to_string();
                                if let Some(bib_file) = bib_file.as_mut() {
                                    bib_file.base_dir = Some(parent_path);
                                } else {
                                    bib_file = Some(BibFile { filename : None, base_dir : Some(parent_path) });
                                }
                            } else {
                                log::warn!("File without valid parent path");
                            }
                        }
                        bib_send.send(bib_file.clone());
                    },

                    // TextChanged is not triggered when text is
                    // first added to sourceview because signal is blocked.
                    // Must know text changes exactly when text is loaded.
                    AnalyzerAction::TextInit(new_txt) | AnalyzerAction::TextChanged(new_txt) => {

                        match crate::typst_tools::parse_doc(Path::new(""), new_txt) {
                            Ok(new_doc) => {
                                println!("Doc parsed");
                                if doc != new_doc || last_err.is_some() {
                                    on_doc_changed.call(new_doc.clone());
                                }
                                last_err = None;
                                doc = new_doc;
                                for obj in doc.objects() {
                                    match obj {
                                        Object::Bibliography(_, new_fname) => {
                                            if let Some(bib_file) = bib_file.as_mut() {
                                                if let Some(old_fname) = &bib_file.filename {
                                                    if &old_fname[..] != &new_fname[..] {
                                                        bib_file.filename = Some(new_fname.to_string());
                                                        bib_send.send(Some(bib_file.clone()));
                                                    }
                                                } else {
                                                    bib_file.filename = Some(new_fname.to_string());
                                                    bib_send.send(Some(bib_file.clone()));
                                                }
                                            } else {
                                                bib_file = Some(BibFile {
                                                    filename : Some(new_fname.to_string()),
                                                    base_dir : None
                                                });
                                                bib_send.send(bib_file.clone());
                                            }
                                            break;
                                        },
                                        _ => { }
                                    }
                                }
                            },
                            Err(errs) => {
                                println!("Doc parsing error");
                                let fst_err = errs.get(0).map(|e| e.1.clone() )
                                    .unwrap_or(String::from("Unknown error"));
                                println!("{}", fst_err);
                                doc = Document::default();
                                on_doc_cleared.call(());
                                on_doc_error.call(TexError { msg : fst_err, line : 0 });
                            }
                        }

                        /*println!("Text init/changed");
                        match Lexer::scan(&new_txt[..]).map(|tks| tks.to_owned() ) {
                            Ok(new_info) => {
                                // Update external file references
                                // Tectonic always require that \bibliographystyle{plain} (or other desired style)
                                // is always present at the document for references to be processed correctly.
                                for tk in new_info.tokens() {
                                    match tk {
                                        Token::Command(Command { cmd : "bibliography", arg : Some(CommandArg::Text(f)), .. }, _) => {
                                            println!("Bib cmd found");
                                            if let Some(bib_file) = bib_file.as_mut() {
                                                bib_file.filename = Some(f.to_string());
                                                bib_send.send(Some(bib_file.clone()));
                                            } else {
                                                bib_file = Some(BibFile {
                                                    filename : Some(f.to_string()),
                                                    base_dir : None
                                                });
                                                bib_send.send(bib_file.clone());
                                            }
                                        },
                                        _ => { }
                                    }
                                }

                                tk_info = new_info;
                                match Parser::from_tokens(tk_info.tokens()) {
                                    Ok(new_doc) => {

                                        // Always update after an error or if the token sequence changed.
                                        // If the token sequence remains the same, there is no update to
                                        // be processed.
                                        if doc != new_doc || last_err.is_some() {
                                            on_doc_changed.call(new_doc.clone());
                                        }

                                        last_err = None;
                                        doc = new_doc;
                                    }

                                    Err(e) => {
                                        last_err = Some(e.clone());
                                        println!("{}", e);
                                        doc = Document::default();
                                        on_doc_cleared.call(());
                                        on_doc_error.call(e.clone());
                                    }
                                }
                            },
                            Err(e) => {
                                last_err = Some(e.clone());
                                tk_info = TokenInfo::default();
                                doc = Document::default();
                                on_doc_cleared.call(());
                                on_doc_error.call(e.clone());
                                println!("{}", e);
                            }
                        }*/
                    },
                    AnalyzerAction::BibChanged(txt) => {
                        match BibParser::parse(&txt[..]) {
                            Ok(refs) =>  {
                                on_refs_cleared.call(());
                                let n = refs.as_ref().len();
                                println!("Have {} refs", n);
                                for (ix, r) in refs.as_ref().iter().enumerate() {
                                    on_reference_changed.call(Difference::Added(ix, r.to_string()));
                                }
                            },
                            Err(e) => {
                                on_doc_error.call(TexError { msg : format!("Bibtex error: {}", e), line : 0 });
                            }
                        }
                    },
                    AnalyzerAction::BibError(e) => {
                        on_doc_error.call(TexError { msg : e, line : 0 });
                    },
                    AnalyzerAction::ItemSelected(sel_ixs) => {
                        /*if let Some(tk_ix) = doc.token_index_at(&sel_ixs[..]) {
                            // Count \n at all tokens before tk_ix
                            // let lines_before = tk_info.pos[0..tk_ix].iter().map(|pos| tk_info.txt[pos.clone()].chars().filter(|c| *c == '\n').count() ).sum::<usize>();
                            let lines_before = tk_info.txt[..tk_info.pos[tk_ix].start].chars().filter(|c| *c == '\n').count();

                            // Add one because we want one past the last line, add +1 because lines count from 1, not zero.
                            on_line_selection.call(lines_before);
                        } else {
                            println!("No token at document index {:?}", sel_ixs);
                        }*/
                        if let Some(line) = doc.get_line(&sel_ixs[..]) {
                            on_line_selection.call(line);
                        }
                    }
                }
                Continue(true)
            }
        });
        Self {
            send,
            on_reference_changed,
            on_section_changed,
            on_doc_changed,
            on_line_selection,
            on_doc_cleared,
            on_doc_error,
            on_refs_cleared,
            on_ref_file_changed,
            on_refs_validated
        }
    }

    pub fn connect_section_changed<F>(&self, f : F)
    where
        F : Fn(Difference) + 'static
    {
        self.on_section_changed.bind(f);
    }

    pub fn connect_reference_changed<F>(&self, f : F)
    where
        F : Fn(Difference) + 'static
    {
        self.on_reference_changed.bind(f);
    }

    pub fn connect_doc_changed<F>(&self, f : F)
    where
        F : Fn(Document) + 'static
    {
        self.on_doc_changed.bind(f);
    }

    pub fn connect_doc_cleared<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_doc_cleared.bind(f);
    }

    pub fn connect_references_cleared<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_refs_cleared.bind(f);
    }

    pub fn connect_references_validated<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_refs_validated.bind(f);
    }

    pub fn connect_doc_error<F>(&self, f : F)
    where
        F : Fn(TexError) + 'static
    {
        self.on_doc_error.bind(f);
    }

    pub fn connect_line_selection<F>(&self, f : F)
    where
        F : Fn(usize) + 'static
    {
        self.on_line_selection.bind(f);
    }

}

/*
hunspell-rs = 0.3.0
let h = Hunspell::new(affpath, dictpath);  path to the .aff file; path to the .dic file
h.check(word)
h.suggest(word)
*/

impl React<FileManager> for Analyzer {

    fn react(&self, manager : &FileManager) {

        // let is_new = Rc::new(RefCell::new(true));

        let send = self.send.clone();
        manager.connect_save({
            move |path| {
                send.send(AnalyzerAction::ChangeBaseDir(Some(path.into())));
            }
        });

        manager.connect_save({
            let send = self.send.clone();
            // let is_new = is_new.clone();
            move |path| {

                // let mut is_new = is_new.borrow_mut();
                // Trigger a document analysis whenever doc is saved, because
                // we might have gone from unknown path -> known path and now
                // we might be able to parse the bibtex file.
                // if *is_new {
                let send = send.clone();
                thread::spawn(move || {
                    match File::open(&path) {
                        Ok(mut f) => {
                            let mut content = String::new();
                            match f.read_to_string(&mut content) {
                                Ok(_) => {
                                    send.send(AnalyzerAction::TextChanged(content));
                                },
                                _ => { }
                            }
                        },
                        Err(_) => { }
                    }
                });
                // *is_new = false;
                // }
            }
        });
        manager.connect_opened({
            let send = self.send.clone();
            // let is_new = is_new.clone();
            move |(path, content)| {
                send.send(AnalyzerAction::ChangeBaseDir(Some(path.into())));
                send.send(AnalyzerAction::TextInit(content));
                // *(is_new.borrow_mut()) = false;
            }
        });
        manager.connect_new({
            let send = self.send.clone();
            // let is_new = is_new.clone();
            move |_| {
                send.send(AnalyzerAction::ChangeBaseDir(None));
                send.send(AnalyzerAction::TextInit(String::new()));
                // *(is_new.borrow_mut()) = true;
            }
        });
    }

}

impl React<DocTree> for Analyzer  {

    fn react(&self, tree : &DocTree) {
        let send = self.send.clone();
        tree.tree_view.selection().connect_changed(move |sel| {
            if sel.count_selected_rows() > 0 {
                let (paths, _) = sel.selected_rows();
                if let Some(path) = paths.get(0) {
                    send.send(AnalyzerAction::ItemSelected(path.indices().iter().map(|ix| *ix as usize ).collect()));
                }
            }
        });
    }

}

impl React<PapersWindow> for Analyzer {

    fn react(&self, window : &PapersWindow) {

        /* It is important to re-parse the document when the popover is opened
        to keep the document lines in sync with the document objects.
        This is a reliable signal that the user needs the most recent version
        of the document, so we can re-parse the document here. */
        window.editor.popover.connect_show({
            let view = window.editor.view.clone();
            let send = self.send.clone();
            move |_| {
                send.send(AnalyzerAction::TextChanged(get_text(&view)));
            }
        });
        window.titlebar.bib_popover.popover.connect_show({
            let view = window.editor.view.clone();
            let send = self.send.clone();
            move |_| {
                send.send(AnalyzerAction::TextChanged(get_text(&view)));
            }
        });
    }

}

fn get_text(view : &sourceview5::View) -> String {
    let buffer = view.buffer();
    buffer.text(&buffer.start_iter(), &buffer.end_iter(), true).to_string()
}


