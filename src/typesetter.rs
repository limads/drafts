/*Copyright (c) 2022 Diego da Silva Lima. All rights reserved.

This work is licensed under the terms of the GPL v3.0 License.
For a copy, see http://www.gnu.org/licenses.*/

use gtk4::*;
use gtk4::prelude::*;
use crate::ui::*;
use crate::*;
use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use std::boxed;
use std::process::Command;
use std::time::Duration;
use tempfile;
use std::sync::mpsc;
use std::io::{Seek, SeekFrom};
use crate::ui::PapersEditor;
use crate::ui::Titlebar;
use std::error::Error;
use std::fmt;
use stateful::{Callbacks, ValuedCallbacks};
use stateful::React;
use std::path::{Path, PathBuf};
use crate::manager::FileManager;
use filecase::SingleArchiverImpl;
use itertools::Itertools;
use crate::typst_tools::Fonts;

#[derive(Debug, Clone)]
pub enum TypesetterTarget {

    /// Carries path to a recently typeset file (with .pdf or .html extension)
    File(String),

    /// Carries binary content of a recently typeset PDF file
    PDFContent(Vec<u8>),

    /// Carries UTF-8 encoded content of a recently typeset HTML file
    HTMLContent(String)

}

impl Default for TypesetterTarget {

    fn default() -> Self {
        Self::PDFContent(Vec::new())
    }

}

pub enum TypesetterAction {

    // Carries content to be typeset.
    Request(String),

    Done(TypesetterTarget),

    // Sets a new basedir (to search for images, references, etc) as the
    // current file dir.
    ChangeBaseDir(Option<PathBuf>),

    Error(String)

}

pub struct Workspace {

    outdir : tempfile::TempDir,

    // file : tempfile::NamedTempFile,

    // out_uri : String
}

impl Workspace {

    pub fn new() -> Self {
        let outdir = tempfile::Builder::new().tempdir().unwrap();
        // println!("Outdir = {}", outdir.path().display());
        // let file = tempfile::Builder::new().suffix(".tex").tempfile().unwrap();
        // println!("Tempfile path = {}", file.path().to_str().unwrap());
        // let out_uri = format!("file://{}/{}.pdf", outdir.path().to_str().unwrap(), file.path().file_stem().unwrap().to_str().unwrap().trim_end_matches(".tex"));
        Self { outdir, /*file, out_uri*/ }
    }

}

pub struct Typesetter {

    send : glib::Sender<TypesetterAction>,

    on_done : Callbacks<TypesetterTarget>,

    on_error : Callbacks<String>

}

fn typeset_document_with_typst(ws : &mut Workspace, file : &Path, send : &glib::Sender<TypesetterAction>, fonts : Fonts) {
    match crate::typst_tools::compile(file, fonts) {
        Ok(pdf_bytes) => {
            use std::io::Write;
            if let Some(fname) = file.file_stem().and_then(|f| f.to_str() ) {
                let mut out_path = PathBuf::from(ws.outdir.path().display().to_string());
                if !out_path.exists() || !out_path.is_dir() {
                    eprintln!("Invalid output path to PDF: {:?}", out_path);
                    return;
                }
                out_path.push(format!("{}.pdf", fname));
                match std::fs::File::create(&out_path) {
                    Ok(mut f) => {
                        if let Ok(_) = f.write_all(&pdf_bytes) {
                            send.send(TypesetterAction::Done(TypesetterTarget::File(out_path.to_str().unwrap().to_string()))).unwrap();
                        } else {
                            eprintln!("Unable to write to temporary file");
                        }
                    },
                    Err(e) => {
                        eprintln!("Unable to create temporary file: {}", e);
                    }
                }
            } else {
                eprintln!("Missing file name");
            }
        },
        Err(errs) => {
            for (line, msg) in errs.iter() {
                send.send(TypesetterAction::Error(format!("(Line {}) {}", (line + 1), msg)));
            }
        }
    }
}

pub struct TypesettingRequest {

    content : String,

    base_path : Option<PathBuf>,

    file :  Option<PathBuf>

}

impl Typesetter {

    pub fn new(fonts : Fonts) -> Self {
        let (send, recv) = glib::MainContext::channel::<TypesetterAction>(glib::PRIORITY_DEFAULT);
        let on_done : Callbacks<TypesetterTarget> = Default::default();
        let on_error : Callbacks<String> = Default::default();
        let (content_send, content_recv) = mpsc::channel::<TypesettingRequest>();

        thread::spawn({
            let send = send.clone();
            move || {
                let mut ws = Workspace::new();
                println!("Outdir: {}", ws.outdir.path().display());
                loop {
                    match content_recv.recv() {
                        Ok(TypesettingRequest { content, base_path, file }) => {
                            // typeset_document_from_lib(&mut ws, &content, base_path.as_ref().map(|p| p.as_path() ), &send);
                            // typeset_document_from_cli(&mut ws, &content, base_path.as_ref().map(|p| p.as_path() ), &send)
                            if let Some(file) = file {
                                typeset_document_with_typst(&mut ws, &file, &send, fonts.clone());
                            } else {
                                println!("Missing current file");
                            }
                        },
                        _ => { }
                    }
                }
            }
        });

        let mut base_path : Option<PathBuf> = None;
        let mut file : Option<PathBuf> = None;
        recv.attach(None, {
            let send = send.clone();
            let on_done = on_done.clone();
            let on_error = on_error.clone();
            move |action| {
                match action {
                    TypesetterAction::Request(txt) => {
                        content_send.send(TypesettingRequest { content : txt, base_path : base_path.clone(), file : file.clone() });
                    },
                    TypesetterAction::Done(target) => {
                        on_done.call(target.clone());
                    },
                    TypesetterAction::Error(e) => {
                        on_error.call(e.clone());
                    },
                    TypesetterAction::ChangeBaseDir(opt_path) => {
                        if let Some(path) = opt_path {
                            if let Some(parent) = Path::new(&path).parent() {
                                base_path = Some(parent.to_owned());
                                file = Some(path.to_owned());
                            } else {
                                log::warn!("File without valid parent path");
                            }
                        } else {
                            base_path = None;
                            file = None;
                        }
                    }
                }
                Continue(true)
            }
        });

        Self { send, on_done, on_error }
    }

    pub fn connect_done<F>(&self, f : F)
    where
        F : Fn(TypesetterTarget) + 'static
    {
        self.on_done.bind(f);
    }

    pub fn connect_error<F>(&self, f : F)
    where
        F : Fn(String) + 'static
    {
        self.on_error.bind(f);
    }

}

fn request_typesetting_file(
    pdf_btn : &Button,
    send : &glib::Sender<TypesetterAction>
) {
    send.send(TypesetterAction::Request(String::new())).unwrap();
    pdf_btn.set_icon_name("timer-symbolic");
    pdf_btn.set_sensitive(false);
}

fn request_typesetting_buffer(
    pdf_btn : &Button,
    view : &sourceview5::View,
    send : &glib::Sender<TypesetterAction>
) {

    let buffer = view.buffer();
    let txt = buffer.text(
        &buffer.start_iter(),
        &buffer.end_iter(),
        true
    ).to_string();

    if txt.is_empty() {
        send.send(TypesetterAction::Error(String::from("Cannot typeset empty document")));
        return;
    }

    send.send(TypesetterAction::Request(txt)).unwrap();
    pdf_btn.set_icon_name("timer-symbolic");
    pdf_btn.set_sensitive(false);
    //refresh_btn.set_sensitive(false);
}

impl React<PapersWindow> for Typesetter {

    fn react(&self, win : &PapersWindow) {
        let (titlebar, editor) = (&win.titlebar, &win.editor);

        titlebar.typeset_action.connect_activate({
            let view = editor.view.clone();
            let send = self.send.clone();
            let pdf_btn = titlebar.pdf_btn.clone();
            move |_, _| {
                request_typesetting_file(&pdf_btn, &send);
            }
        });
        titlebar.pdf_btn.connect_clicked({
            let typeset_action = titlebar.typeset_action.clone();
            move |btn| {
                typeset_action.activate(None);
            }
        });
        /*titlebar.pdf_btn.connect_clicked({
            let view = editor.view.clone();
            let send = self.send.clone();
            move |btn| {
                request_typesetting_buffer(&btn, /*&refresh_btn,*/ &view, &send);
            }
        });*/

        /*titlebar.refresh_btn.connect_clicked({
            let pdf_btn = titlebar.pdf_btn.clone();
            let view = editor.view.clone();
            let send = self.send.clone();
            move |btn| {
                request_typesetting(&pdf_btn, &btn, &view, &send);
            }
        });*/
    }
}

impl React<FileManager> for Typesetter {

    fn react(&self, manager : &FileManager) {
        let send = self.send.clone();
        manager.connect_save({
            move |path| {
                send.send(TypesetterAction::ChangeBaseDir(Some(path.into())));
            }
        });
        manager.connect_opened({
            let send = self.send.clone();
            move |(path, _)| {
                send.send(TypesetterAction::ChangeBaseDir(Some(path.into())));
            }
        });
        manager.connect_new({
            let send = self.send.clone();
            move |_| {
                send.send(TypesetterAction::ChangeBaseDir(None));
            }
        });
    }

}


