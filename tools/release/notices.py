#!/usr/bin/env python3
"""Third-party notices for the software a packaged ClipMill is built from.

The app and clipmilld are compiled from Rust crates, and the app's interface is
bundled from npm packages. For each, this writes a notice that lists every
third-party package with its version, its declared licence and where its
source is, and then the licence texts: each distinct text once, under the
packages that ship it.

A package that ships no licence text of its own is given the standard text of
a licence its expression offers, naming its authors where that licence names a
copyright holder. One whose expression offers no licence with a standard text
here is refused, so a new dependency cannot reach a release without its
notice. ClipMill's own crates and packages are under ClipMill's licence and are
not listed.

    python3 tools/release/notices.py --target aarch64-apple-darwin --out DIR
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import textwrap
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TEXTS = Path(__file__).resolve().parent / "licenses"
WIDTH = 80
# The two programs a packaged app carries: the daemon and the app itself.
PROGRAMS = ("clipmilld", "clipmill-shell")
# The package whose production dependencies are bundled into the interface.
INTERFACE = "@clipmill/desktop"
LICENCE_FILE = re.compile(r"^(licen[cs]e|copying|copyright|notice|unlicense)", re.IGNORECASE)
# Source files that happen to be named like a licence (an icon called
# "copyright", a module called "copying") are code, not texts.
CODE = {".c", ".cc", ".cjs", ".cpp", ".h", ".js", ".json", ".map", ".mjs", ".py", ".rs", ".ts"}
# How deep inside a package to look for the licences of code it vendors.
VENDORED_DEPTH = 3

MIT = """\
Copyright (c) {holders}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

ISC = """\
Copyright (c) {holders}

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
"""

BSD_3_CLAUSE = """\
Copyright (c) {holders}

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""

# Standard texts for a package that ships none: those that name a copyright
# holder are filled with the package's authors; the others are read whole.
TEMPLATES = {"MIT": MIT, "ISC": ISC, "BSD-3-Clause": BSD_3_CLAUSE}
STANDARD_FILES = {"Apache-2.0": "Apache-2.0.txt", "MPL-2.0": "MPL-2.0.txt"}
# When a package offers several licences, the notice takes the earliest here.
PREFERENCE = ("Apache-2.0", "MIT", "ISC", "BSD-3-Clause", "MPL-2.0")
# Who a standard text names where the package's authors are not the holder:
# code it carries from another project, as that code's own headers say.
HOLDERS = {
    ("@bufbuild/protobuf", "BSD-3-Clause"): "2008 Google Inc. All rights reserved.",
    ("victory-vendor", "ISC"): "Mike Bostock",
}


@dataclass(frozen=True)
class Package:
    name: str
    version: str
    # The licence expression the package declares, as it declares it.
    licence: str
    # Where the package's source is published.
    source: str
    # Where its files are on this machine.
    folder: Path
    # Who a standard licence text names as the copyright holder.
    holders: str
    # A licence file its manifest names, relative to its folder.
    licence_file: str | None = None


def say(message: str) -> None:
    print(f"notices: {message}", flush=True)


def alternatives(expression: str) -> list[list[str]]:
    """The choices an SPDX expression offers, each the licences that all apply.

    Only a flat expression is understood: alternatives joined by OR (or
    Cargo's old "/"), each a conjunction joined by AND, the whole optionally
    in parentheses. Anything more nested offers no choice here, so a package
    that would need one is refused rather than guessed at.
    """
    text = expression.strip()
    if text.startswith("(") and text.endswith(")") and text.count("(") == 1:
        text = text[1:-1]
    if not text or "(" in text or ")" in text:
        return []
    choices = re.split(r"\s+OR\s+|\s*/\s*", text)
    return [[term.strip() for term in re.split(r"\s+AND\s+", choice)] for choice in choices]


def standard_texts(package: Package) -> list[tuple[str, str]]:
    """The standard texts for a package that ships no licence text."""
    known = TEMPLATES.keys() | STANDARD_FILES.keys()
    choices = [terms for terms in alternatives(package.licence) if set(terms) <= known]
    if not choices:
        raise SystemExit(
            f"notices: {package.name} {package.version} ships no licence text, and "
            f'"{package.licence}" offers no licence with a standard text here; add it '
            "to notices.py or tools/release/licenses"
        )
    rank = {name: index for index, name in enumerate(PREFERENCE)}
    chosen = min(choices, key=lambda terms: max(rank.get(term, len(rank)) for term in terms))
    texts = []
    for term in chosen:
        if term in STANDARD_FILES:
            texts.append((term, (TEXTS / STANDARD_FILES[term]).read_text(encoding="utf-8")))
        else:
            holder = HOLDERS.get((package.name, term), package.holders)
            texts.append((term, TEMPLATES[term].format(holders=holder)))
    return texts


def is_licence(path: Path) -> bool:
    return (
        path.is_file()
        and LICENCE_FILE.match(path.name) is not None
        and path.suffix.lower() not in CODE
    )


def licence_files(package: Package) -> tuple[list[Path], list[Path]]:
    """The licence texts a package ships, as (its own, the vendored).

    Its own are the files named like a licence at its top level, everything
    in a top-level LICENSES folder, and the file its manifest names. The
    vendored are licence files deeper inside, which belong to code it
    carries from other projects.
    """
    own: dict[Path, Path] = {}
    for entry in sorted(package.folder.iterdir()):
        if is_licence(entry):
            own[entry.resolve()] = entry
        elif entry.is_dir() and entry.name.lower() in {"license", "licenses"}:
            for inner in sorted(entry.iterdir()):
                if inner.is_file():
                    own[inner.resolve()] = inner
    if package.licence_file:
        named = package.folder / package.licence_file
        if named.is_file():
            own.setdefault(named.resolve(), named)
    vendored: dict[Path, Path] = {}
    for base, folders, files in os.walk(package.folder):
        depth = len(Path(base).relative_to(package.folder).parts)
        folders[:] = sorted(name for name in folders if name not in {"node_modules", ".git"})
        if depth >= VENDORED_DEPTH:
            folders.clear()
        if depth == 0:
            continue
        for name in sorted(files):
            path = Path(base) / name
            if is_licence(path) and path.resolve() not in own:
                vendored[path.resolve()] = path
    return list(own.values()), list(vendored.values())


def plain(text: str) -> str:
    """A licence text as written into a notice: Unix line ends, no trailing
    spaces, no blank lines around it. Texts equal after this are one text."""
    lines = [line.rstrip() for line in text.replace("\r\n", "\n").replace("\r", "\n").split("\n")]
    return "\n".join(lines).strip("\n")


def holders(people: list[str], name: str) -> str:
    """Who a package says wrote it, without addresses; or its name's authors."""
    names = [re.sub(r"\s*[<(][^>)]*[>)]", "", person).strip() for person in people]
    names = [person for person in names if person]
    return ", ".join(names) if names else f"the {name} authors"


def shipped(package: Package, path: Path) -> tuple[str, str]:
    """A licence file a package ships, labelled by where it is in the package."""
    label = path.relative_to(package.folder).as_posix()
    return label, path.read_text(encoding="utf-8", errors="replace")


def render(title: str, introduction: str, packages: list[Package]) -> str:
    groups: dict[str, list[str]] = {}
    for package in packages:
        own, vendored = licence_files(package)
        if own:
            texts = [shipped(package, path) for path in own]
        else:
            texts = [(f"standard {term} text", text) for term, text in standard_texts(package)]
        texts += [shipped(package, path) for path in vendored]
        for label, text in texts:
            groups.setdefault(plain(text), []).append(f"{package.name} {package.version} ({label})")
    lines = [title, "", *textwrap.wrap(introduction, WIDTH), ""]
    lines += [
        f"{package.name} {package.version}: {package.licence}, {package.source}"
        for package in packages
    ]
    rule = "=" * WIDTH
    for text, users in groups.items():
        lines += ["", rule, *textwrap.wrap(", ".join(users), WIDTH), rule, "", text]
    return "\n".join(lines) + "\n"


# ---- Rust ---------------------------------------------------------------------


def is_macro(package: dict) -> bool:
    return any("proc-macro" in target["kind"] for target in package["targets"])


def rust_packages(triple: str) -> list[Package]:
    """The third-party crates the two programs are compiled from, for a target.

    Only normal dependencies are followed, and not into procedural macros:
    build scripts, macros and test dependencies run while ClipMill is built,
    and none of them is in the programs.
    """
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", triple],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    crates = {crate["id"]: crate for crate in metadata["packages"]}
    graph = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    roots = [
        key
        for key, crate in crates.items()
        if crate["name"] in PROGRAMS and crate["source"] is None
    ]
    if len(roots) != len(PROGRAMS):
        raise SystemExit(f"notices: the workspace does not define {', '.join(PROGRAMS)}")
    reached: set[str] = set()
    queue = list(roots)
    while queue:
        current = queue.pop()
        if current in reached:
            continue
        reached.add(current)
        for dependency in graph[current]["deps"]:
            normal = any(kind["kind"] is None for kind in dependency["dep_kinds"])
            if normal and not is_macro(crates[dependency["pkg"]]):
                queue.append(dependency["pkg"])
    packages = []
    for key in reached:
        crate = crates[key]
        # A crate without a source is one of ClipMill's own.
        if crate["source"] is None:
            continue
        registry = crate["source"].startswith("registry+")
        packages.append(
            Package(
                name=crate["name"],
                version=crate["version"],
                licence=crate.get("license") or "not stated",
                source=(
                    f"https://crates.io/crates/{crate['name']}/{crate['version']}"
                    if registry
                    else crate.get("repository") or crate["source"]
                ),
                folder=Path(crate["manifest_path"]).parent,
                holders=holders(crate.get("authors") or [], crate["name"]),
                licence_file=crate.get("license_file"),
            )
        )
    return sorted(packages, key=lambda package: (package.name, package.version))


# ---- JavaScript ---------------------------------------------------------------


def npm_people(manifest: dict) -> list[str]:
    people = [manifest.get("author"), *manifest.get("contributors", [])]
    names = []
    for person in people:
        if isinstance(person, dict):
            person = person.get("name")
        if isinstance(person, str) and person.strip():
            names.append(person)
    return names


def npm_licence(manifest: dict) -> str:
    declared = manifest.get("license")
    if isinstance(declared, dict):
        declared = declared.get("type")
    if isinstance(declared, str) and declared.strip():
        return declared.strip()
    # The old form: a list of licences the package is offered under.
    offered = [item.get("type") for item in manifest.get("licenses", []) if isinstance(item, dict)]
    offered = [item for item in offered if isinstance(item, str)]
    return " OR ".join(offered) if offered else "not stated"


def javascript_packages() -> list[Package]:
    """The third-party npm packages the interface's production build draws on.

    Type declarations are left out: they describe code to the compiler and
    none of them reaches the bundle.
    """
    result = subprocess.run(
        ["pnpm", "--filter", INTERFACE, "list", "--prod", "--depth", "Infinity", "--json"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    found: dict[tuple[str, str], Package] = {}
    workspace: set[str] = set()

    def walk(dependencies: dict | None) -> None:
        for name, info in (dependencies or {}).items():
            version = str(info.get("version", ""))
            if version.startswith("link:"):
                # One of ClipMill's own packages: its dependencies are bundled.
                if name not in workspace:
                    workspace.add(name)
                    walk(info.get("dependencies"))
                continue
            if name.startswith("@types/") or (name, version) in found:
                continue
            folder = Path(info["path"])
            manifest = json.loads((folder / "package.json").read_text(encoding="utf-8"))
            found[(name, version)] = Package(
                name=name,
                version=version,
                licence=npm_licence(manifest),
                source=f"https://www.npmjs.com/package/{name}/v/{version}",
                folder=folder,
                holders=holders(npm_people(manifest), name),
            )
            walk(info.get("dependencies"))

    for project in json.loads(result.stdout):
        walk(project.get("dependencies"))
    return sorted(found.values(), key=lambda package: (package.name, package.version))


def write(triple: str, out: Path) -> None:
    rust = rust_packages(triple)
    (out / "THIRD-PARTY-RUST.txt").write_text(
        render(
            "Rust crates in ClipMill",
            "The app and clipmilld are compiled from ClipMill's own crates, under ClipMill's "
            f"licence, and from these {len(rust)} crates, as built for {triple}. Each is listed "
            "with the licence it declares and where its source is; the licence texts follow, "
            "each once, under the crates that ship it. A crate that ships no licence text is "
            "given the standard text of a licence it offers, naming its authors.",
            rust,
        ),
        encoding="utf-8",
    )
    javascript = javascript_packages()
    (out / "THIRD-PARTY-JAVASCRIPT.txt").write_text(
        render(
            "JavaScript packages in ClipMill",
            "The app's interface is bundled from ClipMill's own packages, under ClipMill's "
            f"licence, and from these {len(javascript)} npm packages. Each is listed with the "
            "licence it declares and where its source is; the licence texts follow, each once, "
            "under the packages that ship it. A package that ships no licence text is given the "
            "standard text of a licence it offers, naming its authors. The libraries compiled "
            "into the caption player's WebAssembly, and its fallback font, are in "
            "JASSUB-LIBRARIES.txt.",
            javascript,
        ),
        encoding="utf-8",
    )
    say(f"{len(rust)} crates and {len(javascript)} npm packages")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--target", required=True, help="Rust target triple")
    parser.add_argument("--out", required=True, type=Path, help="folder to write the notices to")
    options = parser.parse_args()
    options.out.mkdir(parents=True, exist_ok=True)
    write(options.target, options.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
