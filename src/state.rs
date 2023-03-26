/*Copyright (c) 2022 Diego da Silva Lima. All rights reserved.

This work is licensed under the terms of the GPL v3.0 License.
For a copy, see http://www.gnu.org/licenses.*/

use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};
use stateful::React;
use std::ops::Deref;
use crate::ui::PapersWindow;
use std::thread;
use stateful::PersistentState;
use gtk4::prelude::*;
use filecase::SingleArchiverImpl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerState {
    pub paned : filecase::PanedState,
    pub window : filecase::WindowState,
    pub recent_files : Vec<String>
}

impl InnerState {

    pub fn push_if_not_present(&mut self, path : &str) {
        if self.recent_files.iter().position(|f| &f[..] == path ).is_none() {
            self.recent_files.insert(0, path.to_string());
        }
    }
}

#[derive(Clone)]
pub struct PapersState(Rc<RefCell<InnerState>>);

impl Deref for PapersState {

    type Target = RefCell<InnerState>;

    fn deref(&self) -> &RefCell<InnerState> {
        &self.0
    }

}

impl Default for PapersState {

    fn default() -> Self {
        PapersState(Rc::new(RefCell::new(InnerState {
            paned : filecase::PanedState { primary : 100, secondary : 400 },
            window : filecase::WindowState { width : 1024, height : 768 },
            recent_files : Vec::new()
        })))
    }

}

impl React<crate::ui::PapersWindow> for PapersState {

    fn react(&self, win : &PapersWindow) {
        let state = self.clone();
        let sidebar_paned = win.editor.sub_paned.clone();
        win.window.connect_close_request(move |win| {
            let mut state = state.borrow_mut();
            filecase::set_win_dims_on_close(&win, &mut state.window);
            gtk4::Inhibit(false)
        });
    }
}


impl React<crate::manager::FileManager> for PapersState {

    fn react(&self, manager : &crate::manager::FileManager) {
        let state = self.clone();
        manager.connect_opened(move |(path, _)| {
            state.borrow_mut().push_if_not_present(&path);
        });
        let state = self.clone();
        manager.connect_save(move |path| {
            state.borrow_mut().push_if_not_present(&path);
        });
    }

}

impl PersistentState<PapersWindow> for PapersState {

    fn recover(path : &str) -> Option<PapersState> {
        Some(PapersState(filecase::load_shared_serializable(path)?))
    }

    fn persist(&self, path : &str) -> thread::JoinHandle<bool> {
        filecase::save_shared_serializable(&self.0, path)
    }

    fn update(&self, papers_win : &PapersWindow) {
        let state = self.borrow();
        papers_win.window.set_default_size(state.window.width, state.window.height);
        for path in state.recent_files.iter() {
            papers_win.start_screen.recent_list.add_row(&path[..], false);
        }
    }

}

