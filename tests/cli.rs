//! The command line's refusals: every unsupported option, input or filter
//! must stop rspandoc with an error that says what was refused, never
//! produce output that quietly differs from pandoc's.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A scratch directory with the inputs the cases use.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("rspandoc-cli-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let files = [
            ("ok.md", "x\n"),
            ("author.md", "---\nauthor: Me\n---\nx\n"),
            ("flow.md", "---\nlist: [a, b]\n---\nx\n"),
            ("image.lua", "function Image(el) return el end\n"),
            ("boom.lua", "function Str(el) error('boom') end\n"),
            ("bad.lua", "function Str(el) return 42 end\n"),
            ("syntax.lua", "syntax error here(\n"),
            ("table.md", "| a | b |\n|---|---|\n| 1 | 2 |\n"),
            ("noop.lua", "function Str(el) return el end\n"),
        ];
        for (file, text) in files {
            std::fs::write(dir.join(file), text).expect("write fixture");
        }
        Self { dir }
    }

    fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Run rspandoc in `dir`; its exit success and standard error.
fn run(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("rspandoc runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn refusals_name_what_was_refused() {
    let fx = Fixture::new("refusals");
    let cases: &[(&[&str], &str)] = &[
        (
            &["--to", "pdf", "ok.md"],
            "output format `pdf` is not supported",
        ),
        (&["-V", "x=y", "ok.md"], "-V/--variable is not supported"),
        (
            &["--pdf-engine=xelatex", "ok.md"],
            "--pdf-engine is not supported",
        ),
        (&["nothere.md"], "could not read nothere.md"),
        (&["--to", "docx", "ok.md"], "give an output file with -o"),
        (&["--css=a.css", "ok.md"], "need --standalone"),
        (&["--standalone", "author.md"], "metadata field `author`"),
        (
            &["--to", "docx", "-o", "out.docx", "author.md"],
            "metadata field `author`",
        ),
        (&["flow.md"], "unsupported YAML value"),
        (
            &["--lua-filter=image.lua", "ok.md"],
            "hook `Image` is not supported",
        ),
        (&["--lua-filter=boom.lua", "ok.md"], "boom"),
        (&["--lua-filter=bad.lua", "ok.md"], "returned a Lua integer"),
        (&["--lua-filter=syntax.lua", "ok.md"], "failed to load"),
        (
            &["--lua-filter=noop.lua", "table.md"],
            "tables are not supported with Lua filters",
        ),
        (
            &["--highlight-style=tango", "ok.md"],
            "--highlight-style tango is not supported",
        ),
        (
            &["--lua-filter=missing.lua", "ok.md"],
            "could not read Lua filter",
        ),
        (&[], "reading from stdin is not supported"),
        (
            &[
                "--standalone",
                "--embed-resources",
                "--css=missing.css",
                "ok.md",
            ],
            "could not fetch resource missing.css",
        ),
    ];
    for (args, message) in cases {
        let (ok, stderr) = run(fx.path(), args);
        assert!(!ok, "{args:?} succeeded");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
    }
}

#[test]
fn version_names_the_build() {
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .arg("--version")
        .output()
        .expect("rspandoc runs");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with(&format!("rspandoc {} by ", env!("CARGO_PKG_VERSION"))));
    for key in [
        "GIT_DESCRIBE",
        "GIT_SHA",
        "GIT_BRANCH",
        "GIT_DIRTY",
        "RUSTC_SEMVER",
        "RUST_EDITION",
        "BUILD_TIMESTAMP",
    ] {
        assert!(text.contains(&format!("{key}: ")), "{key} missing");
    }
}

#[test]
fn output_file_matches_stdout() {
    let fx = Fixture::new("output");
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .current_dir(fx.path())
        .args(["--to", "html", "ok.md"])
        .output()
        .expect("rspandoc runs");
    let (ok, stderr) = run(fx.path(), &["--to", "html", "ok.md", "-o", "ok.html"]);
    assert!(ok, "{stderr}");
    let written = std::fs::read(fx.path().join("ok.html")).expect("output written");
    assert_eq!(written, out.stdout);
}
