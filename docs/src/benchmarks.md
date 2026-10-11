# Speed

rspandoc against pandoc on the same conversions, measured on 2026-10-11 with
`scripts/bench.py`.

## Setup

- rspandoc 0.1.8, the released binary (`dist` profile)
- pandoc 3.7.0.2, Ubuntu's package
- AMD Ryzen AI 7 350 (16 threads), 30 GiB RAM, Ubuntu 26.04.1, Linux 7.0
- Run from teaching-syllabi, with the arguments it builds its documents
  with: two stylesheets (6 KiB) embedded and its Lua filter
- Each conversion is a separate process, as a build tool runs it, so the
  times include process start-up. Peak memory is each process's maximum
  resident set, from the kernel.

## One document

`syllabi/courses/ai/advanced_ai_powered_development.md` (11 KiB), 30 runs of
each after one warm-up:

| Conversion | Tool | Median | Min | Peak memory | Speed-up (median) |
|---|---|---|---|---|---|
| HTML fragment | pandoc | 41.0 ms | 38.8 ms | 109.9 MiB | |
| HTML fragment | rspandoc | 3.2 ms | 2.2 ms | 17.0 MiB | 13x |
| HTML page (CSS embedded, Lua filter) | pandoc | 61.5 ms | 58.7 ms | 113.9 MiB | |
| HTML page (CSS embedded, Lua filter) | rspandoc | 10.5 ms | 8.8 ms | 17.0 MiB | 6x |
| DOCX (Lua filter) | pandoc | 74.2 ms | 71.6 ms | 116.5 MiB | |
| DOCX (Lua filter) | rspandoc | 6.9 ms | 6.1 ms | 17.0 MiB | 11x |

## Whole corpus

All 678 teaching-syllabi documents, one process each, one after another:

| Conversion | pandoc | rspandoc | Speed-up |
|---|---|---|---|
| HTML fragment | 22.8 s | 1.24 s | 18x |
| HTML page (CSS embedded, Lua filter) | 32.5 s | 4.22 s | 8x |
| DOCX (Lua filter) | 39.9 s | 2.90 s | 14x |

A full HTML-and-DOCX rebuild of teaching-syllabi spends 72 s of CPU in
pandoc and 7 s in rspandoc. Builds run conversions in parallel, which
divides both by the number of cores but not their ratio.

## Where the time goes

pandoc spends about 35-40 ms of each run before converting anything
(starting its Haskell runtime and loading its data), so on documents this
size its start-up is most of its time. rspandoc starts in about a
millisecond; on this document its HTML page time splits roughly into:

| Step | Time |
|---|---|
| Start-up, Markdown and HTML body | 3.4 ms |
| Standalone template (`default.html5`, compiled on every run) | about 3 ms |
| Lua filter (a fresh Lua state, the document copied in and out) | about 4 ms |
| Stylesheet embedding | under 1 ms |

The template and the Lua bridge are the obvious places to make faster if it
ever matters: compiling the template once at build time, and handing the
filter only the elements it has functions for.

## Caveats

- The DOCX rows compare different outputs: rspandoc's DOCX has its own
  styles, so it is the same document as pandoc's, not the same file.
- These are small documents; on long ones the conversion itself, not
  start-up, would dominate, and the ratio would be smaller. No document in
  the fleet is long enough to show it.
- Times are from one machine on one day with a lightly loaded system; run
  `scripts/bench.py` to measure your own.

## Reproducing

From a teaching-syllabi checkout, with both tools on the path:

```sh
find syllabi out/processor.generator.generic -name '*.md' | sort > /tmp/corpus.txt
path/to/rspandoc/scripts/bench.py --runs 30 --corpus @/tmp/corpus.txt
```
