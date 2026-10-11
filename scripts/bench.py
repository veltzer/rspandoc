#!/usr/bin/env python
"""Time rspandoc against pandoc on the same conversions.

Run from the directory holding the documents (e.g. teaching-syllabi). For
each scenario (a set of arguments), it converts one document repeatedly
with both tools and reports the median and minimum wall time and the peak
resident memory per run, then converts every listed document once with
each tool, one after another, and reports the total.

Each conversion is one process, as a build tool runs it, so the times
include process start-up. Output goes to a scratch directory, never next
to the inputs.

Usage: bench.py [--rspandoc PATH] [--pandoc PATH] [--runs N]
                [--single FILE] [--corpus @LIST]
"""

import argparse
import os
import statistics
import subprocess
import sys
import tempfile
import time

# The arguments teaching-syllabi builds its documents with, per format.
FILTER_ARGS = [
    "--standalone",
    "--embed-resources",
    "--css=shared/shared-themes/themes.css",
    "--css=print/syllabus.css",
    "--lua-filter=print/pandoc_filter.lua",
]
SCENARIOS = [
    ("HTML fragment", ["--to", "html"], "html"),
    ("HTML page (CSS embedded, Lua filter)", ["--to", "html", *FILTER_ARGS], "html"),
    ("DOCX (Lua filter)", ["--to", "docx", *FILTER_ARGS], "docx"),
]


def run_once(cmd: list[str]) -> tuple[float, int]:
    """Wall seconds and peak resident KiB of one run of `cmd`."""
    start = time.perf_counter()
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    _, status, usage = os.wait4(proc.pid, 0)
    elapsed = time.perf_counter() - start
    if os.waitstatus_to_exitcode(status) != 0:
        err = proc.stderr.read().decode(errors="replace") if proc.stderr else ""
        sys.exit(f"failed: {' '.join(cmd)}\n{err}")
    return elapsed, usage.ru_maxrss


def command(tool: str, args: list[str], doc: str, out: str) -> list[str]:
    return [tool, *args, doc, "-o", out]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rspandoc", default="rspandoc")
    parser.add_argument("--pandoc", default="pandoc")
    parser.add_argument("--runs", type=int, default=30)
    parser.add_argument("--single", default="syllabi/courses/ai/advanced_ai_powered_development.md")
    parser.add_argument("--corpus", help="@FILE listing documents, one per line")
    opts = parser.parse_args()
    corpus: list[str] = []
    if opts.corpus:
        with open(opts.corpus.lstrip("@"), encoding="utf-8") as fh:
            corpus = [line.strip() for line in fh if line.strip()]

    tools = [("pandoc", opts.pandoc), ("rspandoc", opts.rspandoc)]
    with tempfile.TemporaryDirectory() as tmp:
        print(f"## One document ({opts.single}), {opts.runs} runs each\n")
        print("| Conversion | Tool | Median | Min | Peak memory | Speed-up (median) |")
        print("|---|---|---|---|---|---|")
        for name, args, ext in SCENARIOS:
            medians = {}
            for label, tool in tools:
                out = os.path.join(tmp, f"out.{ext}")
                run_once(command(tool, args, opts.single, out))  # warm-up
                samples = [run_once(command(tool, args, opts.single, out)) for _ in range(opts.runs)]
                times = [t for t, _ in samples]
                medians[label] = statistics.median(times)
                mem = max(m for _, m in samples) / 1024
                speedup = "" if label == "pandoc" else f"{medians['pandoc'] / medians[label]:.0f}x"
                print(
                    f"| {name} | {label} | {medians[label] * 1000:.1f} ms | "
                    f"{min(times) * 1000:.1f} ms | {mem:.1f} MiB | {speedup} |"
                )
        if corpus:
            print(f"\n## Whole corpus ({len(corpus)} documents, one process each, sequential)\n")
            print("| Conversion | pandoc | rspandoc | Speed-up |")
            print("|---|---|---|---|")
            for name, args, ext in SCENARIOS:
                totals = {}
                for label, tool in tools:
                    start = time.perf_counter()
                    for i, doc in enumerate(corpus):
                        run_once(command(tool, args, doc, os.path.join(tmp, f"{i}.{ext}")))
                    totals[label] = time.perf_counter() - start
                print(
                    f"| {name} | {totals['pandoc']:.1f} s | {totals['rspandoc']:.2f} s | "
                    f"{totals['pandoc'] / totals['rspandoc']:.0f}x |"
                )
    return 0


if __name__ == "__main__":
    sys.exit(main())
