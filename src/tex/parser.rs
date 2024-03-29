/*Copyright (c) 2022 Diego da Silva Lima. All rights reserved.

This work is licensed under the terms of the GPL v3.0 License.
For a copy, see http://www.gnu.org/licenses.*/

use std::cmp::{Eq, PartialEq};
use super::*;
use either::Either;
use std::convert::AsRef;

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

    Bibliography(usize, String),

    // Paragraph(usize, ObjectIndex)

}

impl Object {

    pub fn index(&self) -> ObjectIndex {
        match self {
            Object::Table(_, ix, _) => *ix,
            Object::Image(_, ix, _) => *ix,
            Object::Equation(_, ix, _) => *ix,
            Object::Code(_, ix, _) => *ix,
            Object::Bibliography(_, _) => ObjectIndex::Root(0),
            // Object::Paragraph(_, ix) => *ix,
        }
    }

}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {

    pub items : Vec<Item>

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

/// Carries item and token index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {

    Section(Section, usize),

    Subsection(Subsection, usize),

    Object(Object, usize)

}

impl Item {

    pub fn line(&self) -> usize {
        match self {
            Item::Section(_, ix) => *ix,
            Item::Subsection(_, ix) => *ix,
            Item::Object(_, ix) => *ix,
        }
    }

    pub fn token_index(&self) -> usize {
        match self {
            Item::Section(_, ix) => *ix,
            Item::Subsection(_, ix) => *ix,
            Item::Object(_, ix) => *ix,
        }
    }
}

pub fn push_to_innermost(
    top : &mut Vec<Item>,
    section : &mut Option<(Section, usize)>,
    subsection : &mut Option<(Subsection, usize)>,
    item : Item
) {
    if let Some((ref mut sub, _)) = subsection {
        sub.items.push(item);
    } else {
        if let Some((ref mut sec, _)) = section {
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
    parent_section : &Option<(Section, usize)>,
    parent_subsection : &Option<(Subsection, usize)>,
    count : &ItemCount
) -> ObjectIndex {
    match parent_subsection {
        Some((Subsection { local_index, items, .. }, _)) => {
            match parent_section {
                Some((sec, _)) => {
                    ObjectIndex::Subsection(sec.index, *local_index, items.len())
                },
                None => {
                    panic!()
                }
            }
        },
        None => {
            match parent_section {
                Some((sec, _)) => {
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
    tk_ix : &mut usize,
    parent_section : &mut Option<(Section, usize)>,
    parent_subsection : &mut Option<(Subsection, usize)>,
    count : &mut ItemCount,
    bl_token : Either<Token<'a>, Block<'a>>
) -> Result<(), String> {

    let curr_tk_ix = *tk_ix;
    match bl_token {
        Either::Left(_) => {
            *tk_ix += 1;
        },
        Either::Right(ref block) => {
            *tk_ix += block.token_count();
        }
    }

    match bl_token {
        Either::Left(Token::Command(Command { cmd : "section", arg, .. }, _)) => {
            match parent_section.take() {
                Some((mut section, sec_tk_ix)) => {
                    if let Some((subsection, sub_tk_ix)) = parent_subsection.take() {
                        section.items.push(Item::Subsection(subsection, sub_tk_ix));
                        count.subsection_global += 1;
                        count.subsection_local = 0;
                    }
                    items.push(Item::Section(section, sec_tk_ix));
                    count.section += 1;
                },
                None => { }
            }
            let name = arg.ok_or(String::from("Unnamed section"))?.to_string();
            *parent_section = Some((Section { name, index : count.section, items : Vec::new() }, curr_tk_ix));
        },
        Either::Left(Token::Command(Command { cmd : "subsection", arg, .. }, _)) => {
            match parent_section {
                Some((ref mut section, _)) => {
                    match parent_subsection.take() {
                        Some((subsection, sub_tk_ix)) => {
                            section.items.push(Item::Subsection(subsection, sub_tk_ix));
                            count.subsection_local += 1;
                            count.subsection_global += 1;
                        },
                        None => { }
                    }
                    let name = arg.ok_or(String::from("Unnamed section"))?.to_string();
                    *parent_subsection = Some((Subsection {
                        name,
                        parent_index : count.section,
                        global_index : count.subsection_global,
                        local_index : count.subsection_local,
                        items : Vec::new()
                    }, curr_tk_ix));
                },
                None => {
                    return Err(String::from("Subsection without section parent"));
                }
            }
        },
        Either::Left(Token::Command(Command { cmd : "includegraphics", .. }, range)) => {

            /*
            \begin{figure}
            \includegraphics{fig}
            \label{fig:galaxy}
            \end{figure}
            */
            push_to_innermost(
                items,
                parent_section,
                parent_subsection, Item::Object(Object::Image(count.image, local_object_index(&items[..],
                &parent_section,
                &parent_subsection,
                &count), None), curr_tk_ix)
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
                None), curr_tk_ix)
            );
            count.math += 1;
        },
        Either::Right(Block { start_cmd : Command { arg, .. }, .. }) => {
            match arg {
                Some(CommandArg::Text("tabular")) => {
                    push_to_innermost(
                        items,
                        parent_section,
                        parent_subsection,
                        Item::Object(Object::Table(count.table, local_object_index(&items[..],
                        &parent_section,
                        &parent_subsection,
                        &count), None), curr_tk_ix)
                    );
                    count.table += 1;
                },
                Some(CommandArg::Text("lstlisting")) => {
                    push_to_innermost(
                        items,
                        parent_section,
                        parent_subsection,
                        Item::Object(Object::Code(count.code, local_object_index(&items[..],
                        &parent_section,
                        &parent_subsection, &count), None), curr_tk_ix)
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
        Either::Right(Block { start_cmd : Command { arg : Some(CommandArg::Text("document")), .. }, .. }) => {
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
            Item::Section(Section { items, .. }, _) => {
                iter_next_item(objs, &items[..]);
            },
            Item::Subsection(Subsection { items, .. }, _) => {
                iter_next_item(objs, &items[..]);
            },
            Item::Object(obj, _) => {
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

    pub fn get_line(&self, sel_ixs : &[usize]) -> Option<usize> {
        match (sel_ixs.get(0), sel_ixs.get(1), sel_ixs.get(2)) {
            (Some(sec_or_obj), None, None) => {
                Some(self.items.get(*sec_or_obj)?.line())
            },
            (Some(sec), Some(subsec_or_obj), None) => {
                match self.items.get(*sec)? {
                    Item::Section(Section { items, .. }, _) => {
                        Some(items.get(*subsec_or_obj)?.line())
                    },
                    _ => None
                }
            },
            (Some(sec), Some(subsec), Some(obj)) => {
                match self.items.get(*sec)? {
                    Item::Section(Section { items, .. }, _) => {
                        match items.get(*subsec)? {
                            Item::Subsection(Subsection { items, .. }, _) => {
                                Some(items.get(*obj)?.line())
                            },
                            _ => None
                        }
                    },
                    _ => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn token_index_at(&self, ixs : &[usize]) -> Option<usize> {

        for ix in ixs.iter() {
            if self.items.len() <= *ix {
                return None;
            }
        }

        match ixs.len() {
            0 => None,
            1 => Some(self.items[ixs[0]].token_index()),
            2 => {
                match &self.items[ixs[0]] {
                    Item::Section(sec, _) => {
                        Some(sec.items[ixs[1]].token_index())
                    },
                    _ => None
                }
            },
            3 => {
                match &self.items[ixs[0]] {
                    Item::Section(sec, _) => {
                        match &sec.items[ixs[1]] {
                            Item::Subsection(sub, _) => {
                                Some(sub.items[ixs[2]].token_index())
                            },
                            _ => None
                        }
                    },
                    _ => None
                }
            },
            _ => None
        }
    }

    // For the root_items, level_one_items, and level_two_items, the first field
    // indexes the element index at the current document tree. The second field indexes
    // the linear token index, used to find the object in the actual text document.
    pub fn root_items(&self) -> Vec<(usize, usize, Either<Section, Object>)> {
        let mut items = Vec::new();
        for (ix, item) in self.items.iter().enumerate() {
            match item {
                Item::Section(s, tk_ix) => {
                    items.push((ix, *tk_ix, Either::Left(s.clone())));
                },
                Item::Object(obj, tk_ix) => {
                    items.push((ix, *tk_ix, Either::Right(obj.clone())));
                },
                _ => { }
            }
        }
        items
    }

    pub fn level_one_items(&self) -> Vec<([usize;2], usize, Either<Subsection, Object>)> {
        let mut items = Vec::new();
        for (root_ix, root_item) in self.items.iter().enumerate() {
            match root_item {
                Item::Section(sec, _) => {
                    for (sec_ix, sec_item) in sec.items.iter().enumerate() {
                        match sec_item {
                            Item::Subsection(sub, tk_ix) => {
                                items.push(([root_ix, sec_ix], *tk_ix, Either::Left(sub.clone())));
                            },
                            Item::Object(obj, tk_ix) => {
                                items.push(([root_ix, sec_ix], *tk_ix, Either::Right(obj.clone())));
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

    pub fn level_two_items(&self) -> Vec<([usize;3], usize, Object)> {
        let mut items = Vec::new();
        for (root_ix, root_item) in self.items.iter().enumerate() {
            match root_item {
                Item::Section(sec, _) => {
                    for (sec_ix, sec_item) in sec.items.iter().enumerate() {
                        match sec_item {
                            Item::Subsection(sub, _) => {
                                for (sub_ix, sub_item) in sub.items.iter().enumerate() {
                                    match sub_item {
                                        Item::Object(obj, tk_ix) => {
                                            items.push(([root_ix, sec_ix, sub_ix], *tk_ix, obj.clone()));
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
                Item::Section(sec, _) => {
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
                Item::Subsection(sub, _) => {
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

    pub fn from_tokens<'a>(mut tks : impl Iterator<Item=Token<'a>> + Clone) -> Result<Document, TexError> {
        let mut all_tks = Vec::new();
        blocked_tokens(Vec::new(), &mut tks, &mut all_tks)
            .map_err(|e| TexError { msg : e, line : 0 })?;
        let mut doc_items : Option<Vec<Item>> = None;

        let mut tk_ix : usize = 0;
        for tk in all_tks {
            if is_doc_block(&tk) {

                // Count beginning of the block
                tk_ix += 1;

                if doc_items.is_some() {
                    return Err(TexError { msg : String::from("Multiple document blocks found"), line : 0 } );
                }

                match tk {
                    Either::Right(Block { inner, .. }) => {
                        let mut curr_section : Option<(Section, usize)> = None;
                        let mut curr_subsection : Option<(Subsection, usize)> = None;
                        let mut count = ItemCount::default();
                        let mut items : Vec<Item> = Vec::new();

                        for in_tk in inner {
                            next_item(&mut items, &mut tk_ix, &mut curr_section, &mut curr_subsection, &mut count, in_tk)
                                .map_err(|e| TexError { msg : e, line : 0 } )?;
                        }

                        // Push any residual subsections into any residual sections.
                        if let Some((subsection, sub_tk_ix)) = curr_subsection.take() {
                            if let Some((ref mut section, _)) = curr_section {
                                section.items.push(Item::Subsection(subsection, sub_tk_ix));
                            } else {
                                return Err(TexError { msg : String::from("Subsection without section parent"), line : 0});
                            }
                        }

                        // Push any residual sections.
                        if let Some((section, sec_tk_ix)) = curr_section {
                            items.push(Item::Section(section, sec_tk_ix));
                        }
                        doc_items = Some(items);
                    },
                    _ => {
                        panic!()
                    }
                }
            } else {
                match tk {
                    Either::Left(_) => {
                        tk_ix += 1;
                    },
                    Either::Right(ref block) => {
                        tk_ix += block.token_count();
                    }
                }
            }
        }

        match doc_items {
            Some(items) => Ok(Document { items }),
            None => Err(TexError { msg : String::from("Missing document block"), line : 0 })
        }
    }

    pub fn parse(s : &str) -> Result<Document, TexError> {
        let tks = Lexer::scan(s)?;
        Self::from_tokens(tks.iter())
    }

}

// TODO test against PLOS latex template
// https://journals.plos.org/plosone/s/latex

#[test]
fn test_bibtex_parser() {

    let txt = r#"
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
    "#;

    println!("{:#?}", BibParser::parse(txt));

}

#[test]
fn templates_are_parseable() {

    /*use crate::ui::*;

    let templates = [
        ARTICLE_TEMPLATE,
        REPORT_TEMPLATE,
        BOOK_TEMPLATE,
        LETTER_TEMPLATE,
        PRESENTATION_TEMPLATE
    ];

    for templ in templates {
        println!("{:#?}", Parser::parse(txt).unwrap());
    }*/
}

#[test]
fn test_latex_parser() {

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

        \end{document}
    "#;

    let cmd = r#"

    \documentclass[12pt,a4paper,oneside,draft]{report}

    "#;

    let math = r#"$$a$$ else"#;

    println!("{:#?}", Lexer::scan(txt).unwrap());
    println!("{:#?}", Parser::parse(txt).unwrap());

    // println!("{:?}", bib_field_value("{Guestrin, E. D. and Eizenman, M.}"));
    // println!("{:?}", bib_field("author = {Guestrin, E. D. and Eizenman, M.}"));
}

#[derive(Debug, Clone)]
pub struct References<'a>(Vec<BibEntry<'a>>);

impl<'a> AsRef<[BibEntry<'a>]> for References<'a> {

    fn as_ref(&self) -> &[BibEntry<'a>] {
        &self.0[..]
    }

}

/*impl<'a> IntoIterator for References<'a> {

    type Item = BibEntry<'a>;

    fn next(&mut self) -> &Self::Item {

    }

}*/

use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::multi::many0;
use nom::combinator::opt;
use nom::sequence::tuple;
use nom::multi::many1;
use nom::multi::separated_list1;
use nom::multi::separated_list0;
use nom::sequence::delimited;

pub struct BibParser {

}

fn space_delimited_bib(txt : &str) -> nom::IResult<&str, BibEntry> {
    let (res, ans) = tuple((many0(multispace1), super::bib_entry, many0(multispace1)))(txt)?;
    Ok((res, ans.1))
}

impl BibParser {

    pub fn parse(txt : &str) -> Result<References, String> {
        let (res, out) = delimited(
            multispace0,
            separated_list0(multispace1, super::bib_entry),
            multispace0
        )(txt).map_err(|e| e.to_string() )?;
        Ok(References(out))

        /*let (rem, _) = (many0(multispace1))(txt).map_err(|e| e.to_string() )?;
        let (_, ans) = separated_list0(super::bib_entry, many1(multispace1))(rem).map_err(|e| e.to_string() )?;
        Ok(ans)*/
    }

}

// cargo test --lib -- bib_parser --nocapture
#[test]
fn bib_parser() {

    /*let content = r#"

        @article{Guestrin2006Jun,
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

        @article{Aslin1975May,
	        author = {Aslin, Richard N. and Salapatek, Philip},
	        title = {{Saccadic localization of visual targets by the very young human infant}},
	        journal = {Percept. Psychophys.},
	        volume = {17},
	        number = {3},
	        pages = {293--302},
	        year = {1975},
	        month = {May},
	        issn = {1532-5962},
	        publisher = {Springer-Verlag},
	        doi = {10.3758/BF03203214}
        }

        "#;*/

    let content = r#"
        @article{Kooiker2016Sep,
	    author = {Kooiker, Marlou J. G. and Pel, Johan J. M. and Verbunt, H{\ifmmode\acute{e}\else\'{e}\fi}l{\ifmmode\grave{e}\else\`{e}\fi}ne J. M. and de Wit, Gerard C. and van Genderen, Maria M. and van der Steen, Johannes},
	    title = {{Quantification of visual function assessment using remote eye tracking in children: validity and applicability}},
	    journal = {Acta Ophthalmol.},
	    volume = {94},
	    number = {6},
	    pages = {599--608},
	    year = {2016},
	    month = {Sep},
	    issn = {1755-375X},
	    publisher = {John Wiley {\&} Sons, Ltd},
	    doi = {10.1111/aos.13038}
    }

    @article{Chang2021Dec,
	        author = {Chang, Melinda Y. and Borchert, Mark S.},
	        title = {{Validity and reliability of eye tracking for visual acuity assessment in children with cortical visual impairment}},
	        journal = {Journal of American Association for Pediatric Ophthalmology and Strabismus},
	        volume = {25},
	        number = {6},
	        pages = {334.e1--334.e5},
	        year = {2021},
	        month = {Dec},
	        issn = {1091-8531},
	        publisher = {Mosby},
	        doi = {10.1016/j.jaapos.2021.07.008}
        }

        @article{Johnson1990Apr,
	        author = {Johnson, Mark H.},
	        title = {{Cortical Maturation and the Development of Visual Attention in Early Infancy}},
	        journal = {J. Cognit. Neurosci.},
	        volume = {2},
	        number = {2},
	        pages = {81--95},
	        year = {1990},
	        month = apr,
	        issn = {0898-929X},
	        publisher = {MIT Press},
	        doi = {10.1162/jocn.1990.2.2.81}
        }

        @article{Valenza1994Jul,
	        author = {Valenza, Eloisa and Simion, Francesca and Umilt{\ifmmode\grave{a}\else\`{a}\fi}, Carlo},
	        title = {{Inhibition of return in newborn infants}},
	        journal = {Infant Behavior and Development},
	        volume = {17},
	        number = {3},
	        pages = {293--302},
	        year = {1994},
	        month = {Jul},
	        issn = {0163-6383},
	        publisher = {JAI},
	        doi = {10.1016/0163-6383(94)90009-4}
        }

    "#;

    use std::io::Read;
    use std::fs::File;
    println!("{:#?}", BibParser::parse(&content));
}

