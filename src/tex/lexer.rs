use std::fmt;
use nom::{
  IResult,
  bytes::complete::*,
  combinator::map_res,
  sequence::tuple,
  branch::alt
};
use nom::multi::many0;
use nom::character::complete::char;
use nom::combinator::opt;
use nom::sequence::delimited;
use nom::character::complete::alphanumeric1;
use nom::multi::{separated_list1, separated_list0};
use nom::character::complete::anychar;
use nom::multi::many1;
use nom::error::ContextError;
use nom::character::complete::alphanumeric0;
use std::cmp::{Eq, PartialEq};
use nom::sequence::separated_pair;
use std::str::FromStr;
use nom::character::complete::space0;
use std::ops::Range;
use nom::error::ErrorKind;
use nom::error::Error;
use either::Either;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command<'a> {

    pub cmd : &'a str,

    // Everything inside brackts like \documentclass[a4,14pt]{article} would contain a4, 14pt
    // If command has empty bracket with no options, Some(Vec::new()), contains some empty vector.
    // If command has no bracket, contains None.
    pub opts : Option<Vec<&'a str>>,

    // Everything inside curly braces like \documentclass[a4,14pt]{article} would contain article
    pub arg : Option<&'a str>,

    // Extra curly brace arguments
    pub extra_arg : Option<&'a str>
}

pub enum BaseCommand {

    Section(String),

    SubSection(String),

    Begin(String),

    End

}

impl<'a> fmt::Display for Command<'a> {

    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let opts = if let Some(opts) = &self.opts {
            if opts.len() > 0 {
                let mut opts_s = String::from("[");
                for i in 0..(opts.len() - 1) {
                    opts_s += opts[i];
                    opts_s += ","
                }
                opts_s += opts.last().unwrap();
                opts_s += "]";
                opts_s
            } else {
                String::from("[]")
            }
        } else {
            String::from("")
        };
        let arg = if let Some(arg) = self.arg {
            format!("{{{}}}", arg)
        } else {
            String::from("")
        };
        write!(f, "\\{}{}{}", self.cmd, opts, arg)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MathQuote {
    Single,
    Double
}

// Borrowed tokenized text representation.
// Last field contains the lenght in bytes taken by the token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token<'a> {

    Command(Command<'a>, usize),

    Text(&'a str, usize),

    Math(&'a str, MathQuote, usize),

    Comment(&'a str, usize),

    Reference(BibEntry<'a>, usize)

}

impl<'a> Token<'a> {

    pub fn kind(&self) -> TokenKind {
        match self {
            Self::Command(_, _) => TokenKind::Command,
            Self::Text(_, _) => TokenKind::Text,
            Self::Math(_, _, _) => TokenKind::Math,
            Self::Comment(_, _) => TokenKind::Comment,
            Self::Reference(_, _) => TokenKind::Reference
        }
    }

    // Can't really implement the trait here because it doesn't preserve lifetime.
    pub fn from_str(s : &str) -> Result<Token<'_>, String> {
        let (rem, tk) = eval_next_token(s).map_err(|e| format!("{}", e) )?;
        if !rem.is_empty() {
            // println!("Still not evaluated = '{}'", rem);
        }
        Ok(tk)
    }

}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Command,
    Text,
    Math,
    Comment,
    Reference
}

// Filter tokens of this type for comparison. Identity is determined by their relative order.
#[derive(Debug, Clone, Copy)]
pub enum Comparison {
    References,
    Sections
}

/// Owned Tokenized text representation. Creates tokens on the fly via
/// methods token_at and tokens, by storing only the tokens byte ranges.
#[derive(Debug, Clone, Default)]
pub struct TokenInfo {

    pub txt : String,

    pub kinds : Vec<TokenKind>,

    pub pos : Vec<Range<usize>>
}

#[derive(Debug, Clone)]
pub enum Difference {
    Added(usize, String),
    Removed(usize),
    Edited(usize, String)
}

#[derive(Debug, Clone)]
pub struct Block<'a> {

    pub start_cmd : Command<'a>,

    // pub start_range : Range<usize>,

    pub end_cmd : Option<Command<'a>>,

    // pub end_range : Range<usize>,

    pub inner : Vec<Either<Token<'a>, Block<'a>>>

}

impl<'a> Block<'a> {

    /// Token count forming this block, including the start and end commands.
    pub fn token_count(&'a self) -> usize {
        let mut count = 2;
        for tk in self.inner.iter() {
            match tk {
                Either::Left(_) => count += 1,
                Either::Right(block) => count += block.token_count()
            }
        }
        count
    }

}

pub fn blocked_tokens<'a>(
    mut curr_blocks : Vec<Block<'a>>,
    tks : &mut (impl Iterator<Item=Token<'a>> + Clone),
    out : &mut Vec<Either<Token<'a>, Block<'a>>>
) -> Result<(), String> {
    match tks.next() {
        Some(Token::Command(Command { cmd : "begin", opts, arg, extra_arg }, _)) => {
            let new_block = Block {
                start_cmd : Command { cmd : "begin", opts, arg, extra_arg },
                end_cmd : None,
                inner : Vec::new()
            };
            curr_blocks.push(new_block);
            blocked_tokens(curr_blocks, tks, out)
        },
        Some(Token::Command(Command { cmd : "end", arg, opts, extra_arg }, _)) => {
            if let Some(mut block) = curr_blocks.pop() {
                if arg == block.start_cmd.arg {
                    block.end_cmd = Some(Command { cmd : "end", arg, opts, extra_arg });
                    if let Some(mut prev_block) = curr_blocks.last_mut() {
                        prev_block.inner.push(Either::Right(block));
                    } else {
                        out.push(Either::Right(block));
                    }
                    blocked_tokens(curr_blocks, tks, out)
                } else {
                    Err(String::from("Invalid arg for end command"))
                }
            } else {
                Err(String::from("Missing begin command"))
            }
        },
        Some(other_token) => {
            if curr_blocks.len() >= 1 {
                if let Some(mut block) = curr_blocks.last_mut() {
                    block.inner.push(Either::Left(other_token));
                }
                blocked_tokens(curr_blocks, tks, out)
            } else {
                out.push(Either::Left(other_token));
                blocked_tokens(curr_blocks, tks, out)
            }
        },
        None => Ok(())
    }

    /*let mut curr_block : Option<usize> = None;
    let mut at_block = false;
    let mut blocked_tks = Vec::new();
    for tk in tks {
        match tk {
            Token::Command(Command { cmd : "begin", opts, arg, extra_arg }, _) => {
                blocked_tks.push(Either::Right(Block { start_cmd : Command { cmd : "begin", opts, arg, extra_arg }, inner : Vec::new(), end_cmd : None }));
                curr_block = Some(blocked_tks.len() - 1);
            },
            Token::Command(Command { cmd : "end", arg, opts, extra_arg }, _) => {
                if let Some(last_block) = blocked_tks.get_mut(curr_block.unwrap()) {
                    match last_block {
                        Either::Right(ref mut block) => {
                            if block.start_cmd.arg == arg {
                                block.end_cmd = Some(Command { cmd : "end", arg, opts, extra_arg});
                                if let Some(block_ix) = curr_block {
                                    if block_ix == 0 {
                                        curr_block = None;
                                    } else {
                                        curr_block = Some(block_ix - 1);
                                    }
                                } else {
                                    panic!()
                                }
                            } else {
                                return Err(String::from("Invalid closing"));
                            }
                        },
                        _ => panic!()
                    }
                } else {
                    return Err(String::from("Invalid command end"));
                }
            },
            other => {
                if let Some(block_ix) = curr_block {
                    match blocked_tks[block_ix] {
                        Either::Right(ref mut block) => {
                            block.inner.push(Either::Left(other));
                        },
                        _ => panic!()
                    }
                } else {
                    blocked_tks.push(Either::Left(other));
                }
            }
        }
    }

    Ok(blocked_tks)*/

    /*let mut blocked_tks = Vec::new();

    // Holds command, its start range and its inner tokens.
    let mut curr_blocks : Vec<(Command<'a>, Vec<Token<'a>>)> = Vec::new();
    for tk in tks {
        match curr_blocks.last_mut() {
            Some((ref start_cmd, ref mut inner_tokens)) => {
                match &tk {
                    Token::Command(cmd, _) => {
                        if cmd.cmd == "end" {
                            if cmd.arg == start_cmd.arg {
                                let inner = blocked_tokens(inner_tokens.drain(..));
                                blocked_tks.push(Either::Right(Block {
                                    start_cmd : start_cmd.clone(),
                                    end_cmd : cmd.clone(), inner,
                                }));
                                curr_block = None;
                            }
                        } else {

                            inner_tokens.push(tk.clone());
                        }
                    },
                    _ => {
                        inner_tokens.push(tk.clone());
                    }
                }
            },
            None => {
                block_start_or_cmd(&mut curr_block, &mut blocked_tks, &tk)?;
            }
        }
    }

    Ok(blocked_tks)*/
}

/*fn block_start_or_cmd<'a>(
    curr_blocks : &mut Vec<(Command<'a>, Vec<Token<'a>>)>,
    blocked_tks : &mut Vec<Either<Token<'a>, Block<'a>>>,
    tk : &Token<'a>
) -> Result<(), String> {
    match &tk {
        Token::Command(cmd, _) => {
            if cmd.cmd == "begin" {
                curr_blocks.push((cmd.clone(), Vec::new()));
            } else {
                blocked_tks.push(Either::Left(tk.clone()));
            }
        },
        _ => {
            blocked_tks.push(Either::Left(tk));
        }
    }
    Ok(())
}*/

impl TokenInfo {

    pub fn sections(&self) -> Vec<Range<usize>> {
        self.kinds.iter()
            .enumerate()
            .filter(|(ix, kind)| {
                let range : Range<usize> = self.pos[*ix].clone();
                **kind == TokenKind::Command && self.txt[range.clone()].starts_with("\\section{" ) && self.txt[range.clone()].ends_with("}")
            })
            .map(|(ix, _)| self.pos[ix].clone() )
            .collect()
    }

    pub fn token_at<'a>(&'a self, ix : usize) -> Token<'a> {
        Token::from_str(&self.txt[self.pos[ix].clone()]).unwrap()
    }

    pub fn tokens<'a>(&'a self) -> impl Iterator<Item=Token<'a>> + Clone + 'a {
        (0..self.kinds.len()).map(|ix| self.token_at(ix) )
    }

    pub fn references(&self) -> Vec<Range<usize>> {

        assert!(self.kinds.len() == self.pos.len() );
        self.kinds.iter()
            .enumerate()
            .filter(|(ix, kind)| **kind == TokenKind::Reference )
            .map(|(ix, _)| self.pos[ix].clone() )
            .collect()
    }

    // Compare tokens of two tokenized texts. Identity of tokens are determined
    // by their order (i.e. section 1 at a is same section 1 at b if they have the
    // same text). Removing a section is therefore represented by editing all
    // sections after the current one and removing the last one.
    pub fn compare_tokens(&self, other : &TokenInfo, kind_comp : Comparison) -> Vec<Difference> {
        let mut diffs = Vec::new();
        let (this_tokens, other_tokens) = match kind_comp {
            Comparison::Sections => (self.sections(), other.sections()),
            Comparison::References => (self.references(), other.references())
        };
        for ix in 0..(this_tokens.len().max(other_tokens.len())) {
            match (this_tokens.get(ix), other_tokens.get(ix)) {
                (Some(this_pos), Some(other_pos)) => {
                    if &self.txt[this_pos.clone()] != &other.txt[other_pos.clone()] {
                        diffs.push(Difference::Edited(ix, other.txt[other_pos.clone()].to_string()));
                    }
                },
                (None, Some(other_pos)) => {
                    diffs.push(Difference::Added(ix, other.txt[other_pos.clone()].to_string()));
                },
                (Some(_), None) => {
                    diffs.push(Difference::Removed(ix));
                },
                (None, None) => {

                }
            }
        }

        diffs
    }

}

fn double_quoted_math(s : &str) -> IResult<&str, &str> {
    delimited(tag("$$"), take_while(|c| c != '$' ), tag("$$"))(s)
}

fn single_quoted_math(s : &str) -> IResult<&str, &str> {
    delimited(tag("$"), take_while(|c| c != '$' ), tag("$"))(s)
}

fn math(s : &str) -> IResult<&str, (&str, MathQuote)> {
    match double_quoted_math(s) {
        Ok((rem, math)) => {
            Ok((rem, (math, MathQuote::Double)))
        },
        Err(_) => {
            single_quoted_math(s).map(|(rem, txt)| (rem, (txt, MathQuote::Single)))
        }
    }
}

fn cmd_options(s : &str) -> IResult<&str, Vec<&str>> {
    delimited(tag("["), separated_list0(tag(","), take_till(|c| c == ']' || c == ',')), tag("]"))(s)
}

fn cmd_arg(s : &str) -> IResult<&str, &str> {
    delimited(tag("{"), take_till(|c| c == '}'), tag("}"))(s)
}

fn comment(s : &str) -> IResult<&str, &str> {
    tuple((char('%'), take_till(|c| c == '\n')))(s).map(|(rem, res)| (rem, res.1) )
}

fn valid_cmd_or_arg<'a>(s : &'a str) -> Result<(), nom::Err<Error<&'a str>>> {
    if s.contains("{") || s.contains("}") || s.contains("\\") || s.contains("\n") {
        Err(nom::Err::Failure(Error::new(s, ErrorKind::Fail)))
    } else {
        Ok(())
    }
}

pub fn command(s : &str) -> IResult<&str, Command> {
    let (rem, cmd) = tuple((char('\\'), is_not("{[\n \t")))(s)?;
    if cmd.1.contains("\\") {
        return Err(nom::Err::Failure(Error::new(cmd.1, ErrorKind::Fail)));
    }
    let (rem, opts) = opt(cmd_options)(rem)?;

    if let Some(opts) = &opts {
        for opt in opts {
            valid_cmd_or_arg(opt)?;
        }
    }

    let (rem, arg) = opt(cmd_arg)(rem)?;
    if let Some(arg) = arg {
        valid_cmd_or_arg(&arg)?;
    }

    let (rem, extra_arg) = opt(cmd_arg)(rem)?;
    if let Some(arg) = extra_arg {
        valid_cmd_or_arg(&arg)?;
    }

    // This means the command argument and/or options were not parsed
    // correctly.
    if rem.starts_with("{") || rem.starts_with("[") {
        return Err(nom::Err::Failure(Error::new(cmd.1, ErrorKind::Fail)));
    }

    Ok((rem, Command { cmd : cmd.1, arg, extra_arg, opts }))
}

fn text(s : &str) -> IResult<&str, &str> {

    // TODO parse up to these characters or end AND cannot start with any of
    // them either: (e.g. s.chars().next().starts_with(..))

    is_not("\\%$@")(s)
}

fn eval_next_token(txt : &str) -> IResult<&str, Token> {
    match command(txt) {
        Ok((rem, cmd)) => Ok((rem, Token::Command(cmd, txt.len() - rem.len()))),
        Err(_) => match math(txt) {
            Ok((rem, (math, quote))) => Ok((rem, Token::Math(math, quote, txt.len() - rem.len()))),
            Err(_) => match comment(txt) {
                Ok((rem, comment)) => Ok((rem, Token::Comment(comment, txt.len() - rem.len()))),
                Err(_) => match bib_entry(txt) {
                    Ok((rem, entry)) => Ok((rem, Token::Reference(entry, txt.len() - rem.len()))),
                    Err(_) => match text(txt) {
                        Ok((rem, text)) => { Ok((rem, Token::Text(text, txt.len() - rem.len()))) },
                        Err(e) => Err(e)
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TexTokens<'a> {

    txt : &'a str,

    tokens : Vec<Token<'a>>,

    offset : usize

}

pub struct Lexer {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Book,
    Booklet,
    Article,
    Conference,
    Inbook,
    Incollection,
    Inproceedings,
    Manual,
    MasterThesis,
    Misc,
    PhdThesis,
    Proceedings,
    TechReport,
    Unpublished
}

impl Entry {

    pub fn pretty(&self) -> &'static str {
        match self {
            Self::Book => "Book",
            Self::Booklet => "Booklet",
            Self::Article => "Article",
            Self::Conference => "Conference",
            Self::Inbook => "In book",
            Self::Incollection => "In collection",
            Self::Inproceedings => "In proceedings",
            Self::Manual => "Manual",
            Self::MasterThesis => "Master thesis",
            Self::Misc => "Misc",
            Self::PhdThesis => "PhD Thesis",
            Self::Proceedings => "Proceedings",
            Self::TechReport => "Tech report",
            Self::Unpublished => "Unpublished"
        }
    }
}

impl FromStr for Entry {

    type Err = ();

    fn from_str(s : &str) -> Result<Self, ()> {
        match s {
            "book" => Ok(Entry::Book),
            "booklet" => Ok(Entry::Booklet),
            "article" => Ok(Entry::Article),
            "conference" => Ok(Entry::Conference),
            "inbook" => Ok(Entry::Inbook),
            "incollection" => Ok(Entry::Incollection),
            "inproceedings" => Ok(Entry::Inproceedings),
            "manual" => Ok(Entry::Manual),
            "masterthesis" => Ok(Entry::MasterThesis),
            "misc" => Ok(Entry::Misc),
            "phdthesis" => Ok(Entry::PhdThesis),
            "proceedings" => Ok(Entry::Proceedings),
            "techreport" => Ok(Entry::TechReport),
            "unpublished" => Ok(Entry::Unpublished),
            _ => Err(())
        }
    }

}

/*
-- Valid fields
address
annote
author
booktitle
chapter
crossref
edition
editor
howpublished
institution
journal
key
month
note
number
organization
pages
publisher
school
series
title
type
volume
year
*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibEntry<'a> {

    entry : Entry,

    key : &'a str,

    fields : Vec<(&'a str, &'a str)>

}

impl<'a> BibEntry<'a> {

    pub fn key(&'a self) -> &'a str {
        self.key
    }

    pub fn find_field(&'a self, key : &'a str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| &key[..] == &k[..] ).map(|(_, v)| *v )
    }

    pub fn title(&'a self) -> Option<&'a str> {
        self.find_field("title")
    }

    pub fn author(&'a self) -> Option<&'a str> {
        self.find_field("author")
    }

    pub fn year(&'a self) -> Option<&'a str> {
        self.find_field("year")
    }

    pub fn entry(&self) -> Entry {
        self.entry
    }

    pub fn entry_pretty(&self) -> &'a str {
        self.entry.pretty()
    }

}

fn bib_field_value(s : &str) -> IResult<&str, &str> {
    alt((
        delimited(tag("{{"), is_not("}"), tag("}}")),
        delimited(tag("{"), is_not("}"), tag("}")),
        delimited(tag("\""), is_not("\""), tag("\"")),
    ))(s)
}

fn bib_field(s : &str) -> IResult<&str, (&str, &str)> {
    tuple((
        opt(space0),
        separated_pair(is_not("="), tuple((space0, char('='), space0)), bib_field_value),
        opt(space0)
    ))(s).map(|(rem, s)| (rem, (s.1.0.trim(), s.1.1.trim())) )
}

fn entry(s : &str) -> IResult<&str, Entry> {
    alt((
        tag("book"),
        tag("booklet"),
        tag("article"),
        tag("conference"),
        tag("inbook"),
        tag("incollection"),
        tag("inproceedings"),
        tag("manual"),
        tag("masterthesis"),
        tag("misc"),
        tag("phdthesis"),
        tag("proceedings"),
        tag("techreport"),
        tag("unpublished")
    ))(s).map(|(rem, s)| (rem, Entry::from_str(s).unwrap()) )
}

fn bib_entry(s : &str) -> IResult<&str, BibEntry> {
    let (rem, entry) = delimited(char('@'), entry, tag("{"))(s)?;
    let (rem, key) = is_not(",")(rem)?;
    let (rem, _) = take(1usize)(rem)?;
    // println!("{}", rem);
    let (rem, fields) = separated_list1(tag(","), bib_field)(rem)?;
    let (rem, _) = is_not("}")(rem)?;
    Ok((rem, BibEntry {
        entry,
        key : key.trim(),
        fields
    }))
}

impl Lexer {

    pub fn scan(s : &str) -> Result<TexTokens<'_>, String> {
        let (rem, tokens) = many0(eval_next_token)(s).map_err(|e| format!("{}", e ) )?;
        if !rem.is_empty() {
            return Err(format!("Could not parse document end: {}", rem));
        }
        /*let lens : Vec<_> = tokens.iter().map(|tk| {
            match tk {
                Token::Command(Command { cmd, opts, arg }) => {
                    let backlash_len = 1;
                    let curly_len = 2;
                    let bracket_len = if opts.is_some() { 2 } else { 0 };
                    let opts_len = if let Some(opts) = &opts {
                        opts.iter().fold(0, |len, opt| len + opt.chars().count() )
                    } else {
                        0
                    };
                    let comma_len = if let Some(opts) = &opts {
                        if opts.len() > 1 {
                            opts.len() - 1
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    let arg_len = arg.map(|a| a.chars().count() ).unwrap_or(0);
                    backlash_len + cmd.len() + curly_len + arg_len + bracket_len + opts_len + comma_len
                },
                Token::Text(txt) => {
                    txt.chars().count()
                },
                Token::Math(math, quote) => {
                    let quote_len = match quote {
                        MathQuote::Single => {
                            2
                        },
                        MathQuote::Double => {
                            4
                        }
                    };
                    math.chars().count() + quote_len
                },
                Token::Comment(comm) => {
                    let percent_len = 1;
                    comm.chars().count() + percent_len
                },
                Token::Reference(entry) => {
                    // TODO implement here
                    0
                }
            }
        }).collect();
        let mut curr_offset = 0;
        let offsets : Vec<_> = lens.iter()
            .map(|len| { let off = curr_offset; curr_offset += len; off } )
            .collect();*/
        // assert offset + len of last token equals string len.
        Ok(TexTokens {
            txt : s,
            tokens,
            offset : 0
            // offsets,
            // lens
        })
    }

    // pub fn offsets(&self) -> impl Iterator<Item=usize> {
    // }
}

impl<'a> TexTokens<'a> {

    /// Returns slice indices taken by each token. Can be used to
    /// index the underlying slice.
    pub fn positions(&'a mut self) -> impl Iterator<Item=Range<usize>> + 'a {
        self.offset = 0;
        self.tokens.iter().map(|tk| {
            match tk {
                Token::Command(_, len) => token_pos(&mut self.offset, *len),
                Token::Text(_, len) => token_pos(&mut self.offset, *len),
                Token::Math(_, _, len) => token_pos(&mut self.offset, *len),
                Token::Comment(_, len) => token_pos(&mut self.offset, *len),
                Token::Reference(_, len) => token_pos(&mut self.offset, *len),
            }
        })
    }

    pub fn kinds(&'a self) -> impl Iterator<Item=TokenKind> + 'a {
        self.tokens.iter().map(|tk| tk.kind() )
    }

    pub fn to_owned(&self) -> TokenInfo {
        let mut tokens = self.clone();
        let n_tokens = tokens.clone().iter().count();
        let kinds : Vec<TokenKind> = tokens.kinds().collect();
        let pos : Vec<Range<usize>> = tokens.positions().collect();

        assert!(n_tokens == kinds.len());
        assert!(n_tokens == pos.len());

        TokenInfo {
            txt : self.txt.to_string(),
            pos,
            kinds
        }
    }

    pub fn iter(&'a self) -> impl Iterator<Item=Token<'a>> + Clone {
        self.tokens.clone().into_iter()
    }

}

fn token_pos(offset : &mut usize, len : usize) -> Range<usize> {
    let range = *offset..(*offset+len);
    *offset += len;
    range
}

