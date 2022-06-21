use papers::manager::*;
use tempfile;
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp::{PartialEq, Eq};

#[derive(Clone, Copy, Eq, PartialEq)]
enum TestAction {
    New,
    SaveUnknown,
    Opened,
    OpenRequest,
    BufferReadRequest
}

#[test]
fn archiver_transitions() {
    /*let manager = FileManager::new();
    let mut temp = Tempfile::new();
    let sequence = Rc::new(RefCell::new(Vec::new()));
    manager.connect_new({
        let sequence = sequence.clone();
        move |_| {
            sequence.borrow_mut().push(TestAction::New);
        }
    });
    manager.connect_save_unknown_path({
        let sequence = sequence.clone();
        |_| {
            sequence.borrow_mut().push(TestAction::SaveUnknownPath);
        }
    });
    manager.connect_opened(|path, content| {
        sequence.borrow_mut().push(TestAction::Opened);
    });
    manager.connect_open_request(|_| {
        sequence.borrow_mut().push(TestAction::OpenRequest);
    });
    manager.connect_buffer_read_request(|_| {
        sequence.borrow_mut().push(TestAction::BufferReadRequest);
    });
    manager.connect_close_confirm({
        move |path| {

        }
    });
    manager.connect_file_changed({
        move |opt_path| {

        }
    });
    manager.connect_window_close({
        move |_| {

        }
    });
    manager.connect_show_open({
        move |_| {

        }
    });*/
}


