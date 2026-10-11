//! Lua filters (`--lua-filter`): the slice of pandoc's Lua filter API the
//! fleet's filters use, written from pandoc's Lua filter documentation.
//!
//! Elements cross into Lua as plain tables with pandoc's field names (`t`
//! and `tag`, `content`, `text`, `format`, `level`, `target`, ...). A
//! filter is the table a script returns, or else the global functions it
//! defines named after element types. Each filter runs in pandoc's order:
//! its inline functions over every inline (innermost first), then its block
//! functions over every block, then `Meta`, then `Pandoc`. A function
//! returning nil keeps the element, an element replaces it, and a list
//! replaces it with the list's elements (`{}` deletes it). Hooks and
//! constructors outside this slice are refused rather than ignored.

use std::path::Path;

use anyhow::{Context as _, Result, anyhow, bail};
use mlua::{Function, Lua, MultiValue, Table, Value};

use crate::ast::{
    Attr, Block, Document, Inline, ListAttributes, ListNumberDelim, ListNumberStyle, MetaValue,
    QuoteType,
};

const INLINE_TAGS: &[&str] = &[
    "Cite",
    "Code",
    "Emph",
    "LineBreak",
    "Link",
    "Quoted",
    "RawInline",
    "SoftBreak",
    "Space",
    "Str",
    "Strong",
];
const BLOCK_TAGS: &[&str] = &[
    "BlockQuote",
    "BulletList",
    "CodeBlock",
    "Header",
    "HorizontalRule",
    "OrderedList",
    "Para",
    "Plain",
    "RawBlock",
];

/// Filter hooks pandoc has that this does not implement.
const UNSUPPORTED_HOOKS: &[&str] = &[
    "Inline",
    "Inlines",
    "Block",
    "Blocks",
    "Image",
    "Math",
    "Note",
    "Span",
    "SmallCaps",
    "Strikeout",
    "Subscript",
    "Superscript",
    "Underline",
    "DefinitionList",
    "Div",
    "Figure",
    "LineBlock",
    "Table",
];

fn lua_err(e: mlua::Error) -> anyhow::Error {
    anyhow!("{e}")
}

// ─── to Lua ─────────────────────────────────────────────────────────────

fn element(lua: &Lua, tag: &str) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("t", tag)?;
    t.set("tag", tag)?;
    Ok(t)
}

fn attr_to_lua(lua: &Lua, a: &Attr) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("identifier", a.id.as_str())?;
    t.set(
        "classes",
        lua.create_sequence_from(a.classes.iter().map(String::as_str))?,
    )?;
    let kvs = lua.create_table()?;
    for (k, v) in &a.kvs {
        kvs.set(k.as_str(), v.as_str())?;
    }
    t.set("attributes", kvs)?;
    Ok(t)
}

fn inlines_to_lua(lua: &Lua, xs: &[Inline]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, x) in xs.iter().enumerate() {
        t.set(i + 1, inline_to_lua(lua, x)?)?;
    }
    Ok(t)
}

fn inline_to_lua(lua: &Lua, x: &Inline) -> mlua::Result<Table> {
    let t = match x {
        Inline::Str(s) => {
            let t = element(lua, "Str")?;
            t.set("text", s.as_str())?;
            t
        }
        Inline::Space => element(lua, "Space")?,
        Inline::SoftBreak => element(lua, "SoftBreak")?,
        Inline::LineBreak => element(lua, "LineBreak")?,
        Inline::Code(a, s) => {
            let t = element(lua, "Code")?;
            t.set("attr", attr_to_lua(lua, a)?)?;
            t.set("text", s.as_str())?;
            t
        }
        Inline::Emph(xs) | Inline::Strong(xs) => {
            let t = element(
                lua,
                if matches!(x, Inline::Emph(_)) {
                    "Emph"
                } else {
                    "Strong"
                },
            )?;
            t.set("content", inlines_to_lua(lua, xs)?)?;
            t
        }
        Inline::Quoted(q, xs) => {
            let t = element(lua, "Quoted")?;
            let qt = match q {
                QuoteType::Single => "SingleQuote",
                QuoteType::Double => "DoubleQuote",
            };
            t.set("quotetype", qt)?;
            t.set("content", inlines_to_lua(lua, xs)?)?;
            t
        }
        Inline::Cite(keys, xs) => {
            let t = element(lua, "Cite")?;
            let cites = lua.create_table()?;
            for (i, k) in keys.iter().enumerate() {
                let c = lua.create_table()?;
                c.set("id", k.as_str())?;
                cites.set(i + 1, c)?;
            }
            t.set("citations", cites)?;
            t.set("content", inlines_to_lua(lua, xs)?)?;
            t
        }
        Inline::Link(a, xs, url, title) => {
            let t = element(lua, "Link")?;
            t.set("attr", attr_to_lua(lua, a)?)?;
            t.set("content", inlines_to_lua(lua, xs)?)?;
            t.set("target", url.as_str())?;
            t.set("title", title.as_str())?;
            t
        }
        Inline::RawHtml(s) => {
            let t = element(lua, "RawInline")?;
            t.set("format", "html")?;
            t.set("text", s.as_str())?;
            t
        }
    };
    Ok(t)
}

fn blocks_to_lua(lua: &Lua, bs: &[Block]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, b) in bs.iter().enumerate() {
        t.set(i + 1, block_to_lua(lua, b)?)?;
    }
    Ok(t)
}

fn block_to_lua(lua: &Lua, b: &Block) -> mlua::Result<Table> {
    let t = match b {
        Block::Plain(xs) | Block::Para(xs) => {
            let t = element(
                lua,
                if matches!(b, Block::Plain(_)) {
                    "Plain"
                } else {
                    "Para"
                },
            )?;
            t.set("content", inlines_to_lua(lua, xs)?)?;
            t
        }
        Block::Header(level, a, xs) => {
            let t = element(lua, "Header")?;
            t.set("level", *level)?;
            t.set("attr", attr_to_lua(lua, a)?)?;
            t.set("identifier", a.id.as_str())?;
            t.set("content", inlines_to_lua(lua, xs)?)?;
            t
        }
        Block::BulletList(items) => {
            let t = element(lua, "BulletList")?;
            t.set("content", items_to_lua(lua, items)?)?;
            t
        }
        Block::OrderedList(la, items) => {
            let t = element(lua, "OrderedList")?;
            t.set("listAttributes", list_attributes_to_lua(lua, la)?)?;
            t.set("content", items_to_lua(lua, items)?)?;
            t
        }
        Block::Code(a, text) => {
            let t = element(lua, "CodeBlock")?;
            t.set("attr", attr_to_lua(lua, a)?)?;
            t.set("identifier", a.id.as_str())?;
            t.set("text", text.as_str())?;
            t
        }
        Block::HorizontalRule => element(lua, "HorizontalRule")?,
        Block::Quote(bs) => {
            let t = element(lua, "BlockQuote")?;
            t.set("content", blocks_to_lua(lua, bs)?)?;
            t
        }
        Block::RawHtml(s) => {
            let t = element(lua, "RawBlock")?;
            t.set("format", "html")?;
            t.set("text", s.as_str())?;
            t
        }
    };
    Ok(t)
}

fn items_to_lua(lua: &Lua, items: &[Vec<Block>]) -> mlua::Result<Table> {
    let list = lua.create_table()?;
    for (i, item) in items.iter().enumerate() {
        list.set(i + 1, blocks_to_lua(lua, item)?)?;
    }
    Ok(list)
}

const fn style_name(s: ListNumberStyle) -> &'static str {
    match s {
        ListNumberStyle::DefaultStyle => "DefaultStyle",
        ListNumberStyle::Example => "Example",
        ListNumberStyle::Decimal => "Decimal",
        ListNumberStyle::LowerRoman => "LowerRoman",
        ListNumberStyle::UpperRoman => "UpperRoman",
        ListNumberStyle::LowerAlpha => "LowerAlpha",
        ListNumberStyle::UpperAlpha => "UpperAlpha",
    }
}

const fn delim_name(d: ListNumberDelim) -> &'static str {
    match d {
        ListNumberDelim::DefaultDelim => "DefaultDelim",
        ListNumberDelim::Period => "Period",
        ListNumberDelim::OneParen => "OneParen",
        ListNumberDelim::TwoParens => "TwoParens",
    }
}

fn list_attributes_to_lua(lua: &Lua, la: &ListAttributes) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("start", la.start)?;
    t.set("style", style_name(la.style))?;
    t.set("delimiter", delim_name(la.delim))?;
    Ok(t)
}

fn meta_to_lua(lua: &Lua, v: &MetaValue) -> mlua::Result<Value> {
    Ok(match v {
        MetaValue::Inlines(xs) => Value::Table(inlines_to_lua(lua, xs)?),
        MetaValue::List(xs) => {
            let t = lua.create_table()?;
            for (i, x) in xs.iter().enumerate() {
                t.set(i + 1, meta_to_lua(lua, x)?)?;
            }
            Value::Table(t)
        }
        MetaValue::Bool(b) => Value::Boolean(*b),
    })
}

fn doc_to_lua(lua: &Lua, doc: &Document) -> mlua::Result<Table> {
    let t = element(lua, "Pandoc")?;
    t.set("blocks", blocks_to_lua(lua, &doc.blocks)?)?;
    let meta = lua.create_table()?;
    for (k, v) in &doc.meta {
        meta.set(k.as_str(), meta_to_lua(lua, v)?)?;
    }
    t.set("meta", meta)?;
    Ok(t)
}

// ─── from Lua ───────────────────────────────────────────────────────────

fn tag_of(t: &Table) -> Result<String> {
    t.get::<String>("t")
        .map_err(|_| anyhow!("Lua value is not a pandoc element"))
}

fn str_field(t: &Table, k: &str) -> Result<String> {
    t.get::<String>(k).map_err(|e| anyhow!("field `{k}`: {e}"))
}

fn attr_from_lua(v: Value) -> Result<Attr> {
    let Value::Table(t) = v else {
        return Ok(Attr::default());
    };
    let mut a = Attr {
        id: t
            .get::<Option<String>>("identifier")
            .map_err(lua_err)?
            .unwrap_or_default(),
        ..Attr::default()
    };
    if let Some(classes) = t.get::<Option<Table>>("classes").map_err(lua_err)? {
        for c in classes.sequence_values::<String>() {
            a.classes.push(c.map_err(lua_err)?);
        }
    }
    if let Some(kvs) = t.get::<Option<Table>>("attributes").map_err(lua_err)? {
        for pair in kvs.pairs::<String, String>() {
            a.kvs.push(pair.map_err(lua_err)?);
        }
        a.kvs.sort();
    }
    Ok(a)
}

fn inlines_from_lua(v: Value) -> Result<Vec<Inline>> {
    match v {
        Value::String(s) => Ok(string_to_inlines(&s.to_str().map_err(lua_err)?)),
        Value::Table(t) => {
            if t.contains_key("t").map_err(lua_err)? {
                return Ok(vec![inline_from_lua(&t)?]);
            }
            let mut out = Vec::new();
            for x in t.sequence_values::<Table>() {
                out.push(inline_from_lua(&x.map_err(lua_err)?)?);
            }
            Ok(out)
        }
        Value::Nil => Ok(Vec::new()),
        other => bail!("expected inlines, got a Lua {}", other.type_name()),
    }
}

/// A string given where inlines are expected: words and spaces.
fn string_to_inlines(s: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    for (i, w) in s.split(' ').enumerate() {
        if i > 0 {
            out.push(Inline::Space);
        }
        if !w.is_empty() {
            out.push(Inline::Str(w.to_owned()));
        }
    }
    out
}

fn inline_from_lua(t: &Table) -> Result<Inline> {
    let tag = tag_of(t)?;
    let content = || inlines_from_lua(t.get::<Value>("content").map_err(lua_err)?);
    Ok(match tag.as_str() {
        "Str" => Inline::Str(str_field(t, "text")?),
        "Space" => Inline::Space,
        "SoftBreak" => Inline::SoftBreak,
        "LineBreak" => Inline::LineBreak,
        "Code" => Inline::Code(
            attr_from_lua(t.get("attr").map_err(lua_err)?)?,
            str_field(t, "text")?,
        ),
        "Emph" => Inline::Emph(content()?),
        "Strong" => Inline::Strong(content()?),
        "Quoted" => {
            let q = match str_field(t, "quotetype")?.as_str() {
                "SingleQuote" => QuoteType::Single,
                "DoubleQuote" => QuoteType::Double,
                other => bail!("unknown quote type {other}"),
            };
            Inline::Quoted(q, content()?)
        }
        "Cite" => {
            let mut keys = Vec::new();
            let cites: Table = t.get("citations").map_err(lua_err)?;
            for c in cites.sequence_values::<Table>() {
                keys.push(str_field(&c.map_err(lua_err)?, "id")?);
            }
            Inline::Cite(keys, content()?)
        }
        "Link" => Inline::Link(
            attr_from_lua(t.get("attr").map_err(lua_err)?)?,
            content()?,
            str_field(t, "target")?,
            t.get::<Option<String>>("title")
                .map_err(lua_err)?
                .unwrap_or_default(),
        ),
        "RawInline" => {
            let format = str_field(t, "format")?;
            if format != "html" {
                bail!("RawInline of format `{format}` is not supported yet");
            }
            Inline::RawHtml(str_field(t, "text")?)
        }
        other => bail!("inline element `{other}` is not supported yet"),
    })
}

fn blocks_from_lua(v: Value) -> Result<Vec<Block>> {
    match v {
        Value::Table(t) => {
            if t.contains_key("t").map_err(lua_err)? {
                return Ok(vec![block_from_lua(&t)?]);
            }
            let mut out = Vec::new();
            for x in t.sequence_values::<Table>() {
                out.push(block_from_lua(&x.map_err(lua_err)?)?);
            }
            Ok(out)
        }
        Value::Nil => Ok(Vec::new()),
        other => bail!("expected blocks, got a Lua {}", other.type_name()),
    }
}

fn block_from_lua(t: &Table) -> Result<Block> {
    let tag = tag_of(t)?;
    let content = || inlines_from_lua(t.get::<Value>("content").map_err(lua_err)?);
    Ok(match tag.as_str() {
        "Plain" => Block::Plain(content()?),
        "Para" => Block::Para(content()?),
        "Header" => {
            let mut attr = attr_from_lua(t.get("attr").map_err(lua_err)?)?;
            if let Some(id) = t.get::<Option<String>>("identifier").map_err(lua_err)? {
                attr.id = id;
            }
            Block::Header(t.get::<usize>("level").map_err(lua_err)?, attr, content()?)
        }
        "BulletList" => Block::BulletList(items_from_lua(t)?),
        "OrderedList" => Block::OrderedList(
            list_attributes_from_lua(t.get("listAttributes").map_err(lua_err)?)?,
            items_from_lua(t)?,
        ),
        "CodeBlock" => {
            let mut attr = attr_from_lua(t.get("attr").map_err(lua_err)?)?;
            if let Some(id) = t.get::<Option<String>>("identifier").map_err(lua_err)? {
                attr.id = id;
            }
            Block::Code(attr, str_field(t, "text")?)
        }
        "HorizontalRule" => Block::HorizontalRule,
        "BlockQuote" => Block::Quote(blocks_from_lua(t.get("content").map_err(lua_err)?)?),
        "RawBlock" => {
            let format = str_field(t, "format")?;
            if format != "html" {
                bail!("RawBlock of format `{format}` is not supported yet");
            }
            Block::RawHtml(str_field(t, "text")?)
        }
        other => bail!("block element `{other}` is not supported yet"),
    })
}

fn items_from_lua(t: &Table) -> Result<Vec<Vec<Block>>> {
    let mut items = Vec::new();
    let list: Table = t.get("content").map_err(lua_err)?;
    for item in list.sequence_values::<Value>() {
        items.push(blocks_from_lua(item.map_err(lua_err)?)?);
    }
    Ok(items)
}

/// A `ListAttributes` table; absent fields take pandoc's defaults.
fn list_attributes_from_lua(v: Value) -> Result<ListAttributes> {
    let mut la = ListAttributes {
        start: 1,
        style: ListNumberStyle::DefaultStyle,
        delim: ListNumberDelim::DefaultDelim,
    };
    let Value::Table(t) = v else {
        return Ok(la);
    };
    if let Some(start) = t.get::<Option<i64>>("start").map_err(lua_err)? {
        la.start = start;
    }
    if let Some(style) = t.get::<Option<String>>("style").map_err(lua_err)? {
        la.style = [
            ListNumberStyle::DefaultStyle,
            ListNumberStyle::Example,
            ListNumberStyle::Decimal,
            ListNumberStyle::LowerRoman,
            ListNumberStyle::UpperRoman,
            ListNumberStyle::LowerAlpha,
            ListNumberStyle::UpperAlpha,
        ]
        .into_iter()
        .find(|&s| style_name(s) == style)
        .ok_or_else(|| anyhow!("unknown list number style `{style}`"))?;
    }
    if let Some(delim) = t.get::<Option<String>>("delimiter").map_err(lua_err)? {
        la.delim = [
            ListNumberDelim::DefaultDelim,
            ListNumberDelim::Period,
            ListNumberDelim::OneParen,
            ListNumberDelim::TwoParens,
        ]
        .into_iter()
        .find(|&d| delim_name(d) == delim)
        .ok_or_else(|| anyhow!("unknown list number delimiter `{delim}`"))?;
    }
    Ok(la)
}

fn meta_from_lua(v: Value) -> Result<MetaValue> {
    match v {
        Value::Boolean(b) => Ok(MetaValue::Bool(b)),
        Value::String(s) => Ok(MetaValue::Inlines(string_to_inlines(
            &s.to_str().map_err(lua_err)?,
        ))),
        Value::Table(t) => {
            if t.contains_key("t").map_err(lua_err)? {
                return Ok(MetaValue::Inlines(vec![inline_from_lua(&t)?]));
            }
            let first: Value = t.get(1).map_err(lua_err)?;
            let is_inlines = match &first {
                Value::Table(f) => f
                    .get::<Option<String>>("t")
                    .map_err(lua_err)?
                    .is_some_and(|tag| INLINE_TAGS.contains(&tag.as_str())),
                _ => false,
            };
            if is_inlines {
                return Ok(MetaValue::Inlines(inlines_from_lua(Value::Table(t))?));
            }
            let mut items = Vec::new();
            for x in t.sequence_values::<Value>() {
                items.push(meta_from_lua(x.map_err(lua_err)?)?);
            }
            Ok(MetaValue::List(items))
        }
        other => bail!(
            "metadata value of Lua type {} is not supported",
            other.type_name()
        ),
    }
}

fn doc_from_lua(t: &Table) -> Result<Document> {
    let blocks = blocks_from_lua(t.get("blocks").map_err(lua_err)?)?;
    let mut meta = Vec::new();
    let m: Table = t.get("meta").map_err(lua_err)?;
    for pair in m.pairs::<String, Value>() {
        let (k, v) = pair.map_err(lua_err)?;
        meta.push((k, meta_from_lua(v)?));
    }
    meta.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(Document { meta, blocks })
}

// ─── the pandoc module ──────────────────────────────────────────────────

fn install_pandoc_module(lua: &Lua) -> mlua::Result<()> {
    let pandoc = lua.create_table()?;
    let simple = |tag: &'static str| lua.create_function(move |lua, ()| element(lua, tag));
    pandoc.set("Space", simple("Space")?)?;
    pandoc.set("SoftBreak", simple("SoftBreak")?)?;
    pandoc.set("LineBreak", simple("LineBreak")?)?;
    pandoc.set(
        "Str",
        lua.create_function(|lua, text: String| {
            let t = element(lua, "Str")?;
            t.set("text", text)?;
            Ok(t)
        })?,
    )?;
    for tag in ["Emph", "Strong", "Para", "Plain"] {
        pandoc.set(
            tag,
            lua.create_function(move |lua, content: Value| {
                let t = element(lua, tag)?;
                t.set("content", content_list(lua, content)?)?;
                Ok(t)
            })?,
        )?;
    }
    pandoc.set(
        "Link",
        lua.create_function(|lua, (content, target, title, attr): (Value, String, Option<String>, Option<Table>)| {
            let t = element(lua, "Link")?;
            t.set("content", content_list(lua, content)?)?;
            t.set("target", target)?;
            t.set("title", title.unwrap_or_default())?;
            match attr {
                Some(a) => t.set("attr", a)?,
                None => t.set("attr", attr_to_lua(lua, &Attr::default())?)?,
            }
            Ok(t)
        })?,
    )?;
    pandoc.set(
        "Header",
        lua.create_function(
            |lua, (level, content, attr): (usize, Value, Option<Table>)| {
                let t = element(lua, "Header")?;
                t.set("level", level)?;
                t.set("content", content_list(lua, content)?)?;
                match attr {
                    Some(a) => t.set("attr", a)?,
                    None => t.set("attr", attr_to_lua(lua, &Attr::default())?)?,
                }
                Ok(t)
            },
        )?,
    )?;
    pandoc.set("HorizontalRule", simple("HorizontalRule")?)?;
    pandoc.set(
        "CodeBlock",
        lua.create_function(|lua, (text, attr): (String, Option<Table>)| {
            let t = element(lua, "CodeBlock")?;
            t.set("text", text)?;
            match attr {
                Some(a) => {
                    t.set("identifier", a.get::<Option<String>>("identifier")?)?;
                    t.set("attr", a)?;
                }
                None => t.set("attr", attr_to_lua(lua, &Attr::default())?)?,
            }
            Ok(t)
        })?,
    )?;
    for tag in ["BulletList", "OrderedList"] {
        pandoc.set(
            tag,
            lua.create_function(move |lua, (items, attrs): (Table, Option<Table>)| {
                let t = element(lua, tag)?;
                t.set("content", items)?;
                if tag == "OrderedList" {
                    let attrs = match attrs {
                        Some(a) => a,
                        None => {
                            let a = lua.create_table()?;
                            a.set("start", 1)?;
                            a.set("style", "DefaultStyle")?;
                            a.set("delimiter", "DefaultDelim")?;
                            a
                        }
                    };
                    t.set("listAttributes", attrs)?;
                }
                Ok(t)
            })?,
        )?;
    }
    for tag in ["RawInline", "RawBlock"] {
        pandoc.set(
            tag,
            lua.create_function(move |lua, (format, text): (String, String)| {
                let t = element(lua, tag)?;
                t.set("format", format)?;
                t.set("text", text)?;
                Ok(t)
            })?,
        )?;
    }
    lua.globals().set("pandoc", pandoc)
}

/// A constructor's content argument as a list: a string becomes words and
/// spaces, a single element a one-element list.
fn content_list(lua: &Lua, v: Value) -> mlua::Result<Table> {
    match v {
        Value::String(s) => inlines_to_lua(lua, &string_to_inlines(&s.to_str()?)),
        Value::Table(t) if t.contains_key("t")? => lua.create_sequence_from([t]),
        Value::Table(t) => Ok(t),
        Value::Nil => lua.create_table(),
        other => Err(mlua::Error::runtime(format!(
            "expected a list of elements, got a {}",
            other.type_name()
        ))),
    }
}

// ─── running filters ────────────────────────────────────────────────────

/// The functions a filter script defines: the table it returns, or else
/// its globals named after element types.
fn filter_functions(lua: &Lua, returned: MultiValue) -> Result<Vec<Table>> {
    if let Some(Value::Table(t)) = returned.into_iter().next() {
        if t.contains_key(1).map_err(lua_err)? {
            return t
                .sequence_values::<Table>()
                .map(|f| f.map_err(lua_err))
                .collect();
        }
        return Ok(vec![t]);
    }
    let f = lua.create_table().map_err(lua_err)?;
    let globals = lua.globals();
    for name in INLINE_TAGS
        .iter()
        .chain(BLOCK_TAGS)
        .chain(&["Meta", "Pandoc"])
        .chain(UNSUPPORTED_HOOKS)
    {
        if let Value::Function(func) = globals.get::<Value>(*name).map_err(lua_err)? {
            f.set(*name, func).map_err(lua_err)?;
        }
    }
    Ok(vec![f])
}

/// Apply the filter's function for each element of `list` (looked up by
/// the element's tag), splicing in what it returns.
fn apply_to_list(filter: &Table, list: &Table) -> Result<()> {
    let len = list.raw_len();
    let mut out: Vec<Value> = Vec::with_capacity(len);
    for i in 1..=len {
        let el: Table = list.get(i).map_err(lua_err)?;
        let tag = tag_of(&el)?;
        let Some(func) = filter
            .get::<Option<Function>>(tag.as_str())
            .map_err(lua_err)?
        else {
            out.push(Value::Table(el));
            continue;
        };
        match func.call::<Value>(el.clone()).map_err(lua_err)? {
            Value::Nil => out.push(Value::Table(el)),
            Value::Table(r) if r.contains_key("t").map_err(lua_err)? => out.push(Value::Table(r)),
            Value::Table(r) => {
                for x in r.sequence_values::<Value>() {
                    out.push(x.map_err(lua_err)?);
                }
            }
            other => bail!("filter function {tag} returned a Lua {}", other.type_name()),
        }
    }
    list.clear().map_err(lua_err)?;
    for (i, v) in out.into_iter().enumerate() {
        list.set(i + 1, v).map_err(lua_err)?;
    }
    Ok(())
}

/// Walk `v` in document order, innermost first, applying the filter to
/// every list of elements whose tags are in `tags`.
fn walk(filter: &Table, v: &Table, tags: &[&str]) -> Result<()> {
    if let Some(tag) = v.get::<Option<String>>("t").map_err(lua_err)? {
        if tag == "Pandoc" {
            let meta: Table = v.get("meta").map_err(lua_err)?;
            let mut keys: Vec<String> = Vec::new();
            for k in meta.pairs::<String, Value>() {
                keys.push(k.map_err(lua_err)?.0);
            }
            keys.sort();
            for k in keys {
                if let Value::Table(m) = meta.get::<Value>(k.as_str()).map_err(lua_err)? {
                    walk(filter, &m, tags)?;
                }
            }
            let blocks: Table = v.get("blocks").map_err(lua_err)?;
            return walk(filter, &blocks, tags);
        }
        if let Value::Table(c) = v.get::<Value>("content").map_err(lua_err)? {
            walk(filter, &c, tags)?;
        }
        return Ok(());
    }
    for child in v.sequence_values::<Value>() {
        if let Value::Table(c) = child.map_err(lua_err)? {
            walk(filter, &c, tags)?;
        }
    }
    let first: Value = v.get(1).map_err(lua_err)?;
    if let Value::Table(f) = first
        && let Some(tag) = f.get::<Option<String>>("t").map_err(lua_err)?
        && tags.contains(&tag.as_str())
    {
        apply_to_list(filter, v)?;
    }
    Ok(())
}

fn run_filter(filter: &Table, doc: Table) -> Result<Table> {
    for hook in UNSUPPORTED_HOOKS {
        if filter.contains_key(*hook).map_err(lua_err)? {
            bail!("the Lua filter hook `{hook}` is not supported yet");
        }
    }
    let has_any = |tags: &[&str]| -> Result<bool> {
        for t in tags {
            if filter.contains_key(*t).map_err(lua_err)? {
                return Ok(true);
            }
        }
        Ok(false)
    };
    if has_any(INLINE_TAGS)? {
        walk(filter, &doc, INLINE_TAGS)?;
    }
    if has_any(BLOCK_TAGS)? {
        walk(filter, &doc, BLOCK_TAGS)?;
    }
    if let Some(f) = filter.get::<Option<Function>>("Meta").map_err(lua_err)? {
        let meta: Table = doc.get("meta").map_err(lua_err)?;
        if let Value::Table(m) = f.call::<Value>(meta).map_err(lua_err)? {
            doc.set("meta", m).map_err(lua_err)?;
        }
    }
    if let Some(f) = filter.get::<Option<Function>>("Pandoc").map_err(lua_err)?
        && let Value::Table(d) = f.call::<Value>(doc.clone()).map_err(lua_err)?
    {
        return Ok(d);
    }
    Ok(doc)
}

/// Run the Lua filter scripts over `doc`, in order.
pub fn run_filters(
    doc: Document,
    scripts: &[impl AsRef<Path>],
    format: &str,
    input: &str,
) -> Result<Document> {
    let mut doc = doc;
    for script in scripts {
        let script = script.as_ref();
        let lua = Lua::new();
        let globals = lua.globals();
        globals.set("FORMAT", format).map_err(lua_err)?;
        let state = lua.create_table().map_err(lua_err)?;
        state
            .set(
                "input_files",
                lua.create_sequence_from([input]).map_err(lua_err)?,
            )
            .map_err(lua_err)?;
        globals.set("PANDOC_STATE", state).map_err(lua_err)?;
        install_pandoc_module(&lua).map_err(lua_err)?;
        let code = std::fs::read_to_string(script)
            .with_context(|| format!("could not read Lua filter {}", script.display()))?;
        let returned = lua
            .load(&code)
            .set_name(script.display().to_string())
            .eval::<MultiValue>()
            .map_err(lua_err)
            .with_context(|| format!("Lua filter {} failed to load", script.display()))?;
        for filter in filter_functions(&lua, returned)? {
            let t = doc_to_lua(&lua, &doc).map_err(lua_err)?;
            let out = run_filter(&filter, t)
                .with_context(|| format!("Lua filter {} failed", script.display()))?;
            doc = doc_from_lua(&out)?;
        }
    }
    Ok(doc)
}
