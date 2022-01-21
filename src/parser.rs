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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command<'a> {
    cmd : &'a str,

    // Everything inside brackts like \documentclass[a4,14pt]{article} would contain a4, 14pt
    // If command has empty bracket with no options, Some(Vec::new()), contains some empty vector.
    // If command has no bracket, contains None.
    opts : Option<Vec<&'a str>>,

    // Everything inside curly braces like \documentclass[a4,14pt]{article} would contain article
    arg : Option<&'a str>
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

/// Owned Tokenized text representation.
#[derive(Debug, Clone)]
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

    pub fn references(&self) -> Vec<Range<usize>> {
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

fn command(s : &str) -> IResult<&str, Command> {
    // TODO also expand is_not to accept any space-like char to end commands without parameters.
    let (rem, cmd) = tuple((char('\\'), is_not("{[\n")))(s)?;
    let (rem, opts) = opt(cmd_options)(rem)?;
    let (rem, arg) = opt(cmd_arg)(rem)?;
    Ok((rem, Command { cmd : cmd.1, arg, opts }))
}

fn text(s : &str) -> IResult<&str, &str> {
    is_not("\\%$@")(s)
    // take_till(|c| c == '\\' || c == '%' || c == '$' )(s)
}

// delimited(char('\section{'), is_not("}"), char('}'))(input)

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
                        Ok((rem, txt)) => Ok((rem, Token::Text(txt, txt.len() - rem.len()))),
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

    // Character offset of each token.
    // offsets : Vec<usize>,

    // Character len of each token.
    // lens : Vec<usize>

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
    TechReprot,
    Unpublished
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
            "techreport" => Ok(Entry::TechReprot),
            "unpublished" => Ok(Entry::Unpublished),
            _ => Err(())
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibEntry<'a> {

    entry : Entry,

    key : &'a str,

    fields : Vec<(&'a str, &'a str)>

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
    ))(s).map(|(rem, s)| (rem, s.1) )
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
    println!("{}", rem);
    let (rem, fields) = separated_list1(tag(","), bib_field)(rem)?;
    let (rem, _) = is_not("}")(rem)?;
    Ok((rem, BibEntry {
        entry,
        key,
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
        let kinds : Vec<TokenKind> = tokens.kinds().collect();
        let pos : Vec<Range<usize>> = tokens.positions().collect();
        TokenInfo {
            txt : self.txt.to_string(),
            pos,
            kinds
        }
    }

}

fn token_pos(offset : &mut usize, len : usize) -> Range<usize> {
    let range = *offset..(*offset+len);
    *offset += len;
    range
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

    // println!("{:?}", bib_field_value("{Guestrin, E. D. and Eizenman, M.}"));
    // println!("{:?}", bib_field("author = {Guestrin, E. D. and Eizenman, M.}"));
}

