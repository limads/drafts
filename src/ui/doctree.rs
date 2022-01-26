use gtk4::*;
use gtk4::prelude::*;
use super::*;
use crate::analyzer::Analyzer;
use crate::tex::{Difference, Token, Command, Object, ObjectIndex};
use gio::prelude::*;
use crate::tex::Subsection;
use crate::tex::Section;
use either::Either;

#[derive(Debug, Clone)]
pub struct DocIcons {
    section_icon : Pixbuf,
    tbl_icon : Pixbuf,
    img_icon : Pixbuf,
    code_icon : Pixbuf,
    eq_icon : Pixbuf
}

#[derive(Debug, Clone)]
pub struct DocTree {
    pub tree_view : TreeView,
    store : TreeStore,
    pub bx : Box,
    doc_icons : DocIcons
}

impl DocTree {

    pub fn build() -> Self {
        let tree_view = TreeView::new();
        tree_view.set_valign(Align::Fill);
        tree_view.set_vexpand(true);
        let store = configure_tree_view(&tree_view);

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

        let doc_icons = DocIcons {
            section_icon : Pixbuf::from_file_at_scale("assets/icons/break-point-symbolic.svg", 16, 16, true).unwrap(),
            tbl_icon : Pixbuf::from_file_at_scale("assets/icons/queries-symbolic.svg", 16, 16, true).unwrap(),
            img_icon : Pixbuf::from_file_at_scale("assets/icons/image-x-generic-symbolic.svg", 16, 16, true).unwrap(),
            code_icon : Pixbuf::from_file_at_scale("assets/icons/gnome-terminal-symbolic.svg", 16, 16, true).unwrap(),
            eq_icon : Pixbuf::from_file_at_scale("assets/icons/equation-symbolic.svg", 16, 16, true).unwrap()
        };
        Self { tree_view, bx, store, doc_icons }
    }
}

fn configure_tree_view(tree_view : &TreeView) -> TreeStore {
    let model = TreeStore::new(&[Pixbuf::static_type(), Type::STRING]);
    tree_view.set_model(Some(&model));
    // tree_view.set_selection_mode(SelectionMode::Single);
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

fn insert_section(iter : TreeIter, store : &TreeStore, sec : Section, icons : &DocIcons) {
    store.set(&iter, &[(0, &icons.section_icon), (1, &sec.name)]);
}

fn insert_subsection(iter : TreeIter, store : &TreeStore, sub : Subsection, icons : &DocIcons) {
    store.set(&iter, &[(0, &icons.section_icon), (1, &sub.name)]);
}

fn insert_object(iter : TreeIter, store : &TreeStore, tree_view : &TreeView, obj : Object, doc_ix : ObjectIndex, icons : &DocIcons) {
    let (icon, name) = match obj {
        Object::Table(order, _, _) => (&icons.tbl_icon, format!("Table {}", order + 1)),
        Object::Image(order, _, _) => (&icons.img_icon, format!("Image {}", order + 1)),
        Object::Equation(order, _, _) => (&icons.eq_icon, format!("Equation {}", order + 1)),
        Object::Code(order, _, _) => (&icons.code_icon, format!("Listing {}", order + 1)),
    };
    /*let (parent_iter, pos) = match doc_ix {
        ObjectIndex::Root(ix) => {
            (None, ix as i32)
        },
        ObjectIndex::Section(sec, obj) => {
            (Some(tree_view.model().unwrap().iter_nth_child(None, sec as i32).unwrap()), obj as i32)
        },
        ObjectIndex::Subsection(sec, sub, obj) => {
            let model = tree_view.model().unwrap();
            let sec_iter = model.iter_nth_child(None, sec as i32).unwrap();
            let subsec_iter = model.iter_nth_child(Some(&sec_iter), sub as i32).unwrap();
            (Some(subsec_iter), obj as i32)
        }
    };*/
    // let iter = store.insert(parent_iter.as_ref(), pos);
    store.set(&iter, &[(0, &icon), (1, &name)]);
}

impl React<Analyzer> for DocTree {

    fn react(&self, analyzer : &Analyzer) {
        analyzer.connect_doc_changed({
            let store = self.store.clone();
            let doc_icons = self.doc_icons.clone();
            let tree_view = self.tree_view.clone();
            move |new_doc| {
                store.clear();

                for (ix, tk_ix, item) in new_doc.root_items() {
                    match item {
                        Either::Left(sec) => {
                            let iter = store.append(None);
                            insert_section(iter, &store, sec, &doc_icons);
                        },
                        Either::Right(obj) => {
                            let iter = store.append(None);
                            insert_object(iter, &store, &tree_view, obj, ObjectIndex::Root(ix), &doc_icons);
                        }
                    }
                }

                for ([root_ix, sec_ix], tk_ix, item) in new_doc.level_one_items() {
                    let section_iter = tree_view.model().unwrap().iter_nth_child(None, root_ix as i32).unwrap();
                    let iter = store.append(Some(&section_iter));
                    match item {
                        Either::Left(sub) => {
                            insert_subsection(iter, &store, sub, &doc_icons);
                        },
                        Either::Right(obj) => {
                            insert_object(iter, &store, &tree_view, obj, ObjectIndex::Section(root_ix, sec_ix), &doc_icons);
                        }
                    }
                }

                for ([root_ix, sec_ix, sub_ix], tk_ix, obj) in new_doc.level_two_items() {
                    let model = tree_view.model().unwrap();
                    let sec_iter = model.iter_nth_child(None, root_ix as i32).unwrap();
                    let subsec_iter = model.iter_nth_child(Some(&sec_iter), sec_ix as i32).unwrap();
                    let iter = store.append(Some(&subsec_iter));
                    insert_object(iter, &store, &tree_view, obj, ObjectIndex::Subsection(root_ix, sec_ix, sub_ix), &doc_icons);
                }

                /*for section in new_doc.sections() {
                    let iter = store.append(None);
                    store.set(&iter, &[(0, &section_icon), (1, &section.name)]);
                }

                for subsection in new_doc.subsections() {
                    let section_iter = tree_view.model().unwrap().iter_nth_child(None, subsection.parent_index as i32).unwrap();
                    let iter = store.insert(Some(&section_iter), subsection.local_index as i32);
                    store.set(&iter, &[(0, &section_icon), (1, &subsection.name)]);
                }

                for object in new_doc.objects() {
                    let iter = store.insert(iter.as_ref(), pos);
                    insert_object(iter, &store, obj, &doc_icons)
                }*/

                tree_view.expand_all();
            }
        });

        analyzer.connect_section_changed({
            let store = self.store.clone();
            let model = self.tree_view.model().unwrap();
            let section_icon = self.doc_icons.section_icon.clone();
            move |diff| {
                match diff {
                    Difference::Added(pos, txt) => {

                        // pass some(iter) to iterate over this element's children
                        // model.iter_children(None);
                        // Pass Some(iter) to insert relative to this parent

                        match Token::from_str(&txt) {
                            Ok(Token::Command(Command { arg : Some(name), .. }, _)) => {
                                if !name.is_empty() {
                                    let iter = store.insert(None, pos as i32);
                                    store.set(&iter, &[(0, &section_icon), (1, &name)]);
                                }
                            },
                            _ => { }
                        }
                    },
                    Difference::Edited(pos, txt) => {
                        match Token::from_str(&txt) {
                            Ok(Token::Command(Command { arg : Some(name), .. }, _)) => {
                                if let Some(iter) = model.iter(&TreePath::from_indices(&[pos as i32])) {
                                    store.set(&iter, &[(0, &section_icon), (1, &name)]);
                                }
                            },
                            _ => { }
                        }
                    },
                    Difference::Removed(pos) => {
                        if let Some(iter) = model.iter(&TreePath::from_indices(&[pos as i32])) {
                            store.remove(&iter);
                        }
                    }
                }
            }
        });
    }

}


