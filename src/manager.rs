use gtk4::*;
use gtk4::prelude::*;
use crate::ui::*;
use crate::*;
use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use std::boxed;

pub enum FileAction {

    NewRequest,

    SaveRequest(Option<String>),

    OpenRequest(String),

    // Carries path and content
    OpenSuccess(String, String),

    OpenError(String)

}

pub struct FileManager {
    send : glib::Sender<FileAction>,
    on_open : Callbacks<(String, String)>
}

impl FileManager {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<FileAction>(glib::PRIORITY_DEFAULT);
        let on_open : Callbacks<(String, String)> = Default::default();
        recv.attach(None, {
            let on_open = on_open.clone();
            let send = send.clone();
            move |action| {
                match action {
                    FileAction::NewRequest => {

                    },
                    FileAction::SaveRequest(opt_path) => {

                    },
                    FileAction::OpenRequest(path) => {
                        thread::spawn({
                            let send = send.clone();
                            move || {
                                match File::open(&path) {
                                    Ok(mut f) => {
                                        let mut content = String::new();
                                        f.read_to_string(&mut content).unwrap();
                                        send.send(FileAction::OpenSuccess(path.to_string(), content)).unwrap();
                                    },
                                    Err(e) => {
                                        send.send(FileAction::OpenError(format!("{}", e ))).unwrap();
                                    }
                                }
                            }
                        });
                    },
                    FileAction::OpenSuccess(path, content) => {
                        on_open.borrow().iter().for_each(|f| f((path.clone(), content.clone())) );
                    },
                    FileAction::OpenError(e) => {
                        println!("{}", e);
                    }
                }
                Continue(true)
            }
        });
        Self { on_open, send }
    }

    pub fn connect_opened<F>(&self, f : F)
    where
        F : Fn((String, String)) + 'static
    {
        self.on_open.borrow_mut().push(boxed::Box::new(f));
    }

}

impl React<MainMenu> for FileManager {

    fn react(&self, menu : &MainMenu) {

        menu.action_new.connect_activate({
            let send = self.send.clone();
            move |_,_| {
                send.send(FileAction::NewRequest).unwrap();
            }
        });
        menu.action_save.connect_activate({
            let send = self.send.clone();
            move |_,_| {
                send.send(FileAction::SaveRequest(None));
            }
        });
    }

}

impl React<OpenDialog> for FileManager {

    fn react(&self, dialog : &OpenDialog) {
        let send = self.send.clone();
        dialog.dialog.connect_response(move |dialog, resp| {
            match resp {
                ResponseType::Accept => {
                    if let Some(path) = dialog.file().and_then(|f| f.path() ) {
                        send.send(FileAction::OpenRequest(path.to_str().unwrap().to_string())).unwrap();
                    }
                },
                _ => { }
            }
        });
    }

}

