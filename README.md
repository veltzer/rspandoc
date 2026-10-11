# rspandoc

A subset of pandoc implemented in Rust.

rspandoc takes pandoc's command line and, for the documents and options it
supports, writes output byte-identical to pandoc 3.7.0.2. Anything outside
that subset is refused with an error, never approximated.

## Status

Supported today: Markdown to HTML, as a fragment or a standalone page
(`--standalone`), with linked stylesheets (`--css`), stylesheet embedding
(`--embed-resources`) and Lua filters (`--lua-filter`); and Markdown to DOCX
(`--to docx -o file.docx`), including `lang` and right-to-left `dir`.

DOCX output uses the style names pandoc's does (Title, Heading1-6,
FirstParagraph, BodyText, Compact, BlockText, VerbatimChar, Hyperlink), but
the style sheet and the rest of the package are rspandoc's own, written
from the Office Open XML specification rather than copied from pandoc's
(GPL) reference document. So its DOCX files look different from pandoc's
and are not byte-identical; what matches is the document: read back with
pandoc's DOCX reader, both give the same structure and text. Output is
deterministic (fixed entry order and timestamps).

```sh
rspandoc --to html --standalone --embed-resources --css=style.css \
    --lua-filter=filter.lua doc.md -o doc.html
```

The Markdown covered is what the fleet's documents use: YAML front matter,
ATX headers, paragraphs, bullet lists, block quotes, HTML comments, code
spans, emphasis, inline, reference and automatic links, citations, entities,
and pandoc's smart typography. Constructs pandoc has and rspandoc does not
(ordered lists, fenced or indented code, tables, images, math, footnotes,
raw TeX, attributes, ...) make rspandoc stop with an error naming them.

Lua filters get the slice of pandoc's Lua API in `src/lua.rs`: `FORMAT`,
`PANDOC_STATE.input_files`, element functions and `Pandoc`/`Meta`, and the
`pandoc` constructors for the elements above.

Not yet: `-V`, PDF output, and the constructs listed above.

`rspandoc --version` prints the build's version information.

## Verification

`scripts/diff-pandoc.py` runs pandoc and rspandoc over a set of files with
the same arguments and reports every output that differs (with `--docx`,
the outputs read back by pandoc's DOCX reader). With their real build
arguments, all 678 documents of teaching-syllabi (with its Lua filter), the
68 of business-syllabi and the CVs in data match: HTML byte for byte, DOCX
as documents. The test suite (`cargo nextest run`) needs no pandoc: `tests/golden/`
holds pandoc's output for fragments by theme (emphasis, quotes, lists,
links, headers, entities, wrapping, ...), whole pages with stylesheets,
embedding and Lua filters, a sample of real fleet documents, and the
template language; DOCX parts are recorded after a read-back check
against pandoc. `tests/cli.rs` checks that every unsupported option,
input and filter is refused with a message naming it.

Speed: on teaching-syllabi rspandoc converts a document in 3-11 ms
against pandoc's 40-75 ms and uses 17 MiB instead of 110 MiB; the whole
corpus builds 8-18x faster. Details and method in `docs/src/benchmarks.md`
(`scripts/bench.py` reproduces them).

Known divergence: emphasis and quotation marks that cross each other
(`"a *b" c*`) are parsed differently from pandoc.

## License

MIT. The layout engine, template language, HTML tokenizer and data tables
are ported from BSD-3-Clause libraries; see `THIRD-PARTY-NOTICES.md`. The
Markdown reader and HTML writer are written from pandoc's documented and
observed behaviour, not from pandoc's GPL source.
