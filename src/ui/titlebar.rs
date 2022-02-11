use super::*;
use crate::analyzer::Analyzer;
use crate::tex::{Difference, BibEntry};
use crate::tex::Token;

#[derive(Debug, Clone)]
pub struct FileActions {
    pub new : gio::SimpleAction,
    pub open : gio::SimpleAction,
    pub save : gio::SimpleAction,
    pub save_as : gio::SimpleAction
}

impl FileActions {

    pub fn new() -> Self {
        let new = gio::SimpleAction::new("new_file", None);
        let open = gio::SimpleAction::new("open_file", None);
        let save = gio::SimpleAction::new("save_file", None);
        let save_as = gio::SimpleAction::new("save_as_file", None);
        Self { new, open, save, save_as }
    }

}

#[derive(Debug, Clone)]
pub struct MainMenu {
    pub popover : PopoverMenu,
    pub actions : FileActions,
    pub open_dialog : OpenDialog,
    pub save_dialog : SaveDialog,
}

impl MainMenu {

    fn build() -> Self {
        let menu = gio::Menu::new();
        menu.append(Some("New"), Some("win.new_file"));
        menu.append(Some("Open"), Some("win.open_file"));
        menu.append(Some("Save"), Some("win.save_file"));
        menu.append(Some("Save as"), Some("win.save_as_file"));
        let popover = PopoverMenu::from_model(Some(&menu));
        let actions = FileActions::new();
        let open_dialog = OpenDialog::build();
        let save_dialog = SaveDialog::build();
        Self { popover, actions, open_dialog, save_dialog }
    }

}

#[derive(Debug, Clone)]
pub struct Titlebar {
    pub header : HeaderBar,
    pub menu_button : MenuButton,
    pub main_menu : MainMenu,
    pub pdf_btn : Button,
    pub sidebar_toggle : ToggleButton,
    pub sidebar_hide_action : gio::SimpleAction,
    pub struct_actions : StructActions,
    pub object_actions : ObjectActions,
    pub math_actions : MathActions,
    pub fmt_popover : FormatPopover,
    pub bib_popover : BibPopover,
    pub symbol_btn : MenuButton
}

#[derive(Debug, Clone)]
pub struct StructActions {
    pub section : gio::SimpleAction,
    pub subsection : gio::SimpleAction,
    pub list : gio::SimpleAction,
}

impl StructActions {

    pub fn build() -> Self {
        let section = gio::SimpleAction::new("section", None);
        let subsection = gio::SimpleAction::new("section", None);
        let list = gio::SimpleAction::new("list", None);
        Self { section, list, subsection }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.section.clone(), self.subsection.clone(), self.list.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct ObjectActions {
    pub image : gio::SimpleAction,
    pub table : gio::SimpleAction,
    pub code : gio::SimpleAction,
}

impl ObjectActions {

    pub fn build() -> Self {
        let image = gio::SimpleAction::new("image", None);
        let table = gio::SimpleAction::new("table", None);
        let code = gio::SimpleAction::new("code", None);
        Self { image, table, code }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.image.clone(), self.table.clone(), self.code.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct MathActions {
    pub operator : gio::SimpleAction,
    pub symbol : gio::SimpleAction,
    pub function : gio::SimpleAction,
}

impl MathActions {

    pub fn build() -> Self {
        let operator = gio::SimpleAction::new("operator", None);
        let symbol = gio::SimpleAction::new("symbol", None);
        let function = gio::SimpleAction::new("function", None);
        Self { operator, symbol, function }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.operator.clone(), self.symbol.clone(), self.function.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct FormatPopover {
    pub bold_btn : Button,
    pub italic_btn : Button,
    pub underline_btn : Button,
    pub strike_btn : Button,
    pub sub_btn : Button,
    pub sup_btn : Button,
    pub popover : Popover
}

impl FormatPopover {

    pub fn build() -> Self {
        let bx = Box::new(Orientation::Vertical, 0);
        let popover = Popover::new();
        let char_bx = Box::new(Orientation::Horizontal, 0);
        let bold_btn = Button::builder().icon_name("format-text-bold-symbolic").build();
        let sub_btn = Button::builder().icon_name("subscript-symbolic").build();
        let sup_btn = Button::builder().icon_name("superscript-symbolic").build();
        let italic_btn = Button::builder().icon_name("format-text-italic-symbolic").build();
        let underline_btn = Button::builder().icon_name("format-text-underline-symbolic").build();
        let strike_btn = Button::builder().icon_name("format-text-strikethrough-symbolic").build();
        for btn in [&bold_btn, &italic_btn, &underline_btn, &strike_btn, &sub_btn, &sup_btn] {
            btn.style_context().add_class("flat");
            char_bx.append(btn);
        }
        bx.append(&Label::new(Some("Character")));
        bx.append(&char_bx);
        popover.set_child(Some(&bx));

        let par_bx = Box::new(Orientation::Vertical, 0);
        let indent_entry = Entry::new();
        indent_entry.set_primary_icon_name(Some("format-indent-more-symbolic"));
        indent_entry.set_placeholder_text(Some("Indentation (mm)"));
        par_bx.append(&indent_entry);

        // \usepackage[a4paper, total={6in, 8in}, left=2cm, right=2cm, top=2cm, bottom=2cm]{geometry}
        // \usepackage[legalpaper, landscape, margin=2in]{geometry}

        let line_height_entry = Entry::new();
        line_height_entry.set_placeholder_text(Some("Line height (em)"));
        //line_height_entry.set_primary_icon_name(Some("size-vertically-symbolic"));
        line_height_entry.set_primary_icon_name(Some("line-height-symbolic"));

        // size-horizontally-symbolic
        // size-height-symbolic

        // Select text, then set each line as a list item by clicking:
        // format-ordered-list-symbolic
        // format-unordered-list-symbolic
        par_bx.append(&line_height_entry);

        par_bx.style_context().add_class("linked");
        let alignment_bx = Box::new(Orientation::Horizontal, 0);
        let center_btn = Button::new();
        center_btn.set_icon_name("format-justify-center-symbolic");
        alignment_bx.append(&center_btn);

        let fill_btn = Button::new();
        fill_btn.set_icon_name("format-justify-fill-symbolic");
        alignment_bx.append(&fill_btn);

        let left_btn = Button::new();
        left_btn.set_icon_name("format-justify-left-symbolic");
        alignment_bx.append(&left_btn);

        let right_btn = Button::new();
        right_btn.set_icon_name("format-justify-right-symbolic");
        alignment_bx.append(&right_btn);
        alignment_bx.style_context().add_class("linked");

        bx.append(&Label::new(Some("Line")));
        bx.append(&par_bx);

        bx.append(&Label::new(Some("Alignment")));
        bx.append(&alignment_bx);

        bx.append(&Label::new(Some("Font")));
        let font_btn = FontButton::new();
        bx.append(&font_btn);

        Self { bold_btn, italic_btn, underline_btn, strike_btn, sub_btn, sup_btn, popover }
    }

}

#[derive(Debug, Clone)]
pub struct BibPopover {
    pub list : ListBox,
    pub popover : Popover,
    pub search_entry : Entry
}

impl BibPopover {

    pub fn build() -> Self {
        let popover = Popover::new();
        let search_entry = Entry::builder().primary_icon_name("search-symbolic").build();
        let list = ListBox::new();
        let bib_scroll = ScrolledWindow::new();
        bib_scroll.set_child(Some(&list));
        bib_scroll.set_width_request(480);
        bib_scroll.set_height_request(280);

        let bx = Box::new(Orientation::Vertical, 0);
        popover.set_child(Some(&bx));
        bx.append(&search_entry);
        bx.append(&bib_scroll);

        search_entry.connect_changed({
            let list = list.clone();
            move |entry| {
                let txt = entry.buffer().text().to_string().to_lowercase();
                let mut ix = 0;
                while let Some(row) = list.row_at_index(ix) {
                    if txt.is_empty() {
                        row.set_visible(true);
                    } else {
                        let ref_row = ReferenceRow::recover(&row);
                        if ref_row.key().to_lowercase().contains(&txt) ||
                            ref_row.authors().to_lowercase().contains(&txt) ||
                            ref_row.title().to_lowercase().contains(&txt) {
                            row.set_visible(true);
                        } else {
                            row.set_visible(false);
                        }
                    }
                    ix += 1;
                }
            }
        });
        BibPopover { list, popover, search_entry }
    }

}

fn build_dash(n : i32) -> Vec<f64> {
    let dash_sz = 10.0 / (n as f64);
    let mut dashes = Vec::<f64>::new();
    for _i in 1..n {
        dashes.push(dash_sz);
    }
    dashes
}

impl Titlebar {

    pub fn build() -> Self {
        let header = HeaderBar::new();
        let menu_button = MenuButton::builder().icon_name("open-menu-symbolic").build();

        let pdf_btn = Button::builder().icon_name("evince-symbolic").build();
        let sidebar_toggle = ToggleButton::builder().icon_name("view-sidebar-symbolic").build();

        // \begin{center}
        // \end{center}

        // \begin{flushleft}
        // \begin{flushright}
        // \noindent - inline command that applies to current paragarph
        // \setlength{\parindent}{20pt} - At document config.

        /*
        \usepackage{multicol}
        \begin{multicols}{2}

        \end{multicols}
        */

        let fmt_popover = FormatPopover::build();
        let fmt_btn = MenuButton::new();
        fmt_btn.set_icon_name("insert-text-symbolic");
        fmt_btn.set_popover(Some(&fmt_popover.popover));

        let bib_popover = BibPopover::build();
        let bib_btn = MenuButton::new();
        bib_btn.set_popover(Some(&bib_popover.popover));
        bib_btn.set_icon_name("user-bookmarks-symbolic");

        let page_popover = Popover::new();
        let page_bx = Box::new(Orientation::Horizontal, 1);

        let page_da = DrawingArea::new();
        page_da.set_width_request(172);
        page_da.set_height_request(144);
        page_da.set_draw_func({
            move |da, ctx, _, _| {
                ctx.save();
                let allocation = da.allocation();

                let color = 0.5843;
                ctx.set_source_rgb(color, color, color);

                ctx.set_line_width(2.0);

                let paper_x_offset = 42.0;
                let paper_y_offset = 10.0;
                let paper_width = 80.0;
                let paper_height = 130.0;
                let fold_sz = 10.0;

                // Draw paper
                ctx.move_to(paper_x_offset, paper_y_offset);
                ctx.line_to(paper_x_offset + paper_width - fold_sz, paper_y_offset);
                ctx.line_to(paper_x_offset + paper_width, paper_y_offset + fold_sz);
                ctx.line_to(paper_x_offset + paper_width, paper_y_offset + paper_height);
                ctx.line_to(paper_x_offset, paper_y_offset + paper_height);
                ctx.line_to(paper_x_offset, paper_y_offset);
                ctx.stroke();

                // Draw top-right fold at paper
                ctx.move_to(paper_x_offset + paper_width - fold_sz, paper_y_offset);
                ctx.line_to(paper_x_offset + paper_width, paper_y_offset + fold_sz);
                ctx.line_to(paper_x_offset + paper_width - fold_sz, paper_y_offset + fold_sz);
                ctx.line_to(paper_x_offset + paper_width - fold_sz, paper_y_offset);
                ctx.fill();

                let margin_left = 10.;
                let margin_right = 10.;
                let margin_bottom = 10.;
                let margin_top = 10.;

                let dashes = build_dash(2);
                ctx.set_dash(&dashes[..], 0.0);

                // Left margin
                ctx.move_to(paper_x_offset + margin_left, paper_y_offset);
                ctx.line_to(paper_x_offset + margin_left, 140.);
                ctx.stroke();

                // Right margin
                ctx.move_to(paper_x_offset + paper_width - margin_right, paper_y_offset);
                ctx.line_to(paper_x_offset + paper_width - margin_right, paper_y_offset + paper_height);
                ctx.stroke();

                // Margin top
                ctx.move_to(paper_x_offset, paper_y_offset + margin_top);
                ctx.line_to(paper_x_offset + paper_width, paper_y_offset + margin_top);
                ctx.stroke();

                // Margin bottom
                ctx.move_to(paper_x_offset, paper_y_offset + paper_height - margin_bottom);
                ctx.line_to(paper_x_offset + paper_width, paper_y_offset + paper_height - margin_bottom);
                ctx.stroke();

                ctx.restore();
            }
        });

        let top_entry = Entry::new();
        top_entry.set_primary_icon_name(Some("margin-top-symbolic"));
        top_entry.set_placeholder_text(Some("Top margin (1cm)"));

        let right_entry = Entry::new();
        right_entry.set_primary_icon_name(Some("margin-right-symbolic"));
        right_entry.set_placeholder_text(Some("Right margin (1cm)"));

        let bottom_entry = Entry::new();
        bottom_entry.set_primary_icon_name(Some("margin-bottom-symbolic"));
        bottom_entry.set_placeholder_text(Some("Bottom margin (1cm)"));

        let left_entry = Entry::new();
        left_entry.set_primary_icon_name(Some("margin-left-symbolic"));
        left_entry.set_placeholder_text(Some("Left margin (1cm)"));

        let margin_bx = Box::new(Orientation::Vertical, 0);
        margin_bx.set_margin_top(10);

        let paper_combo = ComboBoxText::new();
        paper_combo.append(None, "A4");
        paper_combo.append(None, "Letter");
        paper_combo.append(None, "Legal");

        margin_bx.style_context().add_class("linked");
        for entry in [&top_entry, &bottom_entry, &left_entry, &right_entry] {
            margin_bx.append(entry);
        }

        let page_left_bx = Box::new(Orientation::Vertical, 12);
        super::set_margins(&page_left_bx, 6, 6);

        page_left_bx.append(&page_da);
        page_left_bx.append(&paper_combo);

        let page_right_bx = Box::new(Orientation::Vertical, 12);
        super::set_margins(&page_right_bx, 6, 6);
        let update_btn = Button::new();
        update_btn.set_label("Update");

        page_right_bx.append(&margin_bx);
        page_right_bx.append(&update_btn);
        page_bx.append(&page_left_bx);
        page_bx.append(&page_right_bx);

        page_popover.set_child(Some(&page_bx));

        let page_btn = MenuButton::new();
        page_btn.set_icon_name("crop-symbolic");
        page_btn.set_popover(Some(&page_popover));

        // let bx = Box::new(Orientation::Vertical, 0);
        /*let section_btn = Button::with_label("Section");
        for btn in [&section_btn] {
            btn.style_context().add_class("flat");
            bx.append(btn);
        }*/

        let menu = gio::Menu::new();
        menu.append_item(&gio::MenuItem::new(Some("Image"), Some("win.image")));
        menu.append_item(&gio::MenuItem::new(Some("Table"), Some("win.table")));
        menu.append_item(&gio::MenuItem::new(Some("Link to resource"), Some("win.list")));
        menu.append_item(&gio::MenuItem::new(Some("Bibliography file"), Some("win.table")));

        // let struct_submenu = gio::Menu::new();

        // \author{}
        // \date{}
        // \title{}
        // \abstract{}
        /*menu.append_item(&gio::MenuItem::new_submenu(Some("Structure"), &struct_submenu));

        let object_submenu = gio::Menu::new();
        object_submenu.append_item(&gio::MenuItem::new(Some("Code"), Some("win.code")));
        menu.append_item(&gio::MenuItem::new_submenu(Some("Object"), &object_submenu));

        let math_submenu = gio::Menu::new();
        math_submenu.append_item(&gio::MenuItem::new(Some("Symbol"), Some("win.symbol")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Operator"), Some("win.operator")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Function"), Some("win.function")));
        menu.append_item(&gio::MenuItem::new_submenu(Some("Math"), &math_submenu));*/

        // menu.append_item(Some("Math"), &math_submenu);
        let add_popover = PopoverMenu::from_model(Some(&menu));

        // let item = gio::MenuItem::new_submenu(Some("Call"), &submenu);
        // menu.append_item(&item);

        // let math_expander = Expander::new(Some("Math"));
        // bx.append(&math_expander);
        // add_popover.set_child(Some(&bx));

        let add_btn = MenuButton::new();
        add_btn.set_popover(Some(&add_popover));
        add_btn.set_icon_name("mail-attachment-symbolic");

        header.pack_start(&sidebar_toggle);
        header.pack_start(&add_btn);
        header.pack_start(&fmt_btn);
        header.pack_start(&page_btn);
        header.pack_start(&bib_btn);

        // TODO make this another option at a SpinButton.
        // let web_btn = ToggleButton::builder().icon_name("globe-symbolic").build();

        header.pack_end(&menu_button);
        header.pack_end(&pdf_btn);
        // header.pack_end(&web_btn);

        let sidebar_hide_action = gio::SimpleAction::new_stateful("sidebar_hide", None, &(0).to_variant());
        let main_menu = MainMenu::build();
        menu_button.set_popover(Some(&main_menu.popover));

        //let symbol_popover = SymbolPopover::new(&editor);
        //titlebar.symbol_btn.set_popover(Some(&symbol_popover.popover));

        let symbol_btn = MenuButton::new();
        //symbol_btn.set_label("∑");
        symbol_btn.set_icon_name("equation-symbolic");
        header.pack_start(&symbol_btn);

        let org_menu = gio::Menu::new();
        org_menu.append_item(&gio::MenuItem::new(Some("Section"), Some("win.section")));
        org_menu.append_item(&gio::MenuItem::new(Some("Subsection"), Some("win.subsection")));
        org_menu.append_item(&gio::MenuItem::new(Some("List"), Some("win.list")));
        org_menu.append_item(&gio::MenuItem::new(Some("Equation"), Some("win.list")));
        org_menu.append_item(&gio::MenuItem::new(Some("Code listing"), Some("win.list")));
        org_menu.append_item(&gio::MenuItem::new(Some("Bibliography (embedded)"), Some("win.list")));
        let org_popover = PopoverMenu::from_model(Some(&org_menu));

        let org_btn = MenuButton::new();
        org_btn.set_icon_name("format-unordered-list-symbolic");
        org_btn.set_popover(Some(&org_popover));
        header.pack_start(&org_btn);

        Self { symbol_btn, main_menu, header, menu_button, pdf_btn, sidebar_toggle, sidebar_hide_action, bib_popover, math_actions : MathActions::build(), struct_actions : StructActions::build(), object_actions : ObjectActions::build(), fmt_popover }
    }
}

impl React<Typesetter> for Titlebar {

    fn react(&self, typesetter : &Typesetter) {
        let btn = self.pdf_btn.clone();
        typesetter.connect_done({
            move |_| {
                btn.set_icon_name("evince-symbolic");
                btn.set_sensitive(true);
            }
        });
        typesetter.connect_error({
            let btn = self.pdf_btn.clone();
            move |_| {
                btn.set_icon_name("evince-symbolic");
                btn.set_sensitive(true);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceRow {
    pub row : ListBoxRow,
    pub key_label : Label,
    pub authors_label : Label,
    pub title_label : Label,
}

impl ReferenceRow {

    pub fn key(&self) -> String {
        self.key_label.label().to_string().trim_start_matches("<b>").trim_end_matches("</b>").to_string()
    }

    pub fn authors(&self) -> String {
        self.authors_label.label().to_string()
    }

    pub fn title(&self) -> String {
        self.title_label.label().to_string()
    }

    // TODO add different icons for book, article, etc.

    pub fn recover(row : &ListBoxRow) -> Self {
        let bx = row.child().unwrap().downcast::<Box>().unwrap();
        let header_bx = super::get_child_by_index::<Box>(&bx, 0);
        let key_label = super::get_child_by_index::<Label>(&header_bx, 1);
        let authors_label = super::get_child_by_index::<Label>(&header_bx, 2);
        let title_label = super::get_child_by_index::<Label>(&bx, 1);
        Self { row : row.clone(), key_label, authors_label, title_label }
    }

    pub fn update(&self, entry : &BibEntry) {
        // println!("{:?}", entry);
        let key = format!("<b>{}</b>", entry.key());
        let full_title = entry.title().unwrap_or("(Untitled)").trim().to_string();

        let mut title = String::with_capacity(full_title.len());
        let mut should_break = false;
        for (ix, c) in full_title.chars().enumerate() {
            title.push(c);
            if ix > 0 && ix % 60 == 0 {
                should_break = true;
            }
            if c == ' ' && should_break {
                title.push('\n');
                should_break = false;
            }
        }
        let mut authors = entry.author().unwrap_or("(No authors)").trim()
            .trim_start_matches("{").trim_end_matches("}").trim_start_matches("{").trim_end_matches("}");

        let mut broken_authors = String::new();
        if authors.chars().count() > 60 {
            broken_authors = authors.chars().take(60).collect();
            broken_authors += "(...)";
        }
        let year = entry.year().unwrap_or("No date").trim();
        // println!("authors = {}; title = {}; key = {}", authors, title, key);
        self.key_label.set_markup(&key);
        self.authors_label.set_text(&format!("{} ({})", authors, year));
        self.title_label.set_text(&title);
    }

    pub fn build(entry : &BibEntry<'_>) -> Self {
        let bx = Box::new(Orientation::Vertical, 0);
        let key_label = Label::new(None);
        let authors_label = Label::new(None);
        let title_label = Label::new(None);
        for lbl in [&key_label, &authors_label, &title_label] {
            lbl.set_use_markup(true);
            lbl.set_halign(Align::Start);
            lbl.set_justify(Justification::Left);
        }

        let bx_header = Box::new(Orientation::Horizontal, 0);
        let icon = match entry.entry() {
            crate::tex::Entry::Book | crate::tex::Entry::Booklet => "user-bookmarks-symbolic",
            _ => "folder-documents-symbolic"
        };
        let icon = Image::from_icon_name(Some(icon));
        super::set_all_margins(&icon, 6);
        bx_header.append(&icon);
        bx_header.append(&key_label);
        key_label.set_margin_end(6);
        // key_label.set_margin_bottom(6);
        bx_header.append(&authors_label);

        bx.append(&bx_header);
        bx.append(&title_label);
        title_label.set_margin_bottom(6);
        title_label.set_margin_start(6);

        let row = ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(true);

        row.set_child(Some(&bx));
        let ref_row = Self { row, key_label, authors_label, title_label };
        ref_row.update(entry);
        ref_row
    }

}

impl React<Analyzer> for BibPopover {

    fn react(&self, analyzer : &Analyzer) {
        let bib_list = self.list.clone();
        analyzer.connect_reference_changed(move |diff| {
            match diff {
                Difference::Added(pos, txt) => {
                    match Token::from_str(&txt) {
                        Ok(Token::Reference(bib_entry, _)) => {
                            let row = ReferenceRow::build(&bib_entry);
                            bib_list.insert(&row.row, pos as i32);
                        },
                        _ => { }
                    }
                },
                Difference::Edited(pos, txt) => {
                    match Token::from_str(&txt) {
                        Ok(Token::Reference(bib_entry, _)) => {
                            if let Some(row) = bib_list.row_at_index(pos as i32) {
                                let ref_row = ReferenceRow::recover(&row);
                                ref_row.update(&bib_entry);
                            }
                        },
                        _ => { }
                    }
                },
                Difference::Removed(pos) => {
                    if let Some(row) = bib_list.row_at_index(pos as i32) {
                        bib_list.remove(&row);
                    }
                }
            }
        });
        analyzer.connect_doc_cleared({
            let list = self.list.clone();
            move |_| {
                let mut ix = 0;
                while let Some(r) = list.row_at_index(ix) {
                    list.remove(&r);
                    ix += 1;
                }
            }
        });
    }

}

