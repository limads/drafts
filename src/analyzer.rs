use gtk4::*;
use gtk4::prelude::*;
use crate::ui::*;
use crate::parser::*;
use crate::React;
use std::ops::Range;

pub enum AnalyzerAction {
    TextChanged(String)
}

pub struct Analyzer {
    send : glib::Sender<AnalyzerAction>
}

impl Analyzer {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<AnalyzerAction>(glib::PRIORITY_DEFAULT);
        recv.attach(None, {
            let mut tk_info : Option<TokenInfo> = None;
            move |action| {
                match action {
                    AnalyzerAction::TextChanged(new_txt) => {
                        match Lexer::scan(&new_txt[..]).map(|tks| tks.to_owned() ) {
                            Ok(new_info) => {

                                if let Some(info) = &tk_info {
                                    for diff in info.compare_tokens(&new_info, Comparison::Sections) {
                                        println!("{:?}", diff);
                                    }
                                }

                                tk_info = Some(new_info);
                            },
                            Err(e) => {

                            }
                        }
                    }
                }
                Continue(true)
            }
        });
        Self { send }
    }

    pub fn connect_section_added() {

    }

    pub fn connect_reference_added() {

    }

}

impl React<PapersEditor> for Analyzer {

    fn react(&self, editor : &PapersEditor) {

        let send = self.send.clone();
        editor.view.buffer().connect_changed(move |buffer| {
            let txt = buffer.text(
                &buffer.start_iter(),
                &buffer.end_iter(),
                true
            );
            send.send(AnalyzerAction::TextChanged(txt.to_string()));
        });

        /*// Get cursor
        view.iter_at_mark(view.get_insert());
        // Get selection
        match buffer.selection_bounds() {
            Some((from, to,)) => {
                from.text(&to).map(|txt| txt.to_string())
            },
            None => { }
        }

        // Insert - To replace, first we insert ""

        // Also have forward.. versions.
         iter.backward_chars(n)
         iter.backward_cursor_position()
         iter.backward_cursor_positions(n)
         iter.backward_find_char
         iter.forward_to_line_end

         buffer.delete(start_iter, end_iter);
         buffer.insert(&mut textiter, "mytext");
         buffer.insert_at_cursor("mytext");
         buffer.select_range(start, end);
        */
    }

}


