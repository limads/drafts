#[test]
fn archiver_transition() {

    use crate::manager::*;
    use tempfile;
    use std::io::write;

    let manager = FileManager::new();
    let mut temp = Tempfile::new();

    const OPEN_REQUEST : u8 = 0;

    const OPEN : u8 = 1;

    const OPEN_SUCCESS : u8 = 2;

    static mut sequence : Vec<u8> = Vec::new();

    manager.connect_new(move |_| {

    });
    manager.connect_save_unknown_path(move || {

    });
    manager.connect_opened(|path, content| {
        unsafe { sequence.push(OPEN) };
    });
    manager.connect_open_request(|_| {
        unsafe { sequence.push(OPEN_REQUEST) };
    });
    manager.connect_buffer_read_request(|_| {

    });

}


