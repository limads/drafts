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
use tectonic_bridge_core::{SecuritySettings, SecurityStance};
use tectonic::driver;
use tectonic::config;
use tectonic::status;

#[derive(Debug, Clone)]
pub enum TypesetterTarget {

    /// Carries path to a recently typeset file (with .pdf or .html extension)
    File(String),

    /// Carries binary content of a recently typeset PDF file
    PDFContent(Vec<u8>),

    /// Carries UTF-8 encoded content of a recently typeset HTML file
    HTMLContent(String)

}

pub enum TypesetterAction {

    // Carries content to be typeset.
    Request(String),

    Done(TypesetterTarget),

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

    on_done : Callbacks<TypesetterTarget>,

    on_error : Callbacks<String>

}

fn send_error(send : &glib::Sender<TypesetterAction>, err : &tectonic::errors::Error) {
    let mut out = String::new();
    // err.chain()
    for e in err.iter() {
        out += &format!("{}\n", e);
    }
    if let Some(src) = err.source() {
        out += &format!("{}\n", src);
    }
    send.send(TypesetterAction::Error(out));
}

use tectonic::status::*;

/// The StatusBackend implementors provided by tectonic all write to stdout/stderr. For
/// a GUI, what we want is to accumulate the status into a string that will be displayed to
/// a widget.
#[derive(Default)]
struct PapersStatusBackend(String);

impl tectonic::status::StatusBackend for PapersStatusBackend {

    fn report(
        &mut self,
        kind: MessageKind,
        args: fmt::Arguments<'_>,
        err: Option<&anyhow::Error>
    ) {
        use std::fmt::Write;
        match kind {
            MessageKind::Error /*| MessageKind::Warning*/ => {
                write!(&mut self.0, "{}\n", args);
            },
            _ => { }
        }

        /*if let Some(e) = err {
            for item in e.chain() {
                self.0 += &format!("{}\n", item);
            }
        }*/
    }

    fn dump_error_logs(&mut self, output: &[u8]) {

    }

}

fn manual_patterns(msg : &str) -> Option<usize> {
    msg.find("See the LaTeX manual")
        .or_else(|| msg.find("See\nthe LaTeX manual") )
        .or_else(|| msg.find("See the\nLaTeX manual") )
        .or_else(|| msg.find("See the LaTeX\nmanual") )
}

/// The toast does not accept some characters which get mixed up with its markup.
fn format_message(status : &str) -> String {
    let mut last_space = 0;
    let msg = status.chars()
        .map(|c| match c {
            '`' | '"' | '<' | '>' | '“' | '”' => '\'',
            '\n' | '\t' => ' ',
            _ => c
        })
        .map(|c| match c {
            ' ' => {
                if last_space == 10 {
                    last_space = 0;
                    '\n'
                } else {
                    last_space += 1;
                    c
                }
            },
            _ => c
        })
        .collect::<String>();
    if let Some(pos) = manual_patterns(&msg) {
        msg[..pos].to_string()
    } else {
        msg
    }
}

/// Slightly modified version of tectonic::latex_to_pdf, that uses a custom
/// status backed to report errors.
pub fn typeset_document<T: AsRef<str>>(latex: T) -> tectonic::Result<Vec<u8>> {

    use tectonic::errmsg;
    use tectonic::ctry;

    // let mut status = status::plain::PlainStatusBackend::default();
    let mut status = PapersStatusBackend::default();

    let auto_create_config_file = false;
    let config = ctry!(config::PersistentConfig::open(auto_create_config_file);
                       "failed to open the default configuration file");

    let only_cached = false;
    let bundle = ctry!(config.default_bundle(only_cached, &mut status);
                       "failed to load the default resource bundle");

    let format_cache_path = ctry!(config.format_cache_path();
                                  "failed to set up the format cache");

    let mut files = {
        let mut sb = driver::ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(driver::OutputFormat::Pdf)
            .do_not_write_output_files();

        let mut sess =
            ctry!(sb.create(&mut status); "failed to initialize the LaTeX processing session");
        ctry!(sess.run(&mut status); "the LaTeX engine failed");
        sess.into_file_data()
    };

    match files.remove("texput.pdf") {
        Some(file) => Ok(file.data),
        None => Err(errmsg!(
            "LaTeX didn't report failure, but no PDF was created (??)"
        )),
    }
}

/*fn typeset_document(latex : &str, ws : &mut Workspace) -> Result<Vec<u8>, String> {

    let mut status = PapersStatusBackend(String::new());
    //let config = config::PersistentConfig::open(false)
    //    .map_err(|e| format!("Error opening tectonic config: {:#}", e) )?;
    let config = config::PersistentConfig::default();
    let bundle = config.default_bundle(false, &mut status)
        .map_err(|e| format!("Error opening default bundle: {:#}", e) )?;
    let format_cache_path = config.format_cache_path()
        .map_err(|e| format!("Error establishing cache path: {:#}", e) )?;

    println!("Format cache path = {}", format_cache_path.display());
    //println!("Bundle = {}", bundle);

    ws.file.write_all(latex.as_bytes()).unwrap();

    use tectonic::unstable_opts::UnstableOptions;
    /*let unstables = UnstableOptions {
        continue_on_errors : true,
        ..Default::default()
    };*/

    let mut files = {
        // let mut sb = driver::ProcessingSessionBuilder::default();
        let mut sb = driver::ProcessingSessionBuilder::new_with_security(SecuritySettings::new(SecurityStance::DisableInsecures));
        sb
            .bundle(bundle)

            //.unstables(unstables)

            // Overrides primary_input_path and stdin options.
            //.primary_input_buffer(latex.as_bytes())
            //.primary_input_path(ws.file.path())
            .primary_input_path("/home/diego/Downloads/test.tex")

            // Required, or else SessionBuilder panics. This defines the output pdf name
            // by looking at the file stem.
            .tex_input_name("test.tex")

            //.output_dir(&ws.outdir)
            .output_dir("/home/diego/Downloads")

            // A file called latex.fmt will be created if it does not exist yet.
            .format_name("latex")

            // .pass(driver::PassSetting::Tex)
            // .pass(driver::PassSetting::BibtexFirst)

            //.format_name("latex.fmt")
            //.format_name("plain")
            .format_cache_path(format_cache_path)
            //.keep_logs(false)
            //.keep_intermediates(false)
            //.print_stdout(false)
            //.print_stdout(true)
            .output_format(driver::OutputFormat::Pdf);
            //.do_not_write_output_files();
        let mut sess = sb.create(&mut status).map_err(|e| format!("Error creating session builder: {:#}", e) )?;
        match sess.run(&mut status) {
            Ok(_) => { },
            Err(e) => {
                let msg = format_message(&status.0[..]);

                println!("{}", e);

                if msg.is_empty() {
                    let out = sess.get_stdout_content();
                    return Err(format!("Session error: {:#} ({})", e, String::from_utf8(out).unwrap()));
                } else {
                    return Err(msg);
                }
            }
        }
        sess.into_file_data()
    };

    for (file_name, file) in files.iter() {
        println!("Generated file: {}", file_name);
        // pritnln!("{}", )
        if file_name.ends_with(".pdf") {
            return Ok(file.data.clone());
        }
    }
    Err(format!("No PDF output generated"))

    /*match files.remove("texput.pdf") {
        Some(file) => Ok(file.data),
        None => Err(format!("No PDF output generated"))
    }*/
}*/

fn typeset_document_from_cli(ws : &mut Workspace, latex : &str, send : &glib::Sender<TypesetterAction>) {
    ws.file.seek(std::io::SeekFrom::Start(0));
    ws.file.write_all(latex.as_bytes()).unwrap();
    let out = Command::new("tectonic")
        .args(&["-X", "compile", ws.file.path().to_str().unwrap(), "--outdir", ws.outdir.path().to_str().unwrap(), "--outfmt", "pdf"])
        .output()
        .unwrap();
    println!("Command completed.");
    match out.status.success() {
        true => {
            send.send(TypesetterAction::Done(TypesetterTarget::File(ws.out_uri.clone())));
            /*let out = Command::new("evince")
                .args(&[&ws.out_uri])
                .spawn()
                .unwrap();*/

            // sudo apt install libpoppler-dev libpoppler-glib-dev
        },
        false => {
            let e = String::from_utf8(out.stderr).unwrap();
            let out = String::from_utf8(out.stdout).unwrap();
            println!("{}", e);
            send.send(TypesetterAction::Error(format_message(&format!("{}\n{}", e, out))));
        }
    }
}

fn typeset_document_from_lib(ws : &mut Workspace, latex : &str, send : &glib::Sender<TypesetterAction>) {
    match typeset_document(&latex[..]) {
        Ok(pdf_bytes) => {
            match File::create(&ws.out_uri) {
                Ok(mut f) => {
                    match f.write_all(&pdf_bytes) {
                        Ok(_) => {
                            send.send(TypesetterAction::Done(TypesetterTarget::File(ws.out_uri.clone())));
                        },
                        Err(e) => {
                            send.send(TypesetterAction::Error(format!("{}", e)));
                        }
                    }
                },
                Err(e) => {
                    send.send(TypesetterAction::Error(format!("{}", e)));
                }
            }
        },
        Err(e) => {
            send.send(TypesetterAction::Error(format!("{}", e)));
        }
    }
}

/*
On a first run, tectonic will download a **lot** of packages into
~/.cache/Tectonic
*/

impl Typesetter {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<TypesetterAction>(glib::PRIORITY_DEFAULT);
        let on_done : Callbacks<TypesetterTarget> = Default::default();
        let on_error : Callbacks<String> = Default::default();
        let (content_send, content_recv) = mpsc::channel::<String>();
        thread::spawn({
            let send = send.clone();
            move || {
                let mut ws = Workspace::new();
                println!("Outdir: {}", ws.outdir.path().display());

                // TODO parse document and verify that block \begin{document} \end{document} is not empty.

                loop {
                    match content_recv.recv() {
                        Ok(content) => {
                            // typeset_document_from_lib(&mut ws, &content, &send);
                            typeset_document_from_cli(&mut ws, &content, &send)
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
                    TypesetterAction::Done(target) => {
                        on_done.borrow().iter().for_each(|f| f(target.clone()) );
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
        F : Fn(TypesetterTarget) + 'static
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
                    send.send(TypesetterAction::Error(String::from("Cannot typeset empty document")));
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
