use gtk4::*;
use gtk4::prelude::*;
use crate::ui::*;
use crate::tex::*;
use crate::React;
use std::ops::Range;
use std::boxed;
use crate::Callbacks;

pub enum AnalyzerAction {
    TextChanged(String),

    // Item selected from the left sidebar. Calculate char position from byte offset at current
    // document model. Then calculate line from char offset. Propagate line to editor, so the
    // mark can be positioned there.
    ItemSelected(Vec<usize>)

}

pub struct Analyzer {

    send : glib::Sender<AnalyzerAction>,

    on_reference_changed : Callbacks<Difference>,

    on_section_changed : Callbacks<Difference>,

    on_doc_changed : Callbacks<Document>,

    on_doc_cleared : Callbacks<()>,

    on_refs_cleared : Callbacks<()>,

    on_refs_validated : Callbacks<()>,

    on_doc_error : Callbacks<TexError>,

    on_line_selection : Callbacks<usize>

}

impl Analyzer {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<AnalyzerAction>(glib::PRIORITY_DEFAULT);
        let on_reference_changed : Callbacks<Difference> = Default::default();
        let on_section_changed : Callbacks<Difference> = Default::default();
        let on_doc_changed : Callbacks<Document> = Default::default();
        let on_line_selection : Callbacks<usize> = Default::default();
        let on_doc_error : Callbacks<TexError> = Default::default();
        let on_doc_cleared : Callbacks<()> = Default::default();
        let on_refs_cleared : Callbacks<()> = Default::default();
        let on_refs_validated : Callbacks<()> = Default::default();
        // TODO keep an thread watching an external bib file (if any). The user can simply use
        // the embedded bibliography instead.

        recv.attach(None, {
            let mut tk_info = TokenInfo::default();
            let mut doc = Document::default();
            let mut last_err : Option<TexError> = None;
            let on_reference_changed = on_reference_changed.clone();
            let on_section_changed = on_section_changed.clone();
            let on_doc_changed = on_doc_changed.clone();
            let on_line_selection = on_line_selection.clone();
            let on_doc_cleared = on_doc_cleared.clone();
            let on_doc_error = on_doc_error.clone();
            let on_refs_cleared = on_refs_cleared.clone();
            let on_refs_validated = on_refs_validated.clone();
            move |action| {
                match action {
                    AnalyzerAction::TextChanged(new_txt) => {
                        println!("Text changed");
                        match Lexer::scan(&new_txt[..]).map(|tks| tks.to_owned() ) {
                            Ok(new_info) => {

                                //for diff in tk_info.compare_tokens(&new_info, Comparison::Sections) {
                                //    on_section_changed.borrow().iter().for_each(|f| f(diff.clone() ) );
                                //}

                                if new_info.references().is_empty() {
                                    on_refs_cleared.borrow().iter().for_each(|f| f(()) );
                                    println!("References cleared");
                                } else {

                                    // Old tkinfo had empty references, but new one is not empty.
                                    if tk_info.references().is_empty() {
                                        on_refs_validated.borrow().iter().for_each(|f| f(()) );
                                        println!("References validated");
                                    }

                                    for diff in tk_info.compare_tokens(&new_info, Comparison::References) {
                                        // println!("{:?}", diff);
                                        on_reference_changed.borrow().iter().for_each(|f| f(diff.clone() ) );
                                        println!("References changed");
                                    }
                                }

                                tk_info = new_info;
                                match Parser::from_tokens(tk_info.tokens()) {
                                    Ok(new_doc) => {

                                        // Always update after an error or if the token sequence changed.
                                        // If the token sequence remains the same, there is no update to
                                        // be processed.
                                        if doc != new_doc || last_err.is_some() {
                                            on_doc_changed.borrow().iter().for_each(|f| f(new_doc.clone()) );
                                        }

                                        last_err = None;
                                        doc = new_doc;
                                    }

                                    Err(e) => {
                                        last_err = Some(e.clone());
                                        println!("{}", e);
                                        doc = Document::default();
                                        on_doc_cleared.borrow().iter().for_each(|f| f(()) );
                                        on_doc_error.borrow().iter().for_each(|f| f(e.clone()) );
                                    }
                                }
                            },
                            Err(e) => {
                                last_err = Some(e.clone());
                                tk_info = TokenInfo::default();
                                doc = Document::default();
                                on_doc_cleared.borrow().iter().for_each(|f| f(()) );
                                on_doc_error.borrow().iter().for_each(|f| f(e.clone()) );
                                println!("{}", e);
                            }
                        }
                    },
                    AnalyzerAction::ItemSelected(sel_ixs) => {
                        if let Some(tk_ix) = doc.token_index_at(&sel_ixs[..]) {
                            // Count \n at all tokens before tk_ix
                            // let lines_before = tk_info.pos[0..tk_ix].iter().map(|pos| tk_info.txt[pos.clone()].chars().filter(|c| *c == '\n').count() ).sum::<usize>();
                            let lines_before = tk_info.txt[..tk_info.pos[tk_ix].start].chars().filter(|c| *c == '\n').count();

                            // Add one because we want one past the last line, add +1 because lines count from 1, not zero.
                            on_line_selection.borrow().iter().for_each(|f| f(lines_before) );
                            // println!("Token {} at line {}", tk_ix, lines_before);
                        } else {
                            println!("No token at document index {:?}", sel_ixs);
                        }
                    }
                }
                Continue(true)
            }
        });
        Self { send, on_reference_changed, on_section_changed, on_doc_changed, on_line_selection, on_doc_cleared, on_doc_error, on_refs_cleared, on_refs_validated }
    }

    pub fn connect_section_changed<F>(&self, f : F)
    where
        F : Fn(Difference) + 'static
    {
        self.on_section_changed.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_reference_changed<F>(&self, f : F)
    where
        F : Fn(Difference) + 'static
    {
        self.on_reference_changed.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_doc_changed<F>(&self, f : F)
    where
        F : Fn(Document) + 'static
    {
        self.on_doc_changed.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_doc_cleared<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_doc_cleared.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_references_cleared<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_refs_cleared.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_references_validated<F>(&self, f : F)
    where
        F : Fn(()) + 'static
    {
        self.on_refs_validated.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_doc_error<F>(&self, f : F)
    where
        F : Fn(TexError) + 'static
    {
        self.on_doc_error.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_line_selection<F>(&self, f : F)
    where
        F : Fn(usize) + 'static
    {
        self.on_line_selection.borrow_mut().push(boxed::Box::new(f));
    }

}

/*
hunspell-rs = 0.3.0
let h = Hunspell::new(affpath, dictpath);  path to the .aff file; path to the .dic file
h.check(word)
h.suggest(word)
*/

impl React<DocTree> for Analyzer  {

    fn react(&self, tree : &DocTree) {
        let send = self.send.clone();
        tree.tree_view.selection().connect_changed(move |sel| {
            if sel.count_selected_rows() > 0 {
                let (paths, _) = sel.selected_rows();
                if let Some(path) = paths.get(0) {
                    send.send(AnalyzerAction::ItemSelected(path.indices().iter().map(|ix| *ix as usize ).collect()));
                }
            }
        });
    }

}

impl React<PapersEditor> for Analyzer {

    fn react(&self, editor : &PapersEditor) {

        editor.view.buffer().connect_changed({
            let send = self.send.clone();
            let view = editor.view.clone();
            move |buffer| {
                let txt_up_to_cursor = buffer.text(
                    &buffer.start_iter(),
                    &buffer.iter_at_offset(buffer.cursor_position()),
                    true
                );
                if txt_up_to_cursor.ends_with("}") || txt_up_to_cursor.ends_with("$") {
                    let txt = buffer.text(
                        &buffer.start_iter(),
                        &buffer.end_iter(),
                        true
                    );
                    send.send(AnalyzerAction::TextChanged(txt.to_string()));
                }
            }
        });

        editor.view.buffer().connect_delete_range({
            let send = self.send.clone();
            move |buffer, _, _| {
                let txt = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true
                );
                send.send(AnalyzerAction::TextChanged(txt.to_string()));
            }
        });

        editor.view.connect_delete_from_cursor({
            let send = self.send.clone();
            move |view, _, _| {
                let buffer = view.buffer();
                let txt = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true
                );
                send.send(AnalyzerAction::TextChanged(txt.to_string()));
            }
        });

        editor.view.connect_paste_clipboard({
            let send = self.send.clone();
            let view = editor.view.clone();
            move |view| {
                let buffer = view.buffer();
                let txt = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true
                );
                send.send(AnalyzerAction::TextChanged(txt.to_string()));
            }
        });

        // TODO also update on file opened.

        /*editor.view.connect_delete_from_cursor({
            let send = self.send.clone();
            let view = editor.view.clone();
            move |view, _, _| {
                let buffer = view.buffer();
                let txt = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true
                );
                send.send(AnalyzerAction::TextChanged(txt.to_string()));
            }
        });*/

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


