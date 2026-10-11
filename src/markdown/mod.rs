//! A reader for pandoc's Markdown dialect, covering the constructs the
//! fleet's documents use: YAML front matter, ATX headers, paragraphs,
//! bullet lists, block quotes, HTML comments, and the inline syntax in
//! `inline.rs`. Written from pandoc's documented behaviour and observed
//! output, not from its source.
//!
//! Where pandoc's Markdown differs from CommonMark this follows pandoc: a
//! header, block quote or list needs a blank line before it unless it opens
//! its container or follows another block that ended (a header or an HTML
//! block), and a line that does not start a block continues the paragraph
//! it follows, even inside a list item ("lazy" continuation).

mod inline;
mod yaml;

use anyhow::Result;

use std::collections::HashMap;

use crate::ast::{Attr, Block, Document, Inline};

pub use inline::parse_inlines;

/// Link reference definitions: label (lowercased) to URL and title.
pub type Refs = HashMap<String, (String, String)>;

/// Parse a Markdown document.
pub fn read(text: &str) -> Result<Document> {
    let text = text.replace("\r\n", "\n").replace('\t', "    ");
    let lines: Vec<&str> = text.split('\n').collect();
    let (meta, body_start) = yaml::front_matter(&lines)?;
    let body: Vec<String> = lines[body_start..]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    // Links may use a definition that comes later, so the definitions are
    // collected by a first pass.
    let mut refs = Refs::new();
    parse_blocks(&body, false, &mut refs);
    let mut blocks = parse_blocks(&body, false, &mut refs);
    assign_identifiers(&mut blocks);
    Ok(Document { meta, blocks })
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// A bullet list marker line: (marker column, content).
fn bullet(line: &str) -> Option<(usize, &str)> {
    let ind = indent_of(line);
    if ind > 3 {
        return None;
    }
    let rest = &line[ind..];
    let mut chars = rest.chars();
    let m = chars.next()?;
    if !matches!(m, '*' | '-' | '+') {
        return None;
    }
    let after = &rest[1..];
    if after.is_empty() {
        return Some((ind, ""));
    }
    if !after.starts_with(' ') {
        return None;
    }
    // A horizontal rule (`* * *`) is not a list.
    if rest.chars().filter(|c| !c.is_whitespace()).all(|c| c == m)
        && rest.chars().filter(|&c| c == m).count() >= 3
    {
        return None;
    }
    let spaces = indent_of(after);
    let content = if spaces > 4 {
        &after[1..]
    } else {
        &after[spaces..]
    };
    Some((ind, content))
}

fn atx_header(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
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

fn parse_blocks(lines: &[String], in_item: bool, refs: &mut Refs) -> Vec<Block> {
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
        if fresh && indent_of(line) <= 3 {
            if let Some((level, text)) = atx_header(line.trim_start()) {
                blocks.push(Block::Header(
                    level,
                    Attr::default(),
                    parse_inlines(text, refs),
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
                blocks.push(Block::Quote(parse_blocks(&quoted, false, refs)));
                fresh = false;
                continue;
            }
        }
        if (fresh || in_item) && bullet(line).is_some() {
            let (list, next) = parse_list(lines, i, refs);
            blocks.push(list);
            i = next;
            fresh = false;
            continue;
        }
        let mut para = Vec::new();
        while i < lines.len() && !is_blank(&lines[i]) {
            if !para.is_empty() && in_item && bullet(&lines[i]).is_some() {
                break;
            }
            para.push(lines[i].trim_start());
            i += 1;
        }
        let text = para.join("\n");
        let inlines = parse_inlines(text.trim_end(), refs);
        // In a list item, only a paragraph a blank line follows is one;
        // the rest is plain text (until the list decides it is loose).
        if in_item && !lines.get(i).is_some_and(|l| is_blank(l)) {
            blocks.push(Block::Plain(inlines));
        } else {
            blocks.push(Block::Para(inlines));
        }
        fresh = false;
    }
    blocks
}

/// Parse the bullet list starting at `start`; returns it and the index of
/// the first line after it.
///
/// Blank lines between two items belong to the first, so its last
/// paragraph is a real one; blank lines after the last item belong to no
/// item. When any item has a real paragraph the list is loose and all its
/// items' plain text becomes paragraphs.
fn parse_list(lines: &[String], start: usize, refs: &mut Refs) -> (Block, usize) {
    let mut items: Vec<Vec<String>> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let Some((_, content)) = bullet(&lines[i]) else {
            break;
        };
        let mut item = vec![content.to_owned()];
        i += 1;
        let mut after_blank = false;
        while i < lines.len() {
            let l = &lines[i];
            if is_blank(l) {
                // Blank lines belong to the item only when indented content
                // follows them.
                let next = lines[i..].iter().position(|l| !is_blank(l)).map(|p| p + i);
                match next {
                    Some(n) if indent_of(&lines[n]) >= 4 => {
                        item.extend(std::iter::repeat_n(String::new(), n - i));
                        i = n;
                        after_blank = true;
                        continue;
                    }
                    _ => break,
                }
            }
            let ind = indent_of(l);
            if ind < 4 && bullet(l).is_some() {
                break;
            }
            if ind == 0 && after_blank {
                break;
            }
            item.push(dedent(l, 4));
            i += 1;
        }
        let next = lines[i..].iter().position(|l| !is_blank(l)).map(|p| p + i);
        match next {
            Some(n) if bullet(&lines[n]).is_some_and(|(ind, _)| ind < 4) => {
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
        .collect();
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
    (Block::BulletList(items), i)
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
            Block::BulletList(items) => {
                for item in items {
                    visit_headers(item, f);
                }
            }
            Block::Quote(bs) => visit_headers(bs, f),
            Block::Plain(_) | Block::Para(_) | Block::RawHtml(_) => {}
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
