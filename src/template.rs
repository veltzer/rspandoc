//! pandoc's template language: `$var$`, `$if(..)$`, `$for(..)$`, partials,
//! pipes and nesting, compiled to a tree and rendered into a [`Doc`].
//! Ported from doctemplates 0.11.1 (`Text.DocTemplates.Parser` and
//! `.Internal`, BSD-3-Clause, Copyright John MacFarlane), including its
//! source-position rules, which decide where blank lines go and where
//! interpolated text is indented.

use std::collections::BTreeMap;

use anyhow::{Result, bail};

use crate::layout::{self, Doc};

/// A compiled template.
#[derive(Clone, Debug)]
pub enum Template {
    Interpolate(Variable),
    Conditional(Variable, Box<Template>, Box<Template>),
    Iterate(Variable, Box<Template>, Box<Template>),
    Nested(Box<Template>),
    Partial(Vec<Pipe>, Box<Template>),
    Literal(Doc),
    Concat(Vec<Template>),
    Empty,
}

impl Template {
    fn append(self, other: Self) -> Self {
        match (self, other) {
            (x, Self::Empty) | (Self::Empty, x) => x,
            (Self::Concat(mut a), Self::Concat(b)) => {
                a.extend(b);
                Self::Concat(a)
            }
            (Self::Concat(mut a), y) => {
                a.push(y);
                Self::Concat(a)
            }
            (x, Self::Concat(b)) => {
                let mut v = vec![x];
                v.extend(b);
                Self::Concat(v)
            }
            (x, y) => Self::Concat(vec![x, y]),
        }
    }

    fn concat(items: Vec<Self>) -> Self {
        items.into_iter().fold(Self::Empty, Self::append)
    }
}

/// A value transformation applied with `/name`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pipe {
    ToPairs,
    ToUppercase,
    ToLowercase,
    ToLength,
    Reverse,
    FirstItem,
    LastItem,
    Rest,
    AllButLast,
    Chomp,
    ToAlpha,
    ToRoman,
    NoWrap,
}

/// A dotted variable name and its pipes.
#[derive(Clone, Debug, Default)]
pub struct Variable {
    parts: Vec<String>,
    pipes: Vec<Pipe>,
}

/// A template value.
#[derive(Clone, Debug)]
pub enum Val {
    Simple(Doc),
    List(Vec<Val>),
    Map(Context),
    Bool(bool),
    Null,
}

pub type Context = BTreeMap<String, Val>;

// ─── parser ─────────────────────────────────────────────────────────────

/// A Parsec source position: 1-based line and column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pos {
    line: usize,
    col: usize,
}

const START: Pos = Pos { line: 1, col: 1 };

#[derive(Clone)]
struct PState {
    partial_nesting: usize,
    breaking_spaces: bool,
    first_nonspace: Pos,
    nested_col: Option<usize>,
    inside_directive: bool,
}

/// Parser position plus state; saved and restored to backtrack.
#[derive(Clone)]
struct Mark {
    idx: usize,
    pos: Pos,
    st: PState,
}

struct Parser<'a> {
    input: Vec<char>,
    idx: usize,
    pos: Pos,
    st: PState,
    partials: &'a dyn Fn(&str) -> Option<String>,
}

/// What closes a directive: `$` or `}`.
#[derive(Clone, Copy)]
enum Closer {
    Dollar,
    Brace,
}

impl Parser<'_> {
    fn mark(&self) -> Mark {
        Mark {
            idx: self.idx,
            pos: self.pos,
            st: self.st.clone(),
        }
    }

    fn reset(&mut self, m: Mark) {
        self.idx = m.idx;
        self.pos = m.pos;
        self.st = m.st;
    }

    /// Run `f`, restoring the position and state when it fails (Parsec's
    /// `try`; every alternative here backtracks).
    fn attempt<T>(&mut self, f: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let m = self.mark();
        let r = f(self);
        if r.is_none() {
            self.reset(m);
        }
        r
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.idx).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.idx += 1;
        match c {
            '\n' => {
                self.pos.line += 1;
                self.pos.col = 1;
            }
            '\t' => self.pos.col += 8 - ((self.pos.col - 1) % 8),
            _ => self.pos.col += 1,
        }
        Some(c)
    }

    fn satisfy(&mut self, f: impl Fn(char) -> bool) -> Option<char> {
        if self.peek().is_some_and(f) {
            self.advance()
        } else {
            None
        }
    }

    fn char(&mut self, c: char) -> Option<()> {
        self.satisfy(|x| x == c).map(|_| ())
    }

    fn string(&mut self, s: &str) -> Option<()> {
        self.attempt(|p| s.chars().try_for_each(|c| p.char(c)))
    }

    fn at_eof(&self) -> bool {
        self.idx >= self.input.len()
    }

    fn skip_space_or_tab(&mut self) {
        while self.satisfy(|c| c == ' ' || c == '\t').is_some() {}
    }

    fn template(&mut self) -> Template {
        while self.comment().is_some() {}
        let mut items = Vec::new();
        loop {
            let item = self
                .literal()
                .or_else(|| self.newline())
                .or_else(|| self.directive())
                .or_else(|| self.escape());
            let Some(item) = item else { break };
            items.push(item);
            while self.comment().is_some() {}
        }
        Template::concat(items)
    }

    fn line_ending(&mut self) -> Option<String> {
        match self.peek()? {
            '\n' => {
                self.advance();
                Some("\n".to_owned())
            }
            '\r' => {
                self.advance();
                if self.peek() == Some('\n') {
                    self.advance();
                    Some("\r\n".to_owned())
                } else {
                    Some("\r".to_owned())
                }
            }
            _ => None,
        }
    }

    fn newline_or_eof_ahead(&self) -> bool {
        matches!(self.peek(), None | Some('\n' | '\r'))
    }

    fn endline(&mut self) -> Option<String> {
        self.attempt(|p| {
            let nls = p.line_ending()?;
            if let Some(col) = p.st.nested_col {
                while p.pos.col < col && p.satisfy(|c| c == ' ' || c == '\t').is_some() {}
                if !(p.st.inside_directive || p.pos.col >= col) {
                    return None;
                }
            }
            Some(nls)
        })
    }

    fn blank_line(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let nl = p.line_ending()?;
            p.newline_or_eof_ahead()
                .then(|| Template::Literal(layout::literal(&nl)))
        })
    }

    fn newline(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let nls = p.endline()?;
            let mut sps = String::new();
            while let Some(c) = p.satisfy(|c| c == ' ' || c == '\t') {
                sps.push(c);
            }
            p.st.first_nonspace = p.pos;
            Some(Template::Literal(if p.st.breaking_spaces {
                Doc::BreakingSpace
            } else {
                layout::literal(&(nls + &sps))
            }))
        })
    }

    fn literal(&mut self) -> Option<Template> {
        let mut cs = String::new();
        while let Some(c) = self.satisfy(|c| c != '$' && c != '\n' && c != '\r') {
            cs.push(c);
        }
        if cs.is_empty() {
            return None;
        }
        if cs.chars().all(|c| c == ' ' || c == '\t') && self.pos.line == 1 {
            self.st.first_nonspace = self.pos;
        }
        Some(if self.st.breaking_spaces {
            to_breakable(&cs)
        } else {
            Template::Literal(layout::literal(&cs))
        })
    }

    fn escape(&mut self) -> Option<Template> {
        self.string("$$")?;
        self.pos.col -= 1;
        Some(Template::Literal(layout::literal("$")))
    }

    fn directive(&mut self) -> Option<Template> {
        self.conditional()
            .or_else(|| self.for_loop())
            .or_else(|| self.reflow_toggle())
            .or_else(|| self.nested())
            .or_else(|| self.interpolate())
            .or_else(|| self.bare_partial())
    }

    fn open(&mut self) -> Option<Closer> {
        self.attempt(|p| {
            p.char('$')?;
            if matches!(p.peek(), Some('$' | '{')) {
                return None;
            }
            Some(Closer::Dollar)
        })
        .or_else(|| {
            self.attempt(|p| {
                p.string("${")?;
                if p.peek() == Some('}') {
                    return None;
                }
                Some(Closer::Brace)
            })
        })
    }

    fn close(&mut self, c: Closer) -> Option<()> {
        match c {
            Closer::Dollar => self.char('$'),
            Closer::Brace => self.char('}'),
        }
    }

    fn enclosed<T>(&mut self, f: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        self.attempt(|p| {
            let closer = p.open()?;
            p.skip_space_or_tab();
            let r = f(p)?;
            p.skip_space_or_tab();
            p.close(closer)?;
            Some(r)
        })
    }

    fn keyword_var(&mut self, kw: &str) -> Option<Variable> {
        self.enclosed(|p| {
            p.string(kw)?;
            p.char('(')?;
            let v = p.var()?;
            p.char(')')?;
            Some(v)
        })
    }

    fn inside<T>(&mut self, f: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let old = self.st.inside_directive;
        self.st.inside_directive = true;
        let r = f(self);
        self.st.inside_directive = old;
        r
    }

    fn conditional(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let v = p.keyword_var("if")?;
            p.inside(|p| {
                let multiline = p.skip_endline().is_some();
                let if_contents = p.template();
                let else_contents = p
                    .else_part(multiline)
                    .or_else(|| p.elseif())
                    .unwrap_or(Template::Empty);
                p.enclosed(|p| p.string("endif"))?;
                if multiline {
                    p.skip_endline();
                }
                Some(Template::Conditional(
                    v,
                    Box::new(if_contents),
                    Box::new(else_contents),
                ))
            })
        })
    }

    fn else_part(&mut self, multiline: bool) -> Option<Template> {
        self.attempt(|p| {
            p.enclosed(|p| p.string("else"))?;
            if multiline {
                p.skip_endline();
            }
            Some(p.template())
        })
    }

    fn elseif(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let v = p.keyword_var("elseif")?;
            let multiline = p.skip_endline().is_some();
            let if_contents = p.template();
            let else_contents = p
                .else_part(multiline)
                .or_else(|| p.elseif())
                .unwrap_or(Template::Empty);
            Some(Template::Conditional(
                v,
                Box::new(if_contents),
                Box::new(else_contents),
            ))
        })
    }

    fn skip_endline(&mut self) -> Option<()> {
        self.attempt(|p| {
            p.endline()?;
            let m = p.mark();
            p.skip_space_or_tab();
            let pos = p.pos;
            p.reset(m);
            p.st.first_nonspace = pos;
            Some(())
        })
    }

    fn reflow_toggle(&mut self) -> Option<Template> {
        self.enclosed(|p| p.char('~'))?;
        self.st.breaking_spaces = !self.st.breaking_spaces;
        Some(Template::Empty)
    }

    fn nested(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let col = p.pos.col;
            p.enclosed(|p| p.char('^'))?;
            let old = p.st.nested_col;
            p.st.nested_col = Some(col);
            let mut contents = p.template();
            while let Some(more) = p.attempt(|p| {
                let mut blanks = Vec::new();
                while let Some(b) = p.blank_line() {
                    blanks.push(b);
                }
                if blanks.is_empty() {
                    return None;
                }
                let z = p.template();
                Some(Template::concat(blanks).append(z))
            }) {
                contents = contents.append(more);
            }
            p.st.nested_col = old;
            Some(Template::Nested(Box::new(contents)))
        })
    }

    fn for_loop(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let v = p.keyword_var("for")?;
            p.inside(|p| {
                let multiline = p.skip_endline().is_some();
                let contents = p.template();
                let sep = p
                    .attempt(|p| {
                        p.enclosed(|p| p.string("sep"))?;
                        if multiline {
                            p.skip_endline();
                        }
                        Some(p.template())
                    })
                    .unwrap_or(Template::Empty);
                p.enclosed(|p| p.string("endfor"))?;
                if multiline {
                    p.skip_endline();
                }
                Some(Template::Iterate(v, Box::new(contents), Box::new(sep)))
            })
        })
    }

    fn interpolate(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let pos = p.pos;
            let (closer, var) = p.attempt(|p| {
                let cl = p.open()?;
                p.skip_space_or_tab();
                let v = p.var()?;
                if p.peek() == Some('(') {
                    return None;
                }
                Some((cl, v))
            })?;
            let res = if p.peek() == Some(':') {
                p.advance();
                let name = p.partial_name()?;
                p.partial(Some(var), &name)?
            } else if let Some(sep) = p.sep() {
                let it = Variable {
                    parts: vec!["it".to_owned()],
                    pipes: Vec::new(),
                };
                Template::Iterate(var, Box::new(Template::Interpolate(it)), Box::new(sep))
            } else {
                Template::Interpolate(var)
            };
            p.skip_space_or_tab();
            p.close(closer)?;
            Some(p.handle_nesting(false, pos, res))
        })
    }

    fn handle_nesting(&mut self, eat_endline: bool, pos: Pos, templ: Template) -> Template {
        let beginline = self.st.first_nonspace == pos;
        let endofline = self.newline_or_eof_ahead();
        if eat_endline && beginline {
            self.skip_endline();
        }
        if !(beginline && endofline) {
            return templ;
        }
        match templ {
            t @ Template::Nested(_) => t,
            t if pos.col == 1 || Some(pos.col) == self.st.nested_col => t,
            t => Template::Nested(Box::new(t)),
        }
    }

    fn bare_partial(&mut self) -> Option<Template> {
        self.attempt(|p| {
            let pos = p.pos;
            let (closer, name) = p.attempt(|p| {
                let cl = p.open()?;
                p.skip_space_or_tab();
                Some((cl, p.partial_name()?))
            })?;
            let res = p.partial(None, &name)?;
            p.skip_space_or_tab();
            p.close(closer)?;
            Some(p.handle_nesting(true, pos, res))
        })
    }

    fn partial_name(&mut self) -> Option<String> {
        self.attempt(|p| {
            let mut name = String::new();
            while let Some(c) = p.satisfy(|c| c.is_alphanumeric() || "_-./\\".contains(c)) {
                name.push(c);
            }
            if name.is_empty() {
                return None;
            }
            p.string("()")?;
            Some(name)
        })
    }

    fn partial(&mut self, var: Option<Variable>, name: &str) -> Option<Template> {
        let old_st = self.st.clone();
        let separ = self.sep().unwrap_or(Template::Empty);
        let text = (self.partials)(name)?;
        let text = text.strip_suffix('\n').unwrap_or(&text);
        let t = if self.st.partial_nesting > 50 {
            Template::Literal(layout::literal("(loop)"))
        } else {
            let saved_input = std::mem::replace(&mut self.input, text.chars().collect());
            let (saved_idx, saved_pos) = (self.idx, self.pos);
            self.idx = 0;
            self.pos = START;
            self.st.partial_nesting += 1;
            self.st.nested_col = None;
            let res = self.template();
            let complete = self.at_eof();
            self.input = saved_input;
            self.idx = saved_idx;
            self.pos = saved_pos;
            if !complete {
                return None;
            }
            res
        };
        self.st = old_st;
        let mut fs = Vec::new();
        while let Some(f) = self.pipe() {
            fs.push(f);
        }
        let partial = Template::Partial(fs, Box::new(t));
        Some(match var {
            Some(v) => Template::Iterate(v, Box::new(partial), Box::new(separ)),
            None => partial,
        })
    }

    fn sep(&mut self) -> Option<Template> {
        self.attempt(|p| {
            p.char('[')?;
            let mut xs = String::new();
            while let Some(c) = p.satisfy(|c| c != ']') {
                xs.push(c);
            }
            p.char(']')?;
            Some(Template::Literal(layout::literal(&xs)))
        })
    }

    fn comment(&mut self) -> Option<()> {
        let col = self.pos.col;
        self.string("$--")?;
        while self.satisfy(|c| c != '\n').is_some() {}
        if col == 1 {
            self.line_ending();
        }
        Some(())
    }

    fn var(&mut self) -> Option<Variable> {
        let first = self
            .ident_part()
            .or_else(|| self.string("it").map(|()| "it".to_owned()))?;
        let mut parts = vec![first];
        while let Some(part) = self.attempt(|p| {
            p.char('.')?;
            p.ident_part()
        }) {
            parts.push(part);
        }
        let mut pipes = Vec::new();
        while let Some(f) = self.pipe() {
            pipes.push(f);
        }
        Some(Variable { parts, pipes })
    }

    fn pipe(&mut self) -> Option<Pipe> {
        self.attempt(|p| {
            p.char('/')?;
            let mut name = String::new();
            while let Some(c) = p.satisfy(char::is_alphabetic) {
                name.push(c);
            }
            Some(match name.as_str() {
                "uppercase" => Pipe::ToUppercase,
                "lowercase" => Pipe::ToLowercase,
                "pairs" => Pipe::ToPairs,
                "length" => Pipe::ToLength,
                "alpha" => Pipe::ToAlpha,
                "roman" => Pipe::ToRoman,
                "reverse" => Pipe::Reverse,
                "first" => Pipe::FirstItem,
                "rest" => Pipe::Rest,
                "last" => Pipe::LastItem,
                "allbutlast" => Pipe::AllButLast,
                "chomp" => Pipe::Chomp,
                "nowrap" => Pipe::NoWrap,
                _ => return None,
            })
        })
    }

    fn ident_part(&mut self) -> Option<String> {
        self.attempt(|p| {
            let mut part = String::new();
            part.push(p.satisfy(char::is_alphabetic)?);
            while let Some(c) = p.satisfy(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                part.push(c);
            }
            let reserved = [
                "if", "else", "endif", "elseif", "for", "endfor", "sep", "it",
            ];
            (!reserved.contains(&part.as_str())).then_some(part)
        })
    }
}

fn to_breakable(s: &str) -> Template {
    let is_spacy = |c: char| matches!(c, ' ' | '\n' | '\r' | '\t');
    let mut items = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        if rest.starts_with(is_spacy) {
            items.push(Template::Literal(Doc::BreakingSpace));
            rest = rest.trim_start_matches(is_spacy);
        } else {
            let end = rest.find(is_spacy).unwrap_or(rest.len());
            items.push(Template::Literal(layout::literal(&rest[..end])));
            rest = &rest[end..];
        }
    }
    Template::concat(items)
}

/// Compile `text`; `partials` returns the text of a partial by file name.
pub fn compile(text: &str, partials: &dyn Fn(&str) -> Option<String>) -> Result<Template> {
    let mut p = Parser {
        input: text.chars().collect(),
        idx: 0,
        pos: START,
        st: PState {
            partial_nesting: 1,
            breaking_spaces: false,
            first_nonspace: START,
            nested_col: None,
            inside_directive: false,
        },
        partials,
    };
    let t = p.template();
    if !p.at_eof() {
        bail!(
            "template parse error at line {}, column {}",
            p.pos.line,
            p.pos.col
        );
    }
    Ok(t)
}

// ─── rendering ──────────────────────────────────────────────────────────

/// The text of a document's text leaves mapped by `f` (widths unchanged,
/// as doclayout's `fmap` leaves them).
fn map_doc_text(d: &Doc, f: &dyn Fn(&str) -> String) -> Doc {
    match d {
        Doc::Text(n, s) => Doc::Text(*n, f(s)),
        Doc::Prefixed(p, d) => Doc::Prefixed(p.clone(), Box::new(map_doc_text(d, f))),
        Doc::Concat(v) => Doc::Concat(v.iter().map(|d| map_doc_text(d, f)).collect()),
        other => other.clone(),
    }
}

fn map_doc(val: Val, f: &dyn Fn(Doc) -> Doc) -> Val {
    match val {
        Val::Simple(d) => Val::Simple(f(d)),
        Val::Map(m) => Val::Map(m.into_iter().map(|(k, v)| (k, map_doc(v, f))).collect()),
        Val::List(xs) => Val::List(xs.into_iter().map(|v| map_doc(v, f)).collect()),
        other => other,
    }
}

fn rendered(d: &Doc) -> String {
    layout::render(None, d)
}

/// Haskell's `Data.Text.Read.decimal` consuming the whole text.
fn decimal(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn to_roman(x: u64) -> Option<String> {
    const TABLE: [(u64, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    if x >= 4000 {
        return None;
    }
    let mut x = x;
    let mut out = String::new();
    for (n, s) in TABLE {
        while x >= n {
            out.push_str(s);
            x -= n;
        }
    }
    Some(out)
}

fn apply_pipe(pipe: Pipe, val: Val) -> Val {
    match pipe {
        Pipe::ToLength => {
            let len = match &val {
                Val::Simple(d) => rendered(d).chars().count(),
                Val::Map(m) => m.len(),
                Val::List(xs) => xs.len(),
                Val::Bool(_) | Val::Null => 0,
            };
            Val::Simple(layout::literal(&len.to_string()))
        }
        Pipe::ToUppercase => map_doc(val, &|d| map_doc_text(&d, &str::to_uppercase)),
        Pipe::ToLowercase => map_doc(val, &|d| map_doc_text(&d, &str::to_lowercase)),
        Pipe::ToPairs => {
            let pair = |k: String, v: Val| {
                Val::Map(BTreeMap::from([
                    ("key".to_owned(), Val::Simple(layout::literal(&k))),
                    ("value".to_owned(), v),
                ]))
            };
            match val {
                Val::Map(m) => Val::List(m.into_iter().map(|(k, v)| pair(k, v)).collect()),
                Val::List(xs) => Val::List(
                    xs.into_iter()
                        .enumerate()
                        .map(|(i, v)| pair((i + 1).to_string(), v))
                        .collect(),
                ),
                other => other,
            }
        }
        Pipe::FirstItem => match val {
            Val::List(mut xs) if !xs.is_empty() => xs.swap_remove(0),
            other => other,
        },
        Pipe::LastItem => match val {
            Val::List(mut xs) if !xs.is_empty() => xs.pop().unwrap_or(Val::Null),
            other => other,
        },
        Pipe::Rest => match val {
            Val::List(mut xs) if !xs.is_empty() => {
                xs.remove(0);
                Val::List(xs)
            }
            other => other,
        },
        Pipe::AllButLast => match val {
            Val::List(mut xs) if !xs.is_empty() => {
                xs.pop();
                Val::List(xs)
            }
            other => other,
        },
        Pipe::Reverse => match val {
            Val::List(mut xs) => {
                xs.reverse();
                Val::List(xs)
            }
            Val::Simple(d) => Val::Simple(layout::literal(
                &rendered(&d).chars().rev().collect::<String>(),
            )),
            other => other,
        },
        Pipe::Chomp => map_doc(val, &layout::chomp),
        Pipe::ToAlpha => map_doc(val, &|d| match decimal(&rendered(&d)) {
            Some(y) if y > 0 => {
                let off = u8::try_from((y - 1) % 26).unwrap_or(0);
                layout::literal(&char::from(b'a' + off).to_string())
            }
            _ => d,
        }),
        Pipe::ToRoman => map_doc(val, &|d| match decimal(&rendered(&d)).and_then(to_roman) {
            Some(r) => layout::literal(&r),
            None => d,
        }),
        Pipe::NoWrap => map_doc(val, &layout::nowrap),
    }
}

fn apply_pipes(pipes: &[Pipe], val: Val) -> Val {
    pipes.iter().fold(val, |v, &p| apply_pipe(p, v))
}

fn multi_lookup(parts: &[String], val: &Val) -> Val {
    let Some((first, rest)) = parts.split_first() else {
        return val.clone();
    };
    match val {
        Val::Map(m) => m.get(first).map_or(Val::Null, |v| multi_lookup(rest, v)),
        _ => Val::Null,
    }
}

/// Whether a variable is set, and the documents it renders to.
fn resolve_val(pipes: &[Pipe], val: Val) -> (bool, Vec<Doc>) {
    match apply_pipes(pipes, val) {
        Val::List(xs) => {
            let mut any = false;
            let mut docs = Vec::new();
            for x in xs {
                let (b, ds) = resolve_val(&[], x);
                any |= b;
                docs.extend(ds);
            }
            (any, docs)
        }
        Val::Simple(d) if d.is_empty() => (false, Vec::new()),
        Val::Simple(d) => (true, vec![remove_final_nl(d)]),
        Val::Map(_) | Val::Bool(true) => (true, vec![layout::literal("true")]),
        Val::Bool(false) => (false, vec![layout::literal("false")]),
        Val::Null => (false, Vec::new()),
    }
}

fn resolve_variable(v: &Variable, ctx: &Context) -> (bool, Vec<Doc>) {
    resolve_val(&v.pipes, multi_lookup(&v.parts, &Val::Map(ctx.clone())))
}

fn remove_final_nl(d: Doc) -> Doc {
    match d {
        Doc::NewLine | Doc::CarriageReturn => Doc::Empty,
        Doc::Concat(mut v) => {
            if matches!(v.last(), Some(Doc::NewLine | Doc::CarriageReturn)) {
                v.pop();
            }
            layout::hcat(v)
        }
        other => other,
    }
}

fn add_to_context(parts: &[String], x: &Val, ctx: &mut Context) {
    let Some((first, rest)) = parts.split_first() else {
        return;
    };
    if let Some(z) = ctx.get_mut(first) {
        if rest.is_empty() {
            *z = x.clone();
        } else if let Val::Map(m) = z {
            add_to_context(rest, x, m);
        }
    }
}

struct Renderer {
    column: usize,
}

impl Renderer {
    fn update_column(&mut self, d: Doc) -> Doc {
        self.column = layout::update_column(&d, self.column);
        d
    }

    fn with_variable(&mut self, var: &Variable, ctx: &Context, t: &Template) -> Vec<Doc> {
        let set = |x: Val| {
            let mut c = ctx.clone();
            c.insert("it".to_owned(), x.clone());
            add_to_context(&var.parts, &x, &mut c);
            c
        };
        match apply_pipes(&var.pipes, multi_lookup(&var.parts, &Val::Map(ctx.clone()))) {
            Val::Null => Vec::new(),
            Val::List(xs) => xs.into_iter().map(|x| self.render(t, &set(x))).collect(),
            other => vec![self.render(t, &set(other))],
        }
    }

    fn render(&mut self, t: &Template, ctx: &Context) -> Doc {
        match t {
            Template::Literal(d) => self.update_column(d.clone()),
            Template::Interpolate(v) => {
                let (_, xs) = resolve_variable(v, ctx);
                self.update_column(layout::hcat(xs))
            }
            Template::Conditional(v, ift, elset) => {
                if resolve_variable(v, ctx).0 {
                    self.render(ift, ctx)
                } else {
                    self.render(elset, ctx)
                }
            }
            Template::Iterate(v, body, sep) => {
                let xs = self.with_variable(v, ctx, body);
                let sep = self.render(sep, ctx);
                let mut out = Vec::new();
                for (i, x) in xs.into_iter().enumerate() {
                    if i > 0 {
                        out.push(sep.clone());
                    }
                    out.push(x);
                }
                layout::hcat(out)
            }
            Template::Nested(t) => {
                let n = self.column;
                layout::nest(n, self.render(t, ctx))
            }
            Template::Partial(fs, t) => {
                let val = self.render(t, ctx);
                match apply_pipes(fs, Val::Simple(val)) {
                    Val::Simple(x) => x,
                    _ => Doc::Empty,
                }
            }
            Template::Concat(items) => layout::hcat(
                items
                    .iter()
                    .map(|t| self.render(t, ctx))
                    .collect::<Vec<_>>(),
            ),
            Template::Empty => Doc::Empty,
        }
    }
}

/// Render a compiled template with a context.
pub fn render(t: &Template, ctx: &Context) -> Doc {
    Renderer { column: 0 }.render(t, ctx)
}

#[cfg(test)]
mod tests {
    use super::{Context, Val, compile, render};
    use crate::layout;

    fn run(tpl: &str, ctx: &Context) -> String {
        let t = compile(tpl, &|_| None).expect("template compiles");
        layout::render(None, &render(&t, ctx))
    }

    fn simple(s: &str) -> Val {
        Val::Simple(layout::literal(s))
    }

    #[test]
    fn conditionals_and_loops() {
        let ctx = Context::from([
            ("a".to_owned(), simple("x")),
            ("l".to_owned(), Val::List(vec![simple("1"), simple("2")])),
        ]);
        assert_eq!(run("$if(a)$yes$else$no$endif$", &ctx), "yes");
        assert_eq!(run("$if(b)$yes$else$no$endif$", &ctx), "no");
        assert_eq!(run("$for(l)$[$l$]$sep$, $endfor$", &ctx), "[1], [2]");
        assert_eq!(run("$l/length$ $l/first$", &ctx), "2 1");
    }

    #[test]
    fn multiline_directives_swallow_their_lines() {
        let ctx = Context::from([("a".to_owned(), simple("x"))]);
        assert_eq!(
            run("one\n$if(a)$\n$a$\n$endif$\ntwo\n", &ctx),
            "one\nx\ntwo\n"
        );
    }

    #[test]
    fn interpolation_nests_at_its_column() {
        let ctx = Context::from([("a".to_owned(), simple("x\ny"))]);
        assert_eq!(run("  $a$\n", &ctx), "  x\n  y\n");
    }
}
