//! Inline Markdown: code spans, emphasis, links and autolinks, raw HTML,
//! citations, entities, escapes, and pandoc's `smart` typography (curly
//! quotes, dashes, ellipses).
//!
//! The parser is recursive descent over the characters. A construct with
//! an opener (emphasis, quotes) parses its content until a matching closer
//! and backtracks to literal text when none comes.

use anyhow::{Result, bail};

use super::Refs;
use crate::ast::{Attr, Inline, QuoteType};
use crate::entities::ENTITIES;

/// What ends the content being parsed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Closer {
    Star2,
    Star1,
    Under2,
    Under1,
    DoubleQuote,
    SingleQuote,
}

struct Parser<'a> {
    s: Vec<char>,
    i: usize,
    refs: &'a Refs,
    /// A construct pandoc has and this parser does not, once met.
    unsupported: Option<String>,
}

/// Parse Markdown inline text (paragraph lines joined by `\n`).
pub fn parse_inlines(text: &str, refs: &Refs) -> Result<Vec<Inline>> {
    let mut p = Parser {
        s: text.trim().chars().collect(),
        i: 0,
        refs,
        unsupported: None,
    };
    let mut out = p.inlines(&[]).unwrap_or_default();
    if let Some(what) = p.unsupported {
        let excerpt: String = text.trim().chars().take(60).collect();
        bail!("unsupported Markdown ({what}) in: {excerpt}");
    }
    while matches!(out.last(), Some(Inline::Space | Inline::SoftBreak)) {
        out.pop();
    }
    Ok(out)
}

fn push_str(out: &mut Vec<Inline>, s: &str) {
    if let Some(Inline::Str(prev)) = out.last_mut() {
        prev.push_str(s);
    } else {
        out.push(Inline::Str(s.to_owned()));
    }
}

const fn is_ascii_punct(c: char) -> bool {
    c.is_ascii_punctuation()
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn at(&self, k: usize) -> Option<char> {
        self.s.get(k).copied()
    }

    fn prev(&self) -> Option<char> {
        self.i.checked_sub(1).and_then(|k| self.at(k))
    }

    fn starts_with(&self, pat: &str) -> bool {
        pat.chars()
            .enumerate()
            .all(|(k, c)| self.at(self.i + k) == Some(c))
    }

    fn next_is_space(&self, k: usize) -> bool {
        self.at(self.i + k).is_none_or(char::is_whitespace)
    }

    fn prev_is_space(&self) -> bool {
        self.prev().is_none_or(char::is_whitespace)
    }

    /// Whether `closer` ends the content at the current position.
    fn closes(&self, closer: Closer) -> bool {
        // Emphasis closes whatever precedes it; quotes do not close after
        // whitespace.
        match closer {
            Closer::Star2 => self.starts_with("**"),
            Closer::Star1 => self.starts_with("*") && !self.starts_with("**"),
            Closer::Under2 => {
                self.starts_with("__") && !self.at(self.i + 2).is_some_and(char::is_alphanumeric)
            }
            Closer::Under1 => {
                self.starts_with("_")
                    && !self.starts_with("__")
                    && !self.at(self.i + 1).is_some_and(char::is_alphanumeric)
            }
            // Double quotes do not nest: any `"` ends one.
            Closer::DoubleQuote => self.starts_with("\""),
            Closer::SingleQuote => {
                !self.prev_is_space()
                    && self.starts_with("'")
                    && !self.at(self.i + 1).is_some_and(char::is_alphanumeric)
            }
        }
    }

    const fn closer_len(closer: Closer) -> usize {
        match closer {
            Closer::Star2 | Closer::Under2 => 2,
            _ => 1,
        }
    }

    /// Parse inlines until one of `closers` (the innermost is last) ends
    /// them: `Some` with the content when the innermost closer was found
    /// (and consumed), `None` when an outer closer or the end came first.
    fn inlines(&mut self, closers: &[Closer]) -> Option<Vec<Inline>> {
        let mut out: Vec<Inline> = Vec::new();
        while let Some(c) = self.peek() {
            if self.unsupported.is_some() {
                return None;
            }
            if let Some(&inner) = closers.last() {
                if self.closes(inner) {
                    self.i += Self::closer_len(inner);
                    return Some(out);
                }
                if closers[..closers.len() - 1].iter().any(|&o| self.closes(o)) {
                    return None;
                }
            }
            match c {
                ' ' | '\n' => self.whitespace(&mut out),
                '\\' => self.escape(&mut out),
                '`' => self.code(&mut out),
                '*' | '_' => {
                    if !self.emphasis(c, closers, &mut out) {
                        return None;
                    }
                }
                '"' => self.double_quote(closers, &mut out),
                '\'' => self.single_quote(closers, &mut out),
                '[' => self.link(&mut out),
                '<' => self.angle(&mut out),
                '@' => self.cite(&mut out),
                '!' if self.at(self.i + 1) == Some('[') => self.refuse("image"),
                '^' if self.at(self.i + 1) == Some('[') => self.refuse("inline note"),
                '^' if self.tight_pair('^') => self.refuse("superscript"),
                '~' if self.starts_with("~~") => self.refuse("strikeout"),
                '~' if self.tight_pair('~') => self.refuse("subscript"),
                '$' if self.math() => self.refuse("math"),
                '&' => self.entity(&mut out),
                '-' if self.starts_with("---") => {
                    push_str(&mut out, "\u{2014}");
                    self.i += 3;
                }
                '-' if self.starts_with("--") => {
                    push_str(&mut out, "\u{2013}");
                    self.i += 2;
                }
                '.' if self.starts_with("...") => {
                    push_str(&mut out, "\u{2026}");
                    self.i += 3;
                }
                _ => {
                    push_str(&mut out, &c.to_string());
                    self.i += 1;
                }
            }
        }
        if closers.is_empty() { Some(out) } else { None }
    }

    fn refuse(&mut self, what: &str) {
        self.unsupported.get_or_insert_with(|| what.to_owned());
        self.i += 1;
    }

    /// Whether the delimiter `c` here opens a span closed by another `c`
    /// before any whitespace (pandoc's superscript and subscript).
    fn tight_pair(&self, c: char) -> bool {
        if self.next_is_space(1) {
            return false;
        }
        let mut k = self.i + 1;
        while let Some(x) = self.at(k) {
            if x.is_whitespace() {
                return false;
            }
            if x == c {
                return true;
            }
            k += if x == '\\' { 2 } else { 1 };
        }
        false
    }

    /// Whether the `$` here opens TeX math: followed by a non-space, and a
    /// later `$` follows a non-space and is not followed by a digit.
    fn math(&self) -> bool {
        if self.next_is_space(1) {
            return false;
        }
        let mut k = self.i + 1;
        while let Some(x) = self.at(k) {
            if x == '\\' {
                k += 2;
                continue;
            }
            if x == '$'
                && !self.at(k - 1).is_some_and(char::is_whitespace)
                && !self.at(k + 1).is_some_and(|n| n.is_ascii_digit())
            {
                return true;
            }
            k += 1;
        }
        false
    }

    fn whitespace(&mut self, out: &mut Vec<Inline>) {
        let start = self.i;
        while self.peek().is_some_and(|c| c == ' ' || c == '\n') {
            self.i += 1;
        }
        let run: String = self.s[start..self.i].iter().collect();
        if let Some(nl) = run.find('\n') {
            if nl >= 2 {
                out.push(Inline::LineBreak);
            } else {
                out.push(Inline::SoftBreak);
            }
        } else if ends_with_abbreviation(out.last()) {
            push_str(out, "\u{A0}");
        } else {
            out.push(Inline::Space);
        }
    }

    fn escape(&mut self, out: &mut Vec<Inline>) {
        match self.at(self.i + 1) {
            Some('\n') => {
                out.push(Inline::LineBreak);
                self.i += 2;
            }
            Some(c) if is_ascii_punct(c) => {
                push_str(out, &c.to_string());
                self.i += 2;
            }
            Some(c) if c.is_ascii_alphabetic() => self.refuse("raw TeX"),
            _ => {
                push_str(out, "\\");
                self.i += 1;
            }
        }
    }

    fn code(&mut self, out: &mut Vec<Inline>) {
        let start = self.i;
        let n = self.s[start..].iter().take_while(|&&c| c == '`').count();
        let mut k = start + n;
        while k < self.s.len() {
            if self.s[k] == '`' {
                let run = self.s[k..].iter().take_while(|&&c| c == '`').count();
                if run == n {
                    let content: String = self.s[start + n..k].iter().collect();
                    let content = content.replace('\n', " ");
                    out.push(Inline::Code(Attr::default(), content.trim().to_owned()));
                    self.i = k + n;
                    if self.peek() == Some('{') {
                        self.refuse("code attributes");
                    }
                    return;
                }
                k += run;
            } else {
                k += 1;
            }
        }
        // No closing run: the first backtick is literal, and the rest of
        // the run is tried again as an opener.
        push_str(out, "`");
        self.i += 1;
    }

    /// An emphasis delimiter run. A run that cannot open (followed by
    /// whitespace, or `_` inside a word) is literal. One that opens but
    /// finds no closer is literal at the top level, and inside other
    /// emphasis makes that emphasis fail too (returns false), as in pandoc.
    fn emphasis(&mut self, c: char, closers: &[Closer], out: &mut Vec<Inline>) -> bool {
        let double = self.at(self.i + 1) == Some(c);
        let width = if double { 2 } else { 1 };
        let intraword_blocked = c == '_' && self.prev().is_some_and(char::is_alphanumeric);
        if self.next_is_space(width) || intraword_blocked {
            push_str(out, &c.to_string().repeat(width));
            self.i += width;
            return true;
        }
        let closer = match (c, double) {
            ('*', true) => Closer::Star2,
            ('*', false) => Closer::Star1,
            (_, true) => Closer::Under2,
            (_, false) => Closer::Under1,
        };
        let save = self.i;
        self.i += width;
        let mut nested = closers.to_vec();
        nested.push(closer);
        if let Some(content) = self.inlines(&nested)
            && !content.is_empty()
        {
            out.push(if double {
                Inline::Strong(content)
            } else {
                Inline::Emph(content)
            });
            return true;
        }
        self.i = save;
        let inside_emphasis = closers.last().is_some_and(|c| {
            matches!(
                c,
                Closer::Star1 | Closer::Star2 | Closer::Under1 | Closer::Under2
            )
        });
        if inside_emphasis {
            return false;
        }
        push_str(out, &c.to_string().repeat(width));
        self.i += width;
        true
    }

    /// Whether a quote here can open a quotation: at the start, or after
    /// whitespace or opening punctuation.
    fn opener_context(&self) -> bool {
        self.prev()
            .is_none_or(|c| c.is_whitespace() || "([{\"'\u{201C}\u{2018}".contains(c))
    }

    fn double_quote(&mut self, closers: &[Closer], out: &mut Vec<Inline>) {
        if self.opener_context() && !self.next_is_space(1) {
            let save = self.i;
            self.i += 1;
            let mut nested = closers.to_vec();
            nested.push(Closer::DoubleQuote);
            if let Some(mut content) = self.inlines(&nested) {
                // A closing quote after whitespace takes the whitespace in.
                while matches!(content.last(), Some(Inline::Space | Inline::SoftBreak)) {
                    content.pop();
                }
                out.push(Inline::Quoted(QuoteType::Double, content));
                return;
            }
            self.i = save;
        }
        let q = if self.prev_is_space() {
            "\u{201C}"
        } else {
            "\u{201D}"
        };
        push_str(out, q);
        self.i += 1;
    }

    /// A `'`: an apostrophe unless it opens a quotation with a closer.
    /// Single quotes do not nest, and one that opens nothing is an
    /// apostrophe even at the start of a word (`'90s`).
    fn single_quote(&mut self, closers: &[Closer], out: &mut Vec<Inline>) {
        if !self.opener_context() || self.next_is_space(1) || closers.contains(&Closer::SingleQuote)
        {
            push_str(out, "\u{2019}");
            self.i += 1;
            return;
        }
        let save = self.i;
        self.i += 1;
        let mut nested = closers.to_vec();
        nested.push(Closer::SingleQuote);
        if let Some(content) = self.inlines(&nested) {
            out.push(Inline::Quoted(QuoteType::Single, content));
            return;
        }
        self.i = save;
        push_str(out, "\u{2019}");
        self.i += 1;
    }

    /// The index just past the `]` matching the `[` at `open`.
    fn matching_bracket(&self, open: usize) -> Option<usize> {
        let mut depth = 0;
        let mut k = open;
        while k < self.s.len() {
            match self.s[k] {
                '\\' => k += 1,
                '`' => {
                    let n = self.s[k..].iter().take_while(|&&c| c == '`').count();
                    let close = (k + n..self.s.len()).find(|&j| {
                        self.s[j..].iter().take_while(|&&c| c == '`').count() == n
                            && (j == 0 || self.s[j - 1] != '`')
                    });
                    k = close.map_or(k + n - 1, |j| j + n - 1);
                }
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(k + 1);
                    }
                }
                _ => {}
            }
            k += 1;
        }
        None
    }

    /// Inlines nested in a link's text.
    fn sub_inlines(&mut self, text: &str) -> Vec<Inline> {
        match parse_inlines(text, self.refs) {
            Ok(xs) => xs,
            Err(e) => {
                self.unsupported.get_or_insert_with(|| e.to_string());
                Vec::new()
            }
        }
    }

    fn link(&mut self, out: &mut Vec<Inline>) {
        if let Some(end) = self.matching_bracket(self.i) {
            let text: String = self.s[self.i + 1..end - 1].iter().collect();
            let cites = text
                .split(|c: char| c.is_whitespace() || c == ';')
                .any(|w| w.trim_start_matches('-').starts_with('@') && w.len() > 1);
            if cites {
                self.refuse("bracketed citation");
                return;
            }
            if self.at(end) == Some('{') {
                self.refuse("bracketed span or link attributes");
                return;
            }
        }
        if let Some(end) = self.matching_bracket(self.i)
            && self.at(end) == Some('(')
            && let Some((url, title, after)) = self.link_target(end + 1)
        {
            let text: String = self.s[self.i + 1..end - 1].iter().collect();
            let content = self.sub_inlines(&text);
            out.push(Inline::Link(Attr::default(), content, url, title));
            self.i = after;
            if self.peek() == Some('{') {
                self.refuse("link attributes");
            }
            return;
        }
        if let Some(end) = self.matching_bracket(self.i) {
            let text: String = self.s[self.i + 1..end - 1].iter().collect();
            // `[text][label]`, `[text][]` or `[label]`.
            let (label, after) = if self.at(end) == Some('[') {
                let close = (end + 1..self.s.len()).find(|&k| self.s[k] == ']');
                match close {
                    Some(c) => {
                        let l: String = self.s[end + 1..c].iter().collect();
                        (if l.is_empty() { text.clone() } else { l }, c + 1)
                    }
                    None => (text.clone(), end),
                }
            } else {
                (text.clone(), end)
            };
            if let Some((url, title)) = self.refs.get(&label.to_lowercase()) {
                let (url, title) = (escape_uri(url), title.clone());
                let content = self.sub_inlines(&text);
                out.push(Inline::Link(Attr::default(), content, url, title));
                self.i = after;
                return;
            }
        }
        push_str(out, "[");
        self.i += 1;
    }

    /// The `(url "title")` part of an inline link starting after its `(`:
    /// URL, title, and the index after the `)`. The URL may contain spaces
    /// (each run becomes one) and balanced parentheses; only a quoted
    /// string at the end, after whitespace, is a title.
    fn link_target(&self, start: usize) -> Option<(String, String, usize)> {
        let mut k = start;
        let mut depth = 0usize;
        let mut raw = String::new();
        loop {
            let c = self.at(k)?;
            if c == '\\' && self.at(k + 1).is_some_and(is_ascii_punct) {
                raw.push(c);
                raw.push(self.at(k + 1)?);
                k += 2;
                continue;
            }
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            raw.push(c);
            k += 1;
        }
        let inner = raw.trim();
        let mut url_part = inner;
        let mut title = String::new();
        if let Some(q) = inner.chars().last().filter(|&c| c == '"' || c == '\'') {
            let body = &inner[..inner.len() - 1];
            let open = body
                .char_indices()
                .rev()
                .find(|&(p, c)| c == q && body[..p].ends_with(char::is_whitespace))
                .map(|(p, _)| p);
            if let Some(p) = open {
                title = unescape(&body[p + 1..]);
                url_part = body[..p].trim_end();
            }
        }
        let url_part = url_part
            .strip_prefix('<')
            .and_then(|u| u.strip_suffix('>'))
            .unwrap_or(url_part);
        let url = unescape(url_part)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        Some((escape_uri(&url), title, k + 1))
    }

    fn angle(&mut self, out: &mut Vec<Inline>) {
        let rest: String = self.s[self.i..].iter().collect();
        if let Some(body) = rest.strip_prefix("<!--")
            && let Some(end) = body.find("-->")
        {
            let raw = &rest[..4 + end + 3];
            out.push(Inline::RawHtml(raw.to_owned()));
            self.i += raw.chars().count();
            return;
        }
        if let Some(close) = rest.find('>') {
            let inner = &rest[1..close];
            if is_uri(inner) {
                let attr = Attr {
                    classes: vec!["uri".to_owned()],
                    ..Attr::default()
                };
                out.push(Inline::Link(
                    attr,
                    vec![Inline::Str(inner.to_owned())],
                    inner.to_owned(),
                    String::new(),
                ));
                self.i += inner.chars().count() + 2;
                return;
            }
            if is_email(inner) {
                let attr = Attr {
                    classes: vec!["email".to_owned()],
                    ..Attr::default()
                };
                let target = format!("mailto:{inner}");
                out.push(Inline::Link(
                    attr,
                    vec![Inline::Str(inner.to_owned())],
                    target,
                    String::new(),
                ));
                self.i += inner.chars().count() + 2;
                return;
            }
            if is_tag(inner) {
                out.push(Inline::RawHtml(rest[..=close].to_owned()));
                self.i += inner.chars().count() + 2;
                return;
            }
        }
        push_str(out, "<");
        self.i += 1;
    }

    fn cite(&mut self, out: &mut Vec<Inline>) {
        if self.prev().is_none_or(|c| !c.is_alphanumeric() && c != '.') {
            let mut k = self.i + 1;
            let mut key = String::new();
            if self.at(k).is_some_and(|c| c.is_alphanumeric() || c == '_') {
                while let Some(c) = self.at(k) {
                    let internal = ":.#$%&-+?<>~/".contains(c)
                        && self
                            .at(k + 1)
                            .is_some_and(|n| n.is_alphanumeric() || n == '_');
                    if c.is_alphanumeric() || c == '_' || internal {
                        key.push(c);
                        k += 1;
                    } else {
                        break;
                    }
                }
                let content = vec![Inline::Str(format!("@{key}"))];
                out.push(Inline::Cite(vec![key], content));
                self.i = k;
                return;
            }
        }
        push_str(out, "@");
        self.i += 1;
    }

    fn entity(&mut self, out: &mut Vec<Inline>) {
        let rest: String = self.s[self.i + 1..].iter().take(40).collect();
        if let Some(semi) = rest.find(';') {
            let name = &rest[..semi];
            let decoded = if let Some(num) = name.strip_prefix('#') {
                let code = if let Some(hex) = num.strip_prefix(['x', 'X']) {
                    u32::from_str_radix(hex, 16).ok()
                } else {
                    num.parse().ok()
                };
                // A reference to NUL, a surrogate or past Unicode decodes to
                // the replacement character.
                code.map(|c| {
                    char::from_u32(c)
                        .filter(|&ch| ch != '\0')
                        .unwrap_or('\u{FFFD}')
                        .to_string()
                })
            } else if !name.is_empty() && name.chars().all(char::is_alphanumeric) {
                let key = format!("{name};");
                ENTITIES
                    .binary_search_by_key(&key.as_str(), |e| e.0)
                    .ok()
                    .map(|i| ENTITIES[i].1.to_owned())
            } else {
                None
            };
            if let Some(d) = decoded {
                push_str(out, &d);
                self.i += 1 + name.chars().count() + 1;
                return;
            }
        }
        push_str(out, "&");
        self.i += 1;
    }
}

/// Abbreviations after which pandoc's `smart` extension turns a following
/// space into a no-break space (found by probing pandoc).
const ABBREVIATIONS: &[&str] = &[
    "Apr.", "Aug.", "Bros.", "Capt.", "Co.", "Corp.", "Dec.", "Dr.", "Feb.", "Fr.", "Gen.", "Gov.",
    "Hon.", "Inc.", "Jan.", "Jr.", "Jul.", "Jun.", "Ltd.", "Mar.", "Mr.", "Mrs.", "Ms.", "No.",
    "Nov.", "Oct.", "Ph.D.", "Pres.", "Prof.", "Rep.", "Rev.", "Sen.", "Sep.", "Sept.", "Sgt.",
    "Sr.", "St.", "al.", "bk.", "cf.", "ch.", "chap.", "chs.", "e.g.", "ed.", "eds.", "ff.",
    "fig.", "fl.", "fol.", "i.e.", "ill.", "n.b.", "nn.", "pp.", "pt.", "sec.", "univ.", "viz.",
    "vol.", "vs.",
];

/// Whether `last` is text ending in an abbreviation that starts a word.
fn ends_with_abbreviation(last: Option<&Inline>) -> bool {
    let Some(Inline::Str(s)) = last else {
        return false;
    };
    ABBREVIATIONS.iter().any(|a| {
        s.strip_suffix(a).is_some_and(|before| {
            before
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric())
        })
    })
}

/// Text with its backslash escapes resolved.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(&n) = chars.peek()
            && is_ascii_punct(n)
        {
            out.push(n);
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// Percent-encode the characters pandoc does not leave bare in a URL.
fn escape_uri(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for c in url.chars() {
        if c.is_whitespace() || "`\"<>|{}^[]".contains(c) {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn is_uri(s: &str) -> bool {
    let Some((scheme, rest)) = s.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
        && scheme.len() >= 2
        && !rest.is_empty()
        && !s.chars().any(|c| c.is_whitespace() || c == '<' || c == '>')
}

fn is_email(s: &str) -> bool {
    let Some((local, domain)) = s.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local
            .chars()
            .all(|c| c.is_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(c))
        && domain.contains('.')
        && domain
            .split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_alphanumeric() || c == '-'))
}

/// An HTML open or close tag's inside (between `<` and `>`).
fn is_tag(s: &str) -> bool {
    let s = s.strip_prefix('/').unwrap_or(s);
    let s = s.strip_suffix('/').unwrap_or(s);
    let mut chars = s.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    let name_end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(s.len());
    let attrs = &s[name_end..];
    attrs.is_empty() || attrs.starts_with(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::parse_inlines;
    use crate::ast::{Inline, stringify};
    use crate::markdown::Refs;

    #[test]
    fn smart_typography() {
        let x = parse_inlines("it's \"q\" -- a --- b...", &Refs::new()).expect("supported");
        assert_eq!(
            stringify(&x),
            "it\u{2019}s \u{201C}q\u{201D} \u{2013} a \u{2014} b\u{2026}"
        );
    }

    #[test]
    fn emphasis_rules() {
        let x = parse_inlines("snake_case and *em* and 3 * 4", &Refs::new()).expect("supported");
        assert!(x.iter().any(|i| matches!(i, Inline::Emph(_))));
        assert_eq!(stringify(&x), "snake_case and em and 3 * 4");
    }
}
