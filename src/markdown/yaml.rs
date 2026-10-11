//! The YAML front matter of a Markdown document: the subset documents in
//! the fleet use, a mapping of plain scalars and block sequences of plain
//! scalars. Anything else is rejected rather than misread. As in pandoc,
//! string values are themselves Markdown, and `true`/`false` are booleans.

use anyhow::{Result, bail};

use crate::ast::MetaValue;

use super::{Refs, parse_inlines};

/// The metadata in a front matter block opening the document, and the
/// index of the first line after it (0 when there is none).
pub fn front_matter(lines: &[&str]) -> Result<(Vec<(String, MetaValue)>, usize)> {
    if lines.first().is_none_or(|l| l.trim_end() != "---") {
        return Ok((Vec::new(), 0));
    }
    let Some(end) = lines[1..]
        .iter()
        .position(|l| matches!(l.trim_end(), "---" | "..."))
        .map(|p| p + 1)
    else {
        return Ok((Vec::new(), 0));
    };
    // pandoc reads front matter only when a blank line does not follow the
    // opening delimiter.
    if lines.get(1).is_some_and(|l| l.trim().is_empty()) {
        return Ok((Vec::new(), 0));
    }
    let mut meta: Vec<(String, MetaValue)> = Vec::new();
    let mut i = 1;
    while i < end {
        let line = lines[i];
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            i += 1;
            continue;
        }
        if line.starts_with(' ') {
            bail!(
                "front matter line {}: unexpected indentation: {line:?}",
                i + 1
            );
        }
        let Some((key, value)) = line.split_once(':') else {
            bail!(
                "front matter line {}: expected `key: value`: {line:?}",
                i + 1
            );
        };
        let key = key.trim().to_owned();
        let value = value.trim();
        i += 1;
        let val = if value.is_empty() {
            let mut items = Vec::new();
            while i < end {
                let l = lines[i];
                let t = l.trim_start();
                if l.trim().is_empty() {
                    i += 1;
                    continue;
                }
                let Some(item) = t.strip_prefix("- ").or_else(|| (t == "-").then_some("")) else {
                    break;
                };
                if !l.starts_with(' ') && !l.starts_with('-') {
                    break;
                }
                items.push(scalar(item.trim(), i + 1)?);
                i += 1;
            }
            MetaValue::List(items)
        } else {
            scalar(value, i)?
        };
        meta.retain(|(k, _)| *k != key);
        meta.push((key, val));
    }
    meta.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((meta, end + 1))
}

fn scalar(s: &str, line: usize) -> Result<MetaValue> {
    if s.starts_with(['[', '{', '|', '>', '&', '*', '!']) {
        bail!("front matter line {line}: unsupported YAML value: {s:?}");
    }
    let unquoted = if (s.starts_with('"') && s.ends_with('"')
        || s.starts_with('\'') && s.ends_with('\''))
        && s.len() >= 2
    {
        &s[1..s.len() - 1]
    } else {
        s
    };
    Ok(match unquoted {
        "true" | "True" | "TRUE" if unquoted.len() == s.len() => MetaValue::Bool(true),
        "false" | "False" | "FALSE" if unquoted.len() == s.len() => MetaValue::Bool(false),
        _ => MetaValue::Inlines(parse_inlines(unquoted, &Refs::new())?),
    })
}

#[cfg(test)]
mod tests {
    use super::front_matter;
    use crate::ast::MetaValue;

    fn meta(text: &str) -> anyhow::Result<(Vec<(String, MetaValue)>, usize)> {
        let lines: Vec<&str> = text.split('\n').collect();
        front_matter(&lines)
    }

    #[test]
    fn scalars_lists_and_booleans() {
        let (m, end) = meta("---\nb: 'quoted'\na:\n  - x\n  - y\nf: false\nt: true\n---\nbody")
            .expect("valid");
        assert_eq!(end, 8);
        let keys: Vec<&str> = m.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["a", "b", "f", "t"]);
        assert!(matches!(&m[0].1, MetaValue::List(xs) if xs.len() == 2));
        assert_eq!(m[2].1, MetaValue::Bool(false));
        assert_eq!(m[3].1, MetaValue::Bool(true));
    }

    #[test]
    fn not_front_matter() {
        assert_eq!(meta("no front matter").expect("valid").1, 0);
        assert_eq!(meta("---\n\nblank after opener\n---").expect("valid").1, 0);
        assert_eq!(meta("---\nunterminated: x").expect("valid").1, 0);
    }

    #[test]
    fn unsupported_yaml_is_refused() {
        for text in [
            "---\nx: [a]\n---",
            "---\nx: {a: 1}\n---",
            "---\nx: |\n  a\n---",
            "---\n  indented: x\n---",
            "---\nno colon\n---",
        ] {
            assert!(meta(text).is_err(), "{text:?}");
        }
    }
}
