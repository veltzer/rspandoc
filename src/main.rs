//! rspandoc: a subset of pandoc, in Rust. It takes pandoc's command line
//! and produces byte-identical output for the documents and options it
//! supports, and refuses anything else rather than approximating it.

mod ast;
mod embed;
mod entities;
mod html;
mod layout;
mod lua;
mod markdown;
mod template;

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use clap::Parser;

/// Convert Markdown the way pandoc does (a supported subset of it).
#[derive(Debug, Parser)]
#[command(name = "rspandoc", disable_version_flag = true)]
struct Cli {
    /// Output format (only `html` so far).
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

/// Convert `input` per the options; the output text.
fn convert(cli: &Cli, input: &Path) -> Result<String> {
    if cli.to != "html" && cli.to != "html5" {
        bail!(
            "output format `{}` is not supported yet (only html)",
            cli.to
        );
    }
    if !cli.variable.is_empty() {
        bail!("-V/--variable is not supported yet");
    }
    if cli.pdf_engine.is_some() {
        bail!("--pdf-engine is not supported yet");
    }
    let text = std::fs::read_to_string(input)
        .with_context(|| format!("could not read {}", input.display()))?;
    let doc =
        markdown::read(&text).with_context(|| format!("could not parse {}", input.display()))?;
    let doc = lua::run_filters(doc, &cli.lua_filter, "html", &input.to_string_lossy())?;
    if !cli.standalone {
        if !cli.css.is_empty() || cli.embed_resources {
            bail!("--css and --embed-resources need --standalone");
        }
        let body = layout::render(Some(html::COLUMNS), &html::blocks(&doc.blocks));
        return Ok(body + "\n");
    }
    let opts = html::Options {
        css: cli.css.clone(),
        fallback_title: input
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
    };
    let page = html::standalone(&doc, &opts)?;
    if cli.embed_resources {
        embed::embed(&page)
    } else {
        Ok(page)
    }
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
        None => print!("{out}"),
    }
    Ok(())
}
