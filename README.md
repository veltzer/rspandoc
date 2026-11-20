# rspandoc

A pandoc replacement in Rust.

rspandoc takes pandoc's command line and, for the documents and options it
supports, produces the same document pandoc 3.7.0.2 does, many times
faster. It is a replacement, not a byte-for-byte clone: where rspandoc's
own choice is as good as pandoc's (syntax highlighting, DOCX styling), it
makes its own, and every such difference is defined and checked (see
Verification). Anything it does not support is refused with an error,
never approximated.

## Status

Supported today, all from Markdown:

- HTML, as a fragment or a standalone page (`--standalone`), with linked
  stylesheets (`--css`), stylesheet embedding (`--embed-resources`), Lua
  filters (`--lua-filter`) and syntax highlighting (`--no-highlight`
  turns it off).
- DOCX (`--to docx -o file.docx`), including `lang`, right-to-left `dir`
  and a reference document for the styles (`--reference-doc`).
- LaTeX (`--to latex`), as a fragment or a standalone document, with
  template variables (`-V`), `lang`, `dir` and the font variables.
- PDF (`--to pdf -o file.pdf`, or any output file ending in `.pdf`): the
  standalone LaTeX document run through `pdflatex` (the default),
  `xelatex`, `lualatex` or `tectonic` (`--pdf-engine`).

```sh
rspandoc --to html --standalone --embed-resources --css=style.css \
    --lua-filter=filter.lua doc.md -o doc.html
```

```sh
rspandoc --to pdf --pdf-engine=xelatex --lua-filter=filter.lua doc.md -o doc.pdf
```

HTML and LaTeX output are pandoc's, byte for byte, except inside
highlighted code. A PDF is that LaTeX run through the engine the way
pandoc runs it (from the current directory, twice, so the bookmarks are
there), so it is the PDF pandoc makes with the same engine.

Code blocks are highlighted with syntect, using the syntax definitions bat
ships (two-face), so where a token starts and ends is syntect's rather
than pandoc's (skylighting's), and the two know some different languages.
Everything around the tokens is pandoc's: the `sourceCode` markup, the
token classes (`kw`, `st`, `co`, ...), and the stylesheet that colours
them (pandoc's default `pygments` style), so pages and style sheets made
for pandoc's highlighted output work unchanged. In LaTeX the same holds:
the `Shaded` and `Highlighting` environments, the `\KeywordTok`,
`\StringTok`, ... commands and their definitions in the preamble are
pandoc's.

DOCX output uses the style names pandoc's does (Title, Heading1-6,
FirstParagraph, BodyText, Compact, BlockText, SourceCode, VerbatimChar,
Hyperlink, and the token styles KeywordTok, StringTok, ...), but the
style sheet and the rest of the package are rspandoc's own, written from
the Office Open XML specification rather than copied from pandoc's (GPL)
reference document. Read back with pandoc's DOCX reader, rspandoc's file
and pandoc's give the same document. Output is deterministic (fixed entry
order and timestamps). With `--reference-doc` the style sheet is the
reference document's, with the styles rspandoc writes that it lacks added
from rspandoc's own (as pandoc adds its own), and its font table, theme
and page setup come along; a reference document with headers or footers
is refused.

The Markdown covered is what the fleet's documents use: YAML front matter,
ATX headers, paragraphs, bullet and task lists, ordered lists numbered
with digits or `#` (`1.`, `1)`, `(1)`), indented code blocks, fenced code
blocks with or without a language, pipe tables (with pandoc's relative
column widths when a line is too long for the page), horizontal rules,
block quotes, HTML comments, `<details>` and `<summary>` blocks, code
spans, emphasis, inline, reference and automatic links, citations,
entities, and pandoc's smart typography.
Constructs pandoc has and rspandoc does not (lists numbered with letters
or roman numerals, fenced code attributes in braces, table captions and
other table syntaxes, images, math, footnotes, raw TeX, ...) make
rspandoc stop with an error naming them.

Lua filters get the slice of pandoc's Lua API in `src/lua.rs`: `FORMAT`,
`PANDOC_STATE.input_files`, element functions and `Pandoc`/`Meta`, the
`pandoc` constructors for the elements above, raw LaTeX blocks, and the
metadata a filter needs to add to the preamble (`pandoc.MetaList`,
`pandoc.MetaBlocks`, `pandoc.utils.type`). Documents with tables are
refused when a Lua filter is given, until filters can see table cells.

Not yet: `-V` for HTML and DOCX, PDF engines other than the four above,
custom templates, tables of contents, highlighting styles other than
`pygments`, and the constructs listed above. In LaTeX the `author`,
`date`, `subtitle` and `abstract` metadata fields and languages outside
the table in `src/latex.rs` are refused.

`rspandoc --version` prints the build's version information.

## Verification

`scripts/diff-pandoc.py` runs pandoc and rspandoc over a set of files with
the same arguments and reports every output that differs. HTML must match
byte for byte once highlighted code blocks on both sides are turned back
into plain ones (so the code's text must match, its colouring need not);
LaTeX (`--latex`) is held to the same rule as HTML; DOCX (`--docx`) must
match as read back by pandoc's DOCX reader. PDF needs no comparison of
its own: it is the LaTeX document, checked byte for byte, handed to the
same engine.

With their real build arguments, all 689 documents of teaching-syllabi
(with its Lua filter and reference document), the 68 of business-syllabi,
the CVs in data, the 37 exercises of demos-ai-coding and the 6 documents
of demos-lang-md match, in every format each repository builds: HTML,
DOCX and LaTeX.

The test suite (`cargo nextest run`) needs no pandoc. `tests/golden/`
holds pandoc's output, as HTML and as LaTeX, for fragments by theme
(emphasis, quotes, lists, ordered and task lists, code blocks and
highlighting, tables, rules, links, headers, entities, wrapping, ...),
whole pages and LaTeX documents with stylesheets, embedding, template
variables and Lua filters, a sample of real fleet documents, and the
template language. rspandoc's own outputs are recorded too: DOCX parts after a
read-back check against pandoc, and a highlighted page (`highlight/`), so
a change in colouring shows up as a diff to review. `tests/cli.rs` checks
that every unsupported option, input and filter is refused with a message
naming it, and drives the PDF step with a stand-in engine, so the suite
needs no TeX either.

Speed: on teaching-syllabi rspandoc converts a document in 3-10 ms
against pandoc's 36-61 ms and uses 17 MiB instead of 110 MiB; the whole
corpus builds 7-13x faster. On demos-ai-coding's long, code-heavy
exercises, where highlighting dominates, a document takes 38 ms against
pandoc's 290 ms, and the corpus builds 11-14x faster. Details and method in `docs/src/benchmarks.md`
(`scripts/bench.py` reproduces them).

Known divergences besides highlighting: emphasis and quotation marks that
cross each other (`"a *b" c*`), and a `***` run that never closes, are
parsed differently from pandoc.

## License

MIT. The layout engine, template language, HTML tokenizer, data tables and
highlighting stylesheet are ported from BSD-3-Clause libraries; see
`THIRD-PARTY-NOTICES.md`. The embedded syntax definitions carry their own
MIT and Apache-2.0 licences, listed in `THIRD-PARTY-SYNTAXES.md`. The
Markdown reader and HTML writer are written from pandoc's documented and
observed behaviour, not from pandoc's GPL source.
