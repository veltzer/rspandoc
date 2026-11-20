//! PDF output: the standalone LaTeX document run through a TeX engine, as
//! pandoc does it. The engine runs in the current directory, so that paths
//! in the document (fonts, images) mean what they mean to pandoc, and
//! writes everything into a scratch directory that is removed afterwards.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, Result, bail};

/// The engine used when `--pdf-engine` is not given: pandoc's default.
pub const DEFAULT_ENGINE: &str = "pdflatex";

/// The engines rspandoc can drive: the three that take LaTeX's own
/// command line, and tectonic.
const ENGINES: &[&str] = &["lualatex", "pdflatex", "tectonic", "xelatex"];

/// How many times a LaTeX engine runs: the second run is what gives the
/// PDF its bookmarks, which the first one only collects.
const RUNS: usize = 2;

/// The base name of the files in the scratch directory.
const JOB: &str = "input";

/// Refuse an engine rspandoc cannot drive. `engine` may be a path; its
/// file name is what selects the command line.
pub fn check_engine(engine: &str) -> Result<()> {
    if ENGINES.contains(&engine_name(engine).as_str()) {
        return Ok(());
    }
    bail!(
        "--pdf-engine {engine} is not supported yet ({})",
        ENGINES.join(", ")
    )
}

fn engine_name(engine: &str) -> String {
    Path::new(engine)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// A new, empty directory under the system's temporary directory.
fn scratch_dir() -> Result<PathBuf> {
    let base = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("the clock is before 1970")?
        .as_nanos();
    let dir = base.join(format!("rspandoc-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&dir).with_context(|| format!("could not create {}", dir.display()))?;
    Ok(dir)
}

/// The engine's command for one run over `tex`, writing into `dir`.
fn engine_command(engine: &str, dir: &Path, tex: &Path) -> Command {
    let mut cmd = Command::new(engine);
    if engine_name(engine) == "tectonic" {
        cmd.arg("--outdir").arg(dir);
    } else {
        cmd.arg("-halt-on-error")
            .arg("-interaction")
            .arg("nonstopmode")
            .arg("-output-directory")
            .arg(dir);
    }
    cmd.arg(tex);
    cmd
}

/// What went wrong, from the engine's output: TeX reports an error on a
/// line starting with `!`, followed by the place it happened.
fn error_lines(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    match lines.iter().position(|l| l.starts_with('!')) {
        Some(at) => lines[at..].join("\n"),
        None => output.to_owned(),
    }
}

fn run_engine(engine: &str, dir: &Path, tex: &Path) -> Result<Vec<u8>> {
    // tectonic reruns the engine itself until the document settles.
    let runs = if engine_name(engine) == "tectonic" {
        1
    } else {
        RUNS
    };
    for _ in 0..runs {
        let out = engine_command(engine, dir, tex)
            .output()
            .with_context(|| format!("could not run the PDF engine {engine}"))?;
        if !out.status.success() {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            bail!(
                "the PDF engine {engine} failed:\n{}",
                error_lines(&text).trim_end()
            );
        }
    }
    let pdf = dir.join(format!("{JOB}.pdf"));
    std::fs::read(&pdf).with_context(|| format!("{engine} wrote no PDF ({})", pdf.display()))
}

/// The PDF `engine` makes of the LaTeX document `latex`.
pub fn make(latex: &str, engine: &str) -> Result<Vec<u8>> {
    check_engine(engine)?;
    let dir = scratch_dir()?;
    let tex = dir.join(format!("{JOB}.tex"));
    let result = std::fs::write(&tex, latex)
        .with_context(|| format!("could not write {}", tex.display()))
        .and_then(|()| run_engine(engine, &dir, &tex));
    std::fs::remove_dir_all(&dir).with_context(|| format!("could not remove {}", dir.display()))?;
    result
}
