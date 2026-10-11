# rspandoc

A subset of pandoc implemented in Rust.

rspandoc takes pandoc's command line and, for the documents and options it
supports, writes output byte-identical to pandoc 3.7.0.2. Anything outside
that subset is refused with an error, never approximated.

## Status

Supported today: Markdown to HTML, as a fragment or a standalone page
(`--standalone`), with linked stylesheets (`--css`), stylesheet embedding
(`--embed-resources`) and Lua filters (`--lua-filter`).

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

Not yet: `-V`, DOCX and PDF output, and the constructs listed above.

`rspandoc --version` prints the build's version information.

## Verification

`scripts/diff-pandoc.py` runs pandoc and rspandoc over a set of files with
the same arguments and reports every output that differs. Matching byte for
byte with their real build arguments: all 678 documents of
teaching-syllabi (with its Lua filter), the 68 of business-syllabi and the
CVs in data. `tests/golden/` holds pandoc's output for the tricky cases
(emphasis, quotes, lists, links, headers) and a whole embedded page, so
`cargo nextest run` checks them without pandoc installed.

Known divergence: emphasis and quotation marks that cross each other
(`"a *b" c*`) are parsed differently from pandoc.

## License

MIT. The layout engine, template language, HTML tokenizer and data tables
are ported from BSD-3-Clause libraries; see `THIRD-PARTY-NOTICES.md`. The
Markdown reader and HTML writer are written from pandoc's documented and
observed behaviour, not from pandoc's GPL source.
