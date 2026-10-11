//! Golden tests: rspandoc's output against pandoc 3.7.0.2's, recorded in
//! `tests/golden/`. Each `NAME.md` there is converted with `--to html` and
//! must equal `NAME.html` byte for byte; `standalone/` is a whole page
//! with a stylesheet embedded and a Lua filter applied. The expected files
//! were produced by pandoc; regenerate them with pandoc, never by hand.

use std::path::Path;
use std::process::Command;

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
        assert_eq!(rspandoc(&dir, &["--to", "html", name]), expected, "{name}");
        checked += 1;
    }
    assert!(checked >= 8, "only {checked} golden files found");
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
    assert_eq!(got, expected);
}

#[test]
fn unsupported_markdown_is_refused() {
    let dir = std::env::temp_dir().join(format!("rspandoc-refusal-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    for (i, text) in [
        "x $a+b$ y",
        "![i](p.png)",
        "1986. A year",
        "para\n```\ncode\n```",
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
