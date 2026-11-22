# Speed

rspandoc against pandoc on the same conversions, measured with
`scripts/bench.py`: the teaching-syllabi and demos-ai-coding corpora on
2026-10-11 with rspandoc 0.1.10, and this repository's golden corpus on
2026-11-22 with rspandoc 0.1.15.

## Setup

- rspandoc 0.1.10 (and 0.1.15 for the golden corpus), built with the
  release (`dist`) profile
- pandoc 3.7.0.2, Ubuntu's package
- AMD Ryzen AI 7 350 (16 threads), 30 GiB RAM, Ubuntu 26.04.1, Linux 7.0,
  otherwise idle
- Each conversion is a separate process, as a build tool runs it, so the
  times include process start-up. Peak memory is each process's maximum
  resident set, from the kernel.
- Two corpora, each with the arguments its repository builds with:
  teaching-syllabi (two stylesheets, 6 KiB, embedded, and its Lua filter;
  short documents, little code) and demos-ai-coding (no stylesheets or
  filter; long exercises full of highlighted code)
- A third, this repository's golden corpus (`tests/golden/corpus`), with
  no stylesheets or filter, so anyone can rerun it from a checkout alone

## teaching-syllabi

One document, `syllabi/courses/ai/advanced_ai_powered_development.md`
(11 KiB), 30 runs of each after one warm-up:

| Conversion | Tool | Median | Min | Peak memory | Speed-up (median) |
|---|---|---|---|---|---|
| HTML fragment | pandoc | 35.9 ms | 34.3 ms | 110.0 MiB | |
| HTML fragment | rspandoc | 3.3 ms | 2.5 ms | 17.2 MiB | 11x |
| HTML page (CSS embedded, Lua filter) | pandoc | 51.0 ms | 49.6 ms | 114.1 MiB | |
| HTML page (CSS embedded, Lua filter) | rspandoc | 9.7 ms | 7.8 ms | 17.4 MiB | 5x |
| DOCX (Lua filter) | pandoc | 60.5 ms | 59.0 ms | 116.6 MiB | |
| DOCX (Lua filter) | rspandoc | 7.2 ms | 5.5 ms | 17.4 MiB | 8x |

All 678 documents, one process each, one after another:

| Conversion | pandoc | rspandoc | Speed-up |
|---|---|---|---|
| HTML fragment | 18.4 s | 1.37 s | 13x |
| HTML page (CSS embedded, Lua filter) | 26.6 s | 3.84 s | 7x |
| DOCX (Lua filter) | 32.1 s | 2.95 s | 11x |

A full HTML-and-DOCX rebuild of teaching-syllabi spends 59 s of CPU in
pandoc and 7 s in rspandoc. Builds run conversions in parallel, which
divides both by the number of cores but not their ratio.

## demos-ai-coding

One document, `exercises/04_advanced_techniques/03_rag_implementation/exercise.md`
(38 KiB, of which 1,271 lines are code in 17 fenced blocks, almost all
JavaScript), 30 runs of each after one warm-up:

| Conversion | Tool | Median | Min | Peak memory | Speed-up (median) |
|---|---|---|---|---|---|
| HTML fragment | pandoc | 291.1 ms | 286.5 ms | 191.9 MiB | |
| HTML fragment | rspandoc | 37.6 ms | 36.6 ms | 17.5 MiB | 8x |
| HTML page | pandoc | 290.8 ms | 284.0 ms | 192.9 MiB | |
| HTML page | rspandoc | 41.3 ms | 40.8 ms | 17.5 MiB | 7x |
| DOCX | pandoc | 288.6 ms | 282.3 ms | 198.0 MiB | |
| DOCX | rspandoc | 38.1 ms | 36.8 ms | 17.5 MiB | 8x |

All 37 exercises, one process each, one after another:

| Conversion | pandoc | rspandoc | Speed-up |
|---|---|---|---|
| HTML fragment | 6.6 s | 0.54 s | 12x |
| HTML page | 6.8 s | 0.60 s | 11x |
| DOCX | 6.9 s | 0.50 s | 14x |

## Golden corpus

Measured on 2026-11-22 with rspandoc 0.1.15 and pandoc 3.7.0.2, on the same
machine, with no stylesheets or filter (`--plain`).

One document, `tests/golden/corpus/advanced_ai_powered_development.md`
(13 KiB, a later copy of the teaching-syllabi document above), 30 runs of
each after one warm-up:

| Conversion | Tool | Median | Min | Peak memory | Speed-up (median) |
|---|---|---|---|---|---|
| HTML fragment | pandoc | 38.8 ms | 35.1 ms | 109.8 MiB | |
| HTML fragment | rspandoc | 2.4 ms | 2.2 ms | 16.8 MiB | 16x |
| HTML page | pandoc | 39.5 ms | 38.4 ms | 110.4 MiB | |
| HTML page | rspandoc | 8.5 ms | 7.0 ms | 16.9 MiB | 5x |
| DOCX | pandoc | 56.1 ms | 53.7 ms | 114.7 MiB | |
| DOCX | rspandoc | 2.8 ms | 2.2 ms | 16.9 MiB | 20x |

All 15 documents (172 KiB), one process each, one after another:

| Conversion | pandoc | rspandoc | Speed-up |
|---|---|---|---|
| HTML fragment | 0.5 s | 0.04 s | 13x |
| HTML page | 0.5 s | 0.08 s | 6x |
| DOCX | 0.7 s | 0.04 s | 19x |

These rows are not directly comparable with the teaching-syllabi ones:
without the Lua filter and stylesheets the HTML page and DOCX runs skip
about 4 ms of filter work. The HTML page stays the slowest conversion
because the standalone template is still compiled on every run.

## Where the time goes

pandoc spends about 35 ms of each run before converting anything (starting
its Haskell runtime and loading its data), so on short documents its
start-up is most of its time. rspandoc starts in about a millisecond.

On the teaching-syllabi document, rspandoc's HTML page time splits roughly
into:

| Step | Time |
|---|---|
| Start-up, Markdown and HTML body | 3.5 ms |
| Standalone template (`default.html5`, compiled on every run) | about 4 ms |
| Lua filter (a fresh Lua state, the document copied in and out) | about 4 ms |
| Stylesheet embedding | under 1 ms |

On the demos-ai-coding document almost all of it is syntax highlighting:
the fragment takes 37.6 ms highlighted and 2.6 ms with `--no-highlight`,
so its 1,259 lines of JavaScript cost about 28 µs a line, including
compiling the syntax's patterns. pandoc spends far more there: the same
document takes it 292 ms highlighted and 25 ms with `--no-highlight`, so
its highlighter costs about 270 ms to rspandoc's 35.

The template, the Lua bridge and highlighting are the places to make
faster if it ever matters: compiling the template once at build time,
handing the filter only the elements it has functions for, and
highlighting a document's code blocks in parallel.

## Caveats

- The DOCX rows compare different outputs: rspandoc's DOCX has its own
  styles, so it is the same document as pandoc's, not the same file.
- The highlighted rows also compare different outputs: rspandoc's tokens
  are syntect's, not pandoc's (see the introduction).
- Times are from one machine on one day; between runs on the same idle
  machine they moved by a few percent, and the 0.1.8 measurements earlier
  the same day had pandoc about 15% slower. Run `scripts/bench.py` to
  measure your own.

## Reproducing

With both tools on the path, from a teaching-syllabi checkout:

```sh
find syllabi out/processor.generator.generic -name '*.md' | sort > /tmp/corpus.txt
path/to/rspandoc/scripts/bench.py --runs 30 --corpus @/tmp/corpus.txt
```

and from a demos-ai-coding checkout (`--plain`: no stylesheets or filter):

```sh
find exercises -name '*.md' | sort > /tmp/corpus.txt
path/to/rspandoc/scripts/bench.py --plain --runs 30 \
    --single exercises/04_advanced_techniques/03_rag_implementation/exercise.md \
    --corpus @/tmp/corpus.txt
```

and the golden corpus, from this repository's checkout:

```sh
cargo build --profile dist
ls tests/golden/corpus/*.md > /tmp/corpus.txt
scripts/bench.py --plain --runs 30 --rspandoc target/dist/rspandoc \
    --single tests/golden/corpus/advanced_ai_powered_development.md \
    --corpus @/tmp/corpus.txt
```
