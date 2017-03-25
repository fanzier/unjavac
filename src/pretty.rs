use std::cmp;
use std::io;
use std::borrow::Cow;
use std::ops::Add;

pub trait Pretty {
    /// Additional information required to pretty-print:
    type Extra;
    fn pretty_with(&self, extra: Self::Extra) -> Doc;
}

pub trait PlainPretty {
    fn pretty(&self) -> Doc;
}

impl<T: Pretty<Extra = ()>> PlainPretty for T {
    fn pretty(&self) -> Doc {
        self.pretty_with(())
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

pub fn concat<I>(docs: I) -> Doc
    where I: IntoIterator<Item = Doc>
{
    docs.into_iter().fold(empty(), |a, b| a.append(b))
}

pub fn doc<T: Into<Doc>>(data: T) -> Doc {
    data.into()
}

pub fn delim<I: IntoIterator<Item = Doc>>(docs: I, delim: &Doc) -> Doc {
    let mut iter = docs.into_iter();
    let result = if let Some(first) = iter.next() {
        let mut result = breakline() + first;
        for next in iter {
            result = result + delim.clone() + spaceline() + next;
        }
        result
    } else {
        empty()
    };
    result.group()
}

impl<S> From<S> for Doc
    where S: ToString
{
    fn from(s: S) -> Doc {
        let text = s.to_string();
        debug_assert!(!text.contains(|c: char| c == '\n' || c == '\r'));
        Doc::Text(text.into())
    }
}

impl Doc {
    pub fn render<W: ?Sized + io::Write>(&self, width: usize, out: &mut W) -> io::Result<()> {
        best(self, width, out)
    }

    pub fn render_string(&self, width: usize) -> String {
        let mut writer = vec![];
        self.render(width, &mut writer).unwrap();
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
    fn add(self, rhs: U) -> Doc {
        self.append(rhs.into())
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
                    Doc::Space(space) => {
                        match mode {
                            Mode::Flat => {
                                if space {
                                    rem -= 1;
                                }
                            }
                            Mode::Break => {
                                return true;
                            }
                        }
                    }
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
            Doc::Group(ref doc) => {
                match mode {
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
                }
            }
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
