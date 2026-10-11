//! `--embed-resources`: the page re-tokenized, linked stylesheets inlined
//! as `<style>` elements, and stylesheet text compacted, as pandoc does it.
//!
//! The tokenizer and renderer follow tagsoup 0.14.8 (BSD-3-Clause,
//! Copyright Neil Mitchell) for the HTML pandoc produces: entities in text
//! and attribute values are decoded and re-escaped (so `"` and `'` in text
//! come out as `&quot;` and `&#39;`), whitespace inside tags is
//! normalized, attributes with empty values render bare, `script` and
//! `style` contents are raw, and void elements written `<x />` keep that
//! form. The stylesheet handling is written from observed behaviour.

use std::path::Path;

use anyhow::{Context as _, Result, bail};

use crate::entities::ENTITIES;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tag {
    Open(String, Vec<(String, String)>),
    Close(String),
    Text(String),
    Comment(String),
}

/// Elements written `<x />` when empty.
const MINIMIZED: &[&str] = &[
    "hr", "br", "img", "meta", "link", "col", "use", "path", "rect",
];

/// Elements whose content is raw text.
fn is_raw(name: &str) -> bool {
    name.eq_ignore_ascii_case("script") || name.eq_ignore_ascii_case("style")
}

/// Decode the character references in `s`.
fn decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let decoded = rest[1..].find(';').and_then(|semi| {
            let name = &rest[1..=semi];
            let ch = if let Some(num) = name.strip_prefix('#') {
                let code = match num.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => num.parse().ok(),
                };
                code.and_then(char::from_u32).map(String::from)
            } else {
                let key = format!("{name};");
                ENTITIES
                    .binary_search_by_key(&key.as_str(), |e| e.0)
                    .ok()
                    .map(|i| ENTITIES[i].1.to_owned())
            };
            ch.map(|c| (c, semi + 2))
        });
        if let Some((c, len)) = decoded {
            out.push_str(&c);
            rest = &rest[len..];
        } else {
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

fn tokenize(html: &str) -> Result<Vec<Tag>> {
    let s = html;
    let mut tags = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    let flush = |text: &mut String, tags: &mut Vec<Tag>| {
        if !text.is_empty() {
            tags.push(Tag::Text(decode(text)));
            text.clear();
        }
    };
    while i < s.len() {
        let rest = &s[i..];
        if let Some(body) = rest.strip_prefix("<!--") {
            let end = body
                .find("-->")
                .with_context(|| "unterminated HTML comment".to_owned())?;
            flush(&mut text, &mut tags);
            tags.push(Tag::Comment(body[..end].to_owned()));
            i += 4 + end + 3;
            continue;
        }
        let starts_tag = rest
            .strip_prefix('<')
            .and_then(|r| r.chars().next())
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '!' || c == '/');
        if !starts_tag {
            let c = rest.chars().next().unwrap_or_default();
            text.push(c);
            i += c.len_utf8();
            continue;
        }
        flush(&mut text, &mut tags);
        let close = rest
            .find('>')
            .with_context(|| format!("unterminated tag at byte {i}"))?;
        let inner = &rest[1..close];
        i += close + 1;
        if let Some(name) = inner.strip_prefix('/') {
            tags.push(Tag::Close(name.trim().to_owned()));
            continue;
        }
        let (inner, self_closing) = match inner.strip_suffix('/') {
            Some(x) => (x, true),
            None => (inner, false),
        };
        let name_end = inner.find(char::is_whitespace).unwrap_or(inner.len());
        let name = inner[..name_end].to_owned();
        let attrs = parse_attrs(&inner[name_end..])?;
        tags.push(Tag::Open(name.clone(), attrs));
        if self_closing {
            tags.push(Tag::Close(name));
        } else if is_raw(&name) {
            let lower = s[i..].to_ascii_lowercase();
            let end = lower
                .find(&format!("</{}", name.to_ascii_lowercase()))
                .with_context(|| format!("unterminated <{name}>"))?;
            if end > 0 {
                tags.push(Tag::Text(s[i..i + end].to_owned()));
            }
            i += end;
        }
    }
    flush(&mut text, &mut tags);
    Ok(tags)
}

fn parse_attrs(s: &str) -> Result<Vec<(String, String)>> {
    let chars: Vec<char> = s.chars().collect();
    let mut attrs = Vec::new();
    let mut k = 0;
    loop {
        while k < chars.len() && chars[k].is_whitespace() {
            k += 1;
        }
        if k >= chars.len() {
            break;
        }
        let start = k;
        while k < chars.len() && !chars[k].is_whitespace() && chars[k] != '=' {
            k += 1;
        }
        let name: String = chars[start..k].iter().collect();
        if k < chars.len() && chars[k] == '=' {
            k += 1;
            let value: String = match chars.get(k) {
                Some(&q @ ('"' | '\'')) => {
                    let vstart = k + 1;
                    let end = chars[vstart..]
                        .iter()
                        .position(|&c| c == q)
                        .with_context(|| format!("unterminated attribute value in {s:?}"))?;
                    k = vstart + end + 1;
                    chars[vstart..vstart + end].iter().collect()
                }
                Some(_) => {
                    let vstart = k;
                    while k < chars.len() && !chars[k].is_whitespace() {
                        k += 1;
                    }
                    chars[vstart..k].iter().collect()
                }
                None => bail!("attribute without a value in {s:?}"),
            };
            attrs.push((name, decode(&value)));
        } else {
            attrs.push((name, String::new()));
        }
    }
    Ok(attrs)
}

/// tagsoup's `escapeHTML`.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn render(tags: &[Tag]) -> String {
    let open = |out: &mut String, name: &str, attrs: &[(String, String)], shut: &str| {
        out.push('<');
        out.push_str(name);
        for (k, v) in attrs {
            out.push(' ');
            match (k.is_empty(), v.is_empty()) {
                (true, true) => out.push_str("\"\""),
                (false, true) => out.push_str(k),
                (true, false) => {
                    out.push('"');
                    out.push_str(&escape(v));
                    out.push('"');
                }
                (false, false) => {
                    out.push_str(k);
                    out.push_str("=\"");
                    out.push_str(&escape(v));
                    out.push('"');
                }
            }
        }
        out.push_str(shut);
        out.push('>');
    };
    let mut out = String::new();
    let mut i = 0;
    let mut raw: Option<&str> = None;
    while i < tags.len() {
        match &tags[i] {
            Tag::Open(name, attrs) => {
                if let Some(Tag::Close(n2)) = tags.get(i + 1)
                    && n2 == name
                    && MINIMIZED.contains(&name.as_str())
                {
                    open(&mut out, name, attrs, " /");
                    i += 2;
                    continue;
                }
                open(&mut out, name, attrs, "");
                if raw.is_none() && is_raw(name) {
                    raw = Some(name);
                }
            }
            Tag::Close(name) => {
                if raw == Some(name.as_str()) {
                    raw = None;
                }
                out.push_str("</");
                out.push_str(name);
                out.push('>');
            }
            Tag::Text(t) => {
                if raw.is_some() {
                    out.push_str(t);
                } else {
                    out.push_str(&escape(t));
                }
            }
            Tag::Comment(c) => {
                out.push_str("<!--");
                out.push_str(&c.replace("-->", "-- >"));
                out.push_str("-->");
            }
        }
        i += 1;
    }
    out
}

/// Compact stylesheet text the way pandoc's embedding does: comments go
/// (strings are not special), and each run of whitespace becomes its first
/// character.
fn compact_css(css: &str) -> Result<String> {
    if css.contains("url(") || css.contains("@import") {
        bail!("stylesheets with url() or @import are not supported yet");
    }
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(c) = rest.chars().next() {
        if let Some(body) = rest.strip_prefix("/*") {
            let end = body.find("*/").map_or(body.len(), |e| e + 2);
            rest = &body[end..];
        } else if c.is_whitespace() {
            out.push(c);
            rest = rest.trim_start_matches(char::is_whitespace);
        } else {
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    Ok(out)
}

/// The page with its linked stylesheets (paths relative to the current
/// directory) embedded.
pub fn embed(html: &str) -> Result<String> {
    let tags = tokenize(html)?;
    let mut out = Vec::with_capacity(tags.len());
    let mut i = 0;
    while i < tags.len() {
        match &tags[i] {
            Tag::Open(name, attrs) if name == "link" => {
                let get = |k: &str| attrs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
                if get("rel") != Some("stylesheet") {
                    bail!("cannot embed <link> without rel=\"stylesheet\"");
                }
                let href = get("href").context("<link rel=\"stylesheet\"> without href")?;
                let path = Path::new(href);
                let css = std::fs::read_to_string(path)
                    .with_context(|| format!("could not fetch resource {href}"))?;
                out.push(Tag::Open(
                    "style".to_owned(),
                    vec![("type".to_owned(), "text/css".to_owned())],
                ));
                let css = compact_css(&css)?;
                if !css.is_empty() {
                    out.push(Tag::Text(css));
                }
                out.push(Tag::Close("style".to_owned()));
                if matches!(tags.get(i + 1), Some(Tag::Close(n)) if n == "link") {
                    i += 1;
                }
            }
            Tag::Open(name, attrs) if name == "img" || name == "script" => {
                if attrs.iter().any(|(k, _)| k == "src") {
                    bail!("embedding <{name} src> is not supported yet");
                }
                out.push(tags[i].clone());
            }
            Tag::Text(t) if matches!(out.last(), Some(Tag::Open(n, _)) if n == "style") => {
                out.push(Tag::Text(compact_css(t)?));
            }
            other => out.push(other.clone()),
        }
        i += 1;
    }
    Ok(render(&out))
}

#[cfg(test)]
mod tests {
    use super::{compact_css, decode, render, tokenize};

    #[test]
    fn css_compaction() {
        let css = "/*a*/ /*b*/\n x{ content: \"/* c */  y\"; }  \n\n";
        assert_eq!(
            compact_css(css).expect("plain css"),
            " \nx{ content: \" y\"; } "
        );
    }

    #[test]
    fn retokenizing_normalizes_tags_and_escapes() {
        let html = "<!DOCTYPE html>\n<html lang=\"\" a=\"x\">\n<meta charset=\"utf-8\" />\n<p\nclass=\"c\">a \"b\" &amp; it's</p>";
        let out = render(&tokenize(html).expect("tokenizes"));
        assert_eq!(
            out,
            "<!DOCTYPE html>\n<html lang a=\"x\">\n<meta charset=\"utf-8\" />\n<p class=\"c\">a &quot;b&quot; &amp; it&#39;s</p>"
        );
        assert_eq!(decode("&copy;&#65;&bogus;"), "\u{a9}A&bogus;");
    }
}

#[cfg(test)]
mod more_tests {
    use super::{render, tokenize};

    fn roundtrip(html: &str) -> String {
        render(&tokenize(html).expect("tokenizes"))
    }

    #[test]
    fn raw_elements_keep_their_text() {
        assert_eq!(
            roundtrip("<script>if (a < b && c > \"d\") {}</script><style>p > a { x: '1' }</style>"),
            "<script>if (a < b && c > \"d\") {}</script><style>p > a { x: '1' }</style>"
        );
    }

    #[test]
    fn tags_and_attributes() {
        assert_eq!(
            roundtrip("<br/><hr /><img src='a.png' alt=x>"),
            "<br /><hr /><img src=\"a.png\" alt=\"x\">"
        );
        assert_eq!(
            roundtrip("<a title=\"&lt;x&gt; &amp; 'y'\">t</a>"),
            "<a title=\"&lt;x&gt; &amp; &#39;y&#39;\">t</a>"
        );
        assert_eq!(roundtrip("<p>a < b</p>"), "<p>a &lt; b</p>");
    }

    #[test]
    fn unterminated_markup_is_an_error() {
        assert!(tokenize("<p unterminated").is_err());
        assert!(tokenize("<!-- open").is_err());
        assert!(tokenize("<script>never closed").is_err());
    }
}
