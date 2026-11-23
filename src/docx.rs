//! The DOCX writer. The document maps onto the same style names pandoc's
//! DOCX writer uses (Title, Heading1-6, `FirstParagraph`, `BodyText`, Compact,
//! `BlockText`, `SourceCode`, `VerbatimChar`, Hyperlink), so a document styled
//! for pandoc's output behaves the same here. The style sheet, numbering and package
//! parts are rspandoc's own, written from the Office Open XML specification;
//! they are not pandoc's reference document, so the output is not
//! byte-identical to pandoc's, only equivalent in structure.
//!
//! The package is deterministic: entries are stored in a fixed order with
//! a fixed timestamp, so the same input always gives the same bytes.

use std::fmt::Write as _;
use std::io::{Cursor, Write as _};

use anyhow::{Context as _, Result, bail};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use crate::ast::{
    Alignment, Attr, Block, Document, Inline, ListAttributes, ListNumberDelim, ListNumberStyle,
    MetaValue, QuoteType, Table, count_to_f64, stringify, trunc_to_u64,
};
use crate::generator::Mark;
use crate::highlight;
use crate::html::check_meta;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const V_NS: &str = "urn:schemas-microsoft-com:vml";
const O_NS: &str = "urn:schemas-microsoft-com:office:office";
const REL_HYPERLINK: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";

/// The text width, in twentieths of a point, that table grids divide.
const TEXT_WIDTH: u16 = 7920;

/// Twentieths of a point a list level indents by.
const LEVEL_INDENT: usize = 720;

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Metadata the DOCX writer would have to render and does not yet.
const UNSUPPORTED_META: &[&str] = &[
    "abstract",
    "author",
    "date",
    "description",
    "keywords",
    "subtitle",
    "toc",
];

/// Languages written right to left, which Word records as the document's
/// complex-script (`bidi`) language.
const RTL_LANGUAGES: &[&str] = &["ar", "dv", "fa", "he", "ps", "sd", "ug", "ur", "yi"];

/// Run formatting in force.
#[derive(Clone, Copy, Default)]
struct RunFormat {
    bold: bool,
    italic: bool,
    face: Face,
    rtl: bool,
}

/// The character style a run takes; a code style wins over a link style.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Face {
    #[default]
    Plain,
    Link,
    Code,
}

impl RunFormat {
    fn properties(self) -> String {
        let mut p = String::new();
        match self.face {
            Face::Code => p.push_str(r#"<w:rStyle w:val="VerbatimChar"/>"#),
            Face::Link => p.push_str(r#"<w:rStyle w:val="Hyperlink"/>"#),
            Face::Plain => {}
        }
        if self.bold {
            p.push_str("<w:b/><w:bCs/>");
        }
        if self.italic {
            p.push_str("<w:i/><w:iCs/>");
        }
        if self.rtl {
            p.push_str("<w:rtl/>");
        }
        if p.is_empty() {
            p
        } else {
            format!("<w:rPr>{p}</w:rPr>")
        }
    }
}

struct Writer {
    body: String,
    /// The document runs right to left (`dir: rtl`).
    rtl: bool,
    /// Lists written so far, `None` for a bullet list; each gets its own
    /// numbering instance, so adjacent lists stay separate.
    lists: Vec<Option<ListAttributes>>,
    /// External link targets, in relationship order.
    links: Vec<String>,
    bookmark: usize,
    /// The document has a horizontal rule, drawn with VML.
    vml: bool,
    /// Whether to highlight code blocks (no `--no-highlight`).
    highlight: bool,
}

impl Writer {
    fn text_run(&mut self, text: &str, f: RunFormat) {
        let _ = write!(
            self.body,
            r#"<w:r>{}<w:t xml:space="preserve">{}</w:t></w:r>"#,
            f.properties(),
            esc(text)
        );
    }

    fn inlines(&mut self, xs: &[Inline], f: RunFormat) {
        for x in xs {
            self.inline(x, f);
        }
    }

    fn inline(&mut self, x: &Inline, f: RunFormat) {
        match x {
            Inline::Str(s) => self.text_run(s, f),
            Inline::Space | Inline::SoftBreak => self.text_run(" ", f),
            Inline::LineBreak => self.body.push_str("<w:r><w:br/></w:r>"),
            Inline::Code(_, s) => self.text_run(
                s,
                RunFormat {
                    face: Face::Code,
                    ..f
                },
            ),
            Inline::Emph(xs) => self.inlines(xs, RunFormat { italic: true, ..f }),
            Inline::Strong(xs) => self.inlines(xs, RunFormat { bold: true, ..f }),
            Inline::Quoted(q, xs) => {
                let (open, close) = match q {
                    QuoteType::Single => ("\u{2018}", "\u{2019}"),
                    QuoteType::Double => ("\u{201C}", "\u{201D}"),
                };
                self.text_run(open, f);
                self.inlines(xs, f);
                self.text_run(close, f);
            }
            Inline::Cite(_, xs) => self.inlines(xs, f),
            Inline::Link(_, xs, url, _) => {
                if let Some(anchor) = url.strip_prefix('#') {
                    let _ = write!(self.body, r#"<w:hyperlink w:anchor="{}">"#, esc(anchor));
                } else {
                    self.links.push(url.clone());
                    let _ = write!(
                        self.body,
                        r#"<w:hyperlink r:id="{}">"#,
                        link_rel_id(self.links.len())
                    );
                }
                self.inlines(
                    xs,
                    RunFormat {
                        face: if f.face == Face::Plain {
                            Face::Link
                        } else {
                            f.face
                        },
                        ..f
                    },
                );
                self.body.push_str("</w:hyperlink>");
            }
            // Raw HTML has no meaning in a Word document; pandoc drops it too.
            Inline::RawHtml(_) => {}
        }
    }

    /// A paragraph with a style, optional list numbering or indentation,
    /// and an optional bookmark around its content.
    fn paragraph(
        &mut self,
        style: &str,
        list: Option<ListPos>,
        bookmark: Option<&str>,
        xs: &[Inline],
    ) {
        self.paragraph_properties(style, list);
        let id = self.bookmark;
        if let Some(name) = bookmark {
            let _ = write!(
                self.body,
                r#"<w:bookmarkStart w:id="{id}" w:name="{}"/>"#,
                esc(name)
            );
            self.bookmark += 1;
        }
        self.inlines(
            xs,
            RunFormat {
                rtl: self.rtl,
                ..RunFormat::default()
            },
        );
        if bookmark.is_some() {
            let _ = write!(self.body, r#"<w:bookmarkEnd w:id="{id}"/>"#);
        }
        self.body.push_str("</w:p>");
    }

    /// A code block: one paragraph, its lines separated by breaks.
    /// Highlighted when its first class names a language there is a
    /// syntax for, each token a run in its token style.
    fn code_paragraph(&mut self, list: Option<ListPos>, attr: &Attr, code: &str) {
        self.paragraph_properties("SourceCode", list);
        let highlighted = attr
            .classes
            .first()
            .filter(|_| self.highlight)
            .and_then(|lang| highlight::highlight(lang, code));
        if let Some(h) = highlighted {
            for (i, line) in h.lines.iter().enumerate() {
                if i > 0 {
                    self.body.push_str("<w:r><w:br/></w:r>");
                }
                for (t, text) in line {
                    let style = t.map_or("Normal", |t| t.name);
                    let _ = write!(
                        self.body,
                        r#"<w:r><w:rPr><w:rStyle w:val="{style}Tok"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
                        esc(text)
                    );
                }
            }
        } else {
            let f = RunFormat {
                face: Face::Code,
                ..RunFormat::default()
            };
            for (i, line) in code.split('\n').enumerate() {
                if i > 0 {
                    self.body.push_str("<w:r><w:br/></w:r>");
                }
                self.text_run(line, f);
            }
        }
        self.body.push_str("</w:p>");
    }

    /// Open a paragraph and write its properties.
    fn paragraph_properties(&mut self, style: &str, list: Option<ListPos>) {
        self.body.push_str("<w:p><w:pPr>");
        let _ = write!(self.body, r#"<w:pStyle w:val="{style}"/>"#);
        if let Some(ListPos {
            level,
            first: true,
            num,
        }) = list
        {
            let _ = write!(
                self.body,
                r#"<w:numPr><w:ilvl w:val="{level}"/><w:numId w:val="{num}"/></w:numPr>"#
            );
        }
        if self.rtl {
            self.body.push_str("<w:bidi/>");
        }
        // Later paragraphs of an item are indented to its text, outside
        // the numbering (the schema puts `ind` after `bidi`).
        if let Some(ListPos {
            level,
            first: false,
            ..
        }) = list
        {
            let _ = write!(
                self.body,
                r#"<w:ind w:left="{}" w:firstLine="0"/>"#,
                720 * (level + 1)
            );
        }
        self.body.push_str("</w:pPr>");
    }

    /// A table in the `Table` style. Its grid gives the columns pandoc's
    /// widths (equal ones when the table has none), so the DOCX reader
    /// gets the same column widths back; a table with widths spans the
    /// page, one without sizes to its content.
    fn table(&mut self, t: &Table) {
        let fractions = t.width_fractions();
        let n = t.aligns.len().max(1);
        let equal = vec![1.0 / count_to_f64(n); n];
        let (width, grid) = fractions.as_ref().map_or(
            (r#"<w:tblW w:type="auto" w:w="0"/>"#, equal.as_slice()),
            |f| {
                (
                    r#"<w:tblW w:type="pct" w:w="5000"/><w:tblLayout w:type="fixed"/>"#,
                    f.as_slice(),
                )
            },
        );
        let (first_row, look) = if t.head.is_some() {
            (1, "0020")
        } else {
            (0, "0000")
        };
        let _ = write!(
            self.body,
            r#"<w:tbl><w:tblPr><w:tblStyle w:val="Table"/>{width}<w:tblLook w:firstRow="{first_row}" w:lastRow="0" w:firstColumn="0" w:lastColumn="0" w:noHBand="0" w:noVBand="0" w:val="{look}"/></w:tblPr><w:tblGrid>"#
        );
        for f in grid {
            let w = u32::try_from(trunc_to_u64(f64::from(TEXT_WIDTH) * f)).unwrap_or(u32::MAX);
            let _ = write!(self.body, r#"<w:gridCol w:w="{w}"/>"#);
        }
        self.body.push_str("</w:tblGrid>");
        if let Some(head) = &t.head {
            self.table_row(head, &t.aligns, true);
        }
        for row in &t.rows {
            self.table_row(row, &t.aligns, false);
        }
        self.body.push_str("</w:tbl>");
    }

    fn table_row(&mut self, cells: &[Vec<Inline>], aligns: &[Alignment], header: bool) {
        self.body.push_str("<w:tr>");
        if header {
            self.body
                .push_str(r#"<w:trPr><w:tblHeader w:val="on"/></w:trPr>"#);
        }
        for (cell, align) in cells.iter().zip(aligns) {
            self.body
                .push_str(r#"<w:tc><w:tcPr/><w:p><w:pPr><w:pStyle w:val="Compact"/>"#);
            if self.rtl {
                self.body.push_str("<w:bidi/>");
            }
            let jc = match align {
                Alignment::Default => None,
                Alignment::Left => Some("left"),
                Alignment::Center => Some("center"),
                Alignment::Right => Some("right"),
            };
            // An empty cell has no alignment, as in pandoc's.
            if let Some(jc) = jc.filter(|_| !cell.is_empty()) {
                let _ = write!(self.body, r#"<w:jc w:val="{jc}"/>"#);
            }
            self.body.push_str("</w:pPr>");
            self.inlines(
                cell,
                RunFormat {
                    rtl: self.rtl,
                    ..RunFormat::default()
                },
            );
            self.body.push_str("</w:p></w:tc>");
        }
        self.body.push_str("</w:tr>");
    }

    fn blocks(&mut self, bs: &[Block], ctx: Ctx) {
        let mut prev_para = false;
        for (i, b) in bs.iter().enumerate() {
            let list = ctx.level.map(|level| ListPos {
                level,
                first: i == 0 && ctx.item_start,
                num: ctx.num,
            });
            match b {
                Block::Plain(xs) => self.paragraph(
                    if ctx.level.is_some() {
                        "Compact"
                    } else {
                        ctx.para_style("BodyText")
                    },
                    list,
                    None,
                    xs,
                ),
                Block::Para(xs) => {
                    let style = if prev_para {
                        "BodyText"
                    } else {
                        "FirstParagraph"
                    };
                    self.paragraph(ctx.para_style(style), list, None, xs);
                }
                Block::Header(level, attr, xs) => {
                    let style = format!("Heading{level}");
                    let mark = (!attr.id.is_empty()).then_some(attr.id.as_str());
                    self.paragraph(&style, list, mark, xs);
                }
                Block::BulletList(items) | Block::OrderedList(_, items) => {
                    let level = ctx.level.map_or(0, |l| l + 1);
                    self.lists.push(match b {
                        Block::OrderedList(la, _) => Some(*la),
                        _ => None,
                    });
                    let num = CONTINUATION_NUM + self.lists.len();
                    for item in items {
                        self.blocks(
                            item,
                            Ctx {
                                level: Some(level),
                                item_start: true,
                                quote: ctx.quote,
                                num,
                            },
                        );
                    }
                }
                Block::Quote(inner) => self.blocks(
                    inner,
                    Ctx {
                        quote: true,
                        item_start: false,
                        ..ctx
                    },
                ),
                Block::Code(attr, code) => self.code_paragraph(list, attr, code),
                // Word's horizontal line: a VML rule as wide as the text.
                Block::HorizontalRule => {
                    self.vml = true;
                    self.body.push_str(
                        r#"<w:p><w:r><w:pict><v:rect style="width:0;height:1.5pt" o:hralign="center" o:hrstd="t" o:hr="t"/></w:pict></w:r></w:p>"#,
                    );
                }
                Block::Table(t) => self.table(t),
                Block::RawHtml(_) | Block::RawLatex(_) => {}
            }
            prev_para = matches!(b, Block::Para(_));
        }
    }
}

#[derive(Clone, Copy)]
struct ListPos {
    level: usize,
    /// The list's numbering instance.
    num: usize,
    /// The item's first paragraph, which carries the bullet.
    first: bool,
}

#[derive(Clone, Copy, Default)]
struct Ctx {
    level: Option<usize>,
    num: usize,
    item_start: bool,
    quote: bool,
}

impl Ctx {
    const fn para_style(self, style: &'static str) -> &'static str {
        if self.quote { "BlockText" } else { style }
    }
}

fn link_rel_id(n: usize) -> String {
    // rId1-rId3 are the style sheet, numbering and settings.
    format!("rId{}", n + 3)
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/><Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/></Types>
"#;

const PACKAGE_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>
"#;

const SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:defaultTabStop w:val="720"/><w:characterSpacingControl w:val="doNotCompress"/><w:compat><w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/></w:compat></w:settings>
"#;

/// rspandoc's style sheet.
const STYLES: &str = include_str!("../data/docx/styles.xml");

/// The style sheet, with the document's language (`lang`) as the default:
/// a right-to-left language as the complex-script language, any other as
/// the main one.
fn styles(doc: &Document, reference: Option<&Reference>) -> Result<String> {
    let styles = match reference {
        None => STYLES.replace("</w:styles>", &format!("{}</w:styles>", token_styles())),
        Some(r) => with_missing_styles(&r.styles)?,
    };
    let lang = match doc.meta("lang") {
        None => return Ok(styles),
        Some(MetaValue::Inlines(xs)) => stringify(xs),
        Some(_) => bail!("metadata field `lang` must be text"),
    };
    let primary = lang.split('-').next().unwrap_or_default();
    let attr = if RTL_LANGUAGES.contains(&primary) {
        "w:bidi"
    } else {
        "w:val"
    };
    set_default_language(&styles, attr, &lang)
}

/// `styles` with `attr` of the default run properties' `<w:lang>` set to
/// `lang`, adding the attribute or the element when it is not there.
fn set_default_language(styles: &str, attr: &str, lang: &str) -> Result<String> {
    let Some(defaults_end) = styles.find("</w:rPrDefault>") else {
        bail!("the style sheet has no default run properties to set the language in");
    };
    let value = format!(r#"{attr}="{}""#, esc(lang));
    let Some(at) = styles[..defaults_end].find("<w:lang") else {
        // No language yet: add one at the end of the run properties.
        let element = if attr == "w:val" {
            format!("<w:lang {value}/>")
        } else {
            format!(r#"<w:lang w:val="en-US" {value}/>"#)
        };
        let Some(rpr_end) = styles[..defaults_end].rfind("</w:rPr>") else {
            bail!("the style sheet's default run properties are empty");
        };
        return Ok(format!(
            "{}{element}{}",
            &styles[..rpr_end],
            &styles[rpr_end..]
        ));
    };
    let Some(len) = styles[at..].find("/>") else {
        bail!("the style sheet's <w:lang> element is not closed");
    };
    let element = &styles[at..at + len];
    let new_element = match element.find(&format!("{attr}=\"")) {
        Some(a) => {
            let from = a + attr.len() + 2;
            let Some(n) = element[from..].find('"') else {
                bail!("the style sheet's <w:lang> element is malformed");
            };
            format!("{}{value}{}", &element[..a], &element[from + n + 1..])
        }
        None => format!("{element} {value}"),
    };
    Ok(format!(
        "{}{new_element}{}",
        &styles[..at],
        &styles[at + len..]
    ))
}

/// What a reference document (`--reference-doc`) gives the output: its
/// style sheet, and the font table, theme and page setup when it has them.
pub struct Reference {
    styles: String,
    font_table: Option<String>,
    theme: Option<String>,
    section: Option<String>,
}

const FONT_TABLE_PART: &str = "word/fontTable.xml";
const THEME_PART: &str = "word/theme/theme1.xml";

/// Read the reference document at `path`.
pub fn read_reference(path: &std::path::Path) -> Result<Reference> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("could not open the reference document {}", path.display()))?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))
        .with_context(|| format!("{} is not a DOCX file", path.display()))?;
    let mut part = |name: &str| -> Result<Option<String>> {
        let mut entry = match zip.by_name(name) {
            Ok(e) => e,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("could not read {name} from {}", path.display()));
            }
        };
        let mut text = String::new();
        std::io::Read::read_to_string(&mut entry, &mut text)
            .with_context(|| format!("could not read {name} from {}", path.display()))?;
        Ok(Some(text))
    };
    let Some(styles) = part("word/styles.xml")? else {
        bail!(
            "the reference document {} has no style sheet",
            path.display()
        );
    };
    if !styles.contains("<w:styles") || !styles.contains("</w:styles>") {
        bail!(
            "the style sheet of {} is not in the form rspandoc reads (a <w:styles> element)",
            path.display()
        );
    }
    let section = match part("word/document.xml")? {
        Some(document) => section_properties(&document)?,
        None => None,
    };
    Ok(Reference {
        styles,
        font_table: part(FONT_TABLE_PART)?,
        theme: part(THEME_PART)?,
        section,
    })
}

/// The last `<w:sectPr>` of a document: its page size and margins. One
/// that points at headers or footers is refused, since those parts are
/// not carried over yet.
fn section_properties(document: &str) -> Result<Option<String>> {
    let Some(at) = document.rfind("<w:sectPr") else {
        return Ok(None);
    };
    let rest = &document[at..];
    let end = match (rest.find("/>"), rest.find("</w:sectPr>")) {
        // `<w:sectPr/>` or `<w:sectPr attr=".."/>`: closed before any child.
        (Some(short), _) if !rest[..short].contains('>') => short + 2,
        (_, Some(long)) => long + "</w:sectPr>".len(),
        _ => bail!("the reference document's <w:sectPr> is not closed"),
    };
    let section = &rest[..end];
    if section.contains("Reference") {
        bail!("headers and footers in the reference document are not supported yet");
    }
    Ok(Some(section.to_owned()))
}

/// The `<w:style>` elements of a style sheet, each with its style id.
fn style_elements(xml: &str) -> Result<Vec<(&str, &str)>> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find("<w:style ") {
        let Some(len) = rest[at..].find("</w:style>") else {
            bail!("a <w:style> element is not closed");
        };
        let element = &rest[at..at + len + "</w:style>".len()];
        let Some(id_at) = element.find("w:styleId=\"") else {
            bail!("a <w:style> element has no style id");
        };
        let id = &element[id_at + 11..];
        let Some(id_len) = id.find('"') else {
            bail!("a <w:style> element's style id is not closed");
        };
        out.push((&id[..id_len], element));
        rest = &rest[at + element.len()..];
    }
    Ok(out)
}

/// A reference document's style sheet with the styles rspandoc writes
/// that it does not define added from rspandoc's own, as pandoc adds its
/// own to a reference document that lacks them.
fn with_missing_styles(reference: &str) -> Result<String> {
    let defined: Vec<&str> = style_elements(reference)?
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    let tokens = token_styles();
    let mut added = String::new();
    for (id, element) in style_elements(STYLES)?
        .into_iter()
        .chain(style_elements(&tokens)?)
    {
        if !defined.contains(&id) {
            added.push_str(element);
        }
    }
    Ok(reference.replace("</w:styles>", &format!("{added}</w:styles>")))
}

/// A character style per token type of highlighted code (`KeywordTok`,
/// ...), coloured like the HTML stylesheet, and `NormalTok` for the text
/// between tokens.
fn token_styles() -> String {
    let mut out = String::from(
        r#"<w:style w:type="character" w:customStyle="1" w:styleId="NormalTok"><w:name w:val="NormalTok"/><w:basedOn w:val="VerbatimChar"/></w:style>"#,
    );
    for t in highlight::TOKEN_TYPES {
        let mut rpr = String::new();
        if t.bold {
            rpr.push_str("<w:b/><w:bCs/>");
        }
        if t.italic {
            rpr.push_str("<w:i/><w:iCs/>");
        }
        if let Some(c) = t.color {
            let _ = write!(
                rpr,
                r#"<w:color w:val="{}"/>"#,
                c.trim_start_matches('#').to_uppercase()
            );
        }
        let _ = write!(
            out,
            r#"<w:style w:type="character" w:customStyle="1" w:styleId="{n}Tok"><w:name w:val="{n}Tok"/><w:basedOn w:val="VerbatimChar"/><w:rPr>{rpr}</w:rPr></w:style>"#,
            n = t.name
        );
    }
    out.push('\n');
    out
}

/// The numbering instance of an item's later paragraphs; bullet lists
/// number from the one after it.
const CONTINUATION_NUM: usize = 1;

/// Word's number format and level text (for level `lvl`) of an ordered
/// list.
fn number_format(la: &ListAttributes, lvl: usize) -> (&'static str, String) {
    let fmt = match la.style {
        ListNumberStyle::DefaultStyle | ListNumberStyle::Example | ListNumberStyle::Decimal => {
            "decimal"
        }
        ListNumberStyle::LowerRoman => "lowerRoman",
        ListNumberStyle::UpperRoman => "upperRoman",
        ListNumberStyle::LowerAlpha => "lowerLetter",
        ListNumberStyle::UpperAlpha => "upperLetter",
    };
    let n = lvl + 1;
    let text = match la.delim {
        ListNumberDelim::DefaultDelim | ListNumberDelim::Period => format!("%{n}."),
        ListNumberDelim::OneParen => format!("%{n})"),
        ListNumberDelim::TwoParens => format!("(%{n})"),
    };
    (fmt, text)
}

/// Abstract numbering 0: bullets for nine levels, a disc, a circle and a
/// square, repeating; one instance per bullet list. Abstract numbering 1:
/// the same indents with a blank bullet, for an item's later paragraphs.
/// Each ordered list has an abstract numbering of its own, numbered in its
/// style from its first number at every level.
fn numbering(lists: &[Option<ListAttributes>]) -> String {
    let mut nums = String::new();
    let mut ordered = String::new();
    let mut next_abstract = 2;
    for (n, list) in lists.iter().enumerate() {
        let abstract_id = list.as_ref().map_or(0, |la| {
            let id = next_abstract;
            next_abstract += 1;
            let _ = write!(
                ordered,
                r#"<w:abstractNum w:abstractNumId="{id}"><w:multiLevelType w:val="multilevel"/>"#
            );
            for lvl in 0..9 {
                let (fmt, text) = number_format(la, lvl);
                let left = LEVEL_INDENT * (lvl + 1);
                let _ = write!(
                    ordered,
                    r#"<w:lvl w:ilvl="{lvl}"><w:start w:val="{}"/><w:numFmt w:val="{fmt}"/><w:lvlText w:val="{text}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="{left}" w:hanging="360"/></w:pPr></w:lvl>"#,
                    la.start
                );
            }
            ordered.push_str("</w:abstractNum>");
            id
        });
        let _ = write!(
            nums,
            r#"<w:num w:numId="{}"><w:abstractNumId w:val="{abstract_id}"/></w:num>"#,
            CONTINUATION_NUM + n + 1
        );
    }
    let mut bullets = String::new();
    let mut plain = String::new();
    for lvl in 0..9 {
        let bullet = ["\u{2022}", "\u{25E6}", "\u{25AA}"][lvl % 3];
        let left = LEVEL_INDENT * (lvl + 1);
        let _ = write!(
            bullets,
            r#"<w:lvl w:ilvl="{lvl}"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="{bullet}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="{left}" w:hanging="360"/></w:pPr></w:lvl>"#
        );
        let _ = write!(
            plain,
            r#"<w:lvl w:ilvl="{lvl}"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val=" "/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="{left}" w:firstLine="0"/></w:pPr></w:lvl>"#
        );
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="hybridMultilevel"/>{bullets}</w:abstractNum><w:abstractNum w:abstractNumId="1"><w:multiLevelType w:val="hybridMultilevel"/>{plain}</w:abstractNum>{ordered}<w:num w:numId="{CONTINUATION_NUM}"><w:abstractNumId w:val="1"/></w:num>{nums}</w:numbering>
"#
    )
}

fn document_rels(links: &[String], reference: Option<&Reference>) -> String {
    let mut rels = String::from(
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>"#,
    );
    if reference.is_some_and(|r| r.font_table.is_some()) {
        rels.push_str(r#"<Relationship Id="rIdFontTable" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/>"#);
    }
    if reference.is_some_and(|r| r.theme.is_some()) {
        rels.push_str(r#"<Relationship Id="rIdTheme" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>"#);
    }
    for (i, url) in links.iter().enumerate() {
        let _ = write!(
            rels,
            r#"<Relationship Id="{}" Type="{REL_HYPERLINK}" Target="{}" TargetMode="External"/>"#,
            link_rel_id(i + 1),
            esc(url)
        );
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{rels}</Relationships>
"#
    )
}

/// The core properties: the title and, when the mark has them, who ran
/// the conversion (the one who last modified the file) and when (its
/// creation).
fn core_properties(title: Option<&str>, mark: &Mark) -> String {
    let mut props = title.map_or(String::new(), |t| {
        format!("<dc:title>{}</dc:title>", esc(t))
    });
    if let Some(by) = &mark.by {
        let _ = write!(props, "<cp:lastModifiedBy>{}</cp:lastModifiedBy>", esc(by));
    }
    let mut namespaces = "";
    if let Some(date) = &mark.date {
        namespaces = r#" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#;
        let _ = write!(
            props,
            r#"<dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>"#,
            esc(date)
        );
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"{namespaces}>{props}</cp:coreProperties>
"#
    )
}

/// The extended properties: the application that made the file, which is
/// where a DOCX file keeps that (Word writes `Microsoft Office Word`).
fn app_properties(application: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>{}</Application></Properties>
"#,
        esc(application)
    )
}

/// The package's relationships, with the extended properties when the
/// package has them.
fn package_rels(app: bool) -> String {
    if !app {
        return PACKAGE_RELS.to_owned();
    }
    PACKAGE_RELS.replace(
        "</Relationships>",
        r#"<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#,
    )
}

/// The package's content types, with those of the extended properties
/// and of the parts a reference document brought.
fn content_types(reference: Option<&Reference>, app: bool) -> String {
    let mut extra = String::new();
    if app {
        extra.push_str(r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#);
    }
    if reference.is_some_and(|r| r.font_table.is_some()) {
        extra.push_str(r#"<Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/>"#);
    }
    if reference.is_some_and(|r| r.theme.is_some()) {
        extra.push_str(r#"<Override PartName="/word/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>"#);
    }
    CONTENT_TYPES.replace("</Types>", &format!("{extra}</Types>"))
}

/// The DOCX package for `doc`, styled by `reference` when there is one
/// and saying what `mark` says about how it was made.
pub fn write(
    doc: &Document,
    highlight: bool,
    reference: Option<&Reference>,
    mark: &Mark,
) -> Result<Vec<u8>> {
    check_meta(doc, UNSUPPORTED_META)?;
    let rtl = match doc.meta("dir") {
        None => false,
        Some(MetaValue::Inlines(xs)) => match stringify(xs).as_str() {
            "rtl" => true,
            "ltr" => false,
            other => bail!("metadata field `dir` must be rtl or ltr, not {other:?}"),
        },
        Some(_) => bail!("metadata field `dir` must be text"),
    };
    let mut w = Writer {
        body: String::new(),
        rtl,
        lists: Vec::new(),
        links: Vec::new(),
        bookmark: 0,
        vml: false,
        highlight,
    };
    let title = match doc.meta("title") {
        Some(MetaValue::Inlines(xs)) => {
            w.paragraph("Title", None, None, xs);
            Some(stringify(xs))
        }
        Some(_) => bail!("metadata field `title` must be text"),
        None => None,
    };
    w.blocks(&doc.blocks, Ctx::default());
    let vml = if w.vml {
        format!(r#" xmlns:v="{V_NS}" xmlns:o="{O_NS}""#)
    } else {
        String::new()
    };
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="{W_NS}" xmlns:r="{R_NS}"{vml}><w:body>{}{}</w:body></w:document>
"#,
        w.body,
        reference
            .and_then(|r| r.section.as_deref())
            .unwrap_or("<w:sectPr/>")
    );
    let mut parts: Vec<(&str, String)> = vec![
        ("[Content_Types].xml", content_types(reference, mark.named)),
        ("_rels/.rels", package_rels(mark.named)),
        ("docProps/core.xml", core_properties(title.as_deref(), mark)),
        ("word/document.xml", document),
        (
            "word/_rels/document.xml.rels",
            document_rels(&w.links, reference),
        ),
        ("word/styles.xml", styles(doc, reference)?),
        ("word/numbering.xml", numbering(&w.lists)),
        ("word/settings.xml", SETTINGS.to_owned()),
    ];
    if mark.named {
        parts.push(("docProps/app.xml", app_properties(&mark.generator)));
    }
    if let Some(font_table) = reference.and_then(|r| r.font_table.as_ref()) {
        parts.push((FONT_TABLE_PART, font_table.clone()));
    }
    if let Some(theme) = reference.and_then(|r| r.theme.as_ref()) {
        parts.push((THEME_PART, theme.clone()));
    }
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default());
    for (name, content) in parts {
        zip.start_file(name, opts)
            .with_context(|| format!("could not add {name} to the DOCX package"))?;
        zip.write_all(content.as_bytes())
            .with_context(|| format!("could not write {name} into the DOCX package"))?;
    }
    let out = zip.finish().context("could not finish the DOCX package")?;
    Ok(out.into_inner())
}
