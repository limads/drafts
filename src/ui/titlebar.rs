use super::*;

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
    pub sidebar_hide_action : gio::SimpleAction
}

impl Titlebar {

    pub fn build() -> Self {
        let header = HeaderBar::new();
        let menu_button = MenuButton::builder().icon_name("open-menu-symbolic").build();

        let pdf_btn = Button::builder().icon_name("evince-symbolic").build();
        let sidebar_toggle = ToggleButton::builder().icon_name("view-sidebar-symbolic").build();
        let fmt_popover = Popover::new();
        let bx = Box::new(Orientation::Vertical, 0);

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
        bib_list.set_width_request(100);
        bib_list.set_height_request(100);

        let bx = Box::new(Orientation::Vertical, 0);
        bib_popover.set_child(Some(&bx));
        bx.append(&search_entry);
        bx.append(&bib_list);

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
        menu.append_item(&gio::MenuItem::new(Some("Section"), Some("win.section")));
        menu.append_item(&gio::MenuItem::new(Some("List"), Some("win.list")));
        menu.append_item(&gio::MenuItem::new(Some("Image"), Some("win.image")));
        menu.append_item(&gio::MenuItem::new(Some("Table"), Some("win.table")));
        menu.append_item(&gio::MenuItem::new(Some("Code"), Some("win.code")));

        let math_submenu = gio::Menu::new();
        math_submenu.append_item(&gio::MenuItem::new(Some("Bracket"), Some("win.matrix")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Symbol"), Some("win.symbol")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Function"), Some("win.symbol")));
        math_submenu.append_item(&gio::MenuItem::new(Some("Operatior"), Some("win.symbol")));

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
        Self { main_menu, header, menu_button, pdf_btn, sidebar_toggle, sidebar_hide_action }
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

