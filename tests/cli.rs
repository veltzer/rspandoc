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
            ("lang.md", "---\nlang: tlh\n---\nx\n"),
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

/// Run `rspandoc convert` in `dir`; its exit success and standard error.
fn run(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .current_dir(dir)
        .arg("convert")
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
            &["--to", "rtf", "ok.md"],
            "output format `rtf` is not supported",
        ),
        (&["--to", "pdf", "ok.md"], "give an output file with -o"),
        (&["-V", "x=y", "ok.md"], "-V/--variable is not supported"),
        (
            &["--pdf-engine=xelatex", "ok.md"],
            "--pdf-engine needs PDF output",
        ),
        (
            &["--pdf-engine=wkhtmltopdf", "-o", "out.pdf", "ok.md"],
            "--pdf-engine wkhtmltopdf is not supported",
        ),
        (
            &["--to", "latex", "--standalone", "author.md"],
            "metadata field `author`",
        ),
        (
            &["--to", "latex", "--standalone", "lang.md"],
            "language `tlh` is not supported",
        ),
        (
            &[
                "--to",
                "docx",
                "-o",
                "out.docx",
                "--reference-doc=ok.md",
                "ok.md",
            ],
            "is not a DOCX file",
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
        (&[], "<INPUT>"),
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
        .arg("version")
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
        .args(["convert", "--to", "html", "ok.md"])
        .output()
        .expect("rspandoc runs");
    let (ok, stderr) = run(fx.path(), &["--to", "html", "ok.md", "-o", "ok.html"]);
    assert!(ok, "{stderr}");
    let written = std::fs::read(fx.path().join("ok.html")).expect("output written");
    assert_eq!(written, out.stdout);
}

/// A stand-in for a TeX engine, so the PDF step is tested without TeX: it
/// takes LaTeX's command line, appends a line to `runs.log` beside the
/// script for each run, and writes the document it was given as the
/// "PDF". `fail` makes it report a TeX error instead.
fn fake_engine(dir: &Path, name: &str, fail: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;
    let body = if fail {
        "echo 'This is a fake engine'; echo '! Undefined control sequence.'; echo 'l.3 \\nope'; exit 1"
    } else {
        "cp \"$tex\" \"$out/input.pdf\""
    };
    let script = format!(
        "#!/bin/sh\nhere=$(dirname \"$0\")\necho \"$@\" >> \"$here/runs.log\"\nout=\nwhile [ $# -gt 1 ]; do\n  case \"$1\" in -output-directory|--outdir) out=$2; shift;; esac\n  shift\ndone\ntex=$1\n{body}\n"
    );
    let path = dir.join(name);
    std::fs::write(&path, script).expect("write fake engine");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
        .expect("make fake engine executable");
    path
}

#[test]
fn pdf_is_the_latex_document_run_through_the_engine() {
    let fx = Fixture::new("pdf");
    let engine = fake_engine(fx.path(), "xelatex", false);
    let engine_arg = format!("--pdf-engine={}", engine.display());
    let (ok, stderr) = run(fx.path(), &[&engine_arg, "ok.md", "-o", "ok.pdf"]);
    assert!(ok, "{stderr}");
    // The fake engine's "PDF" is the document it was handed: the same
    // one `--to latex --standalone` writes.
    let latex = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .current_dir(fx.path())
        .args(["convert", "--to", "latex", "--standalone", "ok.md"])
        .output()
        .expect("rspandoc runs");
    let pdf = std::fs::read(fx.path().join("ok.pdf")).expect("output written");
    assert_eq!(pdf, latex.stdout);
    // LaTeX runs twice (the second run writes the bookmarks), without
    // stopping for input, into a scratch directory that is gone afterwards.
    let log = std::fs::read_to_string(fx.path().join("runs.log")).expect("engine ran");
    let runs: Vec<&str> = log.lines().collect();
    assert_eq!(runs.len(), 2, "{log}");
    assert!(runs[0].starts_with("-halt-on-error -interaction nonstopmode -output-directory "));
    let scratch = runs[0]
        .split_whitespace()
        .nth(4)
        .expect("output directory argument");
    assert!(!Path::new(scratch).exists(), "{scratch} was left behind");
}

#[test]
fn tectonic_runs_once_with_its_own_arguments() {
    let fx = Fixture::new("tectonic");
    let engine = fake_engine(fx.path(), "tectonic", false);
    let engine_arg = format!("--pdf-engine={}", engine.display());
    let (ok, stderr) = run(
        fx.path(),
        &["--to", "pdf", &engine_arg, "ok.md", "-o", "out.bin"],
    );
    assert!(ok, "{stderr}");
    let log = std::fs::read_to_string(fx.path().join("runs.log")).expect("engine ran");
    assert_eq!(log.lines().count(), 1, "{log}");
    assert!(log.starts_with("--outdir "), "{log}");
}

#[test]
fn a_failing_engine_reports_the_tex_error() {
    let fx = Fixture::new("pdf-fail");
    let engine = fake_engine(fx.path(), "pdflatex", true);
    let engine_arg = format!("--pdf-engine={}", engine.display());
    let (ok, stderr) = run(fx.path(), &[&engine_arg, "ok.md", "-o", "ok.pdf"]);
    assert!(!ok);
    assert!(stderr.contains("! Undefined control sequence."), "{stderr}");
    assert!(stderr.contains("l.3"), "{stderr}");
    // The error starts at TeX's `!` line, not at the engine's banner.
    assert!(!stderr.contains("This is a fake engine"), "{stderr}");
    assert!(!fx.path().join("ok.pdf").exists());
    // One failed run is enough.
    let log = std::fs::read_to_string(fx.path().join("runs.log")).expect("engine ran");
    assert_eq!(log.lines().count(), 1, "{log}");
}

#[test]
fn the_output_extension_selects_the_format() {
    let fx = Fixture::new("extension");
    let (ok, stderr) = run(fx.path(), &["ok.md", "-o", "ok.tex"]);
    assert!(ok, "{stderr}");
    let tex = std::fs::read_to_string(fx.path().join("ok.tex")).expect("output written");
    assert_eq!(tex, "x\n");
    // `--to` wins over the extension.
    let (ok, stderr) = run(fx.path(), &["--to", "html", "ok.md", "-o", "ok.tex"]);
    assert!(ok, "{stderr}");
    let html = std::fs::read_to_string(fx.path().join("ok.tex")).expect("output written");
    assert_eq!(html, "<p>x</p>\n");
}

/// Run rspandoc with `args` as they are; its exit code, stdout and stderr.
fn bare(args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .args(args)
        .output()
        .expect("rspandoc runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn without_a_command_the_commands_are_listed() {
    let (code, stdout, stderr) = bare(&[]);
    assert_eq!(code, Some(2));
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.starts_with("rspandoc needs a command:\n"),
        "{stderr}"
    );
    assert!(stderr.contains("\n  convert  "), "{stderr}");
    assert!(stderr.contains("\n  version  "), "{stderr}");
    // The commands and nothing else: the options are in --help, which the
    // last line points at.
    assert_eq!(stderr.lines().count(), 4, "{stderr}");
    assert!(stderr.ends_with("Run 'rspandoc --help' for the options.\n"));
}

#[test]
fn pandoc_s_bare_command_line_is_refused() {
    // Options belong to `convert`; without it nothing is converted.
    let fx = Fixture::new("bare");
    let out = Command::new(env!("CARGO_BIN_EXE_rspandoc"))
        .current_dir(fx.path())
        .args(["--to", "html", "ok.md"])
        .output()
        .expect("rspandoc runs");
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn the_version_flag_names_the_version() {
    let (code, stdout, _) = bare(&["--version"]);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, format!("rspandoc {}\n", env!("CARGO_PKG_VERSION")));
}
