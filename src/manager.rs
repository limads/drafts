use gtk4::*;
use gtk4::prelude::*;
use crate::ui::*;
use crate::*;
use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use std::boxed;
use std::thread::JoinHandle;
use crate::ui::PapersEditor;

#[derive(Clone, Copy)]
pub enum ActionAfterClose {
    New,
    Open,
    CloseWindow
}

pub enum FileAction {

    // Whether to force or not
    NewRequest(bool),

    SaveRequest(Option<String>),

    SaveSuccess(String),

    FileChanged,

    OpenRequest(String),

    // Carries path and content
    OpenSuccess(String, String),

    OpenError(String),

    RequestShowOpen,

    FileCloseRequest,

    WindowCloseRequest

}

pub struct FileManager {
    pub send : glib::Sender<FileAction>,
    on_open : Callbacks<(String, String)>,
    on_open_request : Callbacks<()>,
    on_new : Callbacks<()>,
    on_buffer_read_request : ValuedCallbacks<(), String>,
    on_file_changed : Callbacks<Option<String>>,
    on_save_unknown_path : Callbacks<String>,
    on_save : Callbacks<String>,
    on_close_confirm : Callbacks<String>,
    on_window_close : Callbacks<()>,
    on_show_open : Callbacks<()>
}

impl FileManager {

    pub fn new() -> Self {

        let (send, recv) = glib::MainContext::channel::<FileAction>(glib::PRIORITY_DEFAULT);
        let on_open : Callbacks<(String, String)> = Default::default();
        let on_show_open : Callbacks<()> = Default::default();
        let on_new : Callbacks<()> = Default::default();
        let on_open_request : Callbacks<()> = Default::default();
        let on_buffer_read_request : ValuedCallbacks<(), String> = Default::default();
        let on_save_unknown_path : Callbacks<String> = Default::default();
        let on_save : Callbacks<String> = Default::default();
        let on_close_confirm : Callbacks<String> = Default::default();
        let on_window_close : Callbacks<()> = Default::default();
        let on_file_changed : Callbacks<Option<String>> = Default::default();
        recv.attach(None, {
            let on_open = on_open.clone();
            let on_new = on_new.clone();
            let send = send.clone();
            let on_buffer_read_request = on_buffer_read_request.clone();
            let on_save_unknown_path = on_save_unknown_path.clone();
            let on_close_confirm = on_close_confirm.clone();
            let on_window_close = on_window_close.clone();
            let on_file_changed = on_file_changed.clone();
            let on_open_request = on_open_request.clone();
            let on_save = on_save.clone();
            let on_show_open = on_show_open.clone();

            // Holds an action that should happen after the currently-opened file is closed.
            // This variable is updated at NewRequest, OpenRequest and WindowCloseRequest.
            let mut after_close = ActionAfterClose::New;

            // Holds optional path and whether the file is saved.
            let mut curr_path : (Option<String>, bool) = (None, true);
            let mut just_opened = true;

            move |action| {
                match action {

                    // To be triggered when "new" action is activated on the main menu.
                    FileAction::NewRequest(force) => {
                        if !force && !curr_path.1 {
                            after_close = ActionAfterClose::New;
                            on_close_confirm.borrow().iter().for_each(|f| f(curr_path.0.clone().unwrap_or(String::from("Untitled.tex"))) );
                        } else {
                            curr_path.0 = None;
                            curr_path.1 = true;
                            on_new.borrow().iter().for_each(|f| f(()) );
                            just_opened = true;
                            println!("Just opened set to true");
                        }
                    },
                    FileAction::SaveRequest(opt_path) => {
                        if let Some(path) = opt_path {
                            let content = on_buffer_read_request.borrow()[0](());
                            spawn_save_file(path, content, send.clone());
                        } else {
                            if let Some(path) = curr_path.0.clone() {
                                let content = on_buffer_read_request.borrow()[0](());
                                spawn_save_file(path, content, send.clone());
                            } else {
                                on_save_unknown_path.borrow().iter().for_each(|f| f(String::new()) );
                            }
                        }
                    },

                    // Called when the buffer changes. Ideally, when the user presses a key to
                    // insert a character. But also when the buffer is changed after a new template is
                    // loaded or a file is opened, which is why the callback is only triggered when
                    // just_opened is false.
                    FileAction::FileChanged => {

                        // Use this decision branch to inhibit buffer changes
                        // when a new file is opened.
                        if just_opened {
                            just_opened = false;
                            println!("(File changed) Just opened set to false");
                        } else {
                            curr_path.1 = false;
                            on_file_changed.borrow().iter().for_each(|f| f(curr_path.0.clone()) );
                            println!("File changed by key press");
                        }
                    },
                    FileAction::SaveSuccess(path) => {
                        curr_path.0 = Some(path.clone());
                        curr_path.1 = true;
                        on_save.borrow().iter().for_each(|f| f(path.clone()) );
                    },
                    FileAction::RequestShowOpen => {
                        if curr_path.1 {
                            on_show_open.borrow().iter().for_each(|f| f(()) );
                        } else {
                            after_close = ActionAfterClose::Open;
                            on_close_confirm.borrow().iter().for_each(|f| f(curr_path.0.clone().unwrap_or(String::from("Untitled.tex"))) );
                        }
                    },
                    FileAction::OpenRequest(path) => {

                        /*if !curr_path.1 {
                            after_close = ActionAfterClose::Open;
                            on_close_confirm.borrow().iter().for_each(|f| f(curr_path.0.clone().unwrap_or(String::from("Untitled.tex"))) );
                            return Continue(true);
                        }*/

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
                        just_opened = true;
                    },
                    FileAction::OpenSuccess(path, content) => {
                        on_open.borrow().iter().for_each(|f| f((path.clone(), content.clone())) );
                        curr_path = (Some(path.clone()), true);
                        just_opened = true;
                        println!("Just opened set to true");
                    },
                    FileAction::OpenError(e) => {
                        println!("{}", e);
                    },

                    // Triggered when the user choses to close an unsaved file at the toast.
                    FileAction::FileCloseRequest => {
                        curr_path = (None, true);
                        match after_close {
                            ActionAfterClose::New => {
                                on_new.borrow().iter().for_each(|f| f(()) );
                                just_opened = true;
                            },
                            ActionAfterClose::Open => {
                                on_show_open.borrow().iter().for_each(|f| f(()) );
                                just_opened = true;
                            },
                            ActionAfterClose::CloseWindow => {
                                on_window_close.borrow().iter().for_each(|f| f(()) );
                            }
                        }
                    },
                    FileAction::WindowCloseRequest => {
                        if !curr_path.1.clone() {
                            after_close = ActionAfterClose::CloseWindow;
                            on_close_confirm.borrow().iter().for_each(|f| f(curr_path.0.clone().unwrap_or(String::from("Untitled.tex"))) );
                            // win_close_request = true;
                        } else {
                            on_window_close.borrow().iter().for_each(|f| f(()) );
                        }
                        // final_state.replace(recent_files.clone());
                    }
                }
                Continue(true)
            }
        });
        Self { on_open, send, on_save_unknown_path, on_buffer_read_request, on_close_confirm, on_window_close, on_new, on_save, on_file_changed, on_open_request, on_show_open }
    }

    pub fn connect_opened<F>(&self, f : F)
    where
        F : Fn((String, String)) + 'static
    {
        self.on_open.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_new<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_new.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_open_request<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_open_request.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_buffer_read_request<F>(&self, f : F)
    where
        F : Fn(())->String + 'static
    {
        self.on_buffer_read_request.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_save_unknown_path<F>(&self, f : F)
    where
        F : Fn(String)->() + 'static
    {
        self.on_save_unknown_path.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_save<F>(&self, f : F)
    where
        F : Fn(String)->() + 'static
    {
        self.on_save.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_close_confirm<F>(&self, f : F)
    where
        F : Fn(String) + 'static
    {
        self.on_close_confirm.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_file_changed<F>(&self, f : F)
    where
        F : Fn(Option<String>) + 'static
    {
        self.on_file_changed.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_window_close<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_window_close.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_show_open<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_show_open.borrow_mut().push(boxed::Box::new(f));
    }

}

pub fn connect_manager_with_file_actions(
    manager : &FileManager,
    actions : &FileActions,
    send : &glib::Sender<FileAction>,
    open_dialog : &OpenDialog
) {
    actions.new.connect_activate({
        let send = send.clone();
        move |_,_| {
            send.send(FileAction::NewRequest(false)).unwrap();
        }
    });
    actions.save.connect_activate({
        let send = send.clone();
        move |_,_| {
            send.send(FileAction::SaveRequest(None));
        }
    });
    let open_dialog = open_dialog.clone();
    actions.open.connect_activate({
        let send = send.clone();
        move |_,_| {
            send.send(FileAction::RequestShowOpen);
        }
    });
}

pub fn connect_manager_with_open_dialog(send : &glib::Sender<FileAction>, dialog : &OpenDialog) {
    let send = send.clone();
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

pub fn connect_manager_with_save_dialog(send : &glib::Sender<FileAction>, dialog : &SaveDialog) {
    let send = send.clone();
    dialog.dialog.connect_response(move |dialog, resp| {
        match resp {
            ResponseType::Accept => {
                if let Some(path) = dialog.file().and_then(|f| f.path() ) {
                    send.send(FileAction::SaveRequest(Some(path.to_str().unwrap().to_string()))).unwrap();
                }
            },
            _ => { }
        }
    });
}

pub fn connect_manager_with_editor(
    send : &glib::Sender<FileAction>,
    view : &sourceview5::View,
    ignore_file_save_action : &gio::SimpleAction
) {
    view.buffer().connect_changed({
        let send = send.clone();
        move |_| {
            send.send(FileAction::FileChanged).unwrap();
        }
    });
    ignore_file_save_action.connect_activate({
        let send = send.clone();
        move |_action, param| {
            send.send(FileAction::FileCloseRequest).unwrap();
        }
    });
}

pub fn connect_manager_with_window(send : &glib::Sender<FileAction>, window : &ApplicationWindow) {
    let send = send.clone();
    window.connect_close_request(move |_win| {
        send.send(FileAction::WindowCloseRequest).unwrap();
        glib::signal::Inhibit(true)
    });
}

impl React<MainMenu> for FileManager {

    fn react(&self, menu : &MainMenu) {
        connect_manager_with_file_actions(self, &menu.actions, &self.send, &menu.open_dialog);
    }

}

impl React<OpenDialog> for FileManager {

    fn react(&self, dialog : &OpenDialog) {
        connect_manager_with_open_dialog(&self.send, &dialog);
    }

}

impl React<SaveDialog> for FileManager {

    fn react(&self, dialog : &SaveDialog) {
        connect_manager_with_save_dialog(&self.send, &dialog);
    }

}

impl React<PapersEditor> for FileManager {

    fn react(&self, editor : &PapersEditor) {
        connect_manager_with_editor(&self.send, &editor.view, &editor.ignore_file_save_action);
    }

}

impl React<PapersWindow> for FileManager {

    fn react(&self, win : &PapersWindow) {
        connect_manager_with_window(&self.send, &win.window);
    }
}

pub fn spawn_save_file(
    path : String,
    content : String,
    send : glib::Sender<FileAction>
) -> JoinHandle<bool> {
    thread::spawn(move || {
        if let Ok(mut f) = File::create(&path) {
            if f.write_all(content.as_bytes()).is_ok() {
                send.send(FileAction::SaveSuccess(path));
                true
            } else {
                false
            }
        } else {
            println!("Unable to write into file");
            false
        }
    })
}



