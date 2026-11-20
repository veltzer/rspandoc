#!/usr/bin/env python
"""Regenerate the expected files under tests/golden/ that come from pandoc.

Runs the `pandoc` found on PATH (or --pandoc PATH) with the arguments each
golden test gives rspandoc, from the directory the test runs in, and writes
pandoc's output over the recorded file. The files that hold rspandoc's own
output (docx/, highlight/expected.*) are not touched: they are recorded
from rspandoc after review.

Usage: regen-golden.py [--pandoc PATH]
"""

import argparse
import pathlib
import subprocess
import sys

GOLDEN = pathlib.Path(__file__).resolve().parent.parent / "tests" / "golden"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--pandoc", default="pandoc")
    pandoc = parser.parse_args().pandoc

    version = subprocess.run([pandoc, "--version"], capture_output=True, text=True, check=True)
    print(version.stdout.splitlines()[0])

    def record(cwd: pathlib.Path, args: list[str], out: pathlib.Path) -> None:
        p = subprocess.run([pandoc, *args], cwd=cwd, capture_output=True, check=False)
        if p.returncode:
            sys.exit(f"pandoc {' '.join(args)} (in {cwd}) failed:\n{p.stderr.decode()}")
        out.write_bytes(p.stdout)

    jobs = 0
    # Fragments: NAME.md as HTML and as LaTeX.
    for md in sorted(GOLDEN.glob("*.md")):
        record(GOLDEN, ["--to", "html", md.name], md.with_suffix(".html"))
        record(GOLDEN, ["--to", "latex", md.name], md.with_suffix(".tex"))
        jobs += 2
    # NAME | ARGUMENTS cases.
    for sub, ext in (("latex", ".tex"), ("pages", ".html")):
        d = GOLDEN / sub
        for line in (d / "cases.txt").read_text().splitlines():
            if not line.strip() or line.startswith("#"):
                continue
            name, args = line.split("|", 1)
            record(d, args.split(), d / f"{name.strip()}{ext}")
            jobs += 1
    # A real syllabus, built the way teaching-syllabi builds it.
    d = GOLDEN / "standalone"
    record(
        d,
        ["--to", "html", "--standalone", "--embed-resources", "--css=style.css",
         "--lua-filter=filter.lua", "syllabi/courses/ai/advanced_ai_powered_development.md"],
        d / "expected.html",
    )
    jobs += 1
    # The corpus: whole pages with standalone/'s stylesheet and filter.
    d = GOLDEN / "corpus"
    for md in sorted(d.glob("*.md")):
        record(
            d,
            ["--to", "html", "--standalone", "--embed-resources", "--css=../standalone/style.css",
             "--lua-filter=../standalone/filter.lua", md.name],
            md.with_suffix(".html"),
        )
        record(
            d,
            ["--to", "latex", "--standalone", "--lua-filter=../standalone/filter.lua", md.name],
            md.with_suffix(".tex"),
        )
        jobs += 2
    # pandoc's page and document for the highlighting input.
    d = GOLDEN / "highlight"
    record(d, ["--to", "html", "--standalone", "input.md"], d / "pandoc.html")
    record(d, ["--to", "latex", "--standalone", "input.md"], d / "pandoc.tex")
    # The template language.
    d = GOLDEN / "template"
    record(d, ["--template=t.html", "data.md"], d / "expected.txt")
    jobs += 3
    print(f"{jobs} files recorded")
    return 0


if __name__ == "__main__":
    sys.exit(main())
