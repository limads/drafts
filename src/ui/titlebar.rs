use super::*;
use crate::analyzer::Analyzer;
use crate::tex::{Difference, BibEntry};
use crate::tex::Token;
use archiver::FileActions;
use std::borrow::Cow;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

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
        let open_dialog = OpenDialog::build("*.tex");
        let save_dialog = SaveDialog::build("*.tex");
        Self { popover, actions, open_dialog, save_dialog }
    }

}

#[derive(Debug, Clone)]
pub struct Titlebar {
    pub header : HeaderBar,
    pub menu_button : MenuButton,
    pub main_menu : MainMenu,
    pub pdf_btn : ToggleButton,
    pub sidebar_toggle : ToggleButton,
    pub sidebar_hide_action : gio::SimpleAction,
    pub sectioning_actions : SectioningActions,
    pub object_actions : ObjectActions,
    pub layout_actions : LayoutActions,
    pub block_actions : BlockActions,
    pub indexing_actions : IndexingActions,
    pub meta_actions : MetaActions,
    pub fmt_popover : FormatPopover,
    pub bib_popover : BibPopover,
    pub symbol_btn : MenuButton,
    pub paper_popover : PaperPopover,
    pub refresh_btn : Button,
    pub zoom_in_btn : Button,
    pub zoom_out_btn : Button,
    pub zoom_action : gio::SimpleAction,
    // pub hide_pdf_btn : Button,
    pub export_pdf_btn : Button,
    pub page_entry : Entry,
    pub page_button : Button
}

#[derive(Debug, Clone)]
pub struct BlockActions {
    pub list : gio::SimpleAction,
    pub verbatim : gio::SimpleAction,
    pub eq : gio::SimpleAction,
    pub bib : gio::SimpleAction,
    pub tbl : gio::SimpleAction
}

impl BlockActions {

    pub fn build() -> Self {
        let list = gio::SimpleAction::new("list", None);
        let verbatim = gio::SimpleAction::new("verbatim", None);
        let eq = gio::SimpleAction::new("eq", None);
        let bib = gio::SimpleAction::new("bib", None);
        let tbl = gio::SimpleAction::new("tbl", None);
        Self { list, verbatim, eq, bib, tbl }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.list.clone(), self.verbatim.clone(), self.eq.clone(), self.bib.clone(), self.tbl.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct LayoutActions {
    pub page_break : gio::SimpleAction,
    pub line_break : gio::SimpleAction,
    pub horizontal_space : gio::SimpleAction,
    pub vertical_space : gio::SimpleAction,
    pub horizontal_fill : gio::SimpleAction,
    pub vertical_fill : gio::SimpleAction,
}

impl LayoutActions {

    pub fn build() -> Self {
        let page_break = gio::SimpleAction::new("page_break", None);
        let line_break = gio::SimpleAction::new("line_break", None);
        let horizontal_space = gio::SimpleAction::new("horizontal_space", None);
        let vertical_space = gio::SimpleAction::new("vertical_space", None);
        let horizontal_fill = gio::SimpleAction::new("horizontal_fill", None);
        let vertical_fill = gio::SimpleAction::new("vertical_fill", None);
        Self { page_break, line_break, horizontal_space, vertical_space, horizontal_fill, vertical_fill }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.page_break.clone(), self.line_break.clone(), self.horizontal_space.clone(), self.vertical_space.clone(), self.horizontal_fill.clone(), self.vertical_fill.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct SectioningActions {
    pub chapter : gio::SimpleAction,
    pub section : gio::SimpleAction,
    pub subsection : gio::SimpleAction,
    pub sub_subsection : gio::SimpleAction,
}

impl SectioningActions {

    pub fn build() -> Self {
        let section = gio::SimpleAction::new("section", None);
        let subsection = gio::SimpleAction::new("subsection", None);
        let sub_subsection = gio::SimpleAction::new("sub_subsection", None);
        let chapter = gio::SimpleAction::new("chapter", None);
        Self { section, subsection, sub_subsection, chapter }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.section.clone(), self.subsection.clone(), self.sub_subsection.clone(), self.chapter.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct ObjectActions {
    pub image : gio::SimpleAction,
    pub table : gio::SimpleAction,
    pub link : gio::SimpleAction,
    pub latex : gio::SimpleAction,
    pub bibfile : gio::SimpleAction,
}

impl ObjectActions {

    pub fn build() -> Self {
        let image = gio::SimpleAction::new("image", None);
        let table = gio::SimpleAction::new("table", None);
        let link = gio::SimpleAction::new("link", None);
        let bibfile = gio::SimpleAction::new("bibfile", None);
        let latex = gio::SimpleAction::new("latex", None);
        Self { image, table, link, bibfile, latex }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.image.clone(), self.table.clone(), self.link.clone(), self.bibfile.clone(), self.latex.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct IndexingActions {
    pub toc : gio::SimpleAction,
    pub lof : gio::SimpleAction,
    pub lot : gio::SimpleAction,
}

impl IndexingActions {

    pub fn build() -> Self {
        let toc = gio::SimpleAction::new("toc", None);
        let lof = gio::SimpleAction::new("lof", None);
        let lot = gio::SimpleAction::new("lot", None);
        Self { toc, lof, lot }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.toc.clone(), self.lof.clone(), self.lot.clone()].into_iter()
    }

}

#[derive(Debug, Clone)]
pub struct MetaActions {
    pub author : gio::SimpleAction,
    pub date : gio::SimpleAction,
    pub title : gio::SimpleAction,
}

impl MetaActions {

    pub fn build() -> Self {
        let author = gio::SimpleAction::new("author", None);
        let date = gio::SimpleAction::new("date", None);
        let title = gio::SimpleAction::new("title", None);
        Self { author, date, title }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=gio::SimpleAction> + 'a {
        [self.author.clone(), self.date.clone(), self.title.clone()].into_iter()
    }

}
/*#[derive(Debug, Clone)]
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

}*/

#[derive(Debug, Clone)]
pub struct FormatPopover {
    pub bold_btn : Button,
    pub italic_btn : Button,
    pub underline_btn : Button,
    pub strike_btn : Button,
    pub sub_btn : Button,
    pub sup_btn : Button,
    pub small_btn : Button,
    pub normal_btn : Button,
    pub large_btn : Button,
    pub huge_btn : Button,
    pub popover : Popover,
    pub par_indent_10 : Button,
    pub par_indent_15 : Button,
    pub par_indent_20 : Button,
    pub line_height_10 : Button,
    pub line_height_15 : Button,
    pub line_height_20 : Button,
    pub onecol_btn : Button,
    pub twocol_btn : Button,
    pub left_btn : Button,
    pub right_btn : Button,
    pub center_btn : Button
}

fn build_fmt_btn(label : &str) -> Button {
    Button::builder().label(label) /*.css_classes(vec![String::from("flat")])*/ .build()
}

impl FormatPopover {

    pub fn build() -> Self {
        let bx = Box::new(Orientation::Horizontal, 24);
        let left_bx = Box::new(Orientation::Vertical, 12);
        let right_bx = Box::new(Orientation::Vertical, 12);
        bx.append(&left_bx);
        bx.append(&right_bx);
        let popover = Popover::new();
        popover.set_child(Some(&bx));

        let char_upper_bx = Box::new(Orientation::Horizontal, 0);
        let char_lower_bx = Box::new(Orientation::Horizontal, 0);

        let bold_btn = Button::builder().icon_name("format-text-bold-symbolic").build();
        let italic_btn = Button::builder().icon_name("format-text-italic-symbolic").build();
        let underline_btn = Button::builder().icon_name("format-text-underline-symbolic").build();
        let strike_btn = Button::builder().icon_name("format-text-strikethrough-symbolic").build();

        // \small \normalsize \large \huge
        // \textsubscript{}
        let sub_btn = Button::builder().icon_name("subscript-symbolic").build();

        // \textsuperscript{}
        let sup_btn = Button::builder().icon_name("superscript-symbolic").build();

        let small_btn = Button::builder().icon_name("text-small-symbolic").build();
        let normal_btn = Button::builder().icon_name("text-normal-symbolic").build();
        let large_btn = Button::builder().icon_name("text-large-symbolic").build();
        let huge_btn = Button::builder().icon_name("text-huge-symbolic").build();

        for btn in [&bold_btn, &italic_btn, &underline_btn, &strike_btn, &sup_btn] {
            btn.style_context().add_class("flat");
            char_upper_bx.append(btn);
        }

        for btn in [&small_btn, &normal_btn, &large_btn, &huge_btn, &sub_btn] {
            btn.style_context().add_class("flat");
            char_lower_bx.append(btn);
        }

        let char_bx = Box::new(Orientation::Vertical, 0);
        char_bx.append(&Label::builder().label("Character").halign(Align::Start).justify(Justification::Left).margin_bottom(6).build());
        char_bx.append(&char_upper_bx);
        char_bx.append(&char_lower_bx);

        let par_bx = Box::new(Orientation::Horizontal, 12);

        // par_bx.append(&Label::builder().label("Paragraph").halign(Align::Start).justify(Justification::Left).margin_bottom(6).build());

        let indent_entry = Entry::new();
        indent_entry.set_primary_icon_name(Some("format-indent-more-symbolic"));
        indent_entry.set_placeholder_text(Some("Indentation (mm)"));

        // par_bx.append(&indent_entry);
        //let line_height_entry = Entry::new();
        //line_height_entry.set_placeholder_text(Some("Line height (em)"));
        //line_height_entry.set_primary_icon_name(Some("line-height-symbolic"));
        // par_bx.append(&line_height_entry);

        let height_bx = Box::new(Orientation::Vertical, 0);
        let height_btn_bx = Box::new(Orientation::Horizontal, 0);
        let (line_height_10, line_height_15, line_height_20) = (build_fmt_btn("1.0"), build_fmt_btn("1.5"), build_fmt_btn("2.0"));
        for btn in [&line_height_10, &line_height_15, &line_height_20] {
            height_btn_bx.append(btn);
        }
        height_btn_bx.style_context().add_class("linked");
        let height_title_bx = PackedImageLabel::build("line-height-symbolic", "Line height");
        height_bx.append(&height_title_bx.bx);
        height_bx.append(&height_btn_bx);
        par_bx.append(&height_bx);

        let indent_bx = Box::new(Orientation::Vertical, 0);
        let indent_btn_bx = Box::new(Orientation::Horizontal, 0);
        let (par_indent_10, par_indent_15, par_indent_20) = (build_fmt_btn("10"), build_fmt_btn("15"), build_fmt_btn("20"));
        for btn in [&par_indent_10, &par_indent_15, &par_indent_20] {
            indent_btn_bx.append(btn);
        }
        indent_btn_bx.style_context().add_class("linked");
        let indent_title_bx = PackedImageLabel::build("format-indent-more-symbolic", "Indentation (pt)");
        indent_bx.append(&indent_title_bx.bx);
        indent_bx.append(&indent_btn_bx);
        par_bx.append(&indent_bx);

        let alignment_bx = Box::new(Orientation::Vertical, 0);
        alignment_bx.set_hexpand(true);
        alignment_bx.set_halign(Align::Fill);

        let alignment_inner_bx = Box::new(Orientation::Horizontal, 0);
        alignment_inner_bx.set_hexpand(true);
        alignment_inner_bx.set_halign(Align::Fill);

        alignment_bx.append(&Label::builder().label("Align").halign(Align::Start).justify(Justification::Left).margin_bottom(6).build());

        let center_btn = Button::new();
        center_btn.set_icon_name("format-justify-center-symbolic");
        center_btn.set_hexpand(true);
        center_btn.set_halign(Align::Fill);
        alignment_inner_bx.append(&center_btn);

        // Latex justifies the text by default - not needed here.
        // let fill_btn = Button::new();
        // fill_btn.set_icon_name("format-justify-fill-symbolic");
        // alignment_inner_bx.append(&fill_btn);

        /*\begin{multicols}{2}
        lots of text
        \end{multicols}*/

        // {\raggedleft Some text flushed right. }
        // {\raggedright Some text flushed right. }
        // {\centering Some text centered. }
        // Use the freeform commands (without {}) to apply to whole document. Use the
        // previous forms when the user selected some text.

        let left_btn = Button::new();
        left_btn.set_hexpand(true);
        left_btn.set_halign(Align::Fill);
        left_btn.set_icon_name("format-justify-left-symbolic");
        alignment_inner_bx.append(&left_btn);

        let right_btn = Button::new();
        right_btn.set_hexpand(true);
        right_btn.set_halign(Align::Fill);
        right_btn.set_icon_name("format-justify-right-symbolic");
        alignment_inner_bx.append(&right_btn);

        alignment_inner_bx.style_context().add_class("linked");
        alignment_bx.append(&alignment_inner_bx);

        let font_bx = Box::new(Orientation::Vertical, 0);
        font_bx.append(&Label::builder().label("Font").halign(Align::Start).justify(Justification::Left).margin_bottom(6).build());
        let font_btn = FontButton::new();
        font_bx.append(&font_btn);

        let cols_bx = Box::new(Orientation::Vertical, 0);
        cols_bx.append(&Label::builder().label("Columns").halign(Align::Start).justify(Justification::Left).margin_bottom(6).build());
        let cols_btn_bx = Box::new(Orientation::Horizontal, 0);
        cols_btn_bx.style_context().add_class("linked");
        let onecol_btn = Button::builder().icon_name("format-justify-fill-symbolic").build();
        let twocol_btn = Button::builder().icon_name("two-columns-symbolic").build();
        onecol_btn.set_hexpand(true);
        onecol_btn.set_halign(Align::Fill);
        twocol_btn.set_hexpand(true);
        twocol_btn.set_halign(Align::Fill);
        cols_btn_bx.append(&onecol_btn);
        cols_btn_bx.append(&twocol_btn);
        cols_bx.append(&cols_btn_bx);

        let layout_bx = Box::new(Orientation::Horizontal, 12);
        layout_bx.set_vexpand(true);
        layout_bx.set_valign(Align::Start);
        layout_bx.append(&alignment_bx);
        layout_bx.append(&cols_bx);

        left_bx.append(&char_bx);
        left_bx.append(&font_bx);

        right_bx.append(&layout_bx);
        right_bx.append(&par_bx);

        Self {
            bold_btn,
            italic_btn,
            underline_btn,
            strike_btn,
            sub_btn,
            sup_btn,
            popover,
            small_btn,
            large_btn,
            normal_btn,
            huge_btn,
            par_indent_10,
            line_height_10,
            onecol_btn,
            twocol_btn,
            par_indent_15,
            par_indent_20,
            line_height_15,
            line_height_20,
            center_btn,
            left_btn,
            right_btn
        }
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
        bib_scroll.set_width_request(520);
        bib_scroll.set_height_request(360);

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
                        if let Some(ref_row) = ReferenceRow::recover(&row) {
                            if ref_row.key().to_lowercase().contains(&txt) ||
                                ref_row.authors().to_lowercase().contains(&txt) ||
                                ref_row.title().to_lowercase().contains(&txt) {
                                row.set_visible(true);
                            } else {
                                row.set_visible(false);
                            }
                        }
                    }
                    ix += 1;
                }
            }
        });
        create_init_row(&list);
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

#[derive(Debug, Clone)]
pub struct PaperPopover {
    pub popover : Popover,
    pub update_btn : Button,
    pub paper_combo : ComboBoxText,
    pub left_entry : Entry,
    pub top_entry : Entry,
    pub right_entry : Entry,
    pub bottom_entry : Entry
}

fn draw_paper_background(
    ctx : &cairo::Context,
    paper_x_offset : f64,
    paper_y_offset : f64,
    paper_width : f64,
    paper_height : f64
) {

    let fold_sz = 10.0;

    // Draw paper background
    ctx.set_line_width(2.0);
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
}

fn draw_paper(da : &DrawingArea, ctx : &cairo::Context, dim : &PaperDimension) {
    ctx.save();
    let allocation = da.allocation();
    let color = 0.5843;
    ctx.set_source_rgb(color, color, color);
    draw_paper_background(&ctx, dim.x_offset, dim.y_offset, dim.width, dim.height);
    draw_margins(&ctx, &dim);
    ctx.restore();
}

struct PaperDimension {
    x_offset : f64,
    y_offset : f64,
    width : f64,
    height : f64,
    margin_left : f64,
    margin_top : f64,
    margin_right : f64,
    margin_bottom : f64
}

fn draw_margins(ctx : &cairo::Context, dim : &PaperDimension) {
    let dashes = build_dash(2);
    ctx.set_dash(&dashes[..], 0.0);

    // Left margin
    ctx.move_to(dim.x_offset + dim.margin_left, dim.y_offset);
    ctx.line_to(dim.x_offset + dim.margin_left, dim.y_offset + dim.height);
    ctx.stroke();

    // Right margin
    ctx.move_to(dim.x_offset + dim.width - dim.margin_right, dim.y_offset);
    ctx.line_to(dim.x_offset + dim.width - dim.margin_right, dim.y_offset + dim.height);
    ctx.stroke();

    // Margin top
    ctx.move_to(dim.x_offset, dim.y_offset + dim.margin_top);
    ctx.line_to(dim.x_offset + dim.width, dim.y_offset + dim.margin_top);
    ctx.stroke();

    // Margin bottom
    ctx.move_to(dim.x_offset, dim.y_offset + dim.height - dim.margin_bottom);
    ctx.line_to(dim.x_offset + dim.width, dim.y_offset + dim.height - dim.margin_bottom);
    ctx.stroke();
}

const PX_PER_MM : f64 = 0.5;

fn update_margin(old_v : &mut f64, entry : &Entry, page_da : &DrawingArea, update_btn : &Button) {
    let txt = entry.buffer().text().to_string();
    if txt.is_empty() {
        *old_v = 20. * PX_PER_MM;
        update_btn.set_sensitive(true);
    } else {
        if let Some(v) = super::parse_int_or_float(&txt) {
            if v > super::MARGIN_MIN && v < super::MARGIN_MAX {
                *old_v = PX_PER_MM * 10. * v;
                update_btn.set_sensitive(true);
            } else {
                *old_v = 20. * PX_PER_MM;
                update_btn.set_sensitive(false);
            }
        } else {
            *old_v = 20. * PX_PER_MM;
            update_btn.set_sensitive(false);
        }
    }
    page_da.queue_draw();
}

impl PaperPopover {

    pub fn build() -> Self {
        let popover = Popover::new();
        let page_bx = Box::new(Orientation::Horizontal, 1);

        let page_da = DrawingArea::new();
        //page_da.set_width_request(172);
        //page_da.set_height_request(144);
        page_da.set_width_request(180);
        page_da.set_height_request(200);
        let update_btn = Button::new();

        let dim = Rc::new(RefCell::new(PaperDimension {
            x_offset : 42.0,
            y_offset : 10.0,
            width : super::A4.0 * PX_PER_MM ,
            height : super::A4.1 * PX_PER_MM,
            margin_left : 20. * PX_PER_MM,
            margin_right : 20. * PX_PER_MM,
            margin_bottom : 20. * PX_PER_MM,
            margin_top : 20. * PX_PER_MM
        }));
        page_da.set_draw_func({
            let dim = dim.clone();
            move |da, ctx, _, _| {
                draw_paper(&da, &ctx, &dim.borrow());
            }
        });

        let top_entry = Entry::new();
        top_entry.set_primary_icon_name(Some("margin-top-symbolic"));
        top_entry.set_placeholder_text(Some("Top margin (cm)"));
        top_entry.connect_changed({
            let page_da = page_da.clone();
            let dim = dim.clone();
            let update_btn = update_btn.clone();
            move |entry| {
                update_margin(&mut dim.borrow_mut().margin_top, &entry, &page_da, &update_btn);

            }
        });

        let right_entry = Entry::new();
        right_entry.set_primary_icon_name(Some("margin-right-symbolic"));
        right_entry.set_placeholder_text(Some("Right margin (cm)"));
        right_entry.connect_changed({
            let page_da = page_da.clone();
            let dim = dim.clone();
            let update_btn = update_btn.clone();
            move |entry| {
                update_margin(&mut dim.borrow_mut().margin_right, &entry, &page_da, &update_btn);
            }
        });

        let bottom_entry = Entry::new();
        bottom_entry.set_primary_icon_name(Some("margin-bottom-symbolic"));
        bottom_entry.set_placeholder_text(Some("Bottom margin (cm)"));
        bottom_entry.connect_changed({
            let page_da = page_da.clone();
            let dim = dim.clone();
            let update_btn = update_btn.clone();
            move |entry| {
                update_margin(&mut dim.borrow_mut().margin_bottom, &entry, &page_da, &update_btn);
            }
        });

        let left_entry = Entry::new();
        left_entry.set_primary_icon_name(Some("margin-left-symbolic"));
        left_entry.set_placeholder_text(Some("Left margin (cm)"));
        left_entry.connect_changed({
            let page_da = page_da.clone();
            let dim = dim.clone();
            let update_btn = update_btn.clone();
            move |entry| {
                update_margin(&mut dim.borrow_mut().margin_left, &entry, &page_da, &update_btn);
            }
        });

        let margin_bx = Box::new(Orientation::Vertical, 0);
        //margin_bx.set_margin_top(24);
        //margin_bx.set_margin_bottom(32);
        margin_bx.set_vexpand(true);
        margin_bx.set_valign(Align::Center);

        let paper_combo = ComboBoxText::new();
        paper_combo.append(Some("A4"), "A4");
        paper_combo.append(Some("Letter"), "Letter");
        paper_combo.append(Some("Legal"), "Legal");
        paper_combo.append(Some("Custom"), "Custom");
        paper_combo.connect_changed({
            let page_da = page_da.clone();
            let dim = dim.clone();
            move |combo| {
                if let Some(id) = combo.active_id() {
                    let dims_mm = match &id.to_string()[..] {
                        "A4" => super::A4,
                        "Letter" => super::LETTER,
                        "Legal" => super::LEGAL,
                        _ => super::A4
                    };
                    let mut dim = dim.borrow_mut();
                    dim.width = dims_mm.0 * PX_PER_MM;
                    dim.height = dims_mm.1 * PX_PER_MM;
                    page_da.queue_draw();
                }
            }
        });

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
        update_btn.set_label("Update");

        page_right_bx.append(&margin_bx);
        page_right_bx.append(&update_btn);
        page_bx.append(&page_left_bx);
        page_bx.append(&page_right_bx);

        popover.set_child(Some(&page_bx));
        Self { popover, update_btn, paper_combo, left_entry, top_entry, right_entry, bottom_entry }
    }

}

pub const DEFAULT_ZOOM_SCALE : f64 = 1.5;

pub const ZOOM_SCALE_INCREMENT : f64 = 0.5;

impl Titlebar {

    pub fn set_typeset_mode(&self, active : bool) {
        self.zoom_in_btn.set_sensitive(active);
        self.zoom_out_btn.set_sensitive(active);
        if !active {
            self.refresh_btn.set_sensitive(active);
        }
        self.export_pdf_btn.set_sensitive(active);
        if !active {
            if self.pdf_btn.is_active() {
                self.pdf_btn.set_active(false);
            }
        }
    }

    pub fn build() -> Self {
        let header = HeaderBar::new();
        let menu_button = MenuButton::builder().icon_name("open-menu-symbolic").build();

        let pdf_btn = ToggleButton::builder().icon_name("evince-symbolic").build();
        // let hide_pdf_btn = Button::builder().icon_name("user-trash-symbolic").build();
        let export_pdf_btn = Button::builder().icon_name("folder-download-symbolic").build();
        let sidebar_toggle = ToggleButton::builder().icon_name("view-sidebar-symbolic").build();
        // hide_pdf_btn.set_sensitive(false);
        export_pdf_btn.set_sensitive(false);
        pdf_btn.set_sensitive(false);

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

        let paper_popover = PaperPopover::build();
        let page_btn = MenuButton::new();
        page_btn.set_icon_name("crop-symbolic");
        page_btn.set_popover(Some(&paper_popover.popover));

        let page_bx = Box::new(Orientation::Horizontal, 0);
        let page_entry = Entry::new();
        page_entry.set_text("0");
        page_entry.set_hexpand(false);
        page_entry.set_halign(Align::Center);
        page_bx.set_hexpand(false);
        page_bx.set_halign(Align::Center);
        //page_entry.set_hfill(false);
        page_entry.set_max_length(4);
        page_entry.set_width_chars(3);
        page_entry.set_max_width_chars(4);
        let page_button = Button::new();
        page_button.set_label("of 0");
        page_button.set_sensitive(false);

        page_entry.set_input_purpose(InputPurpose::Digits);
        page_bx.append(&page_entry);
        page_bx.append(&page_button);
        page_bx.style_context().add_class("linked");

        page_entry.set_sensitive(false);

        // let bx = Box::new(Orientation::Vertical, 0);
        /*let section_btn = Button::with_label("Section");
        for btn in [&section_btn] {
            btn.style_context().add_class("flat");
            bx.append(btn);
        }*/

        let obj_menu = gio::Menu::new();
        obj_menu.append_item(&gio::MenuItem::new(Some("Image"), Some("win.image")));
        obj_menu.append_item(&gio::MenuItem::new(Some("Table"), Some("win.table")));
        obj_menu.append_item(&gio::MenuItem::new(Some("Link to resource"), Some("win.link")));
        obj_menu.append_item(&gio::MenuItem::new(Some("Bibliography file"), Some("win.bibfile")));

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
        let add_popover = PopoverMenu::from_model(Some(&obj_menu));

        // let item = gio::MenuItem::new_submenu(Some("Call"), &submenu);
        // menu.append_item(&item);

        // let math_expander = Expander::new(Some("Math"));
        // bx.append(&math_expander);
        // add_popover.set_child(Some(&bx));

        let add_btn = MenuButton::new();
        add_btn.set_popover(Some(&add_popover));
        add_btn.set_icon_name("mail-attachment-symbolic");

        // TODO make this another option at a SpinButton.
        // let web_btn = ToggleButton::builder().icon_name("globe-symbolic").build();

        let sidebar_hide_action = gio::SimpleAction::new_stateful("sidebar_hide", None, &(0i32).to_variant());
        let main_menu = MainMenu::build();
        menu_button.set_popover(Some(&main_menu.popover));

        // let symbol_popover = SymbolPopover::new(&editor);
        // titlebar.symbol_btn.set_popover(Some(&symbol_popover.popover));

        let symbol_btn = MenuButton::new();
        //symbol_btn.set_label("∑");
        symbol_btn.set_icon_name("equation-symbolic");

        let org_menu = gio::Menu::new();

        let sectioning_submenu = gio::Menu::new();
        sectioning_submenu.append_item(&gio::MenuItem::new(Some("Chapter"), Some("win.chapter")));
        sectioning_submenu.append_item(&gio::MenuItem::new(Some("Section"), Some("win.section")));
        sectioning_submenu.append_item(&gio::MenuItem::new(Some("Subsection"), Some("win.subsection")));
        sectioning_submenu.append_item(&gio::MenuItem::new(Some("Sub-subsection"), Some("win.sub_subsection")));
        org_menu.append_item(&gio::MenuItem::new_submenu(Some("Sectioning"), &sectioning_submenu));

        //\clearpage
        let layout_submenu = gio::Menu::new();
        layout_submenu.append_item(&gio::MenuItem::new(Some("Page break"), Some("win.page_break")));
        layout_submenu.append_item(&gio::MenuItem::new(Some("Line break"), Some("win.line_break")));
        layout_submenu.append_item(&gio::MenuItem::new(Some("Horizontal space"), Some("win.horizontal_space")));
        layout_submenu.append_item(&gio::MenuItem::new(Some("Vertical space"), Some("win.vertical_space")));
        layout_submenu.append_item(&gio::MenuItem::new(Some("Horizontal fill"), Some("win.horizontal_fill")));
        layout_submenu.append_item(&gio::MenuItem::new(Some("Vertical fill"), Some("win.vertical_fill")));

        // \hspace{2cm}
        // \vspace{2cm}

        // Clear page still allows floats to appear; pagebreak avoids any content.
        // \clearpage
        // \pagebreak[0]
        // \newline

        org_menu.append_item(&gio::MenuItem::new_submenu(Some("Layout"), &layout_submenu));

        let block_submenu = gio::Menu::new();
        block_submenu.append_item(&gio::MenuItem::new(Some("Equation"), Some("win.eq")));
        block_submenu.append_item(&gio::MenuItem::new(Some("List"), Some("win.list")));

        // \quote{}
        // block_submenu.append_item(&gio::MenuItem::new(Some("Quote (simple)"), Some("win.function")));

        // \quotation{}
        // block_submenu.append_item(&gio::MenuItem::new(Some("Quote (long)"), Some("win.function")));

        block_submenu.append_item(&gio::MenuItem::new(Some("Verbatim"), Some("win.verbatim")));
        // block_submenu.append_item(&gio::MenuItem::new(Some("Abstract"), Some("win.operator")));
        // block_submenu.append_item(&gio::MenuItem::new(Some("Code listing"), Some("win.code")));
        block_submenu.append_item(&gio::MenuItem::new(Some("Table (embedded)"), Some("win.tbl")));
        block_submenu.append_item(&gio::MenuItem::new(Some("Bibliography (embedded)"), Some("win.bib")));
        org_menu.append_item(&gio::MenuItem::new_submenu(Some("Block"), &block_submenu));

        let meta_submenu = gio::Menu::new();
        meta_submenu.append_item(&gio::MenuItem::new(Some("Author"), Some("win.author")));
        meta_submenu.append_item(&gio::MenuItem::new(Some("Title"), Some("win.title")));
        meta_submenu.append_item(&gio::MenuItem::new(Some("Date"), Some("win.date")));
        org_menu.append_item(&gio::MenuItem::new_submenu(Some("Metadata"), &meta_submenu));

        let indexing_submenu = gio::Menu::new();
        indexing_submenu.append_item(&gio::MenuItem::new(Some("Table of contents"), Some("win.toc")));
        indexing_submenu.append_item(&gio::MenuItem::new(Some("List of tables"), Some("win.lot")));
        indexing_submenu.append_item(&gio::MenuItem::new(Some("List of figures"), Some("win.lof")));
        org_menu.append_item(&gio::MenuItem::new_submenu(Some("Indexing"), &indexing_submenu));

        let org_popover = PopoverMenu::from_model(Some(&org_menu));

        let org_btn = MenuButton::new();
        org_btn.set_icon_name("format-unordered-list-symbolic");
        org_btn.set_popover(Some(&org_popover));

        let zoom_bx = Box::new(Orientation::Horizontal, 0);
        let zoom_in_btn = Button::new();
        let zoom_out_btn = Button::new();
        let refresh_btn = Button::new();
        refresh_btn.set_icon_name("view-refresh-symbolic");

        zoom_in_btn.set_sensitive(false);
        zoom_out_btn.set_sensitive(false);
        refresh_btn.set_sensitive(false);
        zoom_in_btn.set_icon_name("zoom-in-symbolic");
        zoom_out_btn.set_icon_name("zoom-out-symbolic");

        // zoom_bx.style_context().add_class("linked");

        zoom_bx.append(&refresh_btn);
        zoom_bx.append(&zoom_in_btn);
        zoom_bx.append(&zoom_out_btn);

        // Set zoom to minimum whenever the user toggles the sidebar but the
        // typeset PDF is still open, to minimize occlusion of content.
        sidebar_toggle.connect_toggled({
            let zoom_out_btn = zoom_out_btn.clone();
            let pdf_btn = pdf_btn.clone();
            move |toggle| {
                if toggle.is_active() && pdf_btn.is_active() {
                    while zoom_out_btn.is_sensitive() {
                        zoom_out_btn.emit_clicked();
                    }
                }
            }
        });

        header.pack_start(&sidebar_toggle);
        header.pack_start(&org_btn);
        header.pack_start(&page_btn);
        header.pack_start(&fmt_btn);
        header.pack_start(&symbol_btn);
        header.pack_start(&add_btn);
        header.pack_start(&bib_btn);

        header.pack_end(&menu_button);
        header.pack_end(&page_bx);
        header.pack_end(&export_pdf_btn);
        // header.pack_end(&hide_pdf_btn);
        header.pack_end(&zoom_bx);
        header.pack_end(&pdf_btn);

        pdf_btn.connect_toggled({
            let zoom_in_btn = zoom_in_btn.clone();
            let zoom_out_btn = zoom_out_btn.clone();
            let export_pdf_btn = export_pdf_btn.clone();
            let refresh_btn = refresh_btn.clone();
            move |pdf_btn| {
                // hide_pdf_btn.set_sensitive(false);
                if !pdf_btn.is_active() {
                    export_pdf_btn.set_sensitive(false);
                    refresh_btn.set_sensitive(false);
                    zoom_in_btn.set_sensitive(false);
                    zoom_out_btn.set_sensitive(false);
                }
            }
        });

        // let zoom = Rc::new(RefCell::new(DEFAULT_SCALE));
        let zoom_action = gio::SimpleAction::new_stateful("zoom_change", None, &(DEFAULT_ZOOM_SCALE).to_variant());
        let das : Rc<RefCell<Vec<DrawingArea>>> = Rc::new(RefCell::new(Vec::new()));
        zoom_in_btn.connect_clicked({
            // let zoom = zoom.clone();
            // let das = das.clone();
            let zoom_out_btn = zoom_out_btn.clone();
            let zoom_action = zoom_action.clone();
            // let bx = bx.clone();
            move |btn| {
                let mut z : f64 = zoom_action.state().unwrap().get::<f64>().unwrap();
                if z <= 5.0 {
                    z += ZOOM_SCALE_INCREMENT;
                    if z == 5.0 {
                        btn.set_sensitive(false);
                    }
                    if z > 1.0 {
                        zoom_out_btn.set_sensitive(true);
                    }
                } else {
                    btn.set_sensitive(false);
                }
                zoom_action.set_state(&z.to_variant());
                zoom_action.activate(None);
            }
        });
        zoom_out_btn.connect_clicked({
            // let zoom = zoom.clone();
            // let das = das.clone();
            let zoom_in_btn = zoom_in_btn.clone();
            let zoom_action = zoom_action.clone();
            // let bx = bx.clone();
            move |btn| {
                let mut z : f64 = zoom_action.state().unwrap().get::<f64>().unwrap();
                if z > 1.0 {
                    z -= ZOOM_SCALE_INCREMENT;
                    if z == 1.0 {
                        btn.set_sensitive(false);
                    }
                    if z > 1.0 {
                        btn.set_sensitive(true);
                    }
                    if z < 5.0 {
                        zoom_in_btn.set_sensitive(true);
                    }
                } else {
                    btn.set_sensitive(false);
                }
                zoom_action.set_state(&z.to_variant());
                zoom_action.activate(None);
            }
        });


        Self {
            symbol_btn,
            main_menu,
            header,
            menu_button,
            pdf_btn,
            sidebar_toggle,
            sidebar_hide_action,
            bib_popover,
            object_actions : ObjectActions::build(),
            layout_actions : LayoutActions::build(),
            block_actions : BlockActions::build(),
            sectioning_actions : SectioningActions::build(),
            indexing_actions : IndexingActions::build(),
            meta_actions : MetaActions::build(),
            fmt_popover,
            paper_popover,
            zoom_in_btn,
            zoom_out_btn,
            zoom_action,
            refresh_btn,
            // hide_pdf_btn,
            export_pdf_btn,
            page_entry,
            page_button
        }
    }
}

impl React<Typesetter> for Titlebar {

    fn react(&self, typesetter : &Typesetter) {
        let btn = self.pdf_btn.clone();
        let sidebar_toggle = self.sidebar_toggle.clone();
        typesetter.connect_done({
            let refresh_btn = self.refresh_btn.clone();
            move |_| {
                btn.set_icon_name("evince-symbolic");
                btn.set_sensitive(true);
                refresh_btn.set_sensitive(true);

                // Auto-hide overview on document typesettting done.
                // if sidebar_toggle.is_active() {
                //    sidebar_toggle.set_active(false);
                // }
            }
        });
        typesetter.connect_error({
            let btn = self.pdf_btn.clone();
            let titlebar = self.clone();
            move |_| {
                btn.set_icon_name("evince-symbolic");
                btn.set_sensitive(true);
                // btn.set_active(false);
                titlebar.set_typeset_mode(false);
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

fn trim_braces(s : &str) -> &str {
    s.trim_start_matches("{").trim_end_matches("}")
        .trim_start_matches("{").trim_end_matches("}")
}

fn replace_groups(s : &str) -> Cow<str> {

    use nom::bytes::complete::is_not;
    use nom::branch::alt;
    use nom::multi::many0;

    let mut repls = Cow::from(s);
    if let Ok((_, list)) = many0(alt((is_not("{"), crate::tex::group_str)))(s) {
        for seg in list {
            if seg.starts_with("{") {
                println!("{}", seg);
                repls = Cow::from(repls.replace(seg, "?"));
            }
        }
    }
    repls
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

    pub fn recover(row : &ListBoxRow) -> Option<Self> {
        let bx = row.child().unwrap().downcast::<Box>().unwrap();
        let header_bx = super::try_get_child_by_index::<Box>(&bx, 0)?;
        let key_label = super::try_get_child_by_index::<Label>(&header_bx, 1)?;
        let authors_label = super::try_get_child_by_index::<Label>(&header_bx, 2)?;
        let title_label = super::try_get_child_by_index::<Label>(&bx, 1)?;
        Some(Self { row : row.clone(), key_label, authors_label, title_label })
    }

    pub fn update(&self, entry : &BibEntry) {
        // println!("{:?}", entry);
        let key = format!("<b>{}</b>", entry.key());
        let full_title = trim_braces(entry.title().unwrap_or("(Untitled)").trim()).to_string();

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
        let mut title = replace_groups(&title);
        let authors = replace_groups(&trim_braces(entry.author().unwrap_or("(No authors)").trim()));
        let fst_name = regex::Regex::new(r#",.*"#).unwrap();

        let mut sep_authors = authors.split(" and ").collect::<Vec<_>>();
        for mut author in sep_authors.iter_mut() {
            if let Some(m) = fst_name.find(*author) {
                *author = &author[..m.start()]
            }
        }
        let authors_str = match sep_authors.len() {
            0..=1 => {
                Cow::from(sep_authors[0].trim())
            },
            2 => {
                Cow::from(format!("{} & {}", sep_authors[0].trim(), sep_authors[1].trim()))
            },
            3 => {
                Cow::from(format!("{}, {} & {}", sep_authors[0].trim(), sep_authors[1].trim(), sep_authors[2].trim()))
            },
            _ => {
                Cow::from(format!("{} et al.", sep_authors[0].trim()))
            }
        };

        /*let mut broken_authors = String::new();
        if authors.chars().count() > 60 {
            broken_authors = authors.chars().take(60).collect();
            broken_authors += "(...)";
        }*/

        let year = trim_braces(entry.year().unwrap_or("No date").trim());

        // println!("authors = {}; title = {}; key = {}", authors, title, key);
        self.key_label.set_markup(&key);
        self.authors_label.set_text(&format!("{} ({})", authors_str, year));
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

/*impl React<FileManager> for BibPopover {

    fn react(&self, manager : &FileManager) {

    }

}*/

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
                                if let Some(ref_row) = ReferenceRow::recover(&row) {
                                    ref_row.update(&bib_entry);
                                }
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
        analyzer.connect_references_cleared({
            let list = self.list.clone();
            move |_| {
                clear_list(&list);
                // create_init_row(&list);
            }
        });
        analyzer.connect_references_validated({
            let list = self.list.clone();
            move |_| {
                clear_list(&list);
            }
        });

        let last_is_err = Arc::new(AtomicBool::new(false));
        analyzer.connect_doc_error({
            let list = self.list.clone();
            let last_is_err = last_is_err.clone();
            move |err| {
                clear_list(&list);
                create_unique_row(&list, &format!("Parsing error: {}", err), "dialog-error-symbolic");
                last_is_err.store(true, Ordering::Relaxed);
            }
        });
        analyzer.connect_doc_changed({
            let list = self.list.clone();
            move |_| {
                if last_is_err.load(Ordering::Relaxed) {
                    clear_list(&list);
                    create_init_row(&list);
                    last_is_err.store(false, Ordering::Relaxed);
                }
            }
        });
    }

}

pub(super) fn clear_list(list : &ListBox) {
    // let mut ix = 0;
    while let Some(r) = list.row_at_index(0) {
        list.remove(&r);
        // ix += 1;
    }
}

pub(super) fn create_init_row(list : &ListBox) {
    create_unique_row(&list, "Insert a bibliography section and save the file locally \nto search its references here", "user-bookmarks-symbolic");
}

fn create_unique_row(list : &ListBox, label : &str, icon : &str) {
    let row = ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);
    let bx = Box::new(Orientation::Horizontal, 0);
    let icon = Image::from_icon_name(Some(icon));
    super::set_all_margins(&icon, 6);
    bx.append(&icon);
    let label = Label::new(Some(label));
    bx.append(&label);
    row.set_child(Some(&bx));
    list.insert(&row, 0);
}


