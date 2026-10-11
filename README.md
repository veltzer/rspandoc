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

Supported today: Markdown to HTML, as a fragment or a standalone page
(`--standalone`), with linked stylesheets (`--css`), stylesheet embedding
(`--embed-resources`), Lua filters (`--lua-filter`) and syntax
highlighting (`--no-highlight` turns it off); and Markdown to DOCX
(`--to docx -o file.docx`), including `lang` and right-to-left `dir`.

```sh
rspandoc --to html --standalone --embed-resources --css=style.css \
    --lua-filter=filter.lua doc.md -o doc.html
```

HTML output is pandoc's, byte for byte, except inside highlighted code.

Code blocks are highlighted with syntect, using the syntax definitions bat
ships (two-face), so where a token starts and ends is syntect's rather
than pandoc's (skylighting's), and the two know some different languages.
Everything around the tokens is pandoc's: the `sourceCode` markup, the
token classes (`kw`, `st`, `co`, ...), and the stylesheet that colours
them (pandoc's default `pygments` style), so pages and style sheets made
for pandoc's highlighted output work unchanged.

DOCX output uses the style names pandoc's does (Title, Heading1-6,
FirstParagraph, BodyText, Compact, BlockText, SourceCode, VerbatimChar,
Hyperlink, and the token styles KeywordTok, StringTok, ...), but the
style sheet and the rest of the package are rspandoc's own, written from
the Office Open XML specification rather than copied from pandoc's (GPL)
reference document. Read back with pandoc's DOCX reader, rspandoc's file
and pandoc's give the same document. Output is deterministic (fixed entry
order and timestamps).

The Markdown covered is what the fleet's documents use: YAML front matter,
ATX headers, paragraphs, bullet and task lists, ordered lists numbered
with digits or `#` (`1.`, `1)`, `(1)`), indented code blocks, fenced code
blocks with or without a language, horizontal rules, block quotes, HTML
comments, code spans, emphasis, inline, reference and automatic links,
citations, entities, and pandoc's smart typography. Constructs pandoc has
and rspandoc does not (lists numbered with letters or roman numerals,
fenced code attributes in braces, tables, images, math, footnotes, raw
TeX, ...) make rspandoc stop with an error naming them.

Lua filters get the slice of pandoc's Lua API in `src/lua.rs`: `FORMAT`,
`PANDOC_STATE.input_files`, element functions and `Pandoc`/`Meta`, and the
`pandoc` constructors for the elements above.

Not yet: `-V`, PDF output, highlighting styles other than `pygments`, and
the constructs listed above. Pipe tables are the next one the fleet needs
(five of demos-ai-coding's exercises use them).

`rspandoc --version` prints the build's version information.

## Verification

`scripts/diff-pandoc.py` runs pandoc and rspandoc over a set of files with
the same arguments and reports every output that differs. HTML must match
byte for byte once highlighted code blocks on both sides are turned back
into plain ones (so the code's text must match, its colouring need not);
DOCX (`--docx`) must match as read back by pandoc's DOCX reader.

With their real build arguments, all 678 documents of teaching-syllabi
(with its Lua filter), the 68 of business-syllabi and the CVs in data
match, as HTML and as DOCX.

The test suite (`cargo nextest run`) needs no pandoc. `tests/golden/`
holds pandoc's output for fragments by theme (emphasis, quotes, lists,
ordered and task lists, code blocks and highlighting, rules, links,
headers, entities, wrapping, ...), whole pages with stylesheets, embedding
and Lua filters, a sample of real fleet documents, and the template
language. rspandoc's own outputs are recorded too: DOCX parts after a
read-back check against pandoc, and a highlighted page (`highlight/`), so
a change in colouring shows up as a diff to review. `tests/cli.rs` checks
that every unsupported option, input and filter is refused with a message
naming it.

Speed: on teaching-syllabi rspandoc converts a document in 3-11 ms
against pandoc's 40-75 ms and uses 17 MiB instead of 110 MiB; the whole
corpus builds 8-18x faster. Details and method in `docs/src/benchmarks.md`
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
