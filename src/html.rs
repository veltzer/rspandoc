//! The HTML writer: the document tree as a [`Doc`] whose break points are
//! the ones pandoc's HTML output has (spaces between words and before a
//! tag's attributes; nowhere inside code), and the standalone page built
//! from pandoc's `default.html5` template. Written from pandoc's observed
//! output, not from its source.

use std::fmt::Write as _;

use anyhow::{Result, bail};

use crate::ast::{
    Attr, Block, Document, Inline, ListAttributes, ListNumberStyle, MetaValue, QuoteType, stringify,
};
use crate::highlight;
use crate::layout::{self, Doc, cr, literal, space};
use crate::template::{self, Context, Val};

/// The column pandoc wraps HTML at.
pub const COLUMNS: usize = 72;

/// pandoc's HTML text escaping.
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;")
}

/// `<name attr="v" ...>`, with a breakable space before each attribute.
fn open_tag(name: &str, attrs: &[(String, String)]) -> Doc {
    let mut d = literal(&format!("<{name}"));
    for (k, v) in attrs {
        d = d
            .append(space())
            .append(literal(&format!("{k}=\"{}\"", escape_attr(v))));
    }
    d.append(literal(">"))
}

fn close_tag(name: &str) -> Doc {
    literal(&format!("</{name}>"))
}

fn element(name: &str, attrs: &[(String, String)], content: Doc) -> Doc {
    open_tag(name, attrs)
        .append(content)
        .append(close_tag(name))
}

fn attr_pairs(attr: &Attr) -> Vec<(String, String)> {
    let mut v = Vec::new();
    if !attr.id.is_empty() {
        v.push(("id".to_owned(), attr.id.clone()));
    }
    if !attr.classes.is_empty() {
        v.push(("class".to_owned(), attr.classes.join(" ")));
    }
    v.extend(attr.kvs.iter().cloned());
    v
}

/// Text whose spaces are break points, as all text outside code is in
/// pandoc's HTML (a filter can put spaces inside a `Str`).
fn text_doc(s: &str) -> Doc {
    let mut d = Doc::Empty;
    for (i, word) in s.split(' ').enumerate() {
        if i > 0 {
            d = d.append(space());
        }
        if !word.is_empty() {
            d = d.append(literal(word));
        }
    }
    d
}

pub fn inlines(xs: &[Inline]) -> Doc {
    layout::hcat(xs.iter().map(inline))
}

fn inline(x: &Inline) -> Doc {
    match x {
        Inline::Str(s) => text_doc(&escape_text(s)),
        Inline::Space | Inline::SoftBreak => space(),
        Inline::LineBreak => literal("<br />").append(cr()),
        Inline::Code(attr, s) => element("code", &attr_pairs(attr), literal(&escape_text(s))),
        Inline::Emph(xs) => element("em", &[], inlines(xs)),
        Inline::Strong(xs) => element("strong", &[], inlines(xs)),
        Inline::Quoted(q, xs) => {
            let (open, close) = match q {
                QuoteType::Single => ("\u{2018}", "\u{2019}"),
                QuoteType::Double => ("\u{201C}", "\u{201D}"),
            };
            literal(open).append(inlines(xs)).append(literal(close))
        }
        Inline::Cite(keys, xs) => {
            let attrs = [
                ("class".to_owned(), "citation".to_owned()),
                ("data-cites".to_owned(), keys.join(" ")),
            ];
            element("span", &attrs, inlines(xs))
        }
        Inline::Link(attr, xs, url, title) => {
            let mut attrs = vec![("href".to_owned(), url.clone())];
            attrs.extend(attr_pairs(attr));
            if !title.is_empty() {
                attrs.push(("title".to_owned(), title.clone()));
            }
            element("a", &attrs, inlines(xs))
        }
        Inline::RawHtml(s) => literal(s),
    }
}

/// Writes blocks; it numbers the code blocks, as pandoc does, to give
/// their lines identifiers.
pub struct Writer {
    /// Whether to highlight code blocks in a known language.
    highlight: bool,
    /// Code blocks written so far.
    code_blocks: usize,
    /// Whether a code block was highlighted: the page needs the
    /// highlighting stylesheet.
    highlighted: bool,
}

impl Writer {
    pub const fn new(highlight: bool) -> Self {
        Self {
            highlight,
            code_blocks: 0,
            highlighted: false,
        }
    }

    pub fn blocks(&mut self, bs: &[Block]) -> Result<Doc> {
        let mut d = Doc::Empty;
        for (i, b) in bs.iter().enumerate() {
            if i > 0 {
                d = d.append(cr());
            }
            d = d.append(self.block(b)?);
        }
        Ok(d)
    }

    fn list_items(&mut self, items: &[Vec<Block>]) -> Result<Doc> {
        let mut d = Doc::Empty;
        for item in items {
            let content = match task(item) {
                Some((checked, first, rest)) => {
                    let mut input = literal("<input")
                        .append(space())
                        .append(literal(r#"type="checkbox""#));
                    if checked {
                        input = input.append(space()).append(literal(r#"checked="""#));
                    }
                    let label = element(
                        "label",
                        &[],
                        input.append(literal(" />")).append(inlines(first)),
                    );
                    let first = match &item[0] {
                        Block::Para(_) => element("p", &[], label),
                        _ => label,
                    };
                    if rest.is_empty() {
                        first
                    } else {
                        first.append(cr()).append(self.blocks(rest)?)
                    }
                }
                None => self.blocks(item)?,
            };
            d = d.append(cr()).append(element("li", &[], content));
        }
        Ok(d)
    }

    fn block(&mut self, b: &Block) -> Result<Doc> {
        Ok(match b {
            Block::Plain(xs) => inlines(xs),
            Block::Para(xs) => element("p", &[], inlines(xs)),
            Block::Header(level, attr, xs) if *level > 6 => {
                let mut attrs = vec![("class".to_owned(), "heading".to_owned())];
                attrs.extend(attr_pairs(attr));
                element("p", &attrs, inlines(xs))
            }
            Block::Header(level, attr, xs) => {
                element(&format!("h{level}"), &attr_pairs(attr), inlines(xs))
            }
            Block::BulletList(items) => {
                // A list of nothing but tasks is a task list.
                let attrs = if !items.is_empty() && items.iter().all(|i| task(i).is_some()) {
                    vec![("class".to_owned(), "task-list".to_owned())]
                } else {
                    Vec::new()
                };
                open_tag("ul", &attrs)
            }
            .append(self.list_items(items)?)
            .append(cr())
            .append(literal("</ul>")),
            Block::OrderedList(la, items) => open_tag("ol", &ordered_list_attrs(la)?)
                .append(self.list_items(items)?)
                .append(cr())
                .append(literal("</ol>")),
            Block::Code(attr, code) => self.code_block(attr, code),
            Block::HorizontalRule => literal("<hr />"),
            Block::Quote(bs) => literal("<blockquote>")
                .append(cr())
                .append(self.blocks(bs)?)
                .append(cr())
                .append(literal("</blockquote>")),
            Block::RawHtml(s) => literal(s),
        })
    }

    /// A code block: highlighted when its first class names a language
    /// there is a syntax for, otherwise `<pre><code>` with its attributes.
    fn code_block(&mut self, attr: &Attr, code: &str) -> Doc {
        self.code_blocks += 1;
        let highlighted = attr
            .classes
            .first()
            .filter(|_| self.highlight)
            .and_then(|lang| highlight::highlight(lang, code));
        let Some(h) = highlighted else {
            return open_tag("pre", &attr_pairs(attr))
                .append(literal("<code>"))
                .append(literal(&escape_code(code)))
                .append(literal("</code></pre>"));
        };
        self.highlighted = true;
        let id = if attr.id.is_empty() {
            format!("cb{}", self.code_blocks)
        } else {
            attr.id.clone()
        };
        let mut body = String::new();
        for (n, line) in h.lines.iter().enumerate() {
            if n > 0 {
                body.push('\n');
            }
            let line_id = format!("{id}-{}", n + 1);
            let _ = write!(
                body,
                r##"<span id="{line_id}"><a href="#{line_id}" aria-hidden="true" tabindex="-1"></a>"##
            );
            for (t, text) in line {
                match t {
                    Some(t) => {
                        let _ = write!(
                            body,
                            r#"<span class="{}">{}</span>"#,
                            t.class,
                            escape_code(text)
                        );
                    }
                    None => body.push_str(&escape_code(text)),
                }
            }
            body.push_str("</span>");
        }
        let div_attrs = [
            ("class".to_owned(), "sourceCode".to_owned()),
            ("id".to_owned(), id),
        ];
        let pre_attrs = [(
            "class".to_owned(),
            format!("sourceCode {}", attr.classes.join(" ")),
        )];
        open_tag("div", &div_attrs)
            .append(open_tag("pre", &pre_attrs))
            .append(literal(&format!(r#"<code class="sourceCode {}">"#, h.name)))
            .append(literal(&body))
            .append(literal("</code></pre></div>"))
    }
}

/// A task list item: whether it is checked, its first block's text after
/// the box, and its other blocks.
fn task(item: &[Block]) -> Option<(bool, &[Inline], &[Block])> {
    let (Block::Plain(xs) | Block::Para(xs)) = item.first()? else {
        return None;
    };
    let checked = match xs.first()? {
        Inline::Str(s) if s == "\u{2610}" => false,
        Inline::Str(s) if s == "\u{2612}" => true,
        _ => return None,
    };
    if !matches!(xs.get(1), Some(Inline::Space | Inline::SoftBreak)) {
        return None;
    }
    Some((checked, &xs[2..], &item[1..]))
}

/// Text in a code block, which escapes quotes too (unlike inline code).
fn escape_code(s: &str) -> String {
    escape_text(s).replace('"', "&quot;").replace('\'', "&#39;")
}

/// `<ol>`'s attributes: the first number when it is not 1, and the
/// numbering style.
fn ordered_list_attrs(la: &ListAttributes) -> Result<Vec<(String, String)>> {
    let mut attrs = Vec::new();
    if la.start != 1 {
        attrs.push(("start".to_owned(), la.start.to_string()));
    }
    let kind = match la.style {
        ListNumberStyle::DefaultStyle => None,
        ListNumberStyle::Decimal => Some("1"),
        ListNumberStyle::LowerAlpha => Some("a"),
        ListNumberStyle::UpperAlpha => Some("A"),
        ListNumberStyle::LowerRoman => Some("i"),
        ListNumberStyle::UpperRoman => Some("I"),
        ListNumberStyle::Example => bail!("example lists are not supported yet"),
    };
    if let Some(k) = kind {
        attrs.push(("type".to_owned(), k.to_owned()));
    }
    Ok(attrs)
}

/// pandoc's `default.html5` and its `styles.html` partial (dual-licensed
/// GPL-2.0-or-later or BSD-3-Clause; used here under BSD-3-Clause).
const DEFAULT_HTML5: &str = include_str!("../data/templates/default.html5");
const STYLES_HTML: &str = include_str!("../data/templates/styles.html");
const STYLES_CITATIONS_HTML: &str = include_str!("../data/templates/styles.citations.html");

/// Metadata fields the standalone page renders specially; documents that
/// set one are refused until it is supported, rather than rendered wrong.
const UNSUPPORTED_META: &[&str] = &[
    "abstract",
    "author",
    "date",
    "description",
    "header-includes",
    "include-after",
    "include-before",
    "keywords",
    "subtitle",
    "title-prefix",
    "toc",
];

fn meta_val(v: &MetaValue) -> Val {
    match v {
        MetaValue::Inlines(xs) => Val::Simple(inlines(xs)),
        MetaValue::List(xs) => Val::List(xs.iter().map(meta_val).collect()),
        MetaValue::Bool(b) => Val::Bool(*b),
    }
}

/// Refuse a document that sets one of `fields`: metadata a writer renders
/// specially but rspandoc does not support yet.
pub fn check_meta(doc: &Document, fields: &[&str]) -> Result<()> {
    for key in fields {
        if doc.meta(key).is_some() {
            bail!("metadata field `{key}` is not supported yet");
        }
    }
    Ok(())
}

/// Options that shape the page.
pub struct Options {
    /// Stylesheets to link, in order.
    pub css: Vec<String>,
    /// The page title when the document has none: the input's file stem.
    pub fallback_title: String,
    /// Whether to highlight code blocks (no `--no-highlight`).
    pub highlight: bool,
}

/// The standalone HTML page for `doc`.
pub fn standalone(doc: &Document, opts: &Options) -> Result<String> {
    check_meta(doc, UNSUPPORTED_META)?;
    let mut ctx = Context::new();
    for (k, v) in &doc.meta {
        ctx.insert(k.clone(), meta_val(v));
    }
    let pagetitle = match doc.meta("title") {
        Some(MetaValue::Inlines(xs)) => stringify(xs),
        Some(_) => bail!("metadata field `title` must be text"),
        None => opts.fallback_title.clone(),
    };
    ctx.insert(
        "pagetitle".to_owned(),
        Val::Simple(literal(&escape_text(&pagetitle))),
    );
    if opts.css.is_empty() {
        ctx.insert("document-css".to_owned(), Val::Bool(true));
    } else {
        let css = opts.css.iter().map(|c| Val::Simple(literal(c))).collect();
        ctx.insert("css".to_owned(), Val::List(css));
    }
    ctx.insert("displaymath-css".to_owned(), Val::Bool(true));
    let mut w = Writer::new(opts.highlight);
    ctx.insert("body".to_owned(), Val::Simple(w.blocks(&doc.blocks)?));
    if w.highlighted {
        ctx.insert(
            "highlighting-css".to_owned(),
            Val::Simple(literal(highlight::css().trim_end())),
        );
    }
    let partials = |name: &str| match name {
        "styles.html" => Some(STYLES_HTML.to_owned()),
        "styles.citations.html" => Some(STYLES_CITATIONS_HTML.to_owned()),
        _ => None,
    };
    let tpl = template::compile(DEFAULT_HTML5, "default.html5", &partials)?;
    Ok(layout::render(Some(COLUMNS), &template::render(&tpl, &ctx)))
}
