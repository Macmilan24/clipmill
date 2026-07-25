#!/usr/bin/env python3
"""Fail closed when a Node dependency leaves the approved license set."""

from __future__ import annotations

import json
import subprocess
import sys

ALLOWED_LICENSES = {
    "(Apache-2.0 AND BSD-3-Clause)",
    "Apache-2.0",
    "BSD-3-Clause",
    "ISC",
    "MIT",
    "MPL-2.0",
    "Python-2.0",
}


def main() -> int:
    try:
        result = subprocess.run(
            ["pnpm", "licenses", "list", "--json"],
            check=True,
            stdout=subprocess.PIPE,
            text=True,
            timeout=60,
        )
        licenses = json.loads(result.stdout)
    except (OSError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"node-licenses: cannot inspect pnpm dependencies: {error}", file=sys.stderr)
        return 1
    if not isinstance(licenses, dict) or not licenses:
        print("node-licenses: pnpm returned no dependency licenses", file=sys.stderr)
        return 1
    rejected = sorted(set(licenses) - ALLOWED_LICENSES)
    if rejected:
        print(f"node-licenses: unapproved license expressions: {rejected}", file=sys.stderr)
        return 1
    package_count = sum(len(packages) for packages in licenses.values())
    print(f"node-licenses: OK ({package_count} packages, {len(licenses)} approved expressions)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
