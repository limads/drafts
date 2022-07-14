use super::*;
use crate::analyzer::Analyzer;
use glib::signal::SignalHandlerId;

#[derive(Debug, Clone)]
pub struct PapersEditor {
    pub view : View,
    pub scroll : ScrolledWindow,
    pub overlay : libadwaita::ToastOverlay,

    // Holds overlay with overview + sub_paned
    pub paned : Paned,

    // Holds Sourceview scroll + PDF viewer scroll
    pub sub_paned : Paned,

    pub ignore_file_save_action : gio::SimpleAction,
    pub buf_change_handler : Rc<RefCell<Option<SignalHandlerId>>>,
    pub curr_toast : Rc<RefCell<Option<libadwaita::Toast>>>,

    pub pdf_viewer : PdfViewer
}

/*
TODO configure set_enable_undo AND max_undo_levels
*/

// set_right_margin
// set_top_margin
// set_left_margin
// set_bottom_margin

const TEXT_WIDTH : i32 = 820;

const TEXT_VERTICAL_PADDING : i32 = 98;

impl PapersEditor {

    pub fn build() -> Self {
        let view = View::new();
        view.set_hexpand(true);

        // HAlign center : Guarantees the text is centered when we are
        // in distraction-free mode, and the scrollbar is always visible.
        // Halign fill: Avoids hiding the scrollbar when the pdf viewer and
        // overview are both present.
        view.set_halign(Align::Center);

        configure_view(&view);

        // The width request guarantees the text is not wrapped while the
        // center paned is moved (it is simply hidden for very extreme positions).
        // The align center guarantees the margins are aligned.

        let scroll = ScrolledWindow::new();

        view.set_margin_top(TEXT_VERTICAL_PADDING);
        view.set_margin_bottom(TEXT_VERTICAL_PADDING);

        let provider = CssProvider::new();
        provider.load_from_data("* { background-color : #ffffff; } ".as_bytes());

        // scroll.set_kinetic_scrolling(false);

        scroll.style_context().add_provider(&provider, 800);
        scroll.set_child(Some(&view));

        // Guarantees a good portion of the text is always visible.
        view.set_width_request(TEXT_WIDTH);

        // Setting this here prevents the viewer scroll from having a horizontal
        // scroll bar, always favoring the center content instead of the sidebar
        // and typesetter content. BUT the scroll will be hidden under the expanded
        // side widget, and the text won't wrap.
        // scroll.set_width_request(TEXT_WIDTH);

        let overlay = libadwaita::ToastOverlay::builder().opacity(1.0).visible(true).build();

        let sub_paned = Paned::new(Orientation::Horizontal);
        sub_paned.set_shrink_start_child(false);
        sub_paned.set_resize_start_child(false);
        // sub_paned.set_sensitive(false);
        let pdf_viewer = PdfViewer::new();
        sub_paned.set_start_child(&scroll);
        sub_paned.set_end_child(&pdf_viewer.scroll);

        /*sub_paned.connect_visible_notify(|sub_paned| {
            let w = sub_paned.parent().unwrap().allocation().width;
            println!("{}", w);
            sub_paned.set_position(w);
        });*/

        sub_paned.connect_accept_position(move |paned| {
            println!("Paned position accept");
            false
        });

        overlay.set_child(Some(&sub_paned));

        let ignore_file_save_action = gio::SimpleAction::new("ignore_file_save", None);
        let curr_toast : Rc<RefCell<Option<libadwaita::Toast>>> = Rc::new(RefCell::new(None));

        let paned = Paned::new(Orientation::Horizontal);

        // paned.connect_position_set_notify(move |paned| {
        // Move other paned by the same ammount.
        // });

        Self { scroll, view, overlay, paned, sub_paned, ignore_file_save_action, buf_change_handler : Rc::new(RefCell::new(None)), curr_toast, pdf_viewer }
    }
}

impl React<FileManager> for PapersEditor {

    fn react(&self, manager : &FileManager) {
        archiver::connect_manager_to_editor(manager, &self.view, &self.buf_change_handler);
        manager.connect_close_confirm({
            let overlay = self.overlay.clone();
            let curr_toast = self.curr_toast.clone();
            move |file| {
                let mut last_toast = curr_toast.borrow_mut();
                if let Some(t) = last_toast.take() {
                    t.dismiss();
                }
                let toast = libadwaita::Toast::builder()
                    .title(&format!("{} has unsaved changes", file))
                    .button_label("Close anyway")
                    .action_name("win.ignore_file_save")
                    .priority(libadwaita::ToastPriority::High)
                    .timeout(0)
                    .build();
                connect_toast_dismissed(&toast, &curr_toast);
                overlay.add_toast(&toast);
                *last_toast = Some(toast);
            }
        });
        manager.connect_error({
            let overlay = self.overlay.clone();
            let curr_toast = self.curr_toast.clone();
            move |msg| {
                let mut last_toast = curr_toast.borrow_mut();
                if let Some(t) = last_toast.take() {
                    t.dismiss();
                }
                let toast = libadwaita::Toast::builder()
                    .title(&msg)
                    .priority(libadwaita::ToastPriority::High)
                    .timeout(0)
                    .build();
                connect_toast_dismissed(&toast, &curr_toast);
                overlay.add_toast(&toast);
                *last_toast = Some(toast);
            }
        });
    }

}

impl React<Typesetter> for PapersEditor {

    fn react(&self, typesetter : &Typesetter) {
        typesetter.connect_error({
            let overlay = self.overlay.clone();
            let curr_toast = self.curr_toast.clone();
            move |e| {
                let mut last_toast = curr_toast.borrow_mut();
                if let Some(t) = last_toast.take() {
                    t.dismiss();
                }
                let toast = libadwaita::Toast::builder()
                    .title(&e)
                    .priority(libadwaita::ToastPriority::High)
                    .timeout(0)
                    .build();
                connect_toast_dismissed(&toast, &curr_toast);
                overlay.add_toast(&toast);
            }
        });
    }

}

fn insert_at_cursor(view : View, popover : Popover, txt : &str) {
    let buffer = view.buffer();
    buffer.insert_at_cursor(&txt);
    popover.popdown();
    view.grab_focus();

    if txt.ends_with("{}") {
        let iter = buffer.iter_at_offset(buffer.cursor_position() - 1);
        buffer.place_cursor(&iter);
    }

}

pub fn insert_at_cursor_from_action(action : &gio::SimpleAction, view : View, popover : Popover, txt : &'static str) {
    action.connect_activate(move |_, _|{
        insert_at_cursor(view.clone(), popover.clone(), txt);
    });
}

// just insert the given text at cursor.
pub fn insert_at_cursor_from_btn(btn : &Button, view : View, popover : Popover, txt : &'static str) {
    btn.connect_clicked(move|btn|{
        insert_at_cursor(view.clone(), popover.clone(), txt);
    });
}

/// Completely replaces the selected string (if any), or just insert the given text at cursor.
pub fn edit_or_insert_at_cursor(view : &View, txt : &str) {
    let buffer = view.buffer();
    if let Some((mut start, mut end)) = buffer.selection_bounds() {
        buffer.delete(&mut start, &mut end);
        buffer.insert(&mut start, txt);
    } else {
        buffer.insert_at_cursor(&txt);
    }
}

fn wrap_parameter_or_insert_at_cursor(view : View, popover : Popover, tag : &'static str) {
    let buffer = view.buffer();
    let txt = if let Some((start, end)) = buffer.selection_bounds() {
        let prev = buffer.text(&start, &end, true).to_string();
        format!("\\{}{{{}}}", tag, prev)
    } else {
        format!("\\{}{{}}", tag)
    };
    edit_or_insert_at_cursor(&view, &txt[..]);
    popover.popdown();
    view.grab_focus();

    if txt.ends_with("{}") {
        let iter = buffer.iter_at_offset(buffer.cursor_position() - 1);
        buffer.place_cursor(&iter);
    }
}

/* Given a command tag such as \textbf{SomeText}, wrap the selected text as the argument to the given
command, or just insert the empty command if no text is selected */
pub fn wrap_parameter_or_insert_at_cursor_from_btn(btn : &Button, view : View, popover : Popover, tag : &'static str) {
    btn.connect_clicked(move |_| {
        wrap_parameter_or_insert_at_cursor(view.clone(), popover.clone(), tag);
    });
}

pub fn wrap_parameter_or_insert_at_cursor_from_action(action : &gio::SimpleAction, view : View, popover : Popover, tag : &'static str) {
    action.connect_activate(move |_, _| {
        wrap_parameter_or_insert_at_cursor(view.clone(), popover.clone(), tag);
    });
}

/// Wraps a command that can be used as an environment if nothing is selected, but wraps the
/// text in a block if there is something selected.
pub fn environment_or_wrap_at_block(btn : &Button, view : View, popover : Popover, tag : &'static str) {
    btn.connect_clicked(move |_| {
        let buffer = view.buffer();
        let txt = if let Some((start, end)) = buffer.selection_bounds() {
            let prev = buffer.text(&start, &end, true).to_string();
            format!("\\begin{{{}}}\n{}\n\\end{{{}}}", tag, prev, tag)
        } else {
            format!("\\{}", tag)
        };
        edit_or_insert_at_cursor(&view, &txt[..]);
        popover.popdown();
        view.grab_focus();
    });
}

/* Given arbitrary characters, either insert them or, if there is some selected text,
wrap the text between the two tags. */
pub fn enclose_or_insert_at_cursor(btn : &Button, view : View, popover : Popover, start_tag : &'static str, end_tag : &'static str) {
    btn.connect_clicked(move |_| {
        let buffer = view.buffer();
        let txt = if let Some((start, end)) = buffer.selection_bounds() {
            let prev = buffer.text(&start, &end, true).to_string();
            format!("{}{}{}", start_tag, prev, end_tag)
        } else {
            format!("{}{}", start_tag, end_tag)
        };
        edit_or_insert_at_cursor(&view, &txt[..]);
        popover.popdown();
        view.grab_focus();
    });
}

impl React<Titlebar> for PapersEditor {

    fn react(&self, titlebar : &Titlebar) {
        let hide_action = titlebar.sidebar_hide_action.clone();
        let paned = self.paned.clone();
        titlebar.sidebar_toggle.connect_toggled(move |btn| {
            if btn.is_active() {
                let sz = hide_action.state().unwrap().get::<i32>().unwrap();
                if sz > 0 {
                    paned.set_position(sz);
                } else {
                    paned.set_position(320);
                }
            } else {
                hide_action.set_state(&paned.position().to_variant());
                paned.set_position(0);
            }
        });
        titlebar.pdf_btn.connect_toggled({
            let sub_paned = self.sub_paned.clone();
            move |btn| {
                if !btn.is_active() {
                    sub_paned.set_position(sub_paned.allocation().width);
                }
                // sub_paned.set_sensitive(false);
            }
        });
        let view = &self.view;
        let popover = &titlebar.fmt_popover.popover;

        let fmt = [
            (&titlebar.fmt_popover.bold_btn, "textbf"),
            (&titlebar.fmt_popover.underline_btn, "underline"),
            (&titlebar.fmt_popover.italic_btn, "textit"),
            (&titlebar.fmt_popover.strike_btn, "sout"),
            (&titlebar.fmt_popover.sub_btn, "textsubscript"),
            (&titlebar.fmt_popover.sup_btn, "textsuperscript"),
            (&titlebar.fmt_popover.small_btn, "small"),
            (&titlebar.fmt_popover.normal_btn, "normalsize"),
            (&titlebar.fmt_popover.large_btn, "large"),
            (&titlebar.fmt_popover.huge_btn, "huge")
        ];
        for (btn, cmd) in fmt {
            wrap_parameter_or_insert_at_cursor_from_btn(btn, view.clone(), popover.clone(), cmd);
        }

        let par = [
            (&titlebar.fmt_popover.par_indent_10,"\\setlength{\\parindent}{10pt}"),
            (&titlebar.fmt_popover.par_indent_15, "\\setlength{\\parindent}{15pt}"),
            (&titlebar.fmt_popover.par_indent_20, "\\setlength{\\parindent}{20pt}"),
            (&titlebar.fmt_popover.line_height_10, "\\linespread{1.0}"),
            (&titlebar.fmt_popover.line_height_15, "\\linespread{1.5}"),
            (&titlebar.fmt_popover.line_height_20, "\\linespread{2.0}"),
            (&titlebar.fmt_popover.onecol_btn, "\\onecolumn"),
            (&titlebar.fmt_popover.twocol_btn, "\\twocolumn")
        ];
        for (btn, cmd) in par {
            insert_at_cursor_from_btn(btn, view.clone(), popover.clone(), cmd);
        }

        let align = [
            (&titlebar.fmt_popover.center_btn, "center"),
            (&titlebar.fmt_popover.left_btn, "flushleft"),
            (&titlebar.fmt_popover.right_btn, "flushright")
        ];
        for (btn, cmd) in align {
            environment_or_wrap_at_block(btn, view.clone(), popover.clone(), cmd);
        }

        let sectioning = [
            (&titlebar.sectioning_actions.section, "section"),
            (&titlebar.sectioning_actions.subsection, "subsection"),
            (&titlebar.sectioning_actions.sub_subsection, "subsubsection"),
            (&titlebar.sectioning_actions.chapter, "chapter")
        ];
        for (action, cmd) in sectioning {
            wrap_parameter_or_insert_at_cursor_from_action(action, view.clone(), popover.clone(), cmd);
        }

        let block = [
            (&titlebar.block_actions.list, "\\begin{itemize}\nitem a\n\\end{itemize}"),
            (&titlebar.block_actions.verbatim, "\\begin{verbatim}\n\\end{verbatim}"),
            (&titlebar.block_actions.eq, "$$\n$$"),
            (&titlebar.block_actions.tbl, "\\begin{tabular}{ |c|c| }\\hline\na & b & \\\\ c & d \n\\end{tabular}"),
            (&titlebar.block_actions.bib, "\\begin{filecontents}{references.bib}\n\\end{filecontents}\n\\bibliography{references}")
        ];
        for (action, cmd) in block {
            insert_at_cursor_from_action(action, view.clone(), popover.clone(), cmd);
        }

        let layout = [
            (&titlebar.layout_actions.page_break, "\\clearpage"),
            (&titlebar.layout_actions.line_break, "\\newline"),
            (&titlebar.layout_actions.vertical_space, "\\vspace{1cm}"),
            (&titlebar.layout_actions.horizontal_space, "\\hspace{1cm}"),
            (&titlebar.layout_actions.vertical_fill, "\\vfill"),
            (&titlebar.layout_actions.horizontal_fill, "\\hfill")
        ];
        for (action, cmd) in layout {
            insert_at_cursor_from_action(action, view.clone(), popover.clone(), cmd);
        }

        let meta = [
            (&titlebar.meta_actions.author, "author"),
            (&titlebar.meta_actions.date, "date"),
            (&titlebar.meta_actions.title, "title")
        ];
        for (action, cmd) in meta {
            wrap_parameter_or_insert_at_cursor_from_action(action, view.clone(), popover.clone(), cmd);
        }

        let indexing = [
            (&titlebar.indexing_actions.toc, "\\tableofcontents"),
            (&titlebar.indexing_actions.lof, "\\listoffigures"),
            (&titlebar.indexing_actions.lot, "\\listoftables")
        ];
        for (action, cmd) in indexing {
            insert_at_cursor_from_action(action, view.clone(), popover.clone(), cmd);
        }

        titlebar.paper_popover.update_btn.connect_clicked({
            let paper_combo = titlebar.paper_popover.paper_combo.clone();
            let left_entry = titlebar.paper_popover.left_entry.clone();
            let top_entry = titlebar.paper_popover.top_entry.clone();
            let right_entry = titlebar.paper_popover.right_entry.clone();
            let bottom_entry = titlebar.paper_popover.bottom_entry.clone();
            let view = view.clone();
            let popover = titlebar.paper_popover.popover.clone();
            move |_| {

                // TODO support
                // \usepackage[a4paper, total={6in, 8in}, left=2cm, right=2cm, top=2cm, bottom=2cm]{geometry}
                // \usepackage[legalpaper, landscape, margin=2in]{geometry}

                let top = super::parse_int_or_float(&top_entry.buffer().text().to_string()).unwrap_or(2.);
                let left = super::parse_int_or_float(&left_entry.buffer().text().to_string()).unwrap_or(2.);
                let bottom = super::parse_int_or_float(&bottom_entry.buffer().text().to_string()).unwrap_or(2.);
                let right = super::parse_int_or_float(&right_entry.buffer().text().to_string()).unwrap_or(2.);
                let paper = paper_combo.active_id().map(|id| id.to_string().to_lowercase() ).unwrap_or(String::from("a4"));
                let cmd = format!(
                    "\\usepackage[{}paper, left={:.1}cm, right={:.1}cm, top={:.1}cm, bottom={:.1}cm]{{geometry}}",
                    paper,
                    left,
                    right,
                    top,
                    bottom
                );
                insert_at_cursor(view.clone(), popover.clone(), &cmd);
            }
        });
    }
}

impl React<Analyzer> for PapersEditor {

    fn react(&self, analyzer : &Analyzer) {
        let view = self.view.clone();
        analyzer.connect_line_selection(move |line| {
            let buffer = view.buffer();
            if let Some(mut iter) = buffer.iter_at_line(line as i32) {
                buffer.place_cursor(&iter);
                view.scroll_to_iter(&mut iter, 0.0, true, 0.0, 0.5);
                view.grab_focus();
                println!("Cursor placed");
            } else {
                println!("No iter at line {}", line);
            }

            // view.buffer().place_cursor(&iter);
            // view.buffer().move_mark(&mark, &iter);
        });
    }
}

fn move_backwards_to_command_start(buffer : &TextBuffer) -> Option<(TextIter, TextIter, String)> {
    let pos = buffer.cursor_position();
    let pos_iter = buffer.iter_at_offset(pos);
    let mut start = buffer.iter_at_offset(pos);
    let mut ix = 0;
    let mut s = String::new();
    loop {
        ix += 1;
        start = buffer.iter_at_offset(pos-ix);
        println!("Backward = {}", s);
        s = buffer.text(&start, &pos_iter, true).to_string();
        if ix == 1 && (s.starts_with(' ') || s.starts_with('\t') || s.starts_with('\n')) {
            return None;
        }
        if s.starts_with('\n') || s.starts_with("\\") || pos - ix == 0 {
            break;
        }
    }
    if s.starts_with("\\") {
        Some((start, pos_iter, s))
    } else {
        println!("Cmd does not start with \\ but with {:?}", s.chars().next());
        None
    }
}

fn move_forward_to_command_end(buffer : &TextBuffer) -> Option<(TextIter, TextIter, String)> {
    let pos = buffer.cursor_position();
    let pos_iter = buffer.iter_at_offset(pos);
    let mut end = buffer.iter_at_offset(pos);
    let mut ix = 0;
    let mut s = String::new();
    loop {
        ix += 1;
        end = buffer.iter_at_offset(pos+ix);
        s = buffer.text(&pos_iter, &end, true).to_string();
        println!("Forward = {}", s);
        if s.ends_with('\n') || s.ends_with("}") || pos - ix == 0 {
            break;
        }
    }
    if s.ends_with("}") {
        Some((pos_iter, end, s))
    } else {
        None
    }
}

fn extend_citation(citation : &str, new_key : &str) -> Option<String>{

    // Assume the command ends precisely at }, which is 1 byte long always.
    // This is a valid step because we are already working with parsed text.
    if citation.ends_with("}") {
        Some(format!("{},{}}}", &citation[..citation.len()-1], new_key))
    } else {
        None
    }
}

impl React<BibPopover> for PapersEditor {

    fn react(&self, bib_popover : &BibPopover) {
        let search_entry = bib_popover.search_entry.clone();
        let popover = bib_popover.popover.clone();
        let view = self.view.clone();
        bib_popover.list.connect_row_activated(move |_, row| {
            let ref_row = if let Some(row) = ReferenceRow::recover(&row) {
                row
            } else {
                return;
            };
            let key = ref_row.key();
            let buffer = view.buffer();
            let replaced = match move_backwards_to_command_start(&buffer) {
                Some((mut start, mut end, start_txt)) => {
                    // println!("Start text = {}", start_txt);
                    match crate::tex::command(&start_txt[..]) {
                        Ok((_, cmd)) => {
                            if cmd.cmd == "cite" {
                                if let Some(new_citation) = extend_citation(&start_txt, &key) {
                                    buffer.delete(&mut start, &mut end);
                                    buffer.insert(&mut start, &new_citation);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        },
                        _ => {
                            match move_forward_to_command_end(&buffer) {
                                Some((_, mut end, end_txt)) => {
                                    let mut full_txt = start_txt.clone();
                                    full_txt += &end_txt;
                                    // println!("End text = {}", full_txt);
                                    match crate::tex::command(&full_txt[..]) {
                                        Ok((_, cmd)) => {
                                            if cmd.cmd == "cite" {
                                                if let Some(new_citation) = extend_citation(&full_txt, &key) {
                                                    buffer.delete(&mut start, &mut end);
                                                    buffer.insert(&mut start, &new_citation);
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            }
                                        },
                                        _ => false
                                    }
                                },
                                _ => false
                            }
                        }
                    }
                },
                _ => false
            };
            if !replaced {
                edit_or_insert_at_cursor(&view, &format!("\\cite{{{}}}", key)[..]);
            }

            /*let pos = buffer.cursor_position();
            let start = buffer.iter_at_offset(pos-1);
            let end = buffer.iter_at_offset(pos);
            let last_char = buffer.text(&start, &end, true);
            println!("Last char = {}", last_char);*/

            popover.popdown();
            view.grab_focus();
        });
    }

}

fn configure_view(view : &View) {
    let buffer = view.buffer()
        .downcast::<sourceview5::Buffer>().unwrap();
    let manager = sourceview5::StyleSchemeManager::new();
    let scheme = manager.scheme("Adwaita").unwrap();
    buffer.set_style_scheme(Some(&scheme));
    buffer.set_highlight_syntax(true);
    buffer.set_max_undo_levels(40);
    let provider = CssProvider::new();
    provider.load_from_data(b"textview { font-family: \"Sans Regular\"; font-size: 13pt; }");
    let ctx = view.style_context();
    ctx.add_provider(&provider, 800);
    let lang_manager = sourceview5::LanguageManager::default().unwrap();
    let lang = lang_manager.language("latex").unwrap();
    buffer.set_language(Some(&lang));
    view.set_tab_width(4);
    view.set_indent_width(4);
    view.set_auto_indent(true);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_highlight_current_line(false);
    view.set_indent_on_tab(true);
    view.set_show_line_marks(true);
    view.set_enable_snippets(true);
    view.set_wrap_mode(WrapMode::Word);

    // Seems to be working, but only when you click on the the word
    // and **then** press CTRL+Space (simply pressing CTRL+space does not work).
    let completion = view.completion().unwrap();
    let words = sourceview5::CompletionWords::new(Some("main"));
    words.register(&view.buffer());
    completion.add_provider(&words);
    view.set_show_line_numbers(true);
}


