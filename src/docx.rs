//! The DOCX writer. The document maps onto the same style names pandoc's
//! DOCX writer uses (Title, Heading1-6, FirstParagraph, BodyText, Compact,
//! BlockText, SourceCode, VerbatimChar, Hyperlink), so a document styled
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
    Attr, Block, Document, Inline, ListAttributes, ListNumberDelim, ListNumberStyle, MetaValue,
    QuoteType, stringify,
};
use crate::highlight;
use crate::html::check_meta;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const V_NS: &str = "urn:schemas-microsoft-com:vml";
const O_NS: &str = "urn:schemas-microsoft-com:office:office";
const REL_HYPERLINK: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";

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
    code: bool,
    link: bool,
    rtl: bool,
}

impl RunFormat {
    fn properties(self) -> String {
        let mut p = String::new();
        if self.code {
            p.push_str(r#"<w:rStyle w:val="VerbatimChar"/>"#);
        } else if self.link {
            p.push_str(r#"<w:rStyle w:val="Hyperlink"/>"#);
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
            Inline::Code(_, s) => self.text_run(s, RunFormat { code: true, ..f }),
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
                self.inlines(xs, RunFormat { link: true, ..f });
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
                code: true,
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
        match list {
            Some(ListPos {
                level,
                first: true,
                num,
            }) => {
                let _ = write!(
                    self.body,
                    r#"<w:numPr><w:ilvl w:val="{level}"/><w:numId w:val="{num}"/></w:numPr>"#
                );
            }
            // Later paragraphs of an item stay in the list, at its indent,
            // through a numbering whose levels have no bullet.
            Some(ListPos {
                level,
                first: false,
                ..
            }) => {
                let _ = write!(
                    self.body,
                    r#"<w:numPr><w:ilvl w:val="{level}"/><w:numId w:val="{CONTINUATION_NUM}"/></w:numPr>"#
                );
            }
            None => {}
        }
        if self.rtl {
            self.body.push_str("<w:bidi/>");
        }
        self.body.push_str("</w:pPr>");
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
                Block::RawHtml(_) => {}
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
fn styles(doc: &Document) -> Result<String> {
    let styles = STYLES.replace("</w:styles>", &format!("{}</w:styles>", token_styles()));
    let lang = match doc.meta("lang") {
        None => return Ok(styles),
        Some(MetaValue::Inlines(xs)) => stringify(xs),
        Some(_) => bail!("metadata field `lang` must be text"),
    };
    let primary = lang.split('-').next().unwrap_or_default();
    let attr = if RTL_LANGUAGES.contains(&primary) {
        format!(r#"<w:lang w:val="en-US" w:bidi="{}"/>"#, esc(&lang))
    } else {
        format!(r#"<w:lang w:val="{}"/>"#, esc(&lang))
    };
    Ok(styles.replace(r#"<w:lang w:val="en-US"/>"#, &attr))
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
        let abstract_id = match list {
            None => 0,
            Some(la) => {
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
            }
        };
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

fn document_rels(links: &[String]) -> String {
    let mut rels = String::from(
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>"#,
    );
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

fn core_properties(title: Option<&str>) -> String {
    let title = title.map_or(String::new(), |t| {
        format!("<dc:title>{}</dc:title>", esc(t))
    });
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">{title}</cp:coreProperties>
"#
    )
}

/// The DOCX package for `doc`.
pub fn write(doc: &Document, highlight: bool) -> Result<Vec<u8>> {
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
<w:document xmlns:w="{W_NS}" xmlns:r="{R_NS}"{vml}><w:body>{}<w:sectPr/></w:body></w:document>
"#,
        w.body
    );
    let parts: [(&str, String); 8] = [
        ("[Content_Types].xml", CONTENT_TYPES.to_owned()),
        ("_rels/.rels", PACKAGE_RELS.to_owned()),
        ("docProps/core.xml", core_properties(title.as_deref())),
        ("word/document.xml", document),
        ("word/_rels/document.xml.rels", document_rels(&w.links)),
        ("word/styles.xml", styles(doc)?),
        ("word/numbering.xml", numbering(&w.lists)),
        ("word/settings.xml", SETTINGS.to_owned()),
    ];
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
