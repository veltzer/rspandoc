//! rspandoc: a pandoc replacement, in Rust. `rspandoc convert` takes
//! pandoc's options and produces the same document pandoc does for the
//! documents and options it supports (byte for byte for HTML and LaTeX,
//! highlighting aside; a PDF is that LaTeX run through a TeX engine), and
//! refuses anything else rather than approximating it.

mod ast;
mod docx;
mod embed;
mod entities;
mod generator;
mod highlight;
mod html;
mod latex;
mod layout;
mod lua;
mod markdown;
mod pdf;
mod template;

use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rspandoc",
    version,
    about = "A pandoc replacement in Rust",
    arg_required_else_help = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Convert a Markdown file the way pandoc does, with pandoc's options
    Convert(Box<Convert>),
    /// Print version and build details
    Version,
}

/// The options of `rspandoc convert`: pandoc's own, for the part of pandoc
/// rspandoc supports.
#[derive(Debug, Args)]
struct Convert {
    /// Output format: `html`, `docx`, `latex` or `pdf`. Without it the
    /// output file's extension decides, and HTML is the default.
    #[arg(short = 't', long = "to", value_name = "FORMAT")]
    to: Option<String>,

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

    /// Set a template variable (repeatable; LaTeX and PDF output only).
    #[arg(short = 'V', long = "variable", value_name = "KEY[=VALUE]")]
    variable: Vec<String>,

    /// The program that makes the PDF: `pdflatex` (the default),
    /// `xelatex`, `lualatex` or `tectonic`.
    #[arg(long = "pdf-engine", value_name = "PROGRAM")]
    pdf_engine: Option<String>,

    /// The DOCX file whose styles a DOCX output takes (other formats
    /// ignore it, as pandoc does).
    #[arg(long = "reference-doc", value_name = "FILE")]
    reference_doc: Option<PathBuf>,

    /// Do not mark the document as made by rspandoc: write what pandoc
    /// writes (`pandoc` as the generator, no application in a DOCX file).
    #[arg(long)]
    no_mark: bool,

    /// Also write when the conversion happened (the time now, or
    /// `SOURCE_DATE_EPOCH` when it is set) in the document's properties.
    #[arg(long, conflicts_with = "no_mark")]
    mark_date: bool,

    /// Also write who ran the conversion in the document's properties.
    #[arg(long, value_name = "NAME", conflicts_with = "no_mark")]
    mark_by: Option<String>,

    /// The Markdown input file.
    input: PathBuf,
}

/// The bare `rspandoc` lists the commands with a line each and nothing
/// else; the options are in `--help`.
fn parse_cli() -> Cli {
    let command = Cli::command();
    match command.clone().try_get_matches() {
        Ok(matches) => Cli::from_arg_matches(&matches).expect("matches parse"),
        Err(error) => {
            use clap::error::ErrorKind;
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand | ErrorKind::MissingSubcommand
            ) {
                let width = command
                    .get_subcommands()
                    .map(|c| c.get_name().len())
                    .max()
                    .unwrap_or(0);
                eprintln!("rspandoc needs a command:");
                for sub in command.get_subcommands() {
                    eprintln!(
                        "  {:<width$}  {}",
                        sub.get_name(),
                        sub.get_about().map(|a| a.to_string()).unwrap_or_default()
                    );
                }
                eprintln!("Run 'rspandoc --help' for the options.");
                std::process::exit(2);
            }
            error.exit()
        }
    }
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
    Latex,
    Pdf,
}

/// The output format: `--to`, or else what the output file's extension
/// says, or else HTML.
fn output_format(cli: &Convert) -> Result<Format> {
    let extension = cli
        .output
        .as_deref()
        .and_then(Path::extension)
        .map(|e| e.to_string_lossy().to_lowercase());
    let name = match (cli.to.as_deref(), extension.as_deref()) {
        (Some(to), _) => to,
        (None, Some("docx")) => "docx",
        (None, Some("tex" | "latex" | "ltx")) => "latex",
        (None, Some("pdf")) => "pdf",
        (None, _) => "html",
    };
    Ok(match name {
        "html" | "html5" => Format::Html,
        "docx" => Format::Docx,
        "latex" => Format::Latex,
        "pdf" => Format::Pdf,
        other => bail!("output format `{other}` is not supported yet (html, docx, latex, pdf)"),
    })
}

/// `-V KEY[=VALUE]` as a key and a value; a bare key is true.
fn variables(cli: &Convert) -> Vec<(String, String)> {
    cli.variable
        .iter()
        .map(|v| match v.split_once('=') {
            Some((k, v)) => (k.to_owned(), v.to_owned()),
            None => (v.clone(), "true".to_owned()),
        })
        .collect()
}

/// Convert the input per the options; the output bytes.
fn convert(cli: &Convert) -> Result<Vec<u8>> {
    let input = cli.input.as_path();
    let format = output_format(cli)?;
    let tex = matches!(format, Format::Latex | Format::Pdf);
    if !cli.variable.is_empty() && !tex {
        bail!("-V/--variable is not supported yet for this output format (latex, pdf)");
    }
    if let Some(engine) = cli.pdf_engine.as_deref() {
        if format != Format::Pdf {
            bail!("--pdf-engine needs PDF output");
        }
        pdf::check_engine(engine)?;
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
    // A filter sees the format being written, and a PDF is written as LaTeX.
    let mark = generator::Mark::new(cli.no_mark, cli.mark_date, cli.mark_by.as_deref())?;
    // A fragment carries no mark, so there is nowhere to write these.
    let whole = cli.standalone || matches!(format, Format::Docx | Format::Pdf);
    if mark.has_details() && !whole {
        bail!("--mark-date and --mark-by need a whole document (--standalone)");
    }
    let lua_format = match format {
        Format::Html => "html",
        Format::Docx => "docx",
        Format::Latex | Format::Pdf => "latex",
    };
    let doc = lua::run_filters(doc, &cli.lua_filter, lua_format, &input.to_string_lossy())?;
    // DOCX, LaTeX and PDF are not pages: like pandoc, rspandoc ignores
    // the HTML page options for them.
    if format == Format::Docx {
        // A DOCX file is always a whole document.
        if cli.output.is_none() {
            bail!("DOCX output is binary; give an output file with -o");
        }
        let reference = cli
            .reference_doc
            .as_deref()
            .map(docx::read_reference)
            .transpose()?;
        return docx::write(&doc, !cli.no_highlight, reference.as_ref(), &mark);
    }
    if tex {
        if format == Format::Latex && !cli.standalone {
            let body = latex::fragment(&doc, !cli.no_highlight)?;
            return Ok((body + "\n").into_bytes());
        }
        let opts = latex::Options {
            highlight: !cli.no_highlight,
            variables: variables(cli),
            mark,
        };
        let document = latex::standalone(&doc, &opts)?;
        if format == Format::Latex {
            return Ok(document.into_bytes());
        }
        // A PDF is always a whole document.
        if cli.output.is_none() {
            bail!("PDF output is binary; give an output file with -o");
        }
        let engine = cli.pdf_engine.as_deref().unwrap_or(pdf::DEFAULT_ENGINE);
        return pdf::make(&document, engine);
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
        mark,
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
    let cli = match parse_cli().command {
        Commands::Convert(cli) => cli,
        Commands::Version => {
            print_version();
            return Ok(());
        }
    };
    let out = convert(&cli)?;
    match &cli.output {
        Some(path) => std::fs::write(path, out)
            .with_context(|| format!("could not write {}", path.display()))?,
        None => std::io::stdout()
            .write_all(&out)
            .context("could not write to stdout")?,
    }
    Ok(())
}
