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
    # Boost's notice-preserving grant, used by PyTorch's bundled sources:
    # https://spdx.org/licenses/BSL-1.0.html
    "BSL-1.0",
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
    # Pillow's notice + no-endorsement grant, distinct from SPDX MIT:
    # https://spdx.org/licenses/MIT-CMU.html
    "MIT-CMU",
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
    # yt-dlp's source/PyPI package is dedicated to the public domain with
    # an unrestricted fallback grant: https://spdx.org/licenses/Unlicense.html
    # We do not redistribute its differently licensed PyInstaller binaries.
    "Unlicense",
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
# Exact releases whose installed LICENSE files disambiguate legacy metadata.
# Do not alias bare "BSD" globally: it could also mean the advertising clause.
# SymPy's notice additionally includes MIT-licensed latex2sympy sources.
REVIEWED_DECLARATIONS = {
    ("mpmath", "1.3.0", "BSD"): "BSD-3-Clause",
    ("sympy", "1.14.0", "BSD"): "BSD-3-Clause AND MIT",
    ("torchvision", "0.29.0", "BSD"): "BSD-3-Clause",
}
# This exception relaxes Apache obligations; it is not valid with any arbitrary
# base license. https://spdx.org/licenses/LLVM-exception.html
ALLOWED_EXCEPTIONS = {("Apache-2.0", "LLVM-exception")}
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
        license_name = REVIEWED_DECLARATIONS.get((name, distribution.version, raw_license))
        if not raw_license or raw_license == "UNKNOWN":
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
            if len(matches) == 1 and not expression and not _is_identifier(license_name):
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
    """Read SPDX AND/OR expressions, grouping, and explicitly reviewed WITH pairs.

    AND binds more tightly than OR. Every branch is parsed, including branches
    that do not affect the result, so malformed text cannot hide after OR MIT.
    Unknown licenses remain refused unless a valid OR offers an allowed choice.
    """

    if not expression:
        return False
    expression = ALIASES.get(expression, expression)
    tokens = re.findall(r"[A-Za-z0-9][A-Za-z0-9.+:-]*|[()]", expression)
    if "".join(tokens) != re.sub(r"\s+", "", expression):
        return False
    position = 0

    def take(token: str) -> bool:
        nonlocal position
        if position < len(tokens) and tokens[position] == token:
            position += 1
            return True
        return False

    def identifier() -> str:
        nonlocal position
        if position == len(tokens) or tokens[position] in {"(", ")", "AND", "OR", "WITH"}:
            raise ValueError("expected SPDX identifier")
        token = tokens[position]
        position += 1
        return token

    def atom() -> bool:
        if take("("):
            result = disjunction()
            if not take(")"):
                raise ValueError("unclosed SPDX group")
            return result
        license_id = identifier()
        if take("WITH"):
            return (license_id, identifier()) in ALLOWED_EXCEPTIONS
        return license_id in ALLOWED

    def conjunction() -> bool:
        result = atom()
        while take("AND"):
            term = atom()
            result = result and term
        return result

    def disjunction() -> bool:
        result = conjunction()
        while take("OR"):
            term = conjunction()
            result = result or term
        return result

    try:
        result = disjunction()
        return position == len(tokens) and result
    except (ValueError, RecursionError):
        return False


def _is_identifier(value: str | None) -> bool:
    """Whether a declaration is shaped like an SPDX expression at all.

    The point of the distinction is that a package declaring `GPL-3.0-only`
    must be refused on its own words, while a package declaring three
    paragraphs of BSD text has said nothing this tool can act on and its
    classifiers are the better source. A single line of identifier-shaped
    tokens is a claim; a paragraph is not.
    """

    if not value:
        return False
    # Length does not make an explicit composite license declaration prose.
    return (
        re.fullmatch(r"[A-Za-z0-9.+:()\-]+(?:\s+(?:AND|OR|WITH)\s+[A-Za-z0-9.+:()\-]+)*", value)
        is not None
    )


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
