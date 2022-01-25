use std::cmp::{Eq, PartialEq};
use super::*;
use either::Either;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectIndex {

    // Hols item index at document top-level
    Root(usize),

    // Holds section index and item index within the section
    Section(usize, usize),

    // Holds section index, subsection index and item index within subsection
    Subsection(usize, usize, usize)

}

// First field carries the "order" of the object (how many objects of the same time
// were already added before it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {

    Table(usize, ObjectIndex, Option<String>),

    Image(usize, ObjectIndex, Option<String>),

    Equation(usize, ObjectIndex, Option<String>),

    Code(usize, ObjectIndex, Option<String>),

    // Paragraph(usize, ObjectIndex)

}

impl Object {

    pub fn index(&self) -> ObjectIndex {
        match self {
            Object::Table(_, ix, _) => *ix,
            Object::Image(_, ix, _) => *ix,
            Object::Equation(_, ix, _) => *ix,
            Object::Code(_, ix, _) => *ix,
            // Object::Paragraph(_, ix) => *ix,
        }
    }

}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {

    items : Vec<Item>

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name : String,
    pub index : usize,
    pub items : Vec<Item>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subsection {

    pub name : String,

    // Index of the parent section (section.index)
    pub parent_index : usize,

    // Index of the subsection within the current section (number of previous subsections)
    pub local_index : usize,

    // How many subsections were added before it, irrespective of their section affiliation.
    pub global_index : usize,

    pub items : Vec<Item>

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {

    Section(Section),

    Subsection(Subsection),

    Object(Object)

}

pub fn push_to_innermost(
    top : &mut Vec<Item>,
    section : &mut Option<Section>,
    subsection : &mut Option<Subsection>,
    item : Item
) {
    if let Some(sub) = subsection {
        sub.items.push(item);
    } else {
        if let Some(sec) = section {
            sec.items.push(item);
        } else {
            top.push(item);
        }
    }
}

#[derive(Default)]
struct ItemCount {

    section : usize,

    subsection_global : usize,

    subsection_local : usize,

    table : usize,

    image : usize,

    code : usize,

    math : usize,

    // paragraph : usize

}

fn local_object_index(
    items : &[Item],
    parent_section : &Option<Section>,
    parent_subsection : &Option<Subsection>,
    count : &ItemCount
) -> ObjectIndex {
    match parent_subsection {
        Some(Subsection { local_index, items, .. }) => {
            match parent_section {
                Some(sec) => {
                    ObjectIndex::Subsection(sec.index, *local_index, items.len())
                },
                None => {
                    panic!()
                }
            }
        },
        None => {
            match parent_section {
                Some(sec) => {
                    ObjectIndex::Section(sec.index, sec.items.len())
                },
                None => {
                    ObjectIndex::Root(items.len())
                }
            }
        }
    }
}

fn next_item<'a>(
    items : &mut Vec<Item>,
    parent_section : &mut Option<Section>,
    parent_subsection : &mut Option<Subsection>,
    count : &mut ItemCount,
    bl_token : Either<Token<'a>, Block<'a>>
) -> Result<(), String> {
    match bl_token {
        Either::Left(Token::Command(Command { cmd : "section", arg, .. }, _)) => {
            match parent_section.take() {
                Some(mut section) => {
                    if let Some(subsection) = parent_subsection.take() {
                        section.items.push(Item::Subsection(subsection));
                        count.subsection_global += 1;
                        count.subsection_local = 0;
                    }
                    items.push(Item::Section(section));
                    count.section += 1;
                },
                None => { }
            }
            let name = arg.ok_or(String::from("Unnamed section"))?.to_string();
            *parent_section = Some(Section { name, index : count.section, items : Vec::new() });
        },
        Either::Left(Token::Command(Command { cmd : "subsection", arg, .. }, _)) => {
            match parent_section {
                Some(ref mut section) => {
                    match parent_subsection.take() {
                        Some(subsection) => {
                            section.items.push(Item::Subsection(subsection));
                            count.subsection_local += 1;
                            count.subsection_global += 1;
                        },
                        None => { }
                    }
                    let name = arg.ok_or(String::from("Unnamed section"))?.to_string();
                    *parent_subsection = Some(Subsection {
                        name,
                        parent_index : count.section,
                        global_index : count.subsection_global,
                        local_index : count.subsection_local,
                        items : Vec::new()
                    });
                },
                None => {
                    return Err(String::from("Subsection without section parent"));
                }
            }
        },
        Either::Left(Token::Command(Command { cmd : "includegraphics", .. }, range)) => {
            push_to_innermost(
                items,
                parent_section,
                parent_subsection, Item::Object(Object::Image(count.image, local_object_index(&items[..],
                &parent_section,
                &parent_subsection,
                &count), None))
            );
            count.image += 1;
        },
        Either::Left(Token::Text(_, range)) => {
            /*push_to_innermost(
                items,
                parent_section,
                parent_subsection,
                Item::Object(Object::Paragraph(count.paragraph, local_object_index(&items[..],
                &parent_section,
                &parent_subsection,
                &count), ))
            );
            count.paragraph += 1;*/
        },
        Either::Left(Token::Math(_, _, range)) => {
            push_to_innermost(
                items,
                parent_section,
                parent_subsection,
                Item::Object(Object::Equation(count.math, local_object_index(&items[..], &parent_section, &parent_subsection, &count),
                None))
            );
            count.math += 1;
        },
        Either::Right(Block { start_cmd : Command { arg, .. }, .. }) => {
            match arg {
                Some("tabular") => {
                    push_to_innermost(
                        items,
                        parent_section,
                        parent_subsection,
                        Item::Object(Object::Table(count.table, local_object_index(&items[..],
                        &parent_section,
                        &parent_subsection,
                        &count), None))
                    );
                    count.table += 1;
                },
                Some("lstlisting") => {
                    push_to_innermost(
                        items,
                        parent_section,
                        parent_subsection,
                        Item::Object(Object::Code(count.code, local_object_index(&items[..],
                        &parent_section,
                        &parent_subsection, &count), None))
                    );
                    count.code += 1;
                },
                _ => { }
            }
        },
        _ => { }
    }
    Ok(())
}

fn is_doc_block<'a>(bl_token : &Either<Token<'a>, Block<'a>>) -> bool {
    match bl_token {
        Either::Right(Block { start_cmd : Command { arg : Some("document"), .. }, .. }) => {
            true
        },
        _ => {
            false
        }
    }
}

fn iter_next_item(objs : &mut Vec<Object>, tree : &[Item]) {
    match tree.get(0) {
        Some(item) => match item {
            Item::Section(Section { items, .. }) => {
                iter_next_item(objs, &items[..]);
            },
            Item::Subsection(Subsection { items, .. }) => {
                iter_next_item(objs, &items[..]);
            },
            Item::Object(obj) => {
                objs.push(obj.clone());
                if tree.len() > 1 {
                    iter_next_item(objs, &tree[1..]);
                }
            }
        },
        None => { }
    }
}

impl Document {

    pub fn root_items(&self) -> Vec<(usize, Either<Section, Object>)> {
        let mut items = Vec::new();
        for (ix, item) in self.items.iter().enumerate() {
            match item {
                Item::Section(s) => {
                    items.push((ix, Either::Left(s.clone())));
                },
                Item::Object(obj) => {
                    items.push((ix, Either::Right(obj.clone())));
                },
                _ => { }
            }
        }
        items
    }

    pub fn level_one_items(&self) -> Vec<([usize;2], Either<Subsection, Object>)> {
        let mut items = Vec::new();
        for (root_ix, root_item) in self.items.iter().enumerate() {
            match root_item {
                Item::Section(sec) => {
                    for (sec_ix, sec_item) in sec.items.iter().enumerate() {
                        match sec_item {
                            Item::Subsection(sub) => {
                                items.push(([root_ix, sec_ix], Either::Left(sub.clone())));
                            },
                            Item::Object(obj) => {
                                items.push(([root_ix, sec_ix], Either::Right(obj.clone())));
                            },
                            _ => { }
                        }
                    }
                },
                _ => { }
            }
        }
        items
    }

    pub fn level_two_items(&self) -> Vec<([usize;3], Object)> {
        let mut items = Vec::new();
        for (root_ix, root_item) in self.items.iter().enumerate() {
            match root_item {
                Item::Section(sec) => {
                    for (sec_ix, sec_item) in sec.items.iter().enumerate() {
                        match sec_item {
                            Item::Subsection(sub) => {
                                for (sub_ix, sub_item) in sub.items.iter().enumerate() {
                                    match sub_item {
                                        Item::Object(obj) => {
                                            items.push(([root_ix, sec_ix, sub_ix], obj.clone()));
                                        },
                                        _ => { }
                                    }
                                }
                            },
                            _ => { }
                        }
                    }
                },
                _ => { }
            }
        }
        items
    }

    pub fn objects(&self) -> Vec<Object> {
        let mut objs = Vec::new();
        iter_next_item(&mut objs, &self.items[..]);
        objs
    }

    pub fn sections(&self) -> Vec<Section> {
        self.items.iter().filter_map(|item| {
            match item {
                Item::Section(sec) => {
                    Some(sec.clone())
                },
                _ => None
            }
        }).collect()
    }

    pub fn subsections(&self) -> Vec<Subsection> {
        let secs = self.sections();
        secs.iter().map(|sec| sec.items.iter().filter_map(|item| {
            match item {
                Item::Subsection(sub) => {
                    Some(sub.clone())
                },
                _ => None
            }
        })).flatten().collect()
    }

}

pub struct Parser {

}

impl Parser {

    pub fn from_tokens<'a>(mut tks : impl Iterator<Item=Token<'a>> + Clone) -> Result<Document, String> {
        let mut all_tks = Vec::new();
        blocked_tokens(Vec::new(), &mut tks, &mut all_tks)?;
        println!("{:#?}", all_tks);
        let mut doc_items : Option<Vec<Item>> = None;
        for tk in all_tks {
            if is_doc_block(&tk) {
                if doc_items.is_none() {
                    match tk {
                        Either::Right(Block { inner, .. }) => {
                            let mut curr_section : Option<Section> = None;
                            let mut curr_subsection : Option<Subsection> = None;
                            let mut count = ItemCount::default();
                            let mut items : Vec<Item> = Vec::new();
                            for in_tk in inner {
                                next_item(&mut items, &mut curr_section, &mut curr_subsection, &mut count, in_tk)?;
                            }
                            if let Some(subsection) = curr_subsection.take() {
                                if let Some(ref mut section) = curr_section {
                                    section.items.push(Item::Subsection(subsection));
                                } else {
                                    return Err(String::from("Subsection without section parent"));
                                }
                            }
                            if let Some(section) = curr_section {
                                items.push(Item::Section(section));
                            }
                            doc_items = Some(items);
                        },
                        _ => { }
                    }
                } else {
                    return Err(String::from("Multiple document blocks found"));
                }
            }
        }

        match doc_items {
            Some(items) => Ok(Document { items }),
            None => Err(String::from("Missing document block"))
        }
    }

    pub fn parse(s : &str) -> Result<Document, String> {
        let tks = Lexer::scan(s)?;
        Self::from_tokens(tks.iter())
    }

}

#[test]
fn test_parser() {

    let txt = r#"

        \documentclass[12pt,a4paper,oneside,draft]{report}

        \begin{document}

        \section{Hello world}

        % This is a comment

        This is a paragraph

        $$x^2 + 2$$

        This is inline math $a=1$

        This is code

        \begin{lstlisting}
            let a = 1;
            let b = 2;
            a + b
        \end{lstlisting}

        \bibliography

        @article{ Guestrin2006Jun,
	        author = {Guestrin, E. D. and Eizenman, M.},
	        title = {{General theory of remote gaze estimation using the pupil center and corneal reflections}},
	        journal = {IEEE Trans. Biomed. Eng.},
	        volume = {53},
	        number = {6},
	        pages = {1124--1133},
	        year = {2006},
	        month = {Jun},
	        publisher = {IEEE},
	        doi = {10.1109/TBME.2005.863952}
         }

         \end{document}
    "#;

    let cmd = r#"

    \documentclass[12pt,a4paper,oneside,draft]{report}

    "#;

    let math = r#"$$a$$ else"#;

    println!("{:#?}", Lexer::scan(txt));

    println!("{:#?}", Parser::parse(txt));

    // println!("{:?}", bib_field_value("{Guestrin, E. D. and Eizenman, M.}"));
    // println!("{:?}", bib_field("author = {Guestrin, E. D. and Eizenman, M.}"));
}

