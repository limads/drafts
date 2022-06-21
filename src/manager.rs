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
use archiver::SingleArchiver;
use archiver::SingleArchiverAction;
use archiver::SingleArchiverImpl;
use stateful::React;
use archiver::{SaveDialog, OpenDialog};

pub struct FileManager(SingleArchiver);

impl FileManager {

    pub fn new() -> Self {
        FileManager(SingleArchiver::new())
    }

}

impl AsRef<SingleArchiver> for FileManager {

    fn as_ref(&self) -> &SingleArchiver {
        &self.0
    }

}

impl SingleArchiverImpl for FileManager { }

impl React<MainMenu> for FileManager {

    fn react(&self, menu : &MainMenu) {
        archiver::connect_manager_with_file_actions( /*self,*/ &menu.actions, self.sender(), &menu.open_dialog);
    }

}

impl React<OpenDialog> for FileManager {

    fn react(&self, dialog : &OpenDialog) {
        archiver::connect_manager_with_open_dialog(self.sender(), &dialog);
    }

}

impl React<SaveDialog> for FileManager {

    fn react(&self, dialog : &SaveDialog) {
        archiver::connect_manager_with_save_dialog(self.sender(), &dialog);
    }

}

impl React<PapersEditor> for FileManager {

    fn react(&self, editor : &PapersEditor) {
        let handler = archiver::connect_manager_with_editor(self.sender(), &editor.view, &editor.ignore_file_save_action);
        *(editor.buf_change_handler.borrow_mut()) = Some(handler);
    }

}

impl React<PapersWindow> for FileManager {

    fn react(&self, win : &PapersWindow) {
        archiver::connect_manager_responds_window(self.sender(), &win.window);
    }
}

