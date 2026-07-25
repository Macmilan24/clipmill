#!/usr/bin/env python3
"""Fail closed when a Node dependency leaves the approved license set."""

from __future__ import annotations

import json
import subprocess
import sys

# Individual SPDX identifiers we accept. Expressions combining them are
# evaluated below rather than listed here as literal strings.
ALLOWED_TERMS = {
    "0BSD",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "BlueOak-1.0.0",
    # Documentation and data, not code: caniuse-lite ships the browser-support
    # tables every frontend toolchain reads. CC-BY-4.0 permits redistribution
    # with attribution and is one-way compatible with the AGPL we publish under.
    "CC-BY-4.0",
    "CC0-1.0",
    "ISC",
    "MIT",
    # MIT without the attribution clause; strictly more permissive than MIT.
    "MIT-0",
    "MPL-2.0",
    "Python-2.0",
    "Unlicense",
}


def is_allowed(expression: str) -> bool:
    """Evaluate a flat SPDX expression against ALLOWED_TERMS.

    npm license fields are disjunctions of conjunctions — "Apache-2.0 OR MIT",
    "(Apache-2.0 AND BSD-3-Clause)". A dual-licensed package is acceptable when
    *any* alternative is fully acceptable, since we may take that alternative;
    a conjunction is acceptable only when *every* term is. Matching the whole
    string literally, as this did before, rejected dual-licensed packages whose
    every branch was already approved.
    """
    cleaned = expression.replace("(", " ").replace(")", " ")
    alternatives = [part for part in cleaned.split(" OR ")]
    for alternative in alternatives:
        terms = [term.strip() for term in alternative.split(" AND ")]
        terms = [term for term in terms if term]
        if terms and all(term in ALLOWED_TERMS for term in terms):
            return True
    return False


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
    rejected = sorted(expression for expression in licenses if not is_allowed(expression))
    if rejected:
        print(f"node-licenses: unapproved license expressions: {rejected}", file=sys.stderr)
        for expression in rejected:
            names = sorted({package.get("name", "?") for package in licenses[expression]})
            print(f"  {expression}: {', '.join(names)}", file=sys.stderr)
        return 1
    package_count = sum(len(packages) for packages in licenses.values())
    print(f"node-licenses: OK ({package_count} packages, {len(licenses)} approved expressions)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
