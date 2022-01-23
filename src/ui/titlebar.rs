use super::*;
use crate::analyzer::Analyzer;
use crate::tex::{Difference, BibEntry};
use crate::tex::Token;

#[derive(Debug, Clone)]
pub struct MainMenu {
    pub popover : PopoverMenu,
    pub action_new : gio::SimpleAction,
    pub action_open : gio::SimpleAction,
    pub action_save : gio::SimpleAction,
    pub action_save_as : gio::SimpleAction,
}

impl MainMenu {

    fn build() -> Self {
        let menu = gio::Menu::new();
        menu.append(Some("New"), Some("win.new_file"));
        menu.append(Some("Open"), Some("win.open_file"));
        menu.append(Some("Save"), Some("win.save_file"));
        menu.append(Some("Save as"), Some("win.save_as_file"));
        let popover = PopoverMenu::from_model(Some(&menu));
        let action_new = gio::SimpleAction::new("new_file", None);
        let action_open = gio::SimpleAction::new("open_file", None);
        let action_save = gio::SimpleAction::new("save_file", None);
        let action_save_as = gio::SimpleAction::new("save_as_file", None);
        Self { popover, action_new, action_open, action_save, action_save_as }
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
    pub bib_list : ListBox,
    pub struct_actions : StructActions,
    pub object_actions : ObjectActions,
    pub math_actions : MathActions
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

impl Titlebar {

    pub fn build() -> Self {
        let header = HeaderBar::new();
        let menu_button = MenuButton::builder().icon_name("open-menu-symbolic").build();

        let pdf_btn = Button::builder().icon_name("evince-symbolic").build();
        let sidebar_toggle = ToggleButton::builder().icon_name("view-sidebar-symbolic").build();
        let fmt_popover = Popover::new();
        let bx = Box::new(Orientation::Vertical, 0);

        /*
        text-justify-center-symbolic
        text-justify-fill-symbolic
        text-justify-left-symbolic
        text-justify-right-symbolic
        */
        let char_bx = Box::new(Orientation::Horizontal, 0);
        let bold_btn = Button::builder().icon_name("format-text-bold-symbolic").build();
        let italic_btn = Button::builder().icon_name("format-text-italic-symbolic").build();
        let underline_btn = Button::builder().icon_name("format-text-underline-symbolic").build();
        let strike_btn = Button::builder().icon_name("format-text-strikethrough-symbolic").build();
        for btn in [&bold_btn, &italic_btn, &underline_btn, &strike_btn] {
            btn.style_context().add_class("flat");
            char_bx.append(btn);
        }
        bx.append(&Label::new(Some("Character")));
        bx.append(&char_bx);
        fmt_popover.set_child(Some(&bx));

        let fmt_btn = MenuButton::new();
        fmt_btn.set_popover(Some(&fmt_popover));
        fmt_btn.set_icon_name("font-size-symbolic");

        let bib_popover = Popover::new();
        let search_entry = Entry::builder().primary_icon_name("search-symbolic").build();
        let bib_list = ListBox::new();
        let bib_scroll = ScrolledWindow::new();
        bib_scroll.set_child(Some(&bib_list));
        bib_scroll.set_width_request(480);
        bib_scroll.set_height_request(280);

        let bx = Box::new(Orientation::Vertical, 0);
        bib_popover.set_child(Some(&bx));
        bx.append(&search_entry);
        bx.append(&bib_scroll);

        let bib_btn = MenuButton::new();
        bib_btn.set_popover(Some(&bib_popover));
        bib_btn.set_icon_name("user-bookmarks-symbolic");
        let add_btn = MenuButton::new();

        // let bx = Box::new(Orientation::Vertical, 0);
        /*let section_btn = Button::with_label("Section");
        for btn in [&section_btn] {
            btn.style_context().add_class("flat");
            bx.append(btn);
        }*/

        let menu = gio::Menu::new();
        let struct_submenu = gio::Menu::new();
        struct_submenu.append_item(&gio::MenuItem::new(Some("Section"), Some("win.section")));
        struct_submenu.append_item(&gio::MenuItem::new(Some("Subsection"), Some("win.subsection")));
        struct_submenu.append_item(&gio::MenuItem::new(Some("List"), Some("win.list")));
        menu.append_item(&gio::MenuItem::new_submenu(Some("Structure"), &struct_submenu));

        let object_submenu = gio::Menu::new();
        object_submenu.append_item(&gio::MenuItem::new(Some("Image"), Some("win.image")));
        object_submenu.append_item(&gio::MenuItem::new(Some("Table"), Some("win.table")));
        object_submenu.append_item(&gio::MenuItem::new(Some("Code"), Some("win.code")));
        menu.append_item(&gio::MenuItem::new_submenu(Some("Object"), &object_submenu));

        let math_submenu = gio::Menu::new();
        math_submenu.append_item(&gio::MenuItem::new(Some("Symbol"), Some("win.symbol")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Operator"), Some("win.operator")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Function"), Some("win.function")));
        menu.append_item(&gio::MenuItem::new_submenu(Some("Math"), &math_submenu));

        // menu.append_item(Some("Math"), &math_submenu);
        let add_popover = PopoverMenu::from_model(Some(&menu));

        // let item = gio::MenuItem::new_submenu(Some("Call"), &submenu);
        // menu.append_item(&item);

        // let math_expander = Expander::new(Some("Math"));
        // bx.append(&math_expander);
        // add_popover.set_child(Some(&bx));

        add_btn.set_popover(Some(&add_popover));
        add_btn.set_icon_name("list-add-symbolic");

        header.pack_start(&sidebar_toggle);
        header.pack_start(&add_btn);
        header.pack_start(&fmt_btn);
        header.pack_start(&bib_btn);

        // TODO make this another option at a SpinButton.
        // let web_btn = ToggleButton::builder().icon_name("globe-symbolic").build();

        header.pack_end(&menu_button);
        header.pack_end(&pdf_btn);
        // header.pack_end(&web_btn);

        let sidebar_hide_action = gio::SimpleAction::new_stateful("sidebar_hide", None, &(0).to_variant());
        let main_menu = MainMenu::build();
        menu_button.set_popover(Some(&main_menu.popover));
        Self { main_menu, header, menu_button, pdf_btn, sidebar_toggle, sidebar_hide_action, bib_list, math_actions : MathActions::build(), struct_actions : StructActions::build(), object_actions : ObjectActions::build() }
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
        println!("{:?}", entry);
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
        println!("authors = {}; title = {}; key = {}", authors, title, key);
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
        row.set_child(Some(&bx));
        let ref_row = Self { row, key_label, authors_label, title_label };
        ref_row.update(entry);
        ref_row
    }

}

impl React<Analyzer> for Titlebar {

    fn react(&self, analyzer : &Analyzer) {
        let bib_list = self.bib_list.clone();
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
    }

}

