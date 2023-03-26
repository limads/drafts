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
use std::thread::JoinHandle;
use crate::ui::PapersEditor;
use std::time::SystemTime;
use glib::signal::SignalHandlerId;
use filecase::SingleArchiver;
use filecase::SingleArchiverAction;
use filecase::SingleArchiverImpl;
use stateful::React;
use filecase::{SaveDialog, OpenDialog};

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
        filecase::connect_manager_with_file_actions(&menu.actions, self.sender(), &menu.open_dialog);
    }

}

impl React<OpenDialog> for FileManager {

    fn react(&self, dialog : &OpenDialog) {
        filecase::connect_manager_with_open_dialog(self.sender(), &dialog);
    }

}

impl React<SaveDialog> for FileManager {

    fn react(&self, dialog : &SaveDialog) {
        filecase::connect_manager_with_save_dialog(self.sender(), &dialog);
    }

}

impl React<PapersEditor> for FileManager {

    fn react(&self, editor : &PapersEditor) {
        let handler = filecase::connect_manager_with_editor(self.sender(), &editor.view, &editor.ignore_file_save_action);
        *(editor.buf_change_handler.borrow_mut()) = Some(handler);
    }

}

impl React<PapersWindow> for FileManager {

    fn react(&self, win : &PapersWindow) {
        filecase::connect_manager_responds_window(self.sender(), &win.window);
        win.start_screen.recent_list.list.connect_row_activated({
            let send = self.sender().clone();
            move |_, row| {
                let child = row.child().unwrap().downcast::<Box>().unwrap();
                let lbl = PackedImageLabel::extract(&child).unwrap();
                let path = lbl.lbl.text().as_str().to_string();
                send.send(SingleArchiverAction::OpenRequest(path)).unwrap();
            }
        });
    }

}

