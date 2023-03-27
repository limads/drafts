/*Copyright (c) 2022 Diego da Silva Lima. All rights reserved.

This work is licensed under the terms of the GPL v3.0 License.
For a copy, see http://www.gnu.org/licenses.*/

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
                                if let (Some(fname), Some(base_path)) = (bib.filename, bib.base_dir) {
                                    let path = format!("{}/{}", base_path, fname);
                                    if Path::new(&path).exists() {
                                        if let Ok(mut f) = File::open(&path) {
                                            let mut content = String::new();
                                            if let Ok(_) = f.read_to_string(&mut content) {
                                                send.send(AnalyzerAction::BibChanged(content));
                                            } else {
                                                eprintln!("could not read file");
                                            }
                                        } else {
                                            eprintln!("could not open file");
                                        }
                                    } else {
                                        send.send(AnalyzerAction::BibError(format!("Path {} does not exist", path)));
                                    }
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
                                let fst_err = errs.get(0).map(|e| e.1.clone() )
                                    .unwrap_or(String::from("Unknown error"));
                                doc = Document::default();
                                on_doc_cleared.call(());
                                on_doc_error.call(TexError { msg : fst_err, line : 0 });
                            }
                        }

                    },
                    AnalyzerAction::BibChanged(txt) => {
                        match BibParser::parse(&txt[..]) {
                            Ok(refs) =>  {
                                on_refs_cleared.call(());
                                let n = refs.as_ref().len();
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
            move |path| {
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
            }
        });
        manager.connect_opened({
            let send = self.send.clone();
            move |(path, content)| {
                send.send(AnalyzerAction::ChangeBaseDir(Some(path.into())));
                send.send(AnalyzerAction::TextInit(content));
            }
        });
        manager.connect_new({
            let send = self.send.clone();
            move |_| {
                send.send(AnalyzerAction::ChangeBaseDir(None));
                send.send(AnalyzerAction::TextInit(String::new()));
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


