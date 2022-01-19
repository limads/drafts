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

pub enum TypesetterAction {

    // Carries content to be typeset.
    Request(String),

    Done,

    Error(String)

}

pub struct Workspace {
    outdir : tempfile::TempDir,
    file : tempfile::NamedTempFile,
    out_uri : String
}

impl Workspace {

    pub fn new() -> Self {
        let outdir = tempfile::Builder::new().tempdir().unwrap();
        let file = tempfile::Builder::new().suffix(".tex").tempfile().unwrap();
        println!("Tempfile path = {}", file.path().to_str().unwrap());
        let out_uri = format!("file://{}/{}.pdf", outdir.path().to_str().unwrap(), file.path().file_stem().unwrap().to_str().unwrap().trim_end_matches(".tex"));
        Self { outdir, file, out_uri }
    }
}

pub struct Typesetter {

    send : glib::Sender<TypesetterAction>,

    on_done : Callbacks<()>,

    on_error : Callbacks<String>

}

impl Typesetter {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<TypesetterAction>(glib::PRIORITY_DEFAULT);
        let on_done : Callbacks<()> = Default::default();
        let on_error : Callbacks<String> = Default::default();
        let (content_send, content_recv) = mpsc::channel::<String>();

        thread::spawn({
            let send = send.clone();
            move || {
                let mut ws = Workspace::new();
                loop {
                    match content_recv.recv() {
                        Ok(content) => {
                            ws.file.as_file().set_len(0).unwrap();
                            ws.file.as_file().seek(SeekFrom::Start(0)).unwrap();
                            ws.file.write_all(content.as_bytes()).unwrap();

                            println!("{}", ws.outdir.path().to_str().unwrap());

                            // --outfmt html|pdf
                            println!("Running command");
                            let out = Command::new("tectonic")
                                .args(&["-X", "compile", ws.file.path().to_str().unwrap(), "--outdir", ws.outdir.path().to_str().unwrap(), "--outfmt", "pdf"])
                                .output()
                                .unwrap();
                            println!("Command completed.");
                            match out.status.success() {
                                true => {
                                    // gtk4::show_uri(Some(&window), &out_uri, 0);
                                    send.send(TypesetterAction::Done);
                                    let out = Command::new("evince")
                                        .args(&[&ws.out_uri])
                                        .spawn()
                                        .unwrap();
                                    // We must spawn (and not block here) or else
                                    // the file descriptor won't be released for the next call.
                                    /*match out.status.success() {
                                        true => {
                                            println!("Evince closed");
                                        },
                                        false => {
                                            let e = String::from_utf8(out.stderr).unwrap();
                                            println!("{}", e);
                                            send.send(TypesetterAction::Error(e));
                                        }
                                    }*/
                                },
                                false => {
                                    let e = String::from_utf8(out.stderr).unwrap();
                                    println!("{}", e);
                                    send.send(TypesetterAction::Error(e));
                                }
                            }
                        },
                        _ => { }
                    }
                }
            }
        });

        recv.attach(None, {
            let send = send.clone();
            let on_done = on_done.clone();
            let on_error = on_error.clone();
            move |action| {
                match action {
                    TypesetterAction::Request(txt) => {
                        content_send.send(txt);
                    },
                    TypesetterAction::Done => {
                        on_done.borrow().iter().for_each(|f| f(()) );
                    },
                    TypesetterAction::Error(e) => {
                        on_error.borrow().iter().for_each(|f| f(e.clone()) );
                    }
                }
                Continue(true)
            }
        });

        Self { send, on_done, on_error }
    }

    pub fn connect_done<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_done.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_error<F>(&self, f : F)
    where
        F : Fn(String) + 'static
    {
        self.on_error.borrow_mut().push(boxed::Box::new(f));
    }

}

impl React<(Titlebar, PapersEditor)> for Typesetter {

    fn react(&self, (titlebar, editor) : &(Titlebar, PapersEditor)) {
        let send = self.send.clone();
        titlebar.pdf_btn.connect_clicked({
            let view = editor.view.clone();
            // let window = window.clone();
            // let ws = ws.clone();
            move |btn| {
                let buffer = view.buffer();
                let txt = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true
                ).to_string();

                if txt.is_empty() {
                    return;
                }

                send.send(TypesetterAction::Request(txt)).unwrap();
                btn.set_icon_name("timer-symbolic");
                btn.set_sensitive(false);

                // let mut ws = ws.borrow_mut();
                // thread::sleep(Duration::from_secs(200));
            }
        });
    }
}
