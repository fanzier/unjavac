use std::borrow::Cow;
use std::cmp;
use std::io;
use std::ops::{Add, AddAssign};

pub trait PrettyWith<Ctx: ?Sized> {
    fn pretty_with(&self, context: &Ctx) -> Doc;
}

pub trait Pretty {
    fn pretty(&self) -> Doc;
}

impl<T: PrettyWith<()>> Pretty for T {
    fn pretty(&self) -> Doc {
        self.pretty_with(&())
    }
}

pub fn group(doc: Doc) -> Doc {
    doc.group()
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Doc {
    Empty,
    Append(Box<Doc>, Box<Doc>),
    Group(Box<Doc>),
    Nest(usize, Box<Doc>),
    Space(bool),
    Newline,
    Text(Cow<'static, str>),
}

pub fn empty() -> Doc {
    Doc::Empty
}

pub fn newline() -> Doc {
    Doc::Newline
}

pub fn breakline() -> Doc {
    Doc::Space(false)
}

pub fn spaceline() -> Doc {
    Doc::Space(true)
}

pub fn nest(offset: usize, doc: Doc) -> Doc {
    doc.nest(offset)
}

pub fn concat<I>(docs: I) -> Doc
where
    I: IntoIterator<Item = Doc>,
{
    docs.into_iter().fold(empty(), |a, b| a.append(b))
}

pub fn doc<T: Into<Doc>>(data: T) -> Doc {
    data.into()
}

pub fn intersperse<I, S>(docs: I, sep: S) -> Doc
where
    I: IntoIterator<Item = Doc>,
    S: Into<Doc>,
{
    let sep = sep.into();
    let mut iter = docs.into_iter();
    if let Some(first) = iter.next() {
        let mut result = first;
        for next in iter {
            result += sep.clone() + next;
        }
        result
    } else {
        empty()
    }
}

pub fn tupled<I: IntoIterator<Item = Doc>>(docs: I) -> Doc {
    enclose_sep('(', ')', doc(',') + spaceline(), docs)
}

pub fn enclose_sep<L, R, S, I>(left: L, right: R, sep: S, docs: I) -> Doc
where
    L: Into<Doc>,
    R: Into<Doc>,
    S: Into<Doc>,
    I: IntoIterator<Item = Doc>,
{
    let result = left.into() + breakline() + intersperse(docs, sep) + right.into();
    result.nest(4).group()
}

impl<S> From<S> for Doc
where
    S: ToString,
{
    fn from(s: S) -> Doc {
        let text = s.to_string();
        debug_assert!(!text.contains(|c: char| c == '\n' || c == '\r'));
        Doc::Text(text.into())
    }
}

impl Doc {
    pub fn render<I, W>(&self, width_limit: I, out: &mut W) -> io::Result<()>
    where
        I: Into<Option<usize>>,
        W: ?Sized + io::Write,
    {
        best(
            self,
            width_limit.into().unwrap_or(usize::max_value() / 2),
            out,
        )
    }

    pub fn render_string<I>(&self, width_limit: I) -> String
    where
        I: Into<Option<usize>>,
    {
        let mut writer = vec![];
        self.render(width_limit, &mut writer).unwrap();
        String::from_utf8(writer).unwrap()
    }

    pub fn append(self, that: Doc) -> Doc {
        Doc::Append(self.into(), that.into())
    }

    pub fn group(self) -> Doc {
        Doc::Group(self.into())
    }

    pub fn nest(self, offset: usize) -> Doc {
        Doc::Nest(offset, self.into())
    }
}

impl<U: Into<Doc>> Add<U> for Doc {
    type Output = Doc;
    fn add(mut self, rhs: U) -> Doc {
        self.add_assign(rhs);
        self
    }
}

impl<U: Into<Doc>> AddAssign<U> for Doc {
    fn add_assign(&mut self, rhs: U) {
        use std::mem;
        let doc = mem::replace(self, empty());
        mem::replace(self, doc.append(rhs.into()));
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Mode {
    Break,
    Flat,
}

type Cmd<'a> = (usize, Mode, &'a Doc);

fn write_newline<W: ?Sized + io::Write>(ind: usize, out: &mut W) -> io::Result<()> {
    try!(out.write_all(b"\n"));
    write_spaces(ind, out)
}

fn write_spaces<W: ?Sized + io::Write>(spaces: usize, out: &mut W) -> io::Result<()> {
    const SPACES: [u8; 100] = [b' '; 100];
    let mut inserted = 0;
    while inserted < spaces {
        let insert = cmp::min(100, spaces - inserted);
        inserted += try!(out.write(&SPACES[..insert]));
    }
    Ok(())
}

#[inline]
fn fitting<'a>(next: Cmd<'a>, bcmds: &[Cmd<'a>], fcmds: &mut Vec<Cmd<'a>>, mut rem: isize) -> bool {
    let mut bidx = bcmds.len();
    fcmds.clear(); // clear from previous calls from best
    fcmds.push(next);
    while rem >= 0 {
        match fcmds.pop() {
            None => {
                if bidx == 0 {
                    // All commands have been processed
                    return true;
                } else {
                    fcmds.push(bcmds[bidx - 1]);
                    bidx -= 1;
                }
            }
            Some((ind, mode, doc)) => {
                match *doc {
                    Doc::Empty => {}
                    Doc::Append(ref ldoc, ref rdoc) => {
                        fcmds.push((ind, mode, rdoc));
                        // Since appended documents often appear in sequence on the left side we
                        // gain a slight performance increase by batching these pushes (avoiding
                        // to push and directly pop `Append` documents)
                        let mut doc = ldoc;
                        while let Doc::Append(ref l, ref r) = **doc {
                            fcmds.push((ind, mode, r));
                            doc = l;
                        }
                        fcmds.push((ind, mode, doc));
                    }
                    Doc::Group(ref doc) => {
                        fcmds.push((ind, mode, doc));
                    }
                    Doc::Nest(off, ref doc) => {
                        fcmds.push((ind + off, mode, doc));
                    }
                    Doc::Space(space) => match mode {
                        Mode::Flat => {
                            if space {
                                rem -= 1;
                            }
                        }
                        Mode::Break => {
                            return true;
                        }
                    },
                    Doc::Newline => return true,
                    Doc::Text(ref str) => {
                        rem -= str.len() as isize;
                    }
                }
            }
        }
    }
    false
}

#[inline]
pub fn best<W: ?Sized + io::Write>(doc: &Doc, width: usize, out: &mut W) -> io::Result<()> {
    let mut pos = 0usize;
    let mut bcmds = vec![(0usize, Mode::Break, doc)];
    let mut fcmds = vec![];
    while let Some((ind, mode, doc)) = bcmds.pop() {
        match *doc {
            Doc::Empty => {}
            Doc::Append(ref ldoc, ref rdoc) => {
                bcmds.push((ind, mode, rdoc));
                let mut doc = ldoc;
                while let Doc::Append(ref l, ref r) = **doc {
                    bcmds.push((ind, mode, r));
                    doc = l;
                }
                bcmds.push((ind, mode, doc));
            }
            Doc::Group(ref doc) => match mode {
                Mode::Flat => {
                    bcmds.push((ind, Mode::Flat, doc));
                }
                Mode::Break => {
                    let next = (ind, Mode::Flat, &**doc);
                    let rem = width as isize - pos as isize;
                    if fitting(next, &bcmds, &mut fcmds, rem) {
                        bcmds.push(next);
                    } else {
                        bcmds.push((ind, Mode::Break, doc));
                    }
                }
            },
            Doc::Nest(off, ref doc) => {
                bcmds.push((ind + off, mode, doc));
            }
            Doc::Space(space) => {
                match mode {
                    Mode::Flat => {
                        try!(write_spaces(if space { 1 } else { 0 }, out));
                    }
                    Mode::Break => {
                        try!(write_newline(ind, out));
                    }
                }
                pos = ind;
            }
            Doc::Newline => {
                try!(write_newline(ind, out));
                pos = ind;
            }
            Doc::Text(ref s) => {
                try!(out.write_all(&s.as_bytes()));
                pos += s.len();
            }
        }
    }
    Ok(())
}
