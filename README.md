# rspandoc

A subset of pandoc implemented in Rust.

rspandoc takes pandoc's command line and, for the documents and options it
supports, writes output byte-identical to pandoc 3.7.0.2. Anything outside
that subset is refused with an error, never approximated.

## Status

Supported today: Markdown to HTML, as a fragment or a standalone page
(`--standalone`), with linked stylesheets (`--css`) and stylesheet
embedding (`--embed-resources`).

```sh
rspandoc --to html --standalone --embed-resources --css=style.css doc.md -o doc.html
```

The Markdown covered is what the fleet's documents use: YAML front matter,
ATX headers, paragraphs, bullet lists, block quotes, HTML comments, code
spans, emphasis, inline, reference and automatic links, citations, entities,
and pandoc's smart typography. Not yet: Lua filters, `-V`, DOCX and PDF
output.

`rspandoc --version` prints the build's version information.

## Verification

`scripts/diff-pandoc.py` runs pandoc and rspandoc over a set of files with
the same arguments and reports every output that differs. All 678 syllabi
of teaching-syllabi match.

## License

MIT. The layout engine, template language, HTML tokenizer and data tables
are ported from BSD-3-Clause libraries; see `THIRD-PARTY-NOTICES.md`. The
Markdown reader and HTML writer are written from pandoc's documented and
observed behaviour, not from pandoc's GPL source.
