//! Golden tests: rspandoc against pandoc 3.7.0.2, whose outputs are
//! recorded under `tests/golden/`. Regenerate them with pandoc, never by
//! editing them; a difference is a bug in rspandoc unless pandoc's
//! behaviour is what changed.
//!
//! HTML must match byte for byte except for syntax highlighting, which is
//! syntect's rather than pandoc's: both sides have their highlighted code
//! blocks turned back into plain ones before comparing
//! ([`without_highlighting`], as `scripts/diff-pandoc.py` does), and
//! `highlight/` pins rspandoc's own highlighted output.
//!
//! LaTeX is held to the same rule: highlighted blocks (`Shaded`) are
//! reduced to their text on both sides ([`without_latex_highlighting`]).
//!
//! - `*.md`: Markdown fragments, one theme each (`pandoc --to html NAME.md`
//!   and `pandoc --to latex NAME.md`).
//! - `latex/cases.txt`: standalone LaTeX documents, template variables and
//!   a Lua filter that adds to the preamble, one `NAME | ARGUMENTS` case
//!   per line.
//! - `pages/cases.txt`: standalone pages, stylesheets, embedding and Lua
//!   filters, one `NAME | ARGUMENTS` case per line (`pandoc ARGUMENTS`).
//! - `standalone/`: a real syllabus built the way teaching-syllabi builds it.
//! - `corpus/`: real fleet documents, each built as a whole page with
//!   `standalone/`'s stylesheet and Lua filter.
//! - `template/`: the template language (`pandoc --template=t.html data.md`),
//!   checked by a unit test in `src/template.rs`.
//! - `docx/`: rspandoc's own DOCX parts, recorded after checking that
//!   pandoc's DOCX reader reads rspandoc's file and pandoc's as the same
//!   document.

use std::path::Path;
use std::process::Command;

/// `html` with each highlighted code block as the plain `<pre
/// class="LANG"><code>TEXT</code></pre>` pandoc writes for a language it
/// does not know, and without the highlighting stylesheet.
fn without_highlighting(html: &str) -> String {
    let mut s = html.to_owned();
    if let Some(start) = s.find("/* CSS for syntax highlighting */")
        && let Some(len) = s[start..].find("/* Warning */")
    {
        let from = s[..start].trim_end_matches(' ').len() - 1;
        s.replace_range(from..start + len + "/* Warning */".len(), "");
    }
    let mut out = String::new();
    let mut rest = s.as_str();
    while let Some(at) = rest.find(r#"<div class="sourceCode""#) {
        out.push_str(&rest[..at]);
        let block = &rest[at..];
        let end = block
            .find("</code></pre></div>")
            .expect("highlighted block ends");
        let lang_at = block.find(r#"class="sourceCode "#).expect("pre class") + 18;
        let lang = &block[lang_at..lang_at + block[lang_at..].find('"').expect("class ends")];
        let code_at = block.find("<code").expect("code element");
        let code_at = code_at + block[code_at..].find('>').expect("code tag ends") + 1;
        let mut text = String::new();
        let mut in_tag = false;
        for c in block[code_at..end].chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => text.push(c),
                _ => {}
            }
        }
        out.push_str(&format!(r#"<pre class="{lang}"><code>{text}</code></pre>"#));
        rest = &block[end + "</code></pre></div>".len()..];
    }
    out.push_str(rest);
    out
}

/// Compare rspandoc's HTML with pandoc's, highlighting aside.
#[track_caller]
fn assert_html_eq(got: &str, expected: &str, what: &str) {
    assert_eq!(
        without_highlighting(got),
        without_highlighting(expected),
        "{what}"
    );
}

/// `tex` with each highlighted code block (`Shaded`) reduced to its text:
/// the token commands and the braces they add are dropped, as
/// `scripts/diff-pandoc.py --latex` does.
fn without_latex_highlighting(tex: &str) -> String {
    const OPEN: &str = "\\begin{Shaded}\n\\begin{Highlighting}[]\n";
    const CLOSE: &str = "\\end{Highlighting}";
    let mut out = String::new();
    let mut rest = tex;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let block = &rest[at + OPEN.len()..];
        let end = block.find(CLOSE).expect("highlighted block ends");
        out.push_str("[code]\n");
        let mut chars = block[..end].chars().peekable();
        let mut escaped = false;
        let mut word = String::new();
        while let Some(c) = chars.next() {
            if escaped {
                // A token command (`\KeywordTok{`) goes; any other
                // command or escaped character stays.
                if c.is_ascii_alphabetic() {
                    word.push(c);
                    if chars.peek().is_some_and(char::is_ascii_alphabetic) {
                        continue;
                    }
                    if word.ends_with("Tok") && chars.peek() == Some(&'{') {
                        chars.next();
                    } else {
                        out.push('\\');
                        out.push_str(&word);
                    }
                    word.clear();
                } else {
                    out.push('\\');
                    out.push(c);
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c != '{' && c != '}' {
                out.push(c);
            }
        }
        out.push_str("[/code]");
        rest = &block[end..];
    }
    out.push_str(rest);
    out
}

/// Compare rspandoc's LaTeX with pandoc's, highlighting aside.
#[track_caller]
fn assert_latex_eq(got: &str, expected: &str, what: &str) {
    assert_eq!(
        without_latex_highlighting(got),
        without_latex_highlighting(expected),
        "{what}"
    );
}

fn rspandoc(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("rspandoc runs");
    assert!(
        out.status.success(),
        "rspandoc {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("output is UTF-8")
}

#[test]
fn fragments_match_pandoc() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let mut checked = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("tests/golden exists")
        .map(|e| e.expect("directory entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    entries.sort();
    for md in entries {
        let name = md.file_name().and_then(|n| n.to_str()).expect("UTF-8 name");
        let expected =
            std::fs::read_to_string(md.with_extension("html")).expect("expected output exists");
        assert_html_eq(&rspandoc(&dir, &["--to", "html", name]), &expected, name);
        // The same fragment as LaTeX (`pandoc --to latex NAME.md`).
        let expected =
            std::fs::read_to_string(md.with_extension("tex")).expect("expected LaTeX exists");
        assert_latex_eq(&rspandoc(&dir, &["--to", "latex", name]), &expected, name);
        checked += 1;
    }
    assert!(checked >= 8, "only {checked} golden files found");
}

/// `tests/golden/latex/cases.txt`: whole LaTeX documents, template
/// variables and a Lua filter that adds to the preamble, one case per
/// line as `NAME | ARGUMENTS`, against pandoc's `NAME.tex`.
#[test]
fn latex_documents_match_pandoc() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/latex");
    let cases = std::fs::read_to_string(dir.join("cases.txt")).expect("cases.txt exists");
    let mut checked = 0;
    for line in cases
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        let (name, args) = line.split_once('|').expect("NAME | ARGUMENTS");
        let name = name.trim();
        let args: Vec<&str> = args.split_whitespace().collect();
        let expected = std::fs::read_to_string(dir.join(format!("{name}.tex")))
            .expect("expected output exists");
        assert_latex_eq(&rspandoc(&dir, &args), &expected, name);
        checked += 1;
    }
    assert!(checked >= 12, "only {checked} cases found");
}

/// `tests/golden/pages/cases.txt`: whole pages and Lua filters, one case
/// per line as `NAME | ARGUMENTS`, against pandoc's `NAME.html`.
#[test]
fn pages_match_pandoc() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/pages");
    let cases = std::fs::read_to_string(dir.join("cases.txt")).expect("cases.txt exists");
    let mut checked = 0;
    for line in cases
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        let (name, args) = line.split_once('|').expect("NAME | ARGUMENTS");
        let name = name.trim();
        let args: Vec<&str> = args.split_whitespace().collect();
        let expected = std::fs::read_to_string(dir.join(format!("{name}.html")))
            .expect("expected output exists");
        assert_html_eq(&rspandoc(&dir, &args), &expected, name);
        checked += 1;
    }
    assert!(checked >= 9, "only {checked} cases found");
}

#[test]
fn standalone_page_matches_pandoc() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/standalone");
    let expected =
        std::fs::read_to_string(dir.join("expected.html")).expect("expected output exists");
    let got = rspandoc(
        &dir,
        &[
            "--to",
            "html",
            "--standalone",
            "--embed-resources",
            "--css=style.css",
            "--lua-filter=filter.lua",
            "syllabi/courses/ai/advanced_ai_powered_development.md",
        ],
    );
    assert_html_eq(&got, &expected, "standalone");
}

#[test]
fn unsupported_markdown_is_refused() {
    let dir = std::env::temp_dir().join(format!("rspandoc-refusal-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    for (i, text) in [
        "x $a+b$ y",
        "![i](p.png)",
        "a. A letter",
        "``` {.python #main}\ncode\n```",
        "| a | b |\n|---|---|\n| 1 | 2 |\n\nTable: caption",
        "```\nunclosed",
        "text\n---",
        "use \\n here",
    ]
    .iter()
    .enumerate()
    {
        let f = dir.join(format!("{i}.md"));
        std::fs::write(&f, text).expect("write input");
        let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
            .args(["--to", "html"])
            .arg(&f)
            .output()
            .expect("rspandoc runs");
        assert!(!out.status.success(), "{text:?} was accepted");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("unsupported Markdown"),
            "{text:?}"
        );
    }
    std::fs::remove_dir_all(&dir).expect("remove temp dir");
}

/// `tests/golden/highlight/`: rspandoc's own highlighted page, recorded
/// after review, since syntect's tokens are not pandoc's. A change here is
/// a change in how code is coloured: look at it, then record it again.
#[test]
fn highlighting_matches_recorded_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/highlight");
    let expected = std::fs::read_to_string(dir.join("expected.html")).expect("expected output");
    let got = rspandoc(&dir, &["--to", "html", "--standalone", "input.md"]);
    assert_eq!(got, expected);
    // pandoc's page for the same input, highlighting aside.
    let pandoc_page = std::fs::read_to_string(dir.join("pandoc.html")).expect("pandoc's output");
    assert_html_eq(&got, &pandoc_page, "highlight page");
    // The same for LaTeX: rspandoc's own tokens recorded, and pandoc's
    // document for the same input, highlighting aside.
    let expected = std::fs::read_to_string(dir.join("expected.tex")).expect("expected output");
    let got = rspandoc(&dir, &["--to", "latex", "--standalone", "input.md"]);
    assert_eq!(got, expected);
    let pandoc_doc = std::fs::read_to_string(dir.join("pandoc.tex")).expect("pandoc's output");
    assert_latex_eq(&got, &pandoc_doc, "highlight document");
}

/// The named parts of a DOCX package.
fn docx_parts(bytes: &[u8], names: &[&str]) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a zip package");
    names
        .iter()
        .map(|name| {
            let mut part = zip.by_name(name).expect("part exists");
            let mut text = String::new();
            std::io::Read::read_to_string(&mut part, &mut text).expect("part is UTF-8");
            ((*name).to_owned(), text)
        })
        .collect()
}

/// DOCX output against the parts recorded in `tests/golden/docx/NAME/`.
/// rspandoc's styles are its own, so these are not pandoc's bytes; they
/// were checked by reading both rspandoc's and pandoc's DOCX back with
/// pandoc's DOCX reader and getting the same document.
#[test]
fn docx_matches_recorded_parts() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/docx");
    let parts = [
        "docProps/core.xml",
        "word/document.xml",
        "word/numbering.xml",
        "word/styles.xml",
        "word/_rels/document.xml.rels",
    ];
    let out_dir = std::env::temp_dir().join(format!("rspandoc-docx-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).expect("temp dir");
    let cases: [(&str, &[&str]); 5] = [
        ("basic", &["basic.md"]),
        ("blocks", &["blocks.md"]),
        ("tables", &["tables.md"]),
        ("rtl", &["rtl.md"]),
        (
            "filtered",
            &["--lua-filter=../pages/globals.lua", "../pages/lua.md"],
        ),
    ];
    for (name, input) in cases {
        let out = out_dir.join(format!("{name}.docx"));
        let out_arg = out.to_str().expect("UTF-8 path");
        let mut args = vec!["--to", "docx", "-o", out_arg];
        args.extend_from_slice(input);
        rspandoc(&dir, &args);
        let first = std::fs::read(&out).expect("output written");
        rspandoc(&dir, &args);
        assert_eq!(
            first,
            std::fs::read(&out).expect("output written"),
            "{name}: not deterministic"
        );
        for (part, got) in docx_parts(&first, &parts) {
            let expected =
                std::fs::read_to_string(dir.join(name).join(&part)).expect("recorded part exists");
            assert_eq!(got, expected, "{name}: {part}");
        }
    }
    std::fs::remove_dir_all(&out_dir).expect("remove temp dir");
}

/// `tests/golden/corpus/`: real documents from the fleet (the syllabi and
/// CVs that exposed bugs during development), each built as a whole page
/// with a stylesheet embedded and teaching-syllabi's Lua filter, against
/// pandoc's output for the same arguments.
#[test]
fn corpus_sample_matches_pandoc() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/corpus");
    let mut docs: Vec<_> = std::fs::read_dir(&dir)
        .expect("tests/golden/corpus exists")
        .map(|e| e.expect("directory entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    docs.sort();
    for md in &docs {
        let name = md.file_name().and_then(|n| n.to_str()).expect("UTF-8 name");
        let expected =
            std::fs::read_to_string(md.with_extension("html")).expect("expected output exists");
        let got = rspandoc(
            &dir,
            &[
                "--to",
                "html",
                "--standalone",
                "--embed-resources",
                "--css=../standalone/style.css",
                "--lua-filter=../standalone/filter.lua",
                name,
            ],
        );
        assert_html_eq(&got, &expected, name);
        // The same document as LaTeX, with the same filter.
        let expected =
            std::fs::read_to_string(md.with_extension("tex")).expect("expected LaTeX exists");
        let got = rspandoc(
            &dir,
            &[
                "--to",
                "latex",
                "--standalone",
                "--lua-filter=../standalone/filter.lua",
                name,
            ],
        );
        assert_latex_eq(&got, &expected, name);
    }
    assert!(docs.len() >= 15, "only {} documents found", docs.len());
}

/// `--reference-doc`: the output takes the reference document's style
/// sheet (with the styles rspandoc writes that it lacks added), its font
/// table and its page setup. `docx/reference/` holds the parts recorded
/// for `docx/reference.docx`, the fleet's azure theme.
#[test]
fn docx_takes_its_styles_from_the_reference_document() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/docx");
    let out_dir = std::env::temp_dir().join(format!("rspandoc-docx-ref-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).expect("temp dir");
    let out = out_dir.join("out.docx");
    rspandoc(
        &dir,
        &[
            "--to",
            "docx",
            "--reference-doc=reference.docx",
            "-o",
            out.to_str().expect("UTF-8 path"),
            "rtl.md",
        ],
    );
    let bytes = std::fs::read(&out).expect("output written");
    let parts = [
        "[Content_Types].xml",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "word/fontTable.xml",
    ];
    for (part, got) in docx_parts(&bytes, &parts) {
        let expected = std::fs::read_to_string(dir.join("reference").join(&part))
            .expect("recorded part exists");
        assert_eq!(got, expected, "{part}");
    }
    // The reference's styles are kept as they are, and the document's
    // language is set in its defaults.
    let reference = std::fs::read(dir.join("reference.docx")).expect("reference exists");
    let theirs = &docx_parts(&reference, &["word/styles.xml", "word/fontTable.xml"]);
    let ours = &docx_parts(&bytes, &["word/styles.xml", "word/fontTable.xml"]);
    assert_eq!(ours[1].1, theirs[1].1, "font table is the reference's");
    assert!(ours[0].1.contains(r#"w:ascii="Outfit""#));
    assert!(ours[0].1.contains(r#"w:styleId="SourceCode""#));
    assert!(!theirs[0].1.contains(r#"w:styleId="SourceCode""#));
    assert!(ours[0].1.contains(r#"w:bidi="he""#), "language is set");
    std::fs::remove_dir_all(&out_dir).expect("remove temp dir");
}
