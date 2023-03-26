use comemo::Prehashed;
use typst::eval::Library;
use typst::font::{Font, FontBook, FontInfo, FontVariant};
use std::cell::RefCell;
use typst::syntax::{Source, SourceId};
use std::path::{Path, PathBuf};
use typst::util::{Buffer, PathExt};
use once_cell::sync::OnceCell;
use walkdir::WalkDir;
use memmap2::Mmap;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use dirs;
use same_file::{is_same_file, Handle};
use typst::World;
use siphasher::sip128::{Hasher128, SipHasher};
use std::error::Error;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor};
use std::cell::RefMut;
use std::collections::HashMap;
use std::hash::Hash;
use std::fs::File;
use std::io::Read;
use elsa::FrozenVec;
use typst::syntax::{ast::{Expr, Markup, Arg, AstNode}};
use crate::tex::{Section, Subsection, Item};
use typst::diag::{FileError, FileResult, SourceError, StrResult};
use std::rc::Rc;
use gtk4::gio;
use std::sync::Arc;

fn first_text(mark : &Markup) -> String {
    for e in mark.exprs() {
        match e {
            Expr::Text(txt) => {
                return txt.get().to_string();
            },
            _ => { }
        }
    }
    String::new()
}

pub fn push_to_curr_items(curr_subsec : &mut Option<(Subsection, usize)>, curr_sec : &mut Option<(Section, usize)>, its : &mut Vec<Item>, it : Item) {
    if let Some(sub) = curr_subsec.as_mut() {
        sub.0.items.push(it);
    } else if let Some(sec) = curr_sec.as_mut() {
        sec.0.items.push(it);
    } else {
        its.push(it);
    }
}

fn process_errors(source : &Source, errs : Vec<SourceError>) -> Vec<(usize, String)> {
    let mut out = Vec::with_capacity(errs.len());
    for e in errs {
        let line = source.byte_to_line(source.range(e.span).start).unwrap_or(0);
        let msg = e.message.to_string();
        out.push((line, msg));
    }
    out
}

pub fn parse_doc(path : &Path, txt : String) -> Result<crate::tex::Document, Vec<(usize, String)>> {

    use crate::tex::*;

    let mut items = Vec::new();

    let mut curr_sec : Option<(Section, usize)> = None;
    let mut curr_subsec : Option<(Subsection, usize)> = None;
    let mut curr_sec_index = 1;
    let mut curr_subsec_local_index = 1;
    let mut curr_subsec_global_index = 1;
    let source = Source::new(SourceId::detached(), path, txt);
    let mut eq_ix = 1;
    let mut tbl_ix = 1;
    let mut img_ix = 1;
    let mut code_ix = 1;
    let ast = source.ast().map_err(|e| process_errors(&source, *e) )?;
    let mut line;
    for expr in ast.exprs() {
        line = source.byte_to_line(source.range(expr.span()).start).unwrap_or(0);
        match expr {
            Expr::Heading(head) => {
                match head.level().get() {
                    1 => {
                        curr_subsec_local_index = 1;

                        if let (Some(sub), Some(sec)) = (curr_subsec.take(), curr_sec.as_mut()) {
                            sec.0.items.push(Item::Subsection(sub.0, sub.1));
                        }

                        if let Some(prev) = curr_sec.take() {
                            items.push(Item::Section(prev.0, prev.1));
                            curr_sec_index += 1;
                        }
                        curr_sec = Some((Section {
                            name : first_text(&head.body()),
                            index : curr_sec_index,
                            items : Vec::new()
                        }, line));
                    },
                    2 => {
                        if let Some(prev) = curr_subsec.take() {
                            if let Some(sec) = curr_sec.as_mut() {
                                sec.0.items.push(Item::Subsection(prev.0, prev.1));
                                curr_subsec_local_index += 1;
                                curr_subsec_global_index += 1;
                            }
                        }
                        curr_subsec = Some((Subsection {
                            name : first_text(&head.body()),
                            parent_index : curr_sec_index,
                            local_index : curr_subsec_local_index,
                            global_index : curr_subsec_global_index,
                            items : Vec::new()
                        }, line));
                    },
                    _ => { }
                }
            },
            Expr::Equation(_) => {
                let it = Item::Object(Object::Equation(eq_ix, ObjectIndex::Root(0), Some(String::new())), line);
                push_to_curr_items(&mut curr_subsec, &mut curr_sec, &mut items, it);
                eq_ix += 1;
            },
            Expr::Code(_) => {
                let it = Item::Object(Object::Code(code_ix, ObjectIndex::Root(0), Some(String::new())), line);
                push_to_curr_items(&mut curr_subsec, &mut curr_sec, &mut items, it);
                code_ix += 1;
            },
            Expr::FuncCall(call) => {
                match call.callee() {
                    Expr::Ident(id) => {
                        let func = id.get().to_string();
                        match call.args().items().next() {
                            Some(Arg::Pos(expr)) => {
                                match expr {
                                    Expr::Str(s) => {
                                        let arg = s.get().to_string();
                                        match &func[..] {
                                            "image" => {
                                                let it = Item::Object(Object::Image(img_ix, ObjectIndex::Root(0), Some(arg)), line);
                                                push_to_curr_items(&mut curr_subsec, &mut curr_sec, &mut items, it);
                                                img_ix += 1;
                                            },
                                            "bibliography" => {
                                                let it = Item::Object(Object::Bibliography(0, arg), line);
                                                push_to_curr_items(&mut curr_subsec, &mut curr_sec, &mut items, it);
                                            },
                                            "table" => {
                                                let it = Item::Object(Object::Table(tbl_ix, ObjectIndex::Root(0), Some(arg)), line);
                                                push_to_curr_items(&mut curr_subsec, &mut curr_sec, &mut items, it);
                                                tbl_ix += 1;
                                            },
                                            "code" => {

                                            },
                                            _ => {

                                            }
                                        }
                                    },
                                    _ => { }
                                }
                            },
                            _ => { }
                        }
                    },
                    _ => { }
                }
            },
            _ => { }
        }
    }

    if let (Some(sub), Some(sec)) = (curr_subsec.take(), curr_sec.as_mut()) {
        sec.0.items.push(Item::Subsection(sub.0, sub.1));
    }

    if let Some(sec) = curr_sec.take() {
        items.push(Item::Section(sec.0, sec.1));
    }

    Ok(crate::tex::Document { items })
}

type CodespanResult<T> = Result<T, CodespanError>;

type CodespanError = codespan_reporting::files::Error;

#[derive(Clone)]
pub struct Fonts {
    pub book : Arc<Prehashed<FontBook>>,
    pub fonts : Arc<[FontSlot]>
}

impl Fonts {

    pub fn new(res : &gio::Resource) -> Self {
        let searcher = FontSearcher::new(res);
        Self {
            fonts : searcher.fonts.into(),
            book : Arc::new(Prehashed::new(searcher.book)),
        }
    }

}

/// Searches for fonts.
struct FontSearcher {
    book: FontBook,
    fonts: Vec<FontSlot>,
}

impl FontSearcher {

    fn add_embedded(&mut self, resource : &gio::Resource) {
        let mut add = |bytes: &[u8]| {

            // Unsafe required because the returned gio::Bytes doesn't have 'static lifetime.
            // (although it is, the resource is embedded in the binary).
            let buffer = Buffer::from_static(unsafe { std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()) });

            for (i, font) in Font::iter(buffer).enumerate() {
                self.book.push(font.info().clone());
                self.fonts.push(FontSlot {
                    path: PathBuf::new(),
                    index: i as u32,
                    font: OnceCell::from(Some(font)),
                });
            }
        };

        let font_prefix = "/io/github/limads/drafts/fonts/";
        let fonts = [
            "LinLibertine_R.ttf",
            "LinLibertine_RB.ttf",
            "LinLibertine_RBI.ttf",
            "LinLibertine_RI.ttf",
            "NewCMMath-Book.otf",
            "NewCMMath-Regular.otf",
            "DejaVuSansMono.ttf",
            "DejaVuSansMono-Bold.ttf"
        ];
        for font in fonts {
            let font_path = format!("{}{}", font_prefix, font);
            add(resource.lookup_data(&font_path, gio::ResourceLookupFlags::empty()).unwrap().as_ref());
        }
    }

    /// Create a new, empty system searcher.
    fn new(res : &gio::Resource) -> Self {
        let mut searcher = Self { book: FontBook::new(), fonts: vec![] };
        searcher.search_system();
        searcher.add_embedded(res);
        searcher
    }

    /// Search for fonts in the linux system font directories.
    fn search_system(&mut self) {
        self.search_dir("/usr/share/fonts");
        self.search_dir("/usr/local/share/fonts");
        if let Some(dir) = dirs::font_dir() {
            self.search_dir(dir);
        }
    }

    /// Search for all fonts in a directory recursively.
    fn search_dir(&mut self, path: impl AsRef<Path>) {
        for entry in WalkDir::new(path)
            .follow_links(true)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ttf" | "otf" | "TTF" | "OTF" | "ttc" | "otc" | "TTC" | "OTC"),
            ) {
                self.search_file(path);
            }
        }
    }

    /// Index the fonts in the file at the given path.
    fn search_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if let Ok(file) = File::open(path) {
            if let Ok(mmap) = unsafe { Mmap::map(&file) } {
                for (i, info) in FontInfo::iter(&mmap).enumerate() {
                    self.book.push(info);
                    self.fonts.push(FontSlot {
                        path: path.into(),
                        index: i as u32,
                        font: OnceCell::new(),
                    });
                }
            }
        }
    }
}

/// A world that provides access to the operating system.
struct SystemWorld {
    root: PathBuf,
    library: Prehashed<Library>,
    book: Arc<Prehashed<FontBook>>,
    fonts: Arc<[FontSlot]>,
    hashes: RefCell<HashMap<PathBuf, FileResult<PathHash>>>,
    paths: RefCell<HashMap<PathHash, PathSlot>>,
    sources: FrozenVec<Box<Source>>,
    main: SourceId,
}

/// Holds details about the location of a font and lazily the font itself.
pub struct FontSlot {
    path: PathBuf,
    index: u32,
    font: OnceCell<Option<Font>>,
}

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Default)]
struct PathSlot {
    source: OnceCell<FileResult<SourceId>>,
    buffer: OnceCell<FileResult<Buffer>>,
}

impl SystemWorld {
    fn new(root: PathBuf, fonts : Fonts) -> Self {
        Self {
            root,
            library: Prehashed::new(typst_library::build()),
            book: fonts.book.clone(),
            fonts: fonts.fonts.clone(),
            hashes: RefCell::default(),
            paths: RefCell::default(),
            sources: FrozenVec::new(),
            main: SourceId::detached(),
        }
    }
}

impl World for SystemWorld {
    fn root(&self) -> &Path {
        &self.root
    }

    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn main(&self) -> &Source {
        self.source(self.main)
    }

    fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        self.slot(path)?
            .source
            .get_or_init(|| {
                let buf = read(path)?;
                let text = String::from_utf8(buf)?;
                Ok(self.insert(path, text))
            })
            .clone()
    }

    fn source(&self, id: SourceId) -> &Source {
        &self.sources[id.into_u16() as usize]
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn font(&self, id: usize) -> Option<Font> {
        let slot = &self.fonts[id];
        slot.font
            .get_or_init(|| {
                let data = self.file(&slot.path).ok()?;
                Font::new(data, slot.index)
            })
            .clone()
    }

    fn file(&self, path: &Path) -> FileResult<Buffer> {
        self.slot(path)?
            .buffer
            .get_or_init(|| read(path).map(Buffer::from))
            .clone()
    }
}

impl SystemWorld {
    fn slot(&self, path: &Path) -> FileResult<RefMut<PathSlot>> {
        let mut hashes = self.hashes.borrow_mut();
        let hash = match hashes.get(path).cloned() {
            Some(hash) => hash,
            None => {
                let hash = PathHash::new(path);
                if let Ok(canon) = path.canonicalize() {
                    hashes.insert(canon.normalize(), hash.clone());
                }
                hashes.insert(path.into(), hash.clone());
                hash
            }
        }?;

        Ok(std::cell::RefMut::map(self.paths.borrow_mut(), |paths| {
            paths.entry(hash).or_default()
        }))
    }

    fn insert(&self, path: &Path, text: String) -> SourceId {
        let id = SourceId::from_u16(self.sources.len() as u16);
        let source = Source::new(id, path, text);
        self.sources.push(Box::new(source));
        id
    }

    fn relevant(&mut self, event: &notify::Event) -> bool {
        match &event.kind {
            notify::EventKind::Any => {}
            notify::EventKind::Access(_) => return false,
            notify::EventKind::Create(_) => return true,
            notify::EventKind::Modify(kind) => match kind {
                notify::event::ModifyKind::Any => {}
                notify::event::ModifyKind::Data(_) => {}
                notify::event::ModifyKind::Metadata(_) => return false,
                notify::event::ModifyKind::Name(_) => return true,
                notify::event::ModifyKind::Other => return false,
            },
            notify::EventKind::Remove(_) => {}
            notify::EventKind::Other => return false,
        }

        event.paths.iter().any(|path| self.dependant(path))
    }

    fn dependant(&self, path: &Path) -> bool {
        self.hashes.borrow().contains_key(&path.normalize())
            || PathHash::new(path)
                .map_or(false, |hash| self.paths.borrow().contains_key(&hash))
    }

    fn reset(&mut self) {
        self.sources.as_mut().clear();
        self.hashes.borrow_mut().clear();
        self.paths.borrow_mut().clear();
    }
}

pub fn compile(path : &Path, fonts : Fonts) -> Result<Vec<u8>, Vec<(usize, String)>> {
    let parent_path = path.parent()
        // .ok_or(vec![String::from("Missing parent directory"))?
        .unwrap()
        .to_owned();
    let mut world = SystemWorld::new(parent_path, fonts);

    world.reset();
    world.main = world.resolve(&path).unwrap();
        //.map_err(|err| err.to_string())?;

    match typst::compile(&world) {
        Ok(doc) => {
            Ok(typst::export::pdf(&doc))
        },
        Err(errs) => {
            let mut out_errs = Vec::new();
            for e in errs.iter() {
                if let Some(src) = world.sources.iter().find(|s| s.id() == e.span.source() ) {
                    let line = src.byte_to_line(src.range(e.span).start).unwrap_or(0);
                    let msg = e.message.to_string();
                    out_errs.push((line, msg));
                }
            }
            Err(out_errs)
        }
    }

}

/// A hash that is the same for all paths pointing to the same entity.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct PathHash(u128);

impl PathHash {
    fn new(path: &Path) -> FileResult<Self> {
        let f = |e| FileError::from_io(e, path);
        let handle = Handle::from_path(path).map_err(f)?;
        let mut state = SipHasher::new();
        handle.hash(&mut state);
        Ok(Self(state.finish128().as_u128()))
    }
}

/// Read a file.
fn read(path: &Path) -> FileResult<Vec<u8>> {
    let f = |e| FileError::from_io(e, path);
    let mut file = File::open(path).map_err(f)?;
    if file.metadata().map_err(f)?.is_file() {
        let mut data = vec![];
        file.read_to_end(&mut data).map_err(f)?;
        Ok(data)
    } else {
        Err(FileError::IsDirectory)
    }
}

impl<'a> codespan_reporting::files::Files<'a> for SystemWorld {
    type FileId = SourceId;
    type Name = std::path::Display<'a>;
    type Source = &'a str;

    fn name(&'a self, id: SourceId) -> CodespanResult<Self::Name> {
        Ok(World::source(self, id).path().display())
    }

    fn source(&'a self, id: SourceId) -> CodespanResult<Self::Source> {
        Ok(World::source(self, id).text())
    }

    fn line_index(&'a self, id: SourceId, given: usize) -> CodespanResult<usize> {
        let source = World::source(self, id);
        source
            .byte_to_line(given)
            .ok_or_else(|| CodespanError::IndexTooLarge {
                given,
                max: source.len_bytes(),
            })
    }

    fn line_range(
        &'a self,
        id: SourceId,
        given: usize,
    ) -> CodespanResult<std::ops::Range<usize>> {
        let source = World::source(self, id);
        source
            .line_to_range(given)
            .ok_or_else(|| CodespanError::LineTooLarge { given, max: source.len_lines() })
    }

    fn column_number(
        &'a self,
        id: SourceId,
        _: usize,
        given: usize,
    ) -> CodespanResult<usize> {
        let source = World::source(self, id);
        source.byte_to_column(given).ok_or_else(|| {
            let max = source.len_bytes();
            if given <= max {
                CodespanError::InvalidCharBoundary { given }
            } else {
                CodespanError::IndexTooLarge { given, max }
            }
        })
    }
}

