#!/usr/bin/env python
"""Compare rspandoc against pandoc, file by file.

Runs `pandoc <args> FILE` and `rspandoc <args> FILE` for every Markdown
file given (or listed, one per line, in a file passed as @LIST) from the
current directory, and reports which outputs differ, with the first
differing line of each. Exits non-zero when any differ.

Usage: diff-pandoc.py [--rspandoc PATH] [--show N] -- ARGS... -- FILES...
  e.g. (in teaching-syllabi)
  diff-pandoc.py -- --to html --standalone --embed-resources \
      --css=print/syllabus.css -- @files.txt
"""

import argparse
import concurrent.futures
import subprocess
import sys


def run(cmd: list[str]) -> tuple[int, bytes]:
    p = subprocess.run(cmd, capture_output=True, check=False)
    return p.returncode, p.stdout + (b"\n[stderr]\n" + p.stderr if p.returncode else b"")


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
    opts = parser.parse_args(argv[:first])
    args = argv[first + 1 : second]
    files: list[str] = []
    for f in argv[second + 1 :]:
        if f.startswith("@"):
            with open(f[1:], encoding="utf-8") as fh:
                files.extend(line.strip() for line in fh if line.strip())
        else:
            files.append(f)

    def compare(f: str) -> tuple[str, str | None]:
        _, want = run(["pandoc", *args, f])
        _, got = run([opts.rspandoc, *args, f])
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
