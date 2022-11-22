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
// use tectonic_bridge_core::{SecuritySettings, SecurityStance};
use tectonic::driver;
use tectonic::config;
use tectonic::status;
use stateful::{Callbacks, ValuedCallbacks};
use stateful::React;
use std::path::{Path, PathBuf};
use crate::manager::FileManager;
use filecase::SingleArchiverImpl;
use itertools::Itertools;

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
        println!("Outdir = {}", outdir.path().display());
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
/// a widget. This struct wraps this string and implements the tectonic StatusBackend trait.
#[derive(Default)]
struct PapersStatusBackend {

    errors : String,

    warnings : String,

    notes : String

}

/*
Here's how tectonic installs the latex environment:
(1) Verify if there exists a ~/.cache/Tectonic. If not, create it and its sub-directories (files, formats, etc).
(2) Tectonic/indexes contains a space-delimited set of files in the format [File] [value] [number], Where
[File] are tfm (fonts), sty (styles), etc.
(3) Verify if there exists a ~/.config/Tectonic/config.toml. If not, create it. It will contain
[[default_bundles]]
url = "https://relay.fullyjustified.net/default_bundle.tar"

*/

impl tectonic::status::StatusBackend for PapersStatusBackend {

    fn report(
        &mut self,
        kind: MessageKind,
        args: fmt::Arguments<'_>,
        err: Option<&anyhow::Error>
    ) {
        use std::fmt::Write;
        match kind {
            MessageKind::Error => {
                write!(&mut self.errors, "{}\n", args);
            },
            MessageKind::Warning => {
                write!(&mut self.warnings, "{}\n", args);
            },
            MessageKind::Note => {
                write!(&mut self.notes, "{}\n", args);
            }
        }

        /*if let Some(e) = err {
            for item in e.chain() {
                self.0 += &format!("{}\n", item);
            }
        }*/
    }

    fn dump_error_logs(&mut self, output: &[u8]) {
        println!("{}", String::from_utf8(Vec::from(output)).unwrap());
    }

}

fn manual_patterns(msg : &str) -> Option<usize> {
    msg.find("See the LaTeX manual")
        .or_else(|| msg.find("See\nthe LaTeX manual") )
        .or_else(|| msg.find("See the\nLaTeX manual") )
        .or_else(|| msg.find("See the LaTeX\nmanual") )
}

fn potentially_recoverable_patterns(msg : &str) -> Option<usize> {
    msg.find("error: halted on potentially-recoverable error as specified")
        .or_else(|| msg.find("error:\nhalted on potentially-recoverable error as specified") )
        .or_else(|| msg.find("error: halted\non potentially-recoverable error as specified") )
        .or_else(|| msg.find("error: halted on\npotentially-recoverable error as specified") )
        .or_else(|| msg.find("error: halted on potentially-recoverable\nerror as specified") )
        .or_else(|| msg.find("error: halted on potentially-recoverable error\nas specified") )
        .or_else(|| msg.find("error: halted on potentially-recoverable error as\nspecified") )
}

fn format_message(status : &str) -> String {

    // Transform all whitespace sequences of length 1+ to a single space.
    let status = regex::Regex::new(r#"\s+"#).unwrap().replace_all(&status, " ").to_string();

    /// The toast does not accept some characters which get mixed up with its markup.
    /// Transform them here to their closest original meaning, at the same time inserting
    /// line breaks if the current line is too long.
    let mut last_space = 0;
    let mut msg : String = status.trim().chars()
        .map(|c| match c {
            '`' | '"' | '<' | '>' | '“' | '”' => '\'',
            '\n' | '\t' | '!' | '"' => ' ',
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

    // Toast markup require escaping &
    msg = msg.clone().replace("&", "&amp;");

    // Remove duplicated spaces
    msg = msg.clone().chars().dedup_by(|a, b| *a == ' ' && *a == *b ).collect();

    // Remove final parts of the standard tectonic error message that do not give any
    // information to the user specific to the problem at hand.
    let msg_new = if let Some(man_pos) = manual_patterns(&msg) {
        if let Some(rec_pos) = potentially_recoverable_patterns(&msg[..man_pos]) {
            msg[..(man_pos.min(rec_pos))].to_string()
        } else {
            msg[..man_pos].to_string()
        }
    } else {
        if let Some(pos) = potentially_recoverable_patterns(&msg) {
            msg[..pos].to_string()
        } else {
            msg.to_string()
        }
    };

    // Make a pretty line output, ignoring the file name.
    let msg_pretty = if let Some(m) = regex::Regex::new(r#"tex:\d+:"#).unwrap().find(&msg_new[..]) {
        let (start, end) = (m.start(), m.end());
        if let Some(line_n) = msg_new[start..end].split(":").nth(1).and_then(|n_str| n_str.parse::<usize>().ok() ) {
            format!("(Line {}) {}", line_n, &msg_new[end..])
        } else {
            format!("{}", &msg_new[end..])
        }
    } else {
        msg_new.to_string()
    };

    msg_pretty.trim_start_matches("Error: ").to_string()
}

/*/// Slightly modified version of tectonic::latex_to_pdf (0.8.0), that uses a custom
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
        // ctry!(sess.run(&mut status); "the LaTeX engine failed");
        let ans = sess.run(&mut status);
        match &ans {
            Ok(_) => { },
            Err(_) => {
                let msg = format_message(&status.0[..]);
                println!("Error: {}", msg);
                ctry!(sess.run(&mut status); "the LaTeX engine failed");
            }
        }
        sess.into_file_data()
    };

    match files.remove("texput.pdf") {
        Some(file) => Ok(file.data),
        None => Err(errmsg!(
            "LaTeX didn't report failure, but no PDF was created (??)"
        )),
    }
}*/

const URI_BUNDLE : &'static str = "https://relay.fullyjustified.net/default_bundle.tar";

/*
Here is an issue: If we call gtk::init(), any calls to Session::run
Will generate the error: CFF: Parsing CFF DICT failed. (error=-1) for some
obscure reason. The function below works fine in another process where
gtk::init() is not called. Perhaps glib and tectonic rely on static variables with
the same name. Perhaps gtk mess with tectonic launching external CLI tools.
For whatever reason, tectonic and GTK cannot live in the same process.
*/
pub fn typeset_document(latex : &str, base_path : &Path, out_path : &Path) -> Result<Vec<u8>, String> {

    let mut status = PapersStatusBackend{ errors : String::new(), notes : String::new(), warnings : String::new() };
    // let mut status = tectonic::status::plain::PlainStatusBackend::new(tectonic::status::ChatterLevel::Normal);

    //let config = config::PersistentConfig::open(false)
    //    .map_err(|e| format!("Error opening tectonic config: {:#}", e) )?;
    let config = config::PersistentConfig::open(true).unwrap();

    // By setting this to false, we let tectonic downloads files from the network.
    let only_cached_bundle = false;
    //let mut default_bundle = config.default_bundle(only_cached_bundle, &mut status)
    //    .map_err(|e| format!("Error opening default bundle: {:#}", e) )?;
    let uri_bundle = config.make_cached_url_provider(URI_BUNDLE, only_cached_bundle, None, &mut status)
        .map_err(|e| format!("Error opening URI bundle: {:#}", e) )?;

    // Pass a custom bundle path with (path is passed as a CLI option):
    // let local_bundle = sess_builder.bundle(config.make_local_file_provider(path, status)?);

    let format_cache_path = config.format_cache_path()
        .map_err(|e| format!("Error establishing cache path: {:#}", e) )?;

    println!("Format cache path = {}", format_cache_path.display());
    println!("Bundle path = {}", config.default_bundle_loc());

    // println!("Bundle = {:?}", bundle.all_files(&mut status));

    // ws.file.write_all(latex.as_bytes()).unwrap();

    use tectonic::unstable_opts::UnstableOptions;

    /*let unstables = UnstableOptions {
        continue_on_errors : true,
        ..Default::default()
    };*/

    println!("Using {:?} as base path", base_path.to_str().unwrap());

    let mut files = {
        let mut sb = driver::ProcessingSessionBuilder::default();
        // let mut sb = driver::ProcessingSessionBuilder::new_with_security(SecuritySettings::new(SecurityStance::DisableInsecures));
        sb
            .bundle(uri_bundle)
            .filesystem_root(base_path)
            // .bundle(default_bundle)
            // .bundle(local_bundle)

            //.unstables(unstables)

            // Overrides primary_input_path and stdin options.
            .primary_input_buffer(latex.as_bytes())
            //.primary_input_path(ws.file.path())
            // .primary_input_path("/home/diego/Downloads/test.tex")
            //.primary_input_buffer(b"\\documentclass[a4,11pt]{article} \\usepackage{inputenc} \\begin{document}Text\\end{document}")

            // Required, or else SessionBuilder panics. This defines the output pdf name
            // by looking at the file stem.
            .tex_input_name("texput.tex")

            .output_dir(out_path)
            //.output_dir("/home/diego/Downloads")

            // A file called latex.fmt will be created if it does not exist yet.
            .format_name("latex")

            .pass(driver::PassSetting::Default)

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
        // println!("{:?}", sess);
        match sess.run(&mut status) {
            Ok(_) => { },
            Err(e) => {

                let msg = format_message(&status.errors[..]);

                println!("User message: {}", msg);
                println!("{}", e);
                println!("{}", status.warnings);
                println!("{}", status.notes);
                println!("{}", String::from_utf8(sess.get_stdout_content()).unwrap());

                if msg.is_empty() {
                    let out = sess.get_stdout_content();
                    let full_msg = format_message(&format!("Session error: {:#} ({})", e, String::from_utf8(out).unwrap()));
                    return Err(full_msg);
                } else {
                    return Err(msg);
                }
                // println!("Session error: {}",e );
            }
        }
        sess.into_file_data()
    };

    for (file_name, file) in files.iter() {
        if file_name.ends_with("texput.pdf") {
            println!("Generated file: {}", file_name);
            return Ok(file.data.clone());
        }
    }
    Err(format!("No PDF output generated"))

    /*match files.remove("texput.pdf") {
        Some(file) => Ok(file.data),
        None => Err(format!("No PDF output generated"))
    }*/
}

pub fn spawn_helper(ws : &mut Workspace, latex : &str, base_path : Option<&Path>) -> Result<Vec<u8>, String> {

    use std::process::{Command, Stdio};

    let bp = base_path.unwrap_or(ws.outdir.path());
    let args = ["-p", bp.to_str().unwrap(), "-o", ws.outdir.path().to_str().unwrap()];

    println!("Using {:?} as base path", bp.to_str().unwrap());

    let mut res_cmd = Command::new("helper")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    if res_cmd.is_err() {
        if let Ok(exe_path) = std::env::current_exe() {
            res_cmd = Command::new(&format!("/home/diego/Software/gnome/target/debug/helper"))
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            if res_cmd.is_err() {
                return Err(format!("Missing typesetter helper"));
            }
        } else {
            return Err(format!("Missing typesetter helper"));
        }
    }
    let mut cmd = res_cmd.unwrap();
    let mut stdin = cmd.stdin.as_mut().unwrap().write_all(latex.as_bytes()).unwrap();
    match cmd.wait_with_output() {
        Ok(out) => {
            if out.status.success() {
                Ok(out.stdout)
            } else {
                Err(String::from_utf8(out.stderr).unwrap())
            }
        },
        Err(e) => {
            Err(format!("{}",e))
        }
    }
}

pub fn typeset_document_from_cli(ws : &mut Workspace, latex : &str, base_path : Option<&Path>, send : &glib::Sender<TypesetterAction>) {
    /*ws.file.seek(std::io::SeekFrom::Start(0));
    ws.file.write_all(latex.as_bytes()).unwrap();
    let out = Command::new("tectonic")
        .args(&["-X", "compile", ws.file.path().to_str().unwrap(), "--outdir", ws.outdir.path().to_str().unwrap(), "--outfmt", "pdf"])
        .output()
        .unwrap();
    println!("Command completed.");
    match out.status.success() {
        true => {
            unimplemented!()
            // send.send(TypesetterAction::Done(TypesetterTarget::File(ws.out_uri.clone())));
            // sudo apt install libpoppler-dev libpoppler-glib-dev
        },
        false => {
            let e = String::from_utf8(out.stderr).unwrap();
            let out = String::from_utf8(out.stdout).unwrap();
            println!("{}", e);
            send.send(TypesetterAction::Error(format_message(&format!("{}\n{}", e, out))));
        }
    }*/
    // unimplemented!()

    /*
    TODO start helper with the environment variable
    $TECTONIC_CACHE_DIR=$XDG_DATA_DIR/tectonic after tectonic>0.9
    */

    match spawn_helper(ws, latex, base_path) {
        Ok(pdf_content) => {
            let out_path = format!("{}/out.pdf", &ws.outdir.path().display());
            let mut f = File::create(&out_path).unwrap();
            f.write_all(&pdf_content).unwrap();
            send.send(TypesetterAction::Done(TypesetterTarget::File(out_path)));
        },
        Err(e) => {
            if e.is_empty() {
                println!("Error string is empty");
            }
            send.send(TypesetterAction::Error(e));
        }
    }
}

fn typeset_document_from_lib(ws : &mut Workspace, latex : &str, base_path : Option<&Path>, send : &glib::Sender<TypesetterAction>) {

    // Overwrite the GTK4 call of signal(SIGPIPE, SIG_IGN); since tectonic requires
    // them for communicatin with external processes.
    // unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL); }

    println!("Processing: {}", latex);
    let bp = base_path.unwrap_or(ws.outdir.path());
    match typeset_document(&latex[..], bp.as_ref(), ws.outdir.path()) {
        Ok(pdf_bytes) => {
            let out_path = format!("{}/out.pdf", &ws.outdir.path().display());
            match File::create(&out_path) {
                Ok(mut f) => {
                    match f.write_all(&pdf_bytes) {
                        Ok(_) => {
                            send.send(TypesetterAction::Done(TypesetterTarget::File(out_path.clone())));
                        },
                        Err(e) => {
                            send.send(TypesetterAction::Error(format!("File writing error: {}", e)));
                        }
                    }
                },
                Err(e) => {
                    send.send(TypesetterAction::Error(format!("File creation error: {}", e)));
                }
            }
        },
        Err(e) => {
            send.send(TypesetterAction::Error(format!("Typesetting error: {}", e)));
        }
    }
}

/*
On a first run, tectonic will download a **lot** of packages into
~/.cache/Tectonic
*/

pub struct TypesettingRequest {

    content : String,

    base_path : Option<PathBuf>

}

impl Typesetter {

    pub fn new() -> Self {
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
                        Ok(TypesettingRequest { content, base_path }) => {
                            // typeset_document_from_lib(&mut ws, &content, base_path.as_ref().map(|p| p.as_path() ), &send);
                            typeset_document_from_cli(&mut ws, &content, base_path.as_ref().map(|p| p.as_path() ), &send)
                        },
                        _ => { }
                    }
                }
            }
        });

        let mut base_path : Option<PathBuf> = None;
        recv.attach(None, {
            let send = send.clone();
            let on_done = on_done.clone();
            let on_error = on_error.clone();
            move |action| {
                match action {
                    TypesetterAction::Request(txt) => {
                        println!("Base path: {:?}", base_path.clone());
                        content_send.send(TypesettingRequest { content : txt, base_path : base_path.clone() });
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
                            } else {
                                log::warn!("File without valid parent path");
                            }
                        } else {
                            base_path = None;
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

fn request_typesetting(
    pdf_btn : &Button,
    refresh_btn : &Button,
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
    refresh_btn.set_sensitive(false);
}

impl React<PapersWindow> for Typesetter {

    fn react(&self, win : &PapersWindow) {
        let (titlebar, editor) = (&win.titlebar, &win.editor);
        titlebar.pdf_btn.connect_clicked({
            let view = editor.view.clone();
            let send = self.send.clone();
            let refresh_btn = titlebar.refresh_btn.clone();
            // let window = window.clone();
            // let ws = ws.clone();
            move |btn| {
                //if btn.is_active() {
                request_typesetting(&btn, &refresh_btn, &view, &send);
                //}

                // let mut ws = ws.borrow_mut();
                // thread::sleep(Duration::from_secs(200));
            }
        });
        titlebar.refresh_btn.connect_clicked({
            let pdf_btn = titlebar.pdf_btn.clone();
            let view = editor.view.clone();
            let send = self.send.clone();
            move |btn| {
                request_typesetting(&pdf_btn, &btn, &view, &send);
            }
        });
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


