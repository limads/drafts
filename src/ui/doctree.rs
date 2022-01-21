use gtk4::*;
use gtk4::prelude::*;
use super::*;

#[derive(Debug, Clone)]
pub struct DocTree {
    pub tree_view : TreeView,
    pub bx : Box
}

impl DocTree {

    pub fn build() -> Self {
        let tree_view = TreeView::new();
        tree_view.set_valign(Align::Fill);
        tree_view.set_vexpand(true);
        let model = configure_tree_view(&tree_view);

        let title = PackedImageLabel::build("document-edit-symbolic", "Outline");
        title.bx.set_vexpand(false);
        title.bx.set_valign(Align::Start);
        super::set_border_to_title(&title.bx);
        let bx = Box::new(Orientation::Vertical, 0);

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_valign(Align::Fill);
        scroll.set_child(Some(&tree_view));
        bx.append(&title.bx);
        bx.append(&scroll);

        Self { tree_view, bx }
    }
}

fn configure_tree_view(tree_view : &TreeView) -> TreeStore {
    let model = TreeStore::new(&[Pixbuf::static_type(), Type::STRING]);
    tree_view.set_model(Some(&model));
    let pix_renderer = CellRendererPixbuf::new();
    pix_renderer.set_padding(6, 6);
    // pix_renderer.set_property_height(24);
    let txt_renderer = CellRendererText::new();
    // txt_renderer.set_property_height(24);

    let pix_col = TreeViewColumn::new();
    pix_col.pack_start(&pix_renderer, false);
    pix_col.add_attribute(&pix_renderer, "pixbuf", 0);

    let txt_col = TreeViewColumn::new();
    txt_col.pack_start(&txt_renderer, true);
    txt_col.add_attribute(&txt_renderer, "text", 1);

    tree_view.append_column(&pix_col);
    tree_view.append_column(&txt_col);
    tree_view.set_show_expanders(true);
    tree_view.set_can_focus(false);
    tree_view.set_has_tooltip(false);
    tree_view.set_headers_visible(false);

    // tree_view.set_vadjustment(Some(&Adjustment::default()));
    // tree_view.set_vadjustment(Some(&Adjustment::new(0.0, 0.0, 100.0, 10.0, 10.0, 100.0)));
    // tree_view.set_vscroll_policy(ScrollablePolicy::Natural);

    model
}
