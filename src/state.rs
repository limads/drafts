use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};
use stateful::React;
use std::ops::Deref;
use crate::ui::PapersWindow;
use std::thread;
use stateful::PersistentState;
use gtk4::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerState {
    pub paned : filecase::PanedState,
    pub window : filecase::WindowState
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
            window : filecase::WindowState { width : 1024, height : 768 }
        })))
    }

}

impl React<crate::ui::PapersWindow> for PapersState {

    fn react(&self, win : &PapersWindow) {
        let state = self.clone();
        // let main_paned = win.editor.paned.clone();
        let sidebar_paned = win.editor.sub_paned.clone();
        win.window.connect_close_request(move |win| {
            let mut state = state.borrow_mut();
            filecase::set_win_dims_on_close(&win, &mut state.window);
            // filecase::set_paned_on_close(&main_paned, &sidebar_paned, &mut state.paned);
            gtk4::Inhibit(false)
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
        // papers_win.editor.paned.set_position(state.paned.primary);
        // papers_win.editor.sub_paned.set_position(state.paned.secondary);
        papers_win.window.set_default_size(state.window.width, state.window.height);
        /*if state.paned.primary == 0 {
            papers_win.titlebar.sidebar_toggle.set_active(false);
        } else {
            papers_win.titlebar.sidebar_toggle.set_active(true);
        }*/

        // Updating settings winndow goes here.
    }

}

