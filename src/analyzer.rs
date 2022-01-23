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

    on_doc_changed : Callbacks<Document>

}

impl Analyzer {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<AnalyzerAction>(glib::PRIORITY_DEFAULT);
        let on_reference_changed : Callbacks<Difference> = Default::default();
        let on_section_changed : Callbacks<Difference> = Default::default();
        let on_doc_changed : Callbacks<Document> = Default::default();

        // TODO keep an thread watching an external bib file (if any). The user can simply use
        // the embedded bibliography instead.

        recv.attach(None, {
            let mut tk_info = TokenInfo::default();
            let mut doc = Document::default();
            let on_reference_changed = on_reference_changed.clone();
            let on_section_changed = on_section_changed.clone();
            let on_doc_changed = on_doc_changed.clone();
            move |action| {
                match action {
                    AnalyzerAction::TextChanged(new_txt) => {
                        match Lexer::scan(&new_txt[..]).map(|tks| tks.to_owned() ) {
                            Ok(new_info) => {

                                //for diff in tk_info.compare_tokens(&new_info, Comparison::Sections) {
                                //    on_section_changed.borrow().iter().for_each(|f| f(diff.clone() ) );
                                //}

                                for diff in tk_info.compare_tokens(&new_info, Comparison::References) {
                                    on_reference_changed.borrow().iter().for_each(|f| f(diff.clone() ) );
                                }

                                tk_info = new_info;
                                match Parser::from_tokens(tk_info.tokens()) {
                                    Ok(new_doc) => {
                                        println!("{:#?}", new_doc);
                                        if doc != new_doc {
                                            on_doc_changed.borrow().iter().for_each(|f| f(new_doc.clone()) );
                                        }
                                        doc = new_doc;
                                    }
                                    Err(e) => {
                                        println!("{}", e);
                                        doc = Document::default();
                                    }
                                }
                            },
                            Err(e) => {
                                tk_info = TokenInfo::default();
                                println!("{}", e);
                            }
                        }
                    },
                    AnalyzerAction::ItemSelected(sel_ixs) => {

                    }
                }
                Continue(true)
            }
        });
        Self { send, on_reference_changed, on_section_changed, on_doc_changed }
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

}

impl React<DocTree> for Analyzer  {

    fn react(&self, tree : &DocTree) {
        let send = self.send.clone();
        tree.tree_view.selection().connect_changed(move |sel| {
            let (paths, _) = sel.selected_rows();
            if let Some(path) = paths.get(0) {
                send.send(AnalyzerAction::ItemSelected(path.indices().iter().map(|ix| *ix as usize ).collect()));
            }
        });
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


