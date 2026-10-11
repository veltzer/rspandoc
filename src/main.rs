//! rspandoc: a pandoc replacement, in Rust. It takes pandoc's command line
//! and produces the same document pandoc does for the documents and
//! options it supports (byte for byte for HTML, highlighting aside), and
//! refuses anything else rather than approximating it.

mod ast;
mod docx;
mod embed;
mod entities;
mod highlight;
mod html;
mod layout;
mod lua;
mod markdown;
mod template;

use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use clap::Parser;

/// Convert Markdown the way pandoc does (the part of it rspandoc supports).
#[derive(Debug, Parser)]
#[command(name = "rspandoc", disable_version_flag = true)]
struct Cli {
    /// Output format: `html` or `docx`.
    #[arg(
        short = 't',
        long = "to",
        value_name = "FORMAT",
        default_value = "html"
    )]
    to: String,

    /// Write the output here instead of to stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Produce a whole page (pandoc's default HTML template).
    #[arg(short = 's', long)]
    standalone: bool,

    /// Embed linked stylesheets in the page.
    #[arg(long)]
    embed_resources: bool,

    /// Link a stylesheet (repeatable).
    #[arg(short = 'c', long = "css", value_name = "URL")]
    css: Vec<String>,

    /// Run a Lua filter (repeatable; the slice of pandoc's Lua API in
    /// `src/lua.rs`).
    #[arg(long = "lua-filter", value_name = "FILE")]
    lua_filter: Vec<PathBuf>,

    /// Do not highlight code blocks.
    #[arg(long)]
    no_highlight: bool,

    /// The highlighting style: only pandoc's default, `pygments`.
    #[arg(long = "highlight-style", value_name = "STYLE")]
    highlight_style: Option<String>,

    /// Set a template variable (not supported yet).
    #[arg(short = 'V', long = "variable", value_name = "KEY[=VALUE]")]
    variable: Vec<String>,

    /// The PDF engine (not supported yet).
    #[arg(long = "pdf-engine", value_name = "PROGRAM")]
    pdf_engine: Option<String>,

    /// Print version information.
    #[arg(long)]
    version: bool,

    /// The Markdown input file.
    input: Option<PathBuf>,
}

fn print_version() {
    println!(
        "rspandoc {} by {}",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );
    println!("GIT_DESCRIBE: {}", env!("GIT_DESCRIBE"));
    println!("GIT_SHA: {}", env!("GIT_SHA"));
    println!("GIT_BRANCH: {}", env!("GIT_BRANCH"));
    println!("GIT_DIRTY: {}", env!("GIT_DIRTY"));
    println!("RUSTC_SEMVER: {}", env!("RUSTC_SEMVER"));
    println!("RUST_EDITION: {}", env!("RUST_EDITION"));
    println!("BUILD_TIMESTAMP: {}", env!("BUILD_TIMESTAMP"));
}

/// The output formats rspandoc writes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Html,
    Docx,
}

/// Convert `input` per the options; the output bytes.
fn convert(cli: &Cli, input: &Path) -> Result<Vec<u8>> {
    let format = match cli.to.as_str() {
        "html" | "html5" => Format::Html,
        "docx" => Format::Docx,
        other => bail!("output format `{other}` is not supported yet (html, docx)"),
    };
    if !cli.variable.is_empty() {
        bail!("-V/--variable is not supported yet");
    }
    if cli.pdf_engine.is_some() {
        bail!("--pdf-engine is not supported yet");
    }
    if let Some(style) = cli.highlight_style.as_deref()
        && style != "pygments"
    {
        bail!("--highlight-style {style} is not supported yet (pygments)");
    }
    let text = std::fs::read_to_string(input)
        .with_context(|| format!("could not read {}", input.display()))?;
    let doc =
        markdown::read(&text).with_context(|| format!("could not parse {}", input.display()))?;
    let lua_format = match format {
        Format::Html => "html",
        Format::Docx => "docx",
    };
    let doc = lua::run_filters(doc, &cli.lua_filter, lua_format, &input.to_string_lossy())?;
    if format == Format::Docx {
        // A DOCX file is always a whole document; like pandoc, rspandoc
        // ignores the HTML page options for it.
        if cli.output.is_none() {
            bail!("DOCX output is binary; give an output file with -o");
        }
        return docx::write(&doc, !cli.no_highlight);
    }
    if !cli.standalone {
        if !cli.css.is_empty() || cli.embed_resources {
            bail!("--css and --embed-resources need --standalone");
        }
        let mut w = html::Writer::new(!cli.no_highlight);
        let body = layout::render(Some(html::COLUMNS), &w.blocks(&doc.blocks)?);
        return Ok((body + "\n").into_bytes());
    }
    let opts = html::Options {
        css: cli.css.clone(),
        fallback_title: input
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        highlight: !cli.no_highlight,
    };
    let page = html::standalone(&doc, &opts)?;
    let page = if cli.embed_resources {
        embed::embed(&page)?
    } else {
        page
    };
    Ok(page.into_bytes())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.version {
        print_version();
        return Ok(());
    }
    let Some(input) = cli.input.clone() else {
        bail!("reading from stdin is not supported; give an input file");
    };
    let out = convert(&cli, &input)?;
    match &cli.output {
        Some(path) => std::fs::write(path, out)
            .with_context(|| format!("could not write {}", path.display()))?,
        None => std::io::stdout()
            .write_all(&out)
            .context("could not write to stdout")?,
    }
    Ok(())
}
