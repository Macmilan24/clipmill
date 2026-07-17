#!/usr/bin/env python3
"""Schema lint: the rational-time rule (decision D06) as a build gate.

Any JSON Schema property whose name suggests a time quantity
(time/start/end/duration/offset) must not be float-typed ("number").
Times in ClipMill contracts are rational integers — ticks against an
explicit timebase — never float seconds.

Usage: python tools/schema-lint/check.py contracts/schemas/*.json
Exits non-zero listing every violation.
"""

import json
import re
import sys
from pathlib import Path

TIME_NAME = re.compile(r"(?:^|_)(?:time|start|end|duration|offset)(?:_|$)|(?:_at$)")


def walk(node: object, path: str, violations: list[str], source: Path) -> None:
    if isinstance(node, dict):
        props = node.get("properties")
        if isinstance(props, dict):
            for name, subschema in props.items():
                here = f"{path}.{name}" if path else name
                if (
                    TIME_NAME.search(name)
                    and isinstance(subschema, dict)
                    and subschema.get("type") == "number"
                ):
                    violations.append(
                        f"{source}: property '{here}' is float-typed ('number') but named "
                        "like a time quantity - use integer ticks against a timebase (D06)"
                    )
                walk(subschema, here, violations, source)
        for key, value in node.items():
            if key != "properties":
                walk(value, path, violations, source)
    elif isinstance(node, list):
        for item in node:
            walk(item, path, violations, source)


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print("usage: check.py <schema.json> [...]", file=sys.stderr)
        return 2
    violations: list[str] = []
    for arg in argv[1:]:
        source = Path(arg)
        walk(json.loads(source.read_text()), "", violations, source)
    for violation in violations:
        print(f"error: {violation}", file=sys.stderr)
    if violations:
        return 1
    print(f"schema-lint: {len(argv) - 1} schema(s) clean")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
