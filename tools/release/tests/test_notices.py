"""A release's notices must carry every licence text, and refuse to guess one."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "notices", Path(__file__).resolve().parents[1] / "notices.py"
)
assert SPEC is not None and SPEC.loader is not None
notices = importlib.util.module_from_spec(SPEC)
# A dataclass resolves its annotations through the module it was defined in.
sys.modules["notices"] = notices
SPEC.loader.exec_module(notices)


def package(folder: Path, name: str, licence: str, holders: str = "Ada Lovelace"):
    return notices.Package(
        name=name,
        version="1.0.0",
        licence=licence,
        source=f"https://example.invalid/{name}",
        folder=folder,
        holders=holders,
    )


class ExpressionTests(unittest.TestCase):
    def test_flat_expressions_offer_their_choices(self):
        self.assertEqual(notices.alternatives("MIT OR Apache-2.0"), [["MIT"], ["Apache-2.0"]])
        self.assertEqual(notices.alternatives("MIT/Apache-2.0"), [["MIT"], ["Apache-2.0"]])
        self.assertEqual(notices.alternatives("Apache-2.0 / MIT"), [["Apache-2.0"], ["MIT"]])
        self.assertEqual(
            notices.alternatives("(Apache-2.0 AND BSD-3-Clause)"), [["Apache-2.0", "BSD-3-Clause"]]
        )

    def test_a_nested_expression_offers_nothing_rather_than_a_guess(self):
        self.assertEqual(notices.alternatives("(MIT OR Apache-2.0) AND Unicode-3.0"), [])
        self.assertEqual(notices.alternatives(""), [])


class StandardTextTests(unittest.TestCase):
    def setUp(self):
        self.folder = Path(tempfile.mkdtemp())

    def test_a_package_offering_apache_gets_the_text_that_names_no_holder(self):
        texts = notices.standard_texts(
            package(self.folder, "objc2-web-kit", "Zlib OR Apache-2.0 OR MIT")
        )
        self.assertEqual([term for term, _ in texts], ["Apache-2.0"])
        self.assertIn("Apache License", texts[0][1])

    def test_a_holder_is_named_where_the_licence_names_one(self):
        texts = notices.standard_texts(
            package(self.folder, "objc2", "MIT", holders="Mads Marquart")
        )
        self.assertTrue(texts[0][1].startswith("Copyright (c) Mads Marquart\n"))

    def test_code_carried_from_another_project_names_that_project(self):
        texts = dict(
            notices.standard_texts(
                package(self.folder, "@bufbuild/protobuf", "(Apache-2.0 AND BSD-3-Clause)")
            )
        )
        self.assertIn("Copyright (c) 2008 Google Inc.", texts["BSD-3-Clause"])
        self.assertIn("Apache License", texts["Apache-2.0"])

    def test_a_licence_without_a_standard_text_is_refused(self):
        with self.assertRaises(SystemExit) as refusal:
            notices.standard_texts(package(self.folder, "unicode-ident", "Unicode-3.0"))
        self.assertIn("unicode-ident", str(refusal.exception))


class LicenceFileTests(unittest.TestCase):
    def setUp(self):
        self.folder = Path(tempfile.mkdtemp())

    def write(self, relative: str, text: str = "text") -> None:
        path = self.folder / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def test_own_and_vendored_texts_are_found_and_code_is_not(self):
        self.write("LICENSE-MIT")
        self.write("LICENSE-APACHE.md")
        self.write("README.md")
        self.write("LICENSES/Zlib.txt")
        self.write("third_party/fiat/LICENSE")
        self.write("src/copying.rs")
        self.write("dist/icons/copyright.mjs")
        self.write("node_modules/dep/LICENSE")
        own, vendored = notices.licence_files(package(self.folder, "crate", "MIT"))
        root = self.folder.resolve()
        self.assertEqual(
            sorted(path.resolve().relative_to(root).as_posix() for path in own),
            ["LICENSE-APACHE.md", "LICENSE-MIT", "LICENSES/Zlib.txt"],
        )
        self.assertEqual(
            [path.resolve().relative_to(root).as_posix() for path in vendored],
            ["third_party/fiat/LICENSE"],
        )


class RenderTests(unittest.TestCase):
    def test_each_text_is_written_once_under_every_package_that_ships_it(self):
        root = Path(tempfile.mkdtemp())
        for name in ("alpha", "beta"):
            (root / name).mkdir()
            (root / name / "LICENSE").write_text("Shared terms.\r\n\r\n", encoding="utf-8")
        (root / "gamma").mkdir()
        packages = [
            package(root / "alpha", "alpha", "MIT"),
            package(root / "beta", "beta", "MIT"),
            package(root / "gamma", "gamma", "MIT", holders="Grace Hopper"),
        ]
        text = notices.render("Title", "Introduction.", packages)
        self.assertEqual(text.count("Shared terms."), 1)
        self.assertIn("alpha 1.0.0 (LICENSE), beta 1.0.0 (LICENSE)", text)
        self.assertIn("gamma 1.0.0 (standard MIT text)", text)
        self.assertIn("Copyright (c) Grace Hopper", text)
        self.assertIn("gamma 1.0.0: MIT, https://example.invalid/gamma", text)


class HolderTests(unittest.TestCase):
    def test_addresses_are_left_out_and_nobody_named_means_the_authors(self):
        self.assertEqual(
            notices.holders(
                ["Ada <ada@example.invalid> (https://ada.example.invalid)", "Grace"], "x"
            ),
            "Ada, Grace",
        )
        self.assertEqual(notices.holders([], "left-pad"), "the left-pad authors")

    def test_npm_people_and_licences_in_every_form(self):
        manifest = {
            "author": {"name": "Ada"},
            "contributors": ["Grace <grace@example.invalid>"],
            "licenses": [{"type": "MIT"}, {"type": "Apache-2.0"}],
        }
        self.assertEqual(notices.npm_people(manifest), ["Ada", "Grace <grace@example.invalid>"])
        self.assertEqual(notices.npm_licence(manifest), "MIT OR Apache-2.0")
        self.assertEqual(notices.npm_licence({"license": {"type": "ISC"}}), "ISC")
        self.assertEqual(notices.npm_licence({}), "not stated")


if __name__ == "__main__":
    unittest.main()
