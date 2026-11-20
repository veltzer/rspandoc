//! The LaTeX writer: the document tree as the LaTeX pandoc writes for it,
//! and the standalone document built from pandoc's `default.latex`
//! template. A PDF is this document run through a TeX engine (`pdf.rs`).
//! Written from pandoc's observed output, not from its source.

use std::fmt::Write as _;

use anyhow::{Result, bail};

use crate::ast::{
    Alignment, Block, Document, Inline, ListAttributes, ListNumberDelim, ListNumberStyle,
    MetaValue, QuoteType, Table, stringify,
};
use crate::highlight;
use crate::layout::{self, Doc, blankline, cr, flush, literal, nest, space};
use crate::template::{self, Context, Val};

/// The column pandoc wraps LaTeX at.
pub const COLUMNS: usize = 72;

/// Where text is being escaped: running text, or code (`\texttt` and
/// highlighted tokens), where every control word is closed with `{}`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Text,
    Code,
}

/// A control word and what must follow it so that the next character is
/// not read as part of it: `{}` in code, at the end and before a space
/// (which TeX would swallow), a space before a letter, nothing otherwise.
fn control_word(out: &mut String, name: &str, next: Option<char>, mode: Mode) {
    out.push('\\');
    out.push_str(name);
    match next {
        _ if mode == Mode::Code => out.push_str("{}"),
        None => out.push_str("{}"),
        Some(c) if c.is_whitespace() => out.push_str("{}"),
        Some(c) if c.is_alphabetic() => out.push(' '),
        Some(_) => {}
    }
}

/// pandoc's LaTeX escaping of text.
fn escape(s: &str, mode: Mode) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match c {
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '$' => out.push_str("\\$"),
            '%' => out.push_str("\\%"),
            '&' => out.push_str("\\&"),
            '_' => out.push_str("\\_"),
            '#' => out.push_str("\\#"),
            '^' => out.push_str("\\^{}"),
            '[' => out.push_str("{[}"),
            ']' => out.push_str("{]}"),
            '~' => control_word(&mut out, "textasciitilde", next, mode),
            '\\' => control_word(&mut out, "textbackslash", next, mode),
            '<' => control_word(&mut out, "textless", next, mode),
            '>' => control_word(&mut out, "textgreater", next, mode),
            '|' => control_word(&mut out, "textbar", next, mode),
            '\'' => control_word(&mut out, "textquotesingle", next, mode),
            '`' if mode == Mode::Code => control_word(&mut out, "textasciigrave", next, mode),
            // Two hyphens would join into a dash.
            '-' if next == Some('-') => out.push_str("-\\/"),
            ' ' if mode == Mode::Code => out.push_str("\\ "),
            '\u{A0}' => out.push('~'),
            // Typographic characters have TeX spellings in text; code
            // keeps the characters themselves.
            '\u{2026}' if mode == Mode::Text => control_word(&mut out, "ldots", next, mode),
            '\u{2013}' if mode == Mode::Text => out.push_str("--"),
            '\u{2014}' if mode == Mode::Text => out.push_str("---"),
            '\u{2018}' if mode == Mode::Text => out.push('`'),
            '\u{2019}' if mode == Mode::Text => out.push('\''),
            '\u{201C}' if mode == Mode::Text => out.push_str("``"),
            '\u{201D}' if mode == Mode::Text => out.push_str("''"),
            _ => out.push(c),
        }
    }
    out
}

/// A URL inside `\href` and `\url`: the characters TeX would act on are
/// percent-encoded, as pandoc does, and `%` and `#` are escaped.
fn escape_url(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for c in url.chars() {
        match c {
            ' ' => out.push_str("\\%20"),
            '{' => out.push_str("\\%7B"),
            '}' => out.push_str("\\%7D"),
            '^' => out.push_str("\\%5E"),
            '|' => out.push_str("\\%7C"),
            '\\' => out.push('/'),
            '%' => out.push_str("\\%"),
            '#' => out.push_str("\\#"),
            _ => out.push(c),
        }
    }
    out
}

/// A header's identifier as a `\label`: characters outside ASCII are
/// written as `ux` and their code point, which every engine accepts.
fn label(id: &str) -> String {
    let mut out = String::new();
    for c in id.chars() {
        if c.is_ascii() {
            out.push(c);
        } else {
            let _ = write!(out, "ux{:x}", u32::from(c));
        }
    }
    out
}

fn command(name: &str, content: Doc) -> Doc {
    literal(&format!("\\{name}{{"))
        .append(content)
        .append(literal("}"))
}

pub fn inlines(xs: &[Inline]) -> Doc {
    let mut d = Doc::Empty;
    for (i, x) in xs.iter().enumerate() {
        d = d.append(inline(x));
        // A closing quote and an apostrophe after it would join.
        if matches!(x, Inline::Quoted(..))
            && matches!(xs.get(i + 1), Some(Inline::Str(s)) if s.starts_with(['\u{2019}', '\u{2018}']))
        {
            d = d.append(literal("\\,"));
        }
    }
    d
}

fn inline(x: &Inline) -> Doc {
    match x {
        Inline::Str(s) => literal(&escape(s, Mode::Text)),
        Inline::Space | Inline::SoftBreak => space(),
        Inline::LineBreak => literal("\\\\").append(cr()),
        Inline::Code(_, s) => command("texttt", literal(&escape(s, Mode::Code))),
        Inline::Emph(xs) => command("emph", inlines(xs)),
        Inline::Strong(xs) => command("textbf", inlines(xs)),
        Inline::Quoted(q, xs) => {
            let (open, close) = match q {
                QuoteType::Single => ("`", "'"),
                QuoteType::Double => ("``", "''"),
            };
            // Quotation marks that meet would join into the wrong ones;
            // a thin space keeps them apart.
            let starts = matches!(xs.first(), Some(Inline::Quoted(..)));
            let ends = matches!(xs.last(), Some(Inline::Quoted(..)));
            let mut d = literal(open);
            if starts {
                d = d.append(literal("\\,"));
            }
            d = d.append(inlines(xs));
            if ends {
                d = d.append(literal("\\,"));
            }
            d.append(literal(close))
        }
        Inline::Cite(_, xs) => inlines(xs),
        Inline::Link(_, xs, url, _) => link(xs, url),
        // Raw HTML has no place in LaTeX, except a `<span>`, which pandoc
        // reads as a group around its text.
        Inline::RawHtml(s) if is_tag(s, "<span") => literal("{"),
        Inline::RawHtml(s) if is_tag(s, "</span") => literal("}"),
        Inline::RawHtml(_) => Doc::Empty,
    }
}

/// Whether the raw HTML `s` is the tag that `open` starts (`<span`,
/// `</span`).
fn is_tag(s: &str, open: &str) -> bool {
    s.get(..open.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(open))
        && s[open.len()..].starts_with([' ', '>', '\t', '\n'])
}

/// Whether `xs` holds raw HTML at any depth.
fn has_raw(xs: &[Inline]) -> bool {
    xs.iter().any(|x| match x {
        Inline::RawHtml(_) => true,
        Inline::Emph(xs)
        | Inline::Strong(xs)
        | Inline::Quoted(_, xs)
        | Inline::Cite(_, xs)
        | Inline::Link(_, xs, _, _) => has_raw(xs),
        _ => false,
    })
}

fn link(xs: &[Inline], url: &str) -> Doc {
    if let Some(id) = url.strip_prefix('#') {
        return literal(&format!("\\hyperref[{id}]{{"))
            .append(inlines(xs))
            .append(literal("}"));
    }
    let text = match xs {
        [Inline::Str(s)] => Some(s.as_str()),
        _ => None,
    };
    if text == Some(url) {
        return literal(&format!("\\url{{{}}}", escape_url(url)));
    }
    let content = match (text, url.strip_prefix("mailto:")) {
        (Some(t), Some(address)) if t == address => {
            literal(&format!("\\nolinkurl{{{}}}", escape_url(address)))
        }
        _ => inlines(xs),
    };
    literal(&format!("\\href{{{}}}{{", escape_url(url)))
        .append(content)
        .append(literal("}"))
}

/// `xs` without formatting, as a PDF bookmark can show it: the second
/// argument of `\texorpdfstring`.
/// It is one piece of text: its spaces are not places to break a line.
fn plain_inlines(xs: &[Inline], out: &mut String) {
    for x in xs {
        match x {
            Inline::Str(s) | Inline::Code(_, s) => out.push_str(&escape(s, Mode::Text)),
            Inline::Space | Inline::SoftBreak | Inline::LineBreak => out.push(' '),
            Inline::Emph(xs)
            | Inline::Strong(xs)
            | Inline::Link(_, xs, _, _)
            | Inline::Cite(_, xs) => plain_inlines(xs, out),
            Inline::Quoted(q, xs) => {
                let (open, close) = match q {
                    QuoteType::Single => ("`", "'"),
                    QuoteType::Double => ("``", "''"),
                };
                out.push_str(open);
                plain_inlines(xs, out);
                out.push_str(close);
            }
            Inline::RawHtml(_) => {}
        }
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

/// Text inside a highlighted token: the `Highlighting` environment is a
/// verbatim one in which only `\`, `{` and `}` are special.
fn escape_token(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '%' => out.push_str("\\%"),
            '&' => out.push_str("\\&"),
            '_' => out.push_str("\\_"),
            '#' => out.push_str("\\#"),
            '^' => out.push_str("\\^{}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '<' => out.push_str("\\textless{}"),
            '>' => out.push_str("\\textgreater{}"),
            '\'' => out.push_str("\\textquotesingle{}"),
            '`' => out.push_str("\\textasciigrave{}"),
            '-' => out.push_str("{-}"),
            _ => out.push(c),
        }
    }
    out
}

/// `\NameTok{text}`; whitespace alone is written bare.
fn token(out: &mut String, name: &str, text: &str) {
    if text.chars().all(char::is_whitespace) {
        out.push_str(text);
    } else {
        let _ = write!(out, "\\{name}Tok{{{}}}", escape_token(text));
    }
}

/// The definitions the highlighted code needs in the preamble: the
/// `Shaded` and `Highlighting` environments (as skylighting's LaTeX
/// formatter defines them, BSD-3-Clause) and a command per token type in
/// the `pygments` style's colours.
fn highlighting_macros() -> String {
    let mut out = String::from(
        "\\usepackage{color}\n\\usepackage{fancyvrb}\n\\newcommand{\\VerbBar}{|}\n\\newcommand{\\VERB}{\\Verb[commandchars=\\\\\\{\\}]}\n\\DefineVerbatimEnvironment{Highlighting}{Verbatim}{commandchars=\\\\\\{\\}}\n% Add ',fontsize=\\small' for more characters per line\n\\newenvironment{Shaded}{}{}\n",
    );
    // Plain text and region markers are token types too, with no look
    // of their own.
    let mut commands = vec![
        ("Normal", "#1".to_owned()),
        ("RegionMarker", "#1".to_owned()),
    ];
    for t in highlight::TOKEN_TYPES {
        let mut body = "#1".to_owned();
        if t.italic {
            body = format!("\\textit{{{body}}}");
        }
        if t.bold {
            body = format!("\\textbf{{{body}}}");
        }
        if let Some(rgb) = t.color.and_then(rgb_fractions) {
            body = format!("\\textcolor[rgb]{{{rgb}}}{{{body}}}");
        }
        commands.push((t.name, body));
    }
    commands.sort_by_key(|(name, _)| *name);
    for (i, (name, body)) in commands.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let _ = write!(out, "\\newcommand{{\\{name}Tok}}[1]{{{body}}}");
    }
    out
}

/// `#rrggbb` as the `r,g,b` fractions `\textcolor[rgb]` takes.
fn rgb_fractions(hex: &str) -> Option<String> {
    let hex = hex.strip_prefix('#')?;
    let part = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    let (r, g, b) = (part(0)?, part(2)?, part(4)?);
    let f = |v: u8| format!("{:.2}", f64::from(v) / 255.0);
    Some(format!("{},{},{}", f(r), f(g), f(b)))
}

/// Writes blocks; it tracks how deep in ordered lists it is, to name their
/// counters, and what the document used that the preamble must load.
pub struct Writer {
    /// Whether to highlight code blocks that name a language.
    highlight: bool,
    /// Ordered lists around the block being written.
    ordered_depth: usize,
    /// Whether a code block was highlighted: the preamble needs the
    /// highlighting definitions.
    highlighted: bool,
    /// Whether a table was written: the preamble needs longtable.
    tables: bool,
}

impl Writer {
    pub const fn new(highlight: bool) -> Self {
        Self {
            highlight,
            ordered_depth: 0,
            highlighted: false,
            tables: false,
        }
    }

    pub fn blocks(&mut self, bs: &[Block]) -> Result<Doc> {
        let mut d = Doc::Empty;
        for b in bs {
            let one = self.block(b)?;
            if one.is_empty() {
                continue;
            }
            if !d.is_empty() {
                d = d.append(blankline());
            }
            d = d.append(one);
        }
        Ok(d)
    }

    /// The items of a list. Only a bullet list has task items (`tasks`):
    /// a numbered item keeps its number and shows the box as text.
    fn list_items(&mut self, items: &[Vec<Block>], tasks: bool) -> Result<Doc> {
        let mut d = Doc::Empty;
        for item in items {
            let (marker, content) = match task(item).filter(|_| tasks) {
                Some((checked, first, rest)) => {
                    let marker = if checked {
                        "\\item[$\\boxtimes$]"
                    } else {
                        "\\item[$\\square$]"
                    };
                    let mut content = inlines(first);
                    let more = self.blocks(rest)?;
                    if !more.is_empty() {
                        content = content.append(blankline()).append(more);
                    }
                    (marker, content)
                }
                None => ("\\item", self.blocks(item)?),
            };
            d = d
                .append(cr())
                .append(literal(marker))
                .append(cr())
                .append(nest(2, layout::chomp(content)));
        }
        Ok(d)
    }

    fn block(&mut self, b: &Block) -> Result<Doc> {
        Ok(match b {
            Block::Plain(xs) | Block::Para(xs) => inlines(xs),
            Block::Header(level, attr, xs) => {
                if attr.classes.iter().any(|c| c == "unnumbered") {
                    bail!("unnumbered headers are not supported in LaTeX output yet");
                }
                let name = match level {
                    1 => "section",
                    2 => "subsection",
                    3 => "subsubsection",
                    4 => "paragraph",
                    5 => "subparagraph",
                    // The article class has no deeper heading.
                    _ => return Ok(inlines(xs)),
                };
                let full = inlines(xs);
                // The bookmark text is one unbreakable piece.
                let mut plain = String::new();
                plain_inlines(xs, &mut plain);
                let content = if !has_raw(xs) && layout::render(None, &full) == plain {
                    full
                } else {
                    command("texorpdfstring", full)
                        .append(literal("{"))
                        .append(literal(&plain))
                        .append(literal("}"))
                };
                let mut d = command(name, content);
                if !attr.id.is_empty() {
                    d = d.append(literal(&format!("\\label{{{}}}", label(&attr.id))));
                }
                d
            }
            Block::BulletList(items) => {
                let mut d = literal("\\begin{itemize}");
                if is_tight(items) {
                    d = d.append(cr()).append(literal("\\tightlist"));
                }
                d.append(self.list_items(items, true)?)
                    .append(cr())
                    .append(literal("\\end{itemize}"))
            }
            Block::OrderedList(la, items) => {
                self.ordered_depth += 1;
                let result = self.ordered_list(la, items);
                self.ordered_depth -= 1;
                result?
            }
            Block::Code(attr, code) => flush(self.code_block(&attr.classes, code)),
            Block::HorizontalRule => {
                literal("\\begin{center}\\rule{0.5\\linewidth}{0.5pt}\\end{center}")
            }
            Block::Table(t) => {
                self.tables = true;
                table(t)
            }
            Block::Quote(bs) => literal("\\begin{quote}")
                .append(cr())
                .append(self.blocks(bs)?)
                .append(cr())
                .append(literal("\\end{quote}")),
            Block::RawHtml(_) => Doc::Empty,
            Block::RawLatex(s) => literal(s),
        })
    }

    fn ordered_list(&mut self, la: &ListAttributes, items: &[Vec<Block>]) -> Result<Doc> {
        let counter = match self.ordered_depth {
            1 => "enumi",
            2 => "enumii",
            3 => "enumiii",
            4 => "enumiv",
            _ => bail!("ordered lists nested more than four deep are not supported in LaTeX"),
        };
        let mut d = literal("\\begin{enumerate}");
        if la.style != ListNumberStyle::DefaultStyle || la.delim != ListNumberDelim::DefaultDelim {
            let number = match la.style {
                ListNumberStyle::DefaultStyle | ListNumberStyle::Decimal => "arabic",
                ListNumberStyle::LowerAlpha => "alph",
                ListNumberStyle::UpperAlpha => "Alph",
                ListNumberStyle::LowerRoman => "roman",
                ListNumberStyle::UpperRoman => "Roman",
                ListNumberStyle::Example => bail!("example lists are not supported yet"),
            };
            let number = format!("\\{number}{{{counter}}}");
            let shown = match la.delim {
                ListNumberDelim::DefaultDelim | ListNumberDelim::Period => format!("{number}."),
                ListNumberDelim::OneParen => format!("{number})"),
                ListNumberDelim::TwoParens => format!("({number})"),
            };
            d = d
                .append(cr())
                .append(literal(&format!("\\def\\label{counter}{{{shown}}}")));
        }
        if la.start != 1 {
            d = d.append(cr()).append(literal(&format!(
                "\\setcounter{{{counter}}}{{{}}}",
                la.start - 1
            )));
        }
        if is_tight(items) {
            d = d.append(cr()).append(literal("\\tightlist"));
        }
        Ok(d.append(self.list_items(items, false)?)
            .append(cr())
            .append(literal("\\end{enumerate}")))
    }

    /// A code block: highlighted when it names a language (as plain
    /// tokens when there is no syntax for it), otherwise verbatim.
    fn code_block(&mut self, classes: &[String], code: &str) -> Doc {
        let Some(lang) = classes.first().filter(|_| self.highlight) else {
            // The code between soft breaks: empty code leaves no line,
            // and a final empty line of the code is not shown.
            return literal("\\begin{verbatim}")
                .append(cr())
                .append(literal(code))
                .append(cr())
                .append(literal("\\end{verbatim}"));
        };
        self.highlighted = true;
        let mut out = String::from("\\begin{Shaded}\n\\begin{Highlighting}[]\n");
        match highlight::highlight(lang, code) {
            Some(h) => {
                for line in &h.lines {
                    for (t, text) in line {
                        token(&mut out, t.map_or("Normal", |t| t.name), text);
                    }
                    out.push('\n');
                }
            }
            None => {
                for line in code.lines() {
                    token(&mut out, "Normal", line);
                    out.push('\n');
                }
            }
        }
        if code.is_empty() {
            out.push('\n');
        }
        // pandoc ends a highlighted block, and only that, with a line
        // break of its own: it shows as one more empty line when the
        // block is the last thing in the document.
        out.push_str("\\end{Highlighting}\n\\end{Shaded}\n");
        literal(&out)
    }
}

/// A list is tight when no item's first block is a paragraph.
fn is_tight(items: &[Vec<Block>]) -> bool {
    items
        .iter()
        .all(|item| !matches!(item.first(), Some(Block::Para(_))))
}

/// A table, as pandoc's LaTeX writer lays one out: a `longtable` whose
/// columns are `l`, `r` or `c`, or paragraphs of relative width when the
/// table has widths, in which case the header cells are minipages.
fn table(t: &Table) -> Doc {
    let fractions = t.width_fractions();
    let mut d = match &fractions {
        None => {
            let spec: String = t
                .aligns
                .iter()
                .map(|a| match a {
                    Alignment::Default | Alignment::Left => 'l',
                    Alignment::Right => 'r',
                    Alignment::Center => 'c',
                })
                .collect();
            literal(&format!("\\begin{{longtable}}[]{{@{{}}{spec}@{{}}}}"))
        }
        Some(fs) => {
            let seps = 2 * fs.len().saturating_sub(1);
            let mut spec = String::from("\\begin{longtable}[]{@{}");
            for (f, a) in fs.iter().zip(&t.aligns) {
                let _ = write!(
                    spec,
                    "\n  >{{{}\\arraybackslash}}p{{(\\linewidth - {seps}\\tabcolsep) * \\real{{{f:.4}}}}}",
                    align_command(*a)
                );
            }
            spec.push_str("@{}}");
            literal(&spec)
        }
    };
    let row = |cells: &[Vec<Inline>], minipages: bool| {
        // Cells and the `&` between them, a space apart; an empty cell
        // takes no space of its own.
        let mut r = Doc::Empty;
        let mut push = |part: Doc| {
            if part.is_empty() {
                return;
            }
            let so_far = std::mem::replace(&mut r, Doc::Empty);
            r = if so_far.is_empty() {
                part
            } else {
                so_far.append(space()).append(part)
            };
        };
        for (i, (cell, align)) in cells.iter().zip(&t.aligns).enumerate() {
            if i > 0 {
                push(literal("&"));
            }
            push(if minipages {
                literal(&format!(
                    "\\begin{{minipage}}[b]{{\\linewidth}}{}",
                    align_command(*align)
                ))
                .append(cr())
                .append(inlines(cell))
                .append(cr())
                .append(literal("\\end{minipage}"))
            } else {
                inlines(cell)
            });
        }
        r.append(literal(" \\\\"))
    };
    d = d.append(cr()).append(literal("\\toprule\\noalign{}"));
    if let Some(head) = &t.head {
        d = d
            .append(cr())
            .append(row(head, fractions.is_some()))
            .append(cr())
            .append(literal("\\midrule\\noalign{}"));
    }
    d = d
        .append(cr())
        .append(literal("\\endhead"))
        .append(cr())
        .append(literal("\\bottomrule\\noalign{}"))
        .append(cr())
        .append(literal("\\endlastfoot"));
    for r in &t.rows {
        d = d.append(cr()).append(row(r, false));
    }
    d.append(cr()).append(literal("\\end{longtable}"))
}

const fn align_command(a: Alignment) -> &'static str {
    match a {
        Alignment::Default | Alignment::Left => "\\raggedright",
        Alignment::Right => "\\raggedleft",
        Alignment::Center => "\\centering",
    }
}

/// pandoc's `default.latex` and its partials (dual-licensed
/// GPL-2.0-or-later or BSD-3-Clause; used here under BSD-3-Clause).
const DEFAULT_LATEX: &str = include_str!("../data/templates/default.latex");
const PARTIALS: &[(&str, &str)] = &[
    (
        "after-header-includes.latex",
        include_str!("../data/templates/after-header-includes.latex"),
    ),
    (
        "common.latex",
        include_str!("../data/templates/common.latex"),
    ),
    (
        "font-settings.latex",
        include_str!("../data/templates/font-settings.latex"),
    ),
    ("fonts.latex", include_str!("../data/templates/fonts.latex")),
    (
        "hypersetup.latex",
        include_str!("../data/templates/hypersetup.latex"),
    ),
    (
        "passoptions.latex",
        include_str!("../data/templates/passoptions.latex"),
    ),
];

/// Metadata fields the template renders in ways rspandoc has not been
/// checked against pandoc for; documents that set one are refused rather
/// than rendered wrong.
const UNSUPPORTED_META: &[&str] = &[
    "abstract",
    "author",
    "bibliography",
    "date",
    "include-after",
    "include-before",
    "keywords",
    "subject",
    "subtitle",
    "thanks",
    "toc",
];

/// A document language: its tag, the name babel knows it by, and whether
/// babel has to load it on the fly (`provide=*`), as pandoc asks for the
/// languages babel has no classic definition file for.
const LANGUAGES: &[(&str, &str, bool)] = &[
    ("ar", "arabic", true),
    ("de", "ngerman", false),
    ("el", "greek", false),
    ("en", "english", false),
    ("en-GB", "british", false),
    ("en-US", "american", false),
    ("es", "spanish", false),
    ("fr", "french", false),
    ("he", "hebrew", true),
    ("it", "italian", false),
    ("ja", "japanese", true),
    ("nl", "dutch", false),
    ("pl", "polish", false),
    ("pt", "portuguese", false),
    ("pt-BR", "brazilian", false),
    ("ru", "russian", true),
    ("sv", "swedish", false),
    ("tr", "turkish", false),
    ("zh", "chinese", true),
];

fn meta_val(v: &MetaValue) -> Result<Val> {
    Ok(match v {
        MetaValue::Inlines(xs) => Val::Simple(inlines(xs)),
        MetaValue::Blocks(bs) => Val::Simple(Writer::new(false).blocks(bs)?),
        MetaValue::List(xs) => Val::List(xs.iter().map(meta_val).collect::<Result<_>>()?),
        MetaValue::Bool(b) => Val::Bool(*b),
    })
}

/// Options that shape the document.
pub struct Options {
    /// Whether to highlight code blocks (no `--no-highlight`).
    pub highlight: bool,
    /// Template variables from `-V`, in order; a repeated key is a list.
    pub variables: Vec<(String, String)>,
}

/// The body alone, as `--to latex` without `--standalone` writes it.
pub fn fragment(doc: &Document, highlight: bool) -> Result<String> {
    let body = Writer::new(highlight).blocks(&doc.blocks)?;
    Ok(layout::render(Some(COLUMNS), &body)
        .trim_end_matches('\n')
        .to_owned())
}

/// The standalone LaTeX document for `doc`.
pub fn standalone(doc: &Document, opts: &Options) -> Result<String> {
    crate::html::check_meta(doc, UNSUPPORTED_META)?;
    let mut ctx = Context::new();
    for (k, v) in &doc.meta {
        ctx.insert(k.clone(), meta_val(v)?);
    }
    match doc.meta("title") {
        Some(MetaValue::Inlines(xs)) => {
            let plain = escape(&stringify(xs), Mode::Text);
            ctx.insert("title-meta".to_owned(), Val::Simple(literal(&plain)));
        }
        Some(_) => bail!("metadata field `title` must be text"),
        None => {}
    }
    if let Some(lang) = doc.meta("lang") {
        let MetaValue::Inlines(xs) = lang else {
            bail!("metadata field `lang` must be text");
        };
        let tag = stringify(xs);
        let Some((_, babel, provide)) = LANGUAGES.iter().find(|(t, _, _)| *t == tag) else {
            bail!("language `{tag}` is not supported in LaTeX output yet");
        };
        ctx.insert("babel-lang".to_owned(), Val::Simple(literal(babel)));
        if *provide {
            ctx.insert(
                "babeloptions".to_owned(),
                Val::List(vec![Val::Simple(literal("provide=*"))]),
            );
        }
    }
    ctx.entry("documentclass".to_owned())
        .or_insert_with(|| Val::Simple(literal("article")));
    let mut w = Writer::new(opts.highlight);
    ctx.insert("body".to_owned(), Val::Simple(w.blocks(&doc.blocks)?));
    if w.highlighted {
        ctx.insert(
            "highlighting-macros".to_owned(),
            Val::Simple(literal(&highlighting_macros())),
        );
    }
    if w.tables {
        ctx.insert("tables".to_owned(), Val::Bool(true));
    }
    let mut variables: Vec<(String, Vec<Val>)> = Vec::new();
    for (k, v) in &opts.variables {
        let val = Val::Simple(literal(v));
        match variables.iter_mut().find(|(name, _)| name == k) {
            Some((_, vals)) => vals.push(val),
            None => variables.push((k.clone(), vec![val])),
        }
    }
    for (k, mut vals) in variables {
        let val = if vals.len() == 1 {
            vals.remove(0)
        } else {
            Val::List(vals)
        };
        ctx.insert(k, val);
    }
    let partials = |name: &str| {
        PARTIALS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, text)| (*text).to_owned())
    };
    let tpl = template::compile(DEFAULT_LATEX, "default.latex", &partials)?;
    Ok(layout::render(Some(COLUMNS), &template::render(&tpl, &ctx)))
}
