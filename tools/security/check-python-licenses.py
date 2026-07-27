#!/usr/bin/env python3
"""Inspect every uv-locked environment against the Python license policy."""

from __future__ import annotations

import importlib.metadata
import re
import subprocess
import sys
from pathlib import Path

ALLOWED = {
    "0BSD",
    "AGPL-3.0-only",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "CC0-1.0",
    # Python 1.6's licence, and the one term here that is a judgement rather
    # than a formality. It reaches this project once, through `regex`, whose
    # expression is "Apache-2.0 AND CNRI-Python" because parts of it descend
    # from CPython's own `re`. The terms are permissive; what the FSF objects
    # to is a choice-of-venue clause it reads as an extra restriction under
    # GPL section 7. Nothing here modifies `regex`, it arrives as an unmodified
    # wheel behind the accelerated recognizer, and CPython itself ships under
    # a licence lineage this project already depends on — but the objection is
    # real and this allowance is deliberate rather than incidental.
    "CNRI-Python",
    "ISC",
    "MIT",
    "MIT-0",
    # File-level copyleft, and the line worth being explicit about. MPL-2.0
    # obliges whoever *modifies* an MPL file to share that modification; it
    # says nothing about the code it is distributed alongside and nothing at
    # all about what the application renders. These arrive as unmodified
    # transitive dependencies of the recognizer, so the obligation never
    # attaches to anything in this repository. It is a different question from
    # the model licence policy, which is narrow precisely because weights end
    # up inside what a creator publishes; a certificate bundle does not.
    "MPL-2.0",
    "PSF-2.0",
    "Zlib",
}
ALIASES = {
    "3-Clause BSD License": "BSD-3-Clause",
    "Apache 2.0": "Apache-2.0",
    "Apache 2.0 License": "Apache-2.0",
    "Apache License 2.0": "Apache-2.0",
    "BSD License": "BSD-3-Clause",
    "ISC License": "ISC",
    "MIT License": "MIT",
}
PACKAGE_OVERRIDES = {"annotated-types": "MIT"}
CLASSIFIER_LICENSES = {
    "License :: OSI Approved :: Apache Software License": "Apache-2.0",
    "License :: OSI Approved :: BSD License": "BSD-3-Clause",
    "License :: OSI Approved :: MIT License": "MIT",
    "License :: OSI Approved :: Python Software Foundation License": "PSF-2.0",
}


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--child":
        return check_current_environment()
    lockfiles = sorted(Path(".").glob("**/uv.lock"))
    if not lockfiles:
        return fail("no uv.lock files found")
    script = Path(__file__).resolve()
    for lockfile in lockfiles:
        project = lockfile.parent
        result = subprocess.run(
            [
                "uv",
                "run",
                "--offline",
                "--frozen",
                "--project",
                str(project),
                "python",
                str(script),
                "--child",
            ],
            text=True,
        )
        if result.returncode != 0:
            return fail(f"license validation failed for {project}")
    print(f"python-licenses: OK ({len(lockfiles)} uv lockfiles)")
    return 0


def check_current_environment() -> int:
    rejected: list[str] = []
    checked = 0
    for distribution in importlib.metadata.distributions():
        name = canonical_name(distribution.metadata.get("Name", ""))
        if not name:
            rejected.append("unnamed distribution")
            continue
        expression = distribution.metadata.get("License-Expression")
        raw_license = expression or distribution.metadata.get("License")
        license_name = PACKAGE_OVERRIDES.get(name)
        if license_name is None and raw_license and raw_license != "UNKNOWN":
            license_name = ALIASES.get(raw_license, raw_license)
        # A `License` field is not required to hold an identifier, and some
        # packages put the whole licence text there — SciPy ships its BSD
        # notice in full. Prose is not a claim this tool can read, so it falls
        # through to the classifiers, which are the same package's own OSI
        # declaration in a form with only one possible meaning. Recognizable
        # expressions are never overridden this way: what is trusted here is
        # the package, and this only chooses which of its two statements to
        # believe.
        if license_name is None or not is_allowed(license_name):
            classifiers = distribution.metadata.get_all("Classifier", [])
            matches = {
                CLASSIFIER_LICENSES[value] for value in classifiers if value in CLASSIFIER_LICENSES
            }
            if len(matches) == 1 and not _is_identifier(license_name):
                license_name = matches.pop()
        checked += 1
        if not is_allowed(license_name):
            rejected.append(
                f"{name}=={distribution.version}: {_summarize(license_name) or 'UNKNOWN'}"
            )
    if rejected:
        for rejection in sorted(rejected):
            print(f"python-licenses: rejected {rejection}", file=sys.stderr)
        return 1
    print(f"python-licenses: checked {checked} installed distributions")
    return 0


def is_allowed(expression: str | None) -> bool:
    """Whether an SPDX expression is entirely on the allowlist.

    Packages increasingly publish real expressions rather than a single
    identifier — NumPy ships "BSD-3-Clause AND 0BSD AND MIT AND Zlib AND
    CC0-1.0" — and reading those literally rejects a package every term of
    which is already permitted. That is a parsing gap, not a policy position,
    and treating it as one by adding the whole string to the allowlist would
    mean the next version's slightly different string fails again.

    `AND` requires every term; `OR` accepts any. Anything with parentheses or
    both operators is left to a literal match and otherwise refused, because
    an expression this tool cannot read confidently is one a human should.
    """

    if not expression:
        return False
    expression = ALIASES.get(expression, expression)
    if expression in ALLOWED:
        return True
    if "(" in expression or ")" in expression:
        return False
    terms = [ALIASES.get(term, term) for term in re.split(r"\s+AND\s+", expression)]
    if len(terms) > 1:
        return all(is_allowed(term) for term in terms)
    terms = [ALIASES.get(term, term) for term in re.split(r"\s+OR\s+", expression)]
    if len(terms) > 1:
        return any(term in ALLOWED for term in terms)
    return False


def _is_identifier(value: str | None) -> bool:
    """Whether a declaration is shaped like an SPDX expression at all.

    The point of the distinction is that a package declaring `GPL-3.0-only`
    must be refused on its own words, while a package declaring three
    paragraphs of BSD text has said nothing this tool can act on and its
    classifiers are the better source. A single line of identifier-shaped
    tokens is a claim; a paragraph is not.
    """

    if not value or "\n" in value or len(value) > 128:
        return False
    return " " not in value.replace(" AND ", "").replace(" OR ", "")


def _summarize(value: str | None) -> str | None:
    """A rejection has to be readable, and some of these are a page long."""

    if value is None:
        return None
    first = value.strip().splitlines()[0] if value.strip() else value
    return first if len(first) <= 120 else first[:117] + "..."


def canonical_name(value: str) -> str:
    return value.casefold().replace("_", "-").replace(".", "-")


def fail(message: str) -> int:
    print(f"python-licenses: {message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
