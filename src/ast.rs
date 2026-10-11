//! The document tree the reader builds and the writers consume, modelled
//! on the element set pandoc documents for its own AST.

/// An element's identifier, classes and key-value attributes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attr {
    pub id: String,
    pub classes: Vec<String>,
    pub kvs: Vec<(String, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteType {
    Single,
    Double,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Str(String),
    Space,
    SoftBreak,
    LineBreak,
    Code(Attr, String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Quoted(QuoteType, Vec<Inline>),
    /// A citation: its keys, and the text it was written as.
    Cite(Vec<String>, Vec<Inline>),
    /// A link: attributes, text, target URL and title.
    Link(Attr, Vec<Inline>, String, String),
    /// Raw HTML passed through untouched.
    RawHtml(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// Text not wrapped in a paragraph (a tight list item's content).
    Plain(Vec<Inline>),
    Para(Vec<Inline>),
    Header(usize, Attr, Vec<Inline>),
    BulletList(Vec<Vec<Block>>),
    Quote(Vec<Block>),
    /// Raw HTML passed through untouched.
    RawHtml(String),
}

/// A metadata value from the YAML front matter.
#[derive(Clone, Debug, PartialEq)]
pub enum MetaValue {
    Inlines(Vec<Inline>),
    List(Vec<MetaValue>),
    Bool(bool),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    pub meta: Vec<(String, MetaValue)>,
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn meta(&self, key: &str) -> Option<&MetaValue> {
        self.meta.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// The plain text of inlines, without formatting: what pandoc calls
/// stringifying.
pub fn stringify(inlines: &[Inline]) -> String {
    let mut out = String::new();
    stringify_into(inlines, &mut out);
    out
}

fn stringify_into(inlines: &[Inline], out: &mut String) {
    for i in inlines {
        match i {
            Inline::Str(s) | Inline::Code(_, s) => out.push_str(s),
            Inline::Space | Inline::SoftBreak | Inline::LineBreak => out.push(' '),
            Inline::Emph(xs)
            | Inline::Strong(xs)
            | Inline::Cite(_, xs)
            | Inline::Link(_, xs, _, _) => {
                stringify_into(xs, out);
            }
            Inline::Quoted(q, xs) => {
                let (open, close) = match q {
                    QuoteType::Single => ('\u{2018}', '\u{2019}'),
                    QuoteType::Double => ('\u{201C}', '\u{201D}'),
                };
                out.push(open);
                stringify_into(xs, out);
                out.push(close);
            }
            Inline::RawHtml(_) => {}
        }
    }
}
