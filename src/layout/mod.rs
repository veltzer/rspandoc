//! A pretty-printing document, laid out the way the doclayout library lays
//! it out: text with breakable spaces that reflow to a line length, hard
//! and soft line breaks, and prefixed (indented) regions. Ported from
//! doclayout 0.5.0.3 (`Text.DocLayout`, BSD-3-Clause, Copyright John
//! MacFarlane), without the parts the HTML writer and templates never
//! build (blank-line requests, flush and before-non-blank regions, boxes,
//! fonts, hyperlinks); they come back with the writers that need them.
//!
//! doclayout's `Concat` is a binary tree; here it is a flat list whose
//! members are never `Concat` or `Empty`. Every operation doclayout defines
//! by matching on `Concat` (chomp, nestle, the offset walk, ...) only looks
//! at the leftmost or rightmost leaf or walks the leaves in order, so the
//! flat form gives the same answers and keeps the long chains a document
//! builds from overflowing the stack.

mod width;
mod width_tables;

use std::collections::VecDeque;

pub use width::real_length;

/// A document, including the structure that matters for layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Doc {
    /// Text of the given display width; never contains a newline.
    Text(usize, String),
    /// A document with each line prefixed; trailing blanks are left off the
    /// prefix on a line that is otherwise empty.
    Prefixed(String, Box<Doc>),
    /// A space, or a line break when the line is full.
    BreakingSpace,
    /// A newline unless already at the start of a line.
    CarriageReturn,
    /// A newline.
    NewLine,
    /// Documents in sequence; at least two, none `Concat` or `Empty`.
    Concat(Vec<Doc>),
    Empty,
}

impl Doc {
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// The leaves of the document in order.
    fn leaves(&self) -> &[Self] {
        match self {
            Self::Empty => &[],
            Self::Concat(v) => v,
            other => std::slice::from_ref(other),
        }
    }

    fn into_leaves(self) -> Vec<Self> {
        match self {
            Self::Empty => Vec::new(),
            Self::Concat(v) => v,
            other => vec![other],
        }
    }

    fn from_leaves(mut v: Vec<Self>) -> Self {
        match v.len() {
            0 => Self::Empty,
            1 => v.pop().unwrap_or(Self::Empty),
            _ => Self::Concat(v),
        }
    }

    /// `self <> other`.
    #[must_use]
    pub fn append(self, other: Self) -> Self {
        if other.is_empty() {
            return self;
        }
        if self.is_empty() {
            return other;
        }
        let mut v = self.into_leaves();
        v.extend(other.into_leaves());
        Self::Concat(v)
    }
}

/// `mconcat`: the documents in sequence.
pub fn hcat(docs: impl IntoIterator<Item = Doc>) -> Doc {
    let mut v = Vec::new();
    for d in docs {
        v.extend(d.into_leaves());
    }
    Doc::from_leaves(v)
}

/// A document of literal text; newlines in it become hard breaks.
pub fn literal(s: &str) -> Doc {
    let mut v = Vec::new();
    for (i, line) in s.split('\n').enumerate() {
        if i > 0 {
            v.push(Doc::NewLine);
        }
        if !line.is_empty() {
            v.push(Doc::Text(real_length(line), line.to_owned()));
        }
    }
    Doc::from_leaves(v)
}

pub const fn space() -> Doc {
    Doc::BreakingSpace
}

pub const fn cr() -> Doc {
    Doc::CarriageReturn
}

/// Each line of `doc` prefixed with `pref` (except the first, when it does
/// not start a line).
pub fn prefixed(pref: &str, doc: Doc) -> Doc {
    if doc.is_empty() {
        Doc::Empty
    } else {
        Doc::Prefixed(pref.to_owned(), Box::new(doc))
    }
}

/// `doc` indented by `ind` spaces.
pub fn nest(ind: usize, doc: Doc) -> Doc {
    prefixed(&" ".repeat(ind), doc)
}

/// `doc` with its breakable spaces made unbreakable.
pub fn nowrap(doc: Doc) -> Doc {
    Doc::from_leaves(
        doc.into_leaves()
            .into_iter()
            .map(|d| match d {
                Doc::BreakingSpace => Doc::Text(1, " ".to_owned()),
                other => other,
            })
            .collect(),
    )
}

/// `doc` without its trailing line breaks and spaces.
pub fn chomp(doc: Doc) -> Doc {
    match doc {
        Doc::NewLine | Doc::CarriageReturn | Doc::BreakingSpace => Doc::Empty,
        Doc::Prefixed(s, d) => Doc::Prefixed(s, Box::new(chomp(*d))),
        Doc::Concat(mut v) => {
            while let Some(last) = v.pop() {
                let c = chomp(last);
                if !c.is_empty() {
                    v.push(c);
                    break;
                }
            }
            Doc::from_leaves(v)
        }
        other => other,
    }
}

/// doclayout's `getOffset` with breaks never taken: (longest line, current
/// column) after laying out `leaves` from `(l, c)`.
fn get_offset(leaves: &[Doc], mut l: usize, mut c: usize) -> (usize, usize) {
    for d in leaves {
        match d {
            Doc::Text(n, _) => c += n,
            Doc::Empty => {}
            Doc::CarriageReturn | Doc::NewLine => {
                l = l.max(c);
                c = 0;
            }
            Doc::Prefixed(t, d) => {
                let (l2, c2) = get_offset(d.leaves(), 0, 0);
                let tl = real_length(t);
                l = l.max(l2 + tl);
                c = c2 + tl;
            }
            Doc::BreakingSpace => c += 1,
            Doc::Concat(v) => (l, c) = get_offset(v, l, c),
        }
    }
    (l, c)
}

/// The column after laying `doc` out from column `k`, without wrapping.
pub fn update_column(doc: &Doc, k: usize) -> usize {
    get_offset(doc.leaves(), 0, k).1
}

/// Render `doc`, reflowing breakable spaces to `line_length` columns
/// (`None`: never wrap).
pub fn render(line_length: Option<usize>, doc: &Doc) -> String {
    let mut st = RenderState {
        output: String::new(),
        prefix: String::new(),
        prefix_len: 0,
        line_length,
        column: 0,
        newlines: 2,
    };
    st.render_list(normalize(flatten(doc)));
    st.output
}

/// A document with its nesting flattened into a list, as doclayout's
/// renderer consumes it.
#[derive(Clone, Debug)]
enum Flat {
    Text(usize, String),
    Prefixed(String, Vec<Flat>),
    BreakingSpace,
    CarriageReturn,
    NewLine,
}

fn flatten(doc: &Doc) -> Vec<Flat> {
    let mut out = Vec::new();
    flatten_into(doc, &mut out);
    out
}

fn flatten_into(doc: &Doc, out: &mut Vec<Flat>) {
    match doc {
        Doc::Text(n, s) => out.push(Flat::Text(*n, s.clone())),
        Doc::Prefixed(p, d) => {
            let f = flatten(d);
            if !f.is_empty() {
                out.push(Flat::Prefixed(p.clone(), f));
            }
        }
        Doc::BreakingSpace => out.push(Flat::BreakingSpace),
        Doc::CarriageReturn => out.push(Flat::CarriageReturn),
        Doc::NewLine => out.push(Flat::NewLine),
        Doc::Empty => {}
        Doc::Concat(v) => {
            for d in v {
                flatten_into(d, out);
            }
        }
    }
}

/// Merge adjacent breaks: a newline absorbs following spaces and soft
/// breaks, a soft break absorbs following spaces and spaces before it, and a
/// trailing newline becomes a soft break (a trailing space disappears).
fn normalize(input: Vec<Flat>) -> Vec<Flat> {
    let mut rest = input.into_iter().peekable();
    let mut out = Vec::new();
    let mut cur: Option<Flat> = None;
    while let Some(x) = cur.take().or_else(|| rest.next()) {
        let Some(next) = rest.peek() else {
            match x {
                Flat::NewLine => out.push(Flat::CarriageReturn),
                Flat::BreakingSpace => {}
                other => out.push(other),
            }
            break;
        };
        match (&x, next) {
            (Flat::NewLine, Flat::BreakingSpace | Flat::CarriageReturn)
            | (Flat::CarriageReturn, Flat::CarriageReturn | Flat::BreakingSpace)
            | (Flat::BreakingSpace, Flat::BreakingSpace) => {
                cur = Some(x);
                rest.next();
            }
            (Flat::BreakingSpace, Flat::CarriageReturn | Flat::NewLine) => {
                cur = rest.next();
            }
            _ => out.push(x),
        }
    }
    out
}

/// Haskell's `isSpace`: the ASCII spaces, no-break space, and the other
/// Unicode space separators.
fn is_hs_space(c: char) -> bool {
    matches!(c, ' ' | '\t'..='\r' | '\u{A0}')
        || (c > '\u{FF}'
            && matches!(
                c,
                '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
            ))
}

const fn is_breakable(f: &Flat) -> bool {
    matches!(
        f,
        Flat::BreakingSpace | Flat::CarriageReturn | Flat::NewLine
    )
}

const fn offset_of(f: &Flat) -> usize {
    match f {
        Flat::Text(o, _) => *o,
        Flat::BreakingSpace => 1,
        _ => 0,
    }
}

struct RenderState {
    output: String,
    prefix: String,
    prefix_len: usize,
    line_length: Option<usize>,
    column: usize,
    newlines: usize,
}

impl RenderState {
    fn newline(&mut self) {
        if self.column == 0 && !self.prefix.is_empty() {
            self.output
                .push_str(self.prefix.trim_end_matches(is_hs_space));
        }
        self.output.push('\n');
        self.column = 0;
        self.newlines += 1;
    }

    fn outp(&mut self, off: usize, s: &str) {
        if self.column == 0 && !self.prefix.is_empty() {
            self.output.push_str(&self.prefix);
            self.column = self.prefix_len + off;
        } else {
            self.column += off;
        }
        self.output.push_str(s);
        self.newlines = 0;
    }

    fn render_list(&mut self, list: Vec<Flat>) {
        let mut items: VecDeque<Flat> = list.into();
        while let Some(item) = items.pop_front() {
            match item {
                Flat::Text(off, s) => self.outp(off, &s),
                Flat::Prefixed(pref, d) => {
                    let old = (self.prefix.clone(), self.prefix_len);
                    self.prefix.push_str(&pref);
                    self.prefix_len = real_length(&self.prefix);
                    self.render_list(normalize(d));
                    (self.prefix, self.prefix_len) = old;
                }
                Flat::CarriageReturn => {
                    if self.newlines == 0 {
                        self.newline();
                    }
                }
                Flat::NewLine => self.newline(),
                Flat::BreakingSpace => {
                    while items
                        .front()
                        .is_some_and(|n| matches!(n, Flat::BreakingSpace))
                    {
                        items.pop_front();
                    }
                    let off: usize = items
                        .iter()
                        .take_while(|f| !is_breakable(f))
                        .map(offset_of)
                        .sum();
                    match self.line_length {
                        Some(l) if self.column + 1 + off > l => self.newline(),
                        _ => {
                            if self.column > 0 {
                                self.outp(1, " ");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Doc, cr, hcat, literal, nest, render, space};

    fn words(s: &str) -> Doc {
        hcat(s.split(' ').enumerate().map(|(i, w)| {
            if i == 0 {
                literal(w)
            } else {
                space().append(literal(w))
            }
        }))
    }

    #[test]
    fn reflows_at_line_length() {
        let d = words("aaa bbb ccc ddd");
        assert_eq!(render(Some(7), &d), "aaa bbb\nccc ddd");
        assert_eq!(render(None, &d), "aaa bbb ccc ddd");
    }

    #[test]
    fn soft_breaks_merge() {
        let d = literal("a")
            .append(cr())
            .append(space())
            .append(cr())
            .append(literal("b"))
            .append(cr());
        assert_eq!(render(None, &d), "a\nb\n");
    }

    #[test]
    fn nesting_prefixes_lines() {
        let d = literal("x")
            .append(cr())
            .append(nest(2, literal("a").append(cr()).append(literal("b"))));
        assert_eq!(render(None, &d), "x\n  a\n  b");
    }
}
