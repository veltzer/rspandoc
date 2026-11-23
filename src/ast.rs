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

/// How an ordered list numbers its items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListNumberStyle {
    DefaultStyle,
    Example,
    Decimal,
    LowerRoman,
    UpperRoman,
    LowerAlpha,
    UpperAlpha,
}

/// What surrounds an ordered list's numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListNumberDelim {
    DefaultDelim,
    Period,
    OneParen,
    TwoParens,
}

/// An ordered list's first number, numbering style and delimiter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListAttributes {
    pub start: i64,
    pub style: ListNumberStyle,
    pub delim: ListNumberDelim,
}

/// A table column's alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Default,
    Left,
    Center,
    Right,
}

/// A simple table, as a pipe table writes one: a cell is a line of text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub aligns: Vec<Alignment>,
    /// Relative column widths, as the lengths of the separator line's
    /// cells, when a line of the table was too long for the page; `None`
    /// lets the columns size to their content.
    pub widths: Option<Vec<usize>>,
    /// The header row; `None` when all its cells are empty.
    pub head: Option<Vec<Vec<Inline>>>,
    pub rows: Vec<Vec<Vec<Inline>>>,
}

/// A count as an `f64`, saturating at `u32::MAX` (far beyond any column count
/// or width).
pub fn count_to_f64(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// A non-negative `f64` with its fraction dropped; negative and NaN give 0.
pub fn trunc_to_u64(x: f64) -> u64 {
    std::time::Duration::try_from_secs_f64(x.trunc()).map_or(0, |d| d.as_secs())
}

impl Table {
    /// Each column's share of the width, from 0 to 1.
    pub fn width_fractions(&self) -> Option<Vec<f64>> {
        let widths = self.widths.as_ref()?;
        let total: usize = widths.iter().sum();
        Some(
            widths
                .iter()
                .map(|&w| count_to_f64(w) / count_to_f64(total))
                .collect(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Str(String),
    Space,
    SoftBreak,
    LineBreak,
    Code(Attr, String),
    Emph(Vec<Self>),
    Strong(Vec<Self>),
    Quoted(QuoteType, Vec<Self>),
    /// A citation: its keys, and the text it was written as.
    Cite(Vec<String>, Vec<Self>),
    /// A link: attributes, text, target URL and title.
    Link(Attr, Vec<Self>, String, String),
    /// Raw HTML passed through untouched.
    RawHtml(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// Text not wrapped in a paragraph (a tight list item's content).
    Plain(Vec<Inline>),
    Para(Vec<Inline>),
    Header(usize, Attr, Vec<Inline>),
    BulletList(Vec<Vec<Self>>),
    OrderedList(ListAttributes, Vec<Vec<Self>>),
    /// Preformatted text, without its final newline.
    Code(Attr, String),
    HorizontalRule,
    Table(Box<Table>),
    Quote(Vec<Self>),
    /// Raw HTML passed through untouched.
    RawHtml(String),
    /// Raw LaTeX, which only a Lua filter makes: the LaTeX writer passes
    /// it through and the others drop it.
    RawLatex(String),
}

/// A metadata value: from the YAML front matter, or set by a Lua filter.
#[derive(Clone, Debug, PartialEq)]
pub enum MetaValue {
    Inlines(Vec<Inline>),
    /// Blocks, which only a Lua filter makes (`pandoc.MetaBlocks`).
    Blocks(Vec<Block>),
    List(Vec<Self>),
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
