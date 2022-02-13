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
use std::time::SystemTime;
use glib::signal::SignalHandlerId;

#[derive(Clone, Copy)]
pub enum FileState {
    New,
    Editing,
    Open,
    CloseWindow
}

pub enum FileAction {

    // Whether to force or not
    NewRequest(bool),

    SaveRequest(Option<String>),

    SaveSuccess(String),

    SaveError(String),

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
    on_show_open : Callbacks<()>,
    on_error : Callbacks<String>
}

// If file was created via "New" action, path will be None and last_saved will be None.
// If file was opened, path will be Some(path) and last_saved will be None. Every time
// the file is saved via "Save" action, last_saved will be updated and the path is
// persisted. If "Save as" is called, the last_saved AND the path are updated to the
// new path. The "SaveAs" is detected by the difference between the requested and
// actually-held paths.
#[derive(Clone, Debug, Default)]
pub struct OpenedFile {

    pub last_saved : Option<SystemTime>,

    pub path : Option<String>,

    pub just_opened : bool

}

impl OpenedFile {

    pub fn reset(&mut self) {
        self.path = None;
        self.last_saved = Some(SystemTime::now());
        self.just_opened = true;
    }

    pub fn path_or_untitled(&self) -> String {
        self.path.clone().unwrap_or(String::from("Untitled.tex"))
    }

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
        let on_error : Callbacks<String> = Default::default();
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
            let on_error = on_error.clone();

            // Holds an action that should happen after the currently-opened file is closed.
            // This variable is updated at NewRequest, OpenRequest and WindowCloseRequest.
            let mut file_state = FileState::New;

            // Holds optional path and whether the file is saved.
            let mut curr_file : OpenedFile = Default::default();
            curr_file.reset();

            move |action| {
                match action {

                    // To be triggered when "new" action is activated on the main menu.
                    FileAction::NewRequest(force) => {

                        // User requested to create a new file, but the current file has unsaved changes.
                        if !force && !curr_file.last_saved.is_some() {
                            file_state = FileState::New;
                            call(&on_close_confirm, curr_file.path_or_untitled());

                        // User requested to create a new file by clicking the "discard" at the toast
                        // (or there isn't a currently opened path).
                        } else {
                            curr_file.reset();
                            call(&on_new, ());
                            println!("Just opened set to true");
                        }
                    },
                    FileAction::SaveRequest(opt_path) => {
                        if let Some(path) = opt_path {
                            let content = on_buffer_read_request.borrow()[0](());
                            spawn_save_file(path, content, send.clone());
                        } else {
                            if let Some(path) = curr_file.path.clone() {
                                let content = on_buffer_read_request.borrow()[0](());
                                spawn_save_file(path, content, send.clone());
                            } else {
                                call(&on_save_unknown_path, String::new());
                            }
                        }
                    },

                    // Called when the buffer changes. Ideally, when the user presses a key to
                    // insert a character. But also when the buffer is changed after a new template is
                    // loaded or a file is opened, which is why the callback is only triggered when
                    // just_opened is false.
                    FileAction::FileChanged => {

                        println!("File changed");

                        // Use this decision branch to inhibit buffer changes
                        // when a new file is opened.
                        if curr_file.just_opened {
                            curr_file.just_opened = false;
                            // println!("(File changed) Just opened set to false");
                        }

                        //else {
                        curr_file.last_saved = None;
                        println!("File changed by key press");
                        call(&on_file_changed, curr_file.path.clone());
                        //}
                    },
                    FileAction::SaveSuccess(path) => {
                        curr_file.path = Some(path.clone());
                        curr_file.last_saved = Some(SystemTime::now());
                        call(&on_save, path.clone());
                    },
                    FileAction::SaveError(msg) => {
                        println!("{}", msg);
                        call(&on_error, msg.clone());
                    },
                    FileAction::RequestShowOpen => {
                        if curr_file.last_saved.is_some() {
                            call(&on_show_open, ());
                        } else {
                            file_state = FileState::Open;
                            call(&on_close_confirm, curr_file.path_or_untitled());
                        }
                    },
                    FileAction::OpenRequest(path) => {

                        // User tried to open an already-opened file. Ignore the request in this case.
                        if let Some(curr_path) = &curr_file.path {
                            if &curr_path[..] == path {
                                return Continue(true);
                            }
                        }

                        spawn_open_file(path, send.clone());

                        // Just opened should be set here (before the confirmation of the open thread)
                        // because the on_open
                        // curr_file.just_opened = true;
                    },
                    FileAction::OpenSuccess(path, content) => {

                        // It is critical that just_opened is set to true before calling the on_open,
                        // because we must ignore the change to the sourceview buffer.
                        println!("Just opened set to true");
                        curr_file.just_opened = true;
                        curr_file.path = Some(path.clone());
                        curr_file.last_saved = Some(SystemTime::now());

                        call(&on_open, (path.clone(), content.clone()));

                        // println!("Just opened set to true");
                    },

                    FileAction::OpenError(e) => {
                        println!("{}", e);
                        call(&on_error, e.clone());
                    },

                    // Triggered when the user choses to close an unsaved file at the toast.
                    FileAction::FileCloseRequest => {
                        curr_file.reset();
                        match file_state {
                            FileState::New => {
                                call(&on_new, ());
                                curr_file.just_opened = true;
                            },
                            FileState::Open => {
                                call(&on_show_open, ());
                                curr_file.just_opened = true;
                            },
                            FileState::CloseWindow => {
                                call(&on_window_close, ());
                            },
                            FileState::Editing => {

                            }
                        }
                    },
                    FileAction::WindowCloseRequest => {
                        if !curr_file.last_saved.is_some() {
                            file_state = FileState::CloseWindow;
                            call(&on_close_confirm, curr_file.path_or_untitled());
                        } else {
                            call(&on_window_close, ());
                        }
                    }
                }
                Continue(true)
            }
        });
        Self {
            on_open,
            send,
            on_save_unknown_path,
            on_buffer_read_request,
            on_close_confirm,
            on_window_close,
            on_new,
            on_save,
            on_file_changed,
            on_open_request,
            on_show_open,
            on_error
        }
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

        let mut on_buffer_read_request = self.on_buffer_read_request.borrow_mut();

        // Can only connect once, since connecting multiple would assume the data
        // could be read from more than one source. But we have a single sourceview.
        assert!(on_buffer_read_request.len() == 0);

        on_buffer_read_request.push(boxed::Box::new(f));
    }

    pub fn connect_save_unknown_path<F>(&self, f : F)
    where
        F : Fn(String)->() + 'static
    {
        self.on_save_unknown_path.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_error<F>(&self, f : F)
    where
        F : Fn(String)->() + 'static
    {
        self.on_error.borrow_mut().push(boxed::Box::new(f));
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

/// Spawns thread to open a filesystem file. The result of the operation will
/// be sent back to the main thread via the send glib channel.
fn spawn_open_file(path : String, send : glib::Sender<FileAction>) -> JoinHandle<bool> {
    thread::spawn(move || {
        match File::open(&path) {
            Ok(mut f) => {
                let mut content = String::new();
                match f.read_to_string(&mut content) {
                    Ok(_) => {
                        if let Err(e) = send.send(FileAction::OpenSuccess(path.to_string(), content)) {
                            println!("{}", e);
                        }
                        true
                    },
                    Err(e) => {
                        if let Err(e) = send.send(FileAction::OpenError(format!("{}", e ))) {
                            println!("{}", e);
                        }
                        false
                    }
                }
            },
            Err(e) => {
                if let Err(e) = send.send(FileAction::OpenError(format!("{}", e ))) {
                    println!("{}", e);
                }
                false
            }
        }
    })
}

pub fn spawn_save_file(
    path : String,
    content : String,
    send : glib::Sender<FileAction>
) -> JoinHandle<bool> {
    thread::spawn(move || {
        match File::create(&path) {
            Ok(mut f) => {
                match f.write_all(content.as_bytes()) {
                    Ok(_) => {
                        send.send(FileAction::SaveSuccess(path));
                        true
                    },
                    Err(e) => {
                        send.send(FileAction::SaveError(format!("{}",e )));
                        false
                    }
                }
            }
            Err(e) => {
                send.send(FileAction::SaveError(format!("{}",e )));
                false
            }
        }
    })
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
) -> SignalHandlerId {
    ignore_file_save_action.connect_activate({
        let send = send.clone();
        move |_action, param| {
            send.send(FileAction::FileCloseRequest).unwrap();
        }
    });
    view.buffer().connect_changed({
        let send = send.clone();
        move |buf| {
            println!("Buffer changed to {}", buf.text(&buf.start_iter(), &buf.end_iter(), false));
            send.send(FileAction::FileChanged).unwrap();
        }
    })
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
        *(editor.buf_change_handler.borrow_mut()) = Some(connect_manager_with_editor(&self.send, &editor.view, &editor.ignore_file_save_action));
    }

}

impl React<PapersWindow> for FileManager {

    fn react(&self, win : &PapersWindow) {
        connect_manager_with_window(&self.send, &win.window);
    }
}



