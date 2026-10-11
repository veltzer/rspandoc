#!/usr/bin/env python
"""Compare rspandoc against pandoc, file by file.

Runs `pandoc <args> FILE` and `rspandoc <args> FILE` for every Markdown
file given (or listed, one per line, in a file passed as @LIST) from the
current directory, and reports which outputs differ, with the first
differing line of each. Exits non-zero when any differ.

HTML is compared byte for byte except for syntax highlighting: rspandoc
highlights with syntect, not pandoc's skylighting, so the tokens differ by
design, and so do the languages each knows. Before comparing, every
highlighted block in both outputs is turned back into the plain
`<pre class="LANG"><code>TEXT</code></pre>` pandoc writes for a language
it does not know, and the highlighting stylesheet is dropped from the
page; the code's text and everything around it must still be identical.

With --docx, both write DOCX files (rspandoc's styles are its own, so the
bytes differ by design) and what is compared is each file read back by
pandoc's DOCX reader: the same document structure and text.

Usage: diff-pandoc.py [--rspandoc PATH] [--show N] -- ARGS... -- FILES...
  e.g. (in teaching-syllabi)
  diff-pandoc.py -- --to html --standalone --embed-resources \
      --css=print/syllabus.css -- @files.txt
"""

import argparse
import concurrent.futures
import os
import re
import subprocess
import sys
import tempfile


def run(cmd: list[str]) -> tuple[int, bytes]:
    p = subprocess.run(cmd, capture_output=True, check=False)
    return p.returncode, p.stdout + (b"\n[stderr]\n" + p.stderr if p.returncode else b"")


HIGHLIGHTED = re.compile(
    rb'<div class="sourceCode" id="[^"]*"><pre\s+class="sourceCode ([^"]*)">'
    rb'<code class="sourceCode[^"]*">(.*?)</code></pre></div>',
    re.S,
)
TOKEN_SPAN = re.compile(rb'<span class="[a-z]{2}">([^<]*)</span>')
LINE_START = re.compile(rb'<span id="[^"]*"><a href="[^"]*" aria-hidden="true" tabindex="-1"></a>')
HIGHLIGHTING_CSS = re.compile(rb"\n *?/\* CSS for syntax highlighting \*/.*?/\* Warning \*/", re.S)


def plain_code(m: re.Match[bytes]) -> bytes:
    text = LINE_START.sub(b"", TOKEN_SPAN.sub(rb"\1", m.group(2))).replace(b"</span>", b"")
    return b'<pre class="' + m.group(1) + b'"><code>' + text + b"</code></pre>"


def without_highlighting(html: bytes) -> bytes:
    """`html` with highlighted code blocks as plain ones and no highlighting CSS."""
    return HIGHLIGHTING_CSS.sub(b"", HIGHLIGHTED.sub(plain_code, html))


def first_diff(a: bytes, b: bytes) -> str:
    la, lb = a.split(b"\n"), b.split(b"\n")
    for n, (x, y) in enumerate(zip(la, lb, strict=False), 1):
        if x != y:
            return f"line {n}:\n  pandoc:   {x.decode(errors='replace')}\n  rspandoc: {y.decode(errors='replace')}"
    return f"length: pandoc {len(la)} lines, rspandoc {len(lb)} lines"


def main() -> int:
    argv = sys.argv[1:]
    if argv.count("--") != 2:
        print(__doc__, file=sys.stderr)
        return 2
    first = argv.index("--")
    second = argv.index("--", first + 1)
    parser = argparse.ArgumentParser()
    parser.add_argument("--rspandoc", default="rspandoc")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument("--docx", action="store_true")
    opts = parser.parse_args(argv[:first])
    args = argv[first + 1 : second]
    files: list[str] = []
    for f in argv[second + 1 :]:
        if f.startswith("@"):
            with open(f[1:], encoding="utf-8") as fh:
                files.extend(line.strip() for line in fh if line.strip())
        else:
            files.append(f)

    def via_docx(tool: str, f: str, tmp: str) -> bytes:
        out = os.path.join(tmp, os.path.basename(tool) + ".docx")
        code, err = run([tool, *args, f, "-o", out])
        if code:
            return err
        return run(["pandoc", "-f", "docx", "-t", "native", out])[1]

    def compare(f: str) -> tuple[str, str | None]:
        if opts.docx:
            with tempfile.TemporaryDirectory() as tmp:
                want = via_docx("pandoc", f, tmp)
                got = via_docx(opts.rspandoc, f, tmp)
        else:
            _, want = run(["pandoc", *args, f])
            _, got = run([opts.rspandoc, *args, f])
            want, got = without_highlighting(want), without_highlighting(got)
        return f, None if want == got else first_diff(want, got)

    with concurrent.futures.ThreadPoolExecutor() as ex:
        results = list(ex.map(compare, files))
    bad = [(f, d) for f, d in results if d is not None]
    for f, d in bad[: opts.show]:
        print(f"DIFF {f}: {d}")
    print(f"{len(files) - len(bad)}/{len(files)} identical")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
