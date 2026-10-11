//! A reader for pandoc's Markdown dialect, covering the constructs the
//! fleet's documents use: YAML front matter, ATX headers, paragraphs,
//! bullet lists, decimal ordered lists, indented and fenced code blocks
//! (without a language), horizontal rules, block quotes, HTML comments,
//! and the inline syntax in `inline.rs`. Written from pandoc's documented
//! behaviour and observed output, not from its source.
//!
//! Where pandoc's Markdown differs from CommonMark this follows pandoc: a
//! header, block quote or list needs a blank line before it unless it opens
//! its container or follows another block that ended (a header or an HTML
//! block), and a line that does not start a block continues the paragraph
//! it follows, even inside a list item ("lazy" continuation).

mod inline;
mod yaml;

use anyhow::{Result, bail};

use std::collections::HashMap;

use crate::ast::{Attr, Block, Document, Inline, ListAttributes, ListNumberDelim, ListNumberStyle};

pub use inline::parse_inlines;

/// Link reference definitions: label (lowercased) to URL and title.
pub type Refs = HashMap<String, (String, String)>;

/// Parse a Markdown document.
pub fn read(text: &str) -> Result<Document> {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<String> = text.split('\n').map(expand_tabs).collect();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let (meta, body_start) = yaml::front_matter(&lines)?;
    let body: Vec<String> = lines[body_start..]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    // Links may use a definition that comes later, so the definitions are
    // collected by a first pass.
    let mut refs = Refs::new();
    parse_blocks(&body, false, &mut refs)?;
    let mut blocks = parse_blocks(&body, false, &mut refs)?;
    assign_identifiers(&mut blocks);
    Ok(Document { meta, blocks })
}

/// The tab stop pandoc expands tabs to (`--tab-stop`).
const TAB_STOP: usize = 4;

/// `line` with its tabs expanded to spaces up to the next tab stop.
fn expand_tabs(line: &str) -> String {
    if !line.contains('\t') {
        return line.to_owned();
    }
    let mut out = String::with_capacity(line.len() + TAB_STOP);
    let mut col = 0;
    for c in line.chars() {
        if c == '\t' {
            let n = TAB_STOP - col % TAB_STOP;
            out.extend(std::iter::repeat_n(' ', n));
            col += n;
        } else {
            out.push(c);
            col += 1;
        }
    }
    out
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// What kind of list a marker line continues or starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marker {
    Bullet,
    Ordered(ListAttributes),
}

impl Marker {
    /// Whether an item with this marker continues a list started by
    /// `first`: any bullet continues a bullet list, and an ordered item one
    /// with the same delimiter, whatever its number. (`#` stands for any
    /// number, and `#.`'s default delimiter is a period; the numbering
    /// styles this reader supports are all decimal.)
    fn continues(self, first: Self) -> bool {
        let delim = |la: ListAttributes| match la.delim {
            ListNumberDelim::DefaultDelim => ListNumberDelim::Period,
            d => d,
        };
        match (self, first) {
            (Self::Bullet, Self::Bullet) => true,
            (Self::Ordered(a), Self::Ordered(b)) => delim(a) == delim(b),
            _ => false,
        }
    }
}

/// A list marker line this reader supports: (marker, content column,
/// content). Bullets are `*`, `-` and `+`; ordered markers are a decimal
/// number or `#`, as `1.`, `1)` or `(1)`.
fn list_marker(line: &str) -> Option<(Marker, usize, &str)> {
    let ind = indent_of(line);
    if ind > 3 {
        return None;
    }
    let rest = &line[ind..];
    let (marker, after) = if let Some(after) = rest.strip_prefix(['*', '-', '+']) {
        // A horizontal rule (`* * *`) is not a list.
        if horizontal_rule(line) {
            return None;
        }
        (Marker::Bullet, after)
    } else {
        let (body, delim, after) = if let Some(r) = rest.strip_prefix('(') {
            let close = r.find(')')?;
            (&r[..close], ListNumberDelim::TwoParens, &r[close + 1..])
        } else {
            let end = rest.find(['.', ')'])?;
            let delim = if rest[end..].starts_with('.') {
                ListNumberDelim::Period
            } else {
                ListNumberDelim::OneParen
            };
            (&rest[..end], delim, &rest[end + 1..])
        };
        let attrs = if body == "#" {
            let delim = if delim == ListNumberDelim::Period {
                ListNumberDelim::DefaultDelim
            } else {
                delim
            };
            ListAttributes {
                start: 1,
                style: ListNumberStyle::DefaultStyle,
                delim,
            }
        } else if (1..=9).contains(&body.len()) && body.chars().all(|c| c.is_ascii_digit()) {
            ListAttributes {
                start: body.parse().ok()?,
                style: ListNumberStyle::Decimal,
                delim,
            }
        } else {
            return None;
        };
        (Marker::Ordered(attrs), after)
    };
    let marker_end = line.len() - after.len();
    if after.is_empty() {
        return Some((marker, marker_end + 1, ""));
    }
    if !after.starts_with(' ') {
        return None;
    }
    let spaces = indent_of(after);
    // More than four spaces after the marker start indented content, which
    // keeps all but one of them.
    let spaces = if spaces > 4 { 1 } else { spaces };
    Some((marker, marker_end + spaces, &after[spaces..]))
}

/// An item's first line with a task box (`[ ]`, `[x]`) as the ballot box
/// character pandoc reads it as (☐, ☒), which writers turn into a
/// checkbox.
fn task_box(content: &str) -> String {
    let boxed = |rest: &str, mark: char| {
        let rest = rest.strip_prefix(' ')?.trim_start();
        (!rest.is_empty()).then(|| format!("{mark} {rest}"))
    };
    content
        .strip_prefix("[ ]")
        .and_then(|r| boxed(r, '\u{2610}'))
        .or_else(|| {
            content
                .strip_prefix("[x]")
                .or_else(|| content.strip_prefix("[X]"))
                .and_then(|r| boxed(r, '\u{2612}'))
        })
        .unwrap_or_else(|| content.to_owned())
}

/// Whether `line` is a list marker of any kind pandoc knows, supported or
/// not: it ends the list item before it.
fn any_list_marker(line: &str) -> bool {
    list_marker(line).is_some() || (indent_of(line) <= 3 && ordered_marker(line.trim()))
}

/// A horizontal rule: three or more `*`, `-` or `_`, all the same,
/// with optional spaces between.
fn horizontal_rule(line: &str) -> bool {
    if indent_of(line) > 3 {
        return false;
    }
    let t = line.trim();
    let Some(c) = t.chars().next() else {
        return false;
    };
    matches!(c, '*' | '-' | '_')
        && t.chars().all(|x| x == c || x == ' ')
        && t.chars().filter(|&x| x == c).count() >= 3
}

/// The opening line of a fenced code block: its indent, fence character,
/// fence length and info string.
struct Fence<'a> {
    indent: usize,
    ch: char,
    len: usize,
    info: &'a str,
}

fn fence(line: &str) -> Option<Fence<'_>> {
    let indent = indent_of(line);
    if indent > 3 {
        return None;
    }
    let t = &line[indent..];
    let ch = t.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let len = t.chars().take_while(|&c| c == ch).count();
    if len < 3 {
        return None;
    }
    let info = t[len..].trim();
    // A backtick fence's info string has no backticks; with them the line
    // is text with code spans.
    if ch == '`' && info.contains('`') {
        return None;
    }
    Some(Fence {
        indent,
        ch,
        len,
        info,
    })
}

/// Whether `line` closes the fence `f`: the same character, at least as
/// many, and nothing else.
fn closes_fence(line: &str, f: &Fence) -> bool {
    let t = line.trim();
    indent_of(line) <= 3 && t.chars().count() >= f.len && t.chars().all(|c| c == f.ch)
}

fn atx_header(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    // Seven or more `#`s are a header too (pandoc 3 renders it as a
    // paragraph styled "heading" in HTML).
    if level == 0 {
        return None;
    }
    let rest = &line[level..];
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let mut text = rest.trim();
    // An optional closing sequence of #s, preceded by a space.
    let stripped = text.trim_end_matches('#');
    if stripped.len() < text.len() && (stripped.is_empty() || stripped.ends_with(' ')) {
        text = stripped.trim_end();
    }
    Some((level, text))
}

/// Strip up to `n` leading spaces.
fn dedent(line: &str, n: usize) -> String {
    line[indent_of(line).min(n)..].to_owned()
}

/// A link reference definition line: label, URL, title.
fn reference_definition(line: &str) -> Option<(String, String, String)> {
    if indent_of(line) > 3 {
        return None;
    }
    let rest = line.trim_start().strip_prefix('[')?;
    let close = rest.find(']')?;
    let label = &rest[..close];
    if label.is_empty() {
        return None;
    }
    let rest = rest[close + 1..].strip_prefix(':')?.trim_start();
    let (url, rest) = if let Some(r) = rest.strip_prefix('<') {
        let end = r.find('>')?;
        (&r[..end], &r[end + 1..])
    } else {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if end == 0 {
            return None;
        }
        (&rest[..end], &rest[end..])
    };
    let rest = rest.trim();
    let title = if rest.is_empty() {
        ""
    } else {
        let (open, close) = match rest.chars().next()? {
            '"' => ('"', '"'),
            '\'' => ('\'', '\''),
            '(' => ('(', ')'),
            _ => return None,
        };
        rest.strip_prefix(open)?.strip_suffix(close)?
    };
    Some((label.to_lowercase(), url.to_owned(), title.to_owned()))
}

/// HTML elements that make a line starting with them a raw HTML block.
const HTML_BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "canvas",
    "center",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "meta",
    "nav",
    "noframes",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "pre",
    "script",
    "section",
    "source",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "video",
];

/// A pandoc Markdown construct this reader does not implement, starting at
/// `line`. `may_start` says whether a block that needs a fresh start (a
/// list, indented code, an HTML block) could begin here; the others are
/// checked on every line, since they interrupt or reinterpret paragraphs.
fn unsupported_block(line: &str, may_start: bool) -> Option<&'static str> {
    let t = line.trim();
    if let Some(f) = fence(line)
        && f.info.contains(|c: char| c.is_whitespace() || c == '{')
    {
        return Some("fenced code block attributes");
    }
    if t.starts_with(":::") {
        return Some("fenced div");
    }
    if t.starts_with("[^") && t.contains("]:") {
        return Some("footnote");
    }
    if t.starts_with("\\begin{") {
        return Some("LaTeX environment");
    }
    if indent_of(line) <= 3 {
        if t.chars().all(|c| "-=".contains(c)) {
            return Some("setext header or horizontal rule");
        }
        let marks = t.chars().filter(|c| "-*_".contains(*c)).count();
        if marks >= 3 && t.chars().all(|c| "-*_ ".contains(c)) {
            return Some("horizontal rule");
        }
        if t.starts_with('|') || (t.contains("--") && t.chars().all(|c| "-=:|+ ".contains(c))) {
            return Some("table or line block");
        }
        if t.starts_with(": ") || t.starts_with("~ ") {
            return Some("definition list");
        }
    }
    if !may_start {
        return None;
    }
    if indent_of(line) >= 4 {
        return Some("indented code block");
    }
    if ordered_marker(t) && list_marker(line).is_none() {
        return Some("ordered list numbered by letters or roman numerals");
    }
    if let Some(rest) = t.strip_prefix('<') {
        let name: String = rest
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if HTML_BLOCK_TAGS.contains(&name.as_str()) {
            return Some("HTML block");
        }
    }
    None
}

/// Whether `t` starts with an ordered list marker: a number, `#`, a letter
/// or a roman numeral, followed by `.` or `)` and a space, or `(n)`.
fn ordered_marker(t: &str) -> bool {
    let (body, rest) = if let Some(r) = t.strip_prefix('(') {
        match r.find(')') {
            Some(c) => (&r[..c], &r[c + 1..]),
            None => return false,
        }
    } else {
        match t.find(['.', ')']) {
            Some(c) => (&t[..c], &t[c + 1..]),
            None => return false,
        }
    };
    let numeral = !body.is_empty()
        && (body.chars().all(|c| c.is_ascii_digit())
            || body == "#"
            || (body.chars().count() == 1 && body.chars().all(|c| c.is_ascii_alphabetic()))
            || body.chars().all(|c| "ivxlcdm".contains(c))
            || body.chars().all(|c| "IVXLCDM".contains(c)));
    numeral && (rest.is_empty() || rest.starts_with(' '))
}

fn parse_blocks(lines: &[String], in_item: bool, refs: &mut Refs) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut i = 0;
    // Whether a block that needs a fresh start (header, list, quote, HTML
    // block) may begin here.
    let mut fresh = true;
    while i < lines.len() {
        let line = lines[i].as_str();
        if is_blank(line) {
            fresh = true;
            i += 1;
            continue;
        }
        if let Some(f) = fence(line)
            && !f.info.contains(|c: char| c.is_whitespace() || c == '{')
        {
            let Some(len) = lines[i + 1..].iter().position(|l| closes_fence(l, &f)) else {
                bail!(
                    "unsupported Markdown (unclosed fenced code block) at: {}",
                    line.trim()
                );
            };
            let code: Vec<String> = lines[i + 1..i + 1 + len]
                .iter()
                .map(|l| dedent(l, f.indent))
                .collect();
            // A one-word info string is the code's language, its class.
            let attr = Attr {
                classes: if f.info.is_empty() {
                    Vec::new()
                } else {
                    vec![f.info.to_owned()]
                },
                ..Attr::default()
            };
            blocks.push(Block::Code(attr, code.join("\n")));
            i += len + 2;
            fresh = true;
            continue;
        }
        if fresh && horizontal_rule(line) {
            // `---` before a non-blank line may open a YAML metadata block.
            if line.trim().starts_with('-') && lines.get(i + 1).is_some_and(|l| !is_blank(l)) {
                bail!(
                    "unsupported Markdown (YAML metadata block) at: {}",
                    line.trim()
                );
            }
            blocks.push(Block::HorizontalRule);
            i += 1;
            fresh = true;
            continue;
        }
        if fresh && indent_of(line) >= 4 {
            let mut end = i;
            let mut j = i;
            while j < lines.len() && (is_blank(&lines[j]) || indent_of(&lines[j]) >= 4) {
                if !is_blank(&lines[j]) {
                    end = j + 1;
                }
                j += 1;
            }
            let code: Vec<String> = lines[i..end].iter().map(|l| dedent(l, 4)).collect();
            blocks.push(Block::Code(Attr::default(), code.join("\n")));
            i = end;
            fresh = true;
            continue;
        }
        if let Some(what) = unsupported_block(line, fresh || in_item) {
            bail!("unsupported Markdown ({what}) at: {}", line.trim());
        }
        if fresh && indent_of(line) <= 3 {
            if let Some((level, text)) = atx_header(line.trim_start()) {
                if text.ends_with('}') && text.contains(" {") {
                    bail!(
                        "unsupported Markdown (header attributes) at: {}",
                        line.trim()
                    );
                }
                blocks.push(Block::Header(
                    level,
                    Attr::default(),
                    parse_inlines(text, refs)?,
                ));
                i += 1;
                continue;
            }
            if let Some((label, url, title)) = reference_definition(line) {
                refs.entry(label).or_insert((url, title));
                i += 1;
                continue;
            }
            if line.trim_start().starts_with("<!--") {
                let start = i;
                while i < lines.len() && !lines[i].contains("-->") {
                    i += 1;
                }
                let end = (i + 1).min(lines.len());
                let raw = lines[start..end].join("\n");
                blocks.push(Block::RawHtml(raw.trim().to_owned()));
                i = end;
                continue;
            }
            if line.trim_start().starts_with('>') {
                let mut quoted = Vec::new();
                while i < lines.len() && !is_blank(&lines[i]) {
                    let l = lines[i].trim_start();
                    let l = l
                        .strip_prefix('>')
                        .map_or(l, |r| r.strip_prefix(' ').unwrap_or(r));
                    quoted.push(l.to_owned());
                    i += 1;
                }
                blocks.push(Block::Quote(parse_blocks(&quoted, false, refs)?));
                fresh = false;
                continue;
            }
        }
        if (fresh || in_item) && list_marker(line).is_some() {
            let (list, next) = parse_list(lines, i, refs)?;
            blocks.push(list);
            i = next;
            // A list ends at a blank line or at the marker of a different
            // list, which starts at once.
            fresh = lines.get(i).is_some_and(|l| any_list_marker(l));
            continue;
        }
        let mut para = Vec::new();
        let mut fenced = false;
        while i < lines.len() && !is_blank(&lines[i]) {
            if !para.is_empty() && in_item && any_list_marker(&lines[i]) {
                break;
            }
            // A code fence ends a paragraph, which is then a real one.
            if !para.is_empty() && fence(&lines[i]).is_some() {
                fenced = true;
                break;
            }
            if !para.is_empty()
                && let Some(what) = unsupported_block(&lines[i], false)
            {
                bail!("unsupported Markdown ({what}) at: {}", lines[i].trim());
            }
            para.push(lines[i].trim_start());
            i += 1;
        }
        let text = para.join("\n");
        let inlines = parse_inlines(text.trim_end(), refs)?;
        // In a list item, only a paragraph a blank line follows is one;
        // the rest is plain text (until the list decides it is loose).
        if in_item && !fenced && !lines.get(i).is_some_and(|l| is_blank(l)) {
            blocks.push(Block::Plain(inlines));
        } else {
            blocks.push(Block::Para(inlines));
        }
        fresh = false;
    }
    Ok(blocks)
}

/// Parse the list starting at `start`; returns it and the index of
/// the first line after it.
///
/// Blank lines between two items belong to the first, so its last
/// paragraph is a real one; blank lines after the last item belong to no
/// item. When any item has a real paragraph the list is loose and all its
/// items' plain text becomes paragraphs.
fn parse_list(lines: &[String], start: usize, refs: &mut Refs) -> Result<(Block, usize)> {
    let mut items: Vec<Vec<String>> = Vec::new();
    let mut i = start;
    let Some((first, _, _)) = list_marker(&lines[start]) else {
        bail!("no list starts at: {}", lines[start].trim());
    };
    while i < lines.len() {
        let Some((marker, col, content)) = list_marker(&lines[i]) else {
            break;
        };
        if !marker.continues(first) {
            break;
        }
        let mut item = vec![task_box(content)];
        i += 1;
        while i < lines.len() {
            let l = &lines[i];
            if is_blank(l) {
                // Blank lines belong to the item only when content indented
                // to its content column follows them.
                let next = lines[i..].iter().position(|l| !is_blank(l)).map(|p| p + i);
                match next {
                    Some(n) if indent_of(&lines[n]) >= col => {
                        item.extend(std::iter::repeat_n(String::new(), n - i));
                        i = n;
                        continue;
                    }
                    _ => break,
                }
            }
            let ind = indent_of(l);
            // A marker short of the content column starts the next item; one
            // at or past it nests inside this one.
            if ind < col && any_list_marker(l) {
                break;
            }
            // Anything else short of the content column is a lazy
            // continuation of the item's paragraph, except a code fence,
            // which pandoc treats inconsistently there.
            if ind < col && fence(l).is_some() {
                bail!(
                    "unsupported Markdown (code fence not indented into its list item) at: {}",
                    l.trim()
                );
            }
            item.push(dedent(l, col));
            i += 1;
        }
        let next = lines[i..].iter().position(|l| !is_blank(l)).map(|p| p + i);
        match next {
            Some(n) if list_marker(&lines[n]).is_some_and(|(m, _, _)| m.continues(first)) => {
                item.extend(std::iter::repeat_n(String::new(), n - i));
                items.push(item);
                i = n;
            }
            _ => {
                items.push(item);
                break;
            }
        }
    }
    let mut items: Vec<Vec<Block>> = items
        .iter()
        .map(|lines| parse_blocks(lines, true, refs))
        .collect::<Result<_>>()?;
    let loose = items.iter().flatten().any(|b| matches!(b, Block::Para(_)));
    if loose {
        for item in &mut items {
            for b in item.iter_mut() {
                if let Block::Plain(xs) = b {
                    *b = Block::Para(std::mem::take(xs));
                }
            }
        }
    }
    let list = match first {
        Marker::Bullet => Block::BulletList(items),
        Marker::Ordered(attrs) => Block::OrderedList(attrs, items),
    };
    Ok((list, i))
}

/// Give every header an identifier from its text, as pandoc's
/// `auto_identifiers` does, numbering repeats.
fn assign_identifiers(blocks: &mut [Block]) {
    let mut used = std::collections::HashSet::new();
    visit_headers(blocks, &mut |attr, inlines| {
        let base = identifier(inlines);
        let mut id = base.clone();
        let mut n = 0;
        while used.contains(&id) {
            n += 1;
            id = format!("{base}-{n}");
        }
        used.insert(id.clone());
        attr.id = id;
    });
}

fn visit_headers(blocks: &mut [Block], f: &mut dyn FnMut(&mut Attr, &[Inline])) {
    for b in blocks {
        match b {
            Block::Header(_, attr, inlines) => f(attr, inlines),
            Block::BulletList(items) | Block::OrderedList(_, items) => {
                for item in items {
                    visit_headers(item, f);
                }
            }
            Block::Quote(bs) => visit_headers(bs, f),
            Block::Plain(_)
            | Block::Para(_)
            | Block::RawHtml(_)
            | Block::Code(..)
            | Block::HorizontalRule => {}
        }
    }
}

/// pandoc's identifier rule: keep letters, digits, `_`, `-` and `.`, join
/// the words with `-`, lowercase, and drop everything before the first
/// letter; `section` when nothing is left.
fn identifier(inlines: &[Inline]) -> String {
    let text = plain_for_id(inlines);
    let kept: String = text
        .chars()
        .filter(|&c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.') || c.is_whitespace())
        .collect();
    let joined = kept
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase();
    let id = joined.trim_start_matches(|c: char| !c.is_alphabetic());
    if id.is_empty() {
        "section".to_owned()
    } else {
        id.to_owned()
    }
}

fn plain_for_id(inlines: &[Inline]) -> String {
    crate::ast::stringify(inlines)
}
