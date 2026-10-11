//! The HTML writer: the document tree as a [`Doc`] whose break points are
//! the ones pandoc's HTML output has (spaces between words and before a
//! tag's attributes; nowhere inside code), and the standalone page built
//! from pandoc's `default.html5` template. Written from pandoc's observed
//! output, not from its source.

use anyhow::{Result, bail};

use crate::ast::{Attr, Block, Document, Inline, MetaValue, QuoteType, stringify};
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

pub fn blocks(bs: &[Block]) -> Doc {
    let mut d = Doc::Empty;
    for (i, b) in bs.iter().enumerate() {
        if i > 0 {
            d = d.append(cr());
        }
        d = d.append(block(b));
    }
    d
}

fn block(b: &Block) -> Doc {
    match b {
        Block::Plain(xs) => inlines(xs),
        Block::Para(xs) => element("p", &[], inlines(xs)),
        Block::Header(level, attr, xs) => {
            element(&format!("h{level}"), &attr_pairs(attr), inlines(xs))
        }
        Block::BulletList(items) => {
            let mut d = literal("<ul>");
            for item in items {
                d = d.append(cr()).append(element("li", &[], blocks(item)));
            }
            d.append(cr()).append(literal("</ul>"))
        }
        Block::Quote(bs) => literal("<blockquote>")
            .append(cr())
            .append(blocks(bs))
            .append(cr())
            .append(literal("</blockquote>")),
        Block::RawHtml(s) => literal(s),
    }
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
    "dir",
    "header-includes",
    "include-after",
    "include-before",
    "keywords",
    "lang",
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

/// Options that shape the page.
pub struct Options {
    /// Stylesheets to link, in order.
    pub css: Vec<String>,
    /// The page title when the document has none: the input's file stem.
    pub fallback_title: String,
}

/// The standalone HTML page for `doc`.
pub fn standalone(doc: &Document, opts: &Options) -> Result<String> {
    for key in UNSUPPORTED_META {
        if doc.meta(key).is_some() {
            bail!("metadata field `{key}` is not supported yet");
        }
    }
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
    ctx.insert("body".to_owned(), Val::Simple(blocks(&doc.blocks)));
    let partials = |name: &str| match name {
        "styles.html" => Some(STYLES_HTML.to_owned()),
        "styles.citations.html" => Some(STYLES_CITATIONS_HTML.to_owned()),
        _ => None,
    };
    let tpl = template::compile(DEFAULT_HTML5, &partials)?;
    Ok(layout::render(Some(COLUMNS), &template::render(&tpl, &ctx)))
}
