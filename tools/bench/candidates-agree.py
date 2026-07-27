#!/usr/bin/env python3
"""The benchmark measures exactly what the daemon can plan, and nothing else.

Two lists have to agree: the implementations the daemon will bind a capability
to, and the implementations the benchmark measures. They live in different
languages because one is planning and the other is measurement, and each is in
the right place — but a drift between them fails silently in the worst way. A
benchmark entry the daemon does not know is a measurement nobody reads; a
daemon candidate the benchmark skips is a capability that can never be selected
on merit, only fallen back to.

So this reads both and refuses a mismatch. It parses Rust rather than importing
it, which is crude, and the alternative — a generated table, or a third file
both derive from — buys less than it costs for five rows that change once a
phase.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REGISTRY = Path("crates/clipmilld/src/implementations.rs")
BENCHMARK = Path("tools/bench/speech-benchmark.py")

RUST_ENTRY = re.compile(
    r"Implementation\s*\{\s*"
    r'name:\s*"(?P<name>[^"]+)",\s*'
    r'capability:\s*"(?P<capability>[^"]+)",\s*'
    r'stage:\s*"[^"]+",\s*'
    r'model:\s*"(?P<model>[^"]+)",',
)
PYTHON_ENTRY = re.compile(
    r'Candidate\(\s*"(?P<name>[^"]+)",\s*"(?P<capability>[^"]+)",\s*"(?P<model>[^"]+)"\s*\)',
)


def main() -> int:
    planned = _read(REGISTRY, RUST_ENTRY)
    measured = _read(BENCHMARK, PYTHON_ENTRY)
    if not planned:
        print(f"candidates-agree: no implementations parsed from {REGISTRY}", file=sys.stderr)
        return 1

    unmeasured = planned - measured
    unplanned = measured - planned
    for name, capability, model in sorted(unmeasured):
        print(
            f"candidates-agree: the daemon can plan {name} ({capability}, {model}) "
            "but the benchmark never measures it",
            file=sys.stderr,
        )
    for name, capability, model in sorted(unplanned):
        print(
            f"candidates-agree: the benchmark measures {name} ({capability}, {model}) "
            "but no daemon candidate matches it",
            file=sys.stderr,
        )
    if unmeasured or unplanned:
        return 1
    print(f"candidates-agree: OK ({len(planned)} implementations, planned and measured)")
    return 0


def _read(path: Path, pattern: re.Pattern[str]) -> set[tuple[str, str, str]]:
    text = path.read_text(encoding="utf-8")
    # The pattern matches inside a #[cfg(test)] module too, which is harmless:
    # neither file declares implementations in its tests, and a set makes a
    # duplicate a no-op rather than an error.
    return {
        (match.group("name"), match.group("capability"), match.group("model"))
        for match in pattern.finditer(text)
    }


if __name__ == "__main__":
    raise SystemExit(main())
