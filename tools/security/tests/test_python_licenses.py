"""The license gate must fail closed without rejecting reviewed SPDX syntax."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import unittest
from email.message import Message
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "python_license_policy", Path(__file__).resolve().parents[1] / "check-python-licenses.py"
)
assert SPEC is not None and SPEC.loader is not None
policy = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(policy)


def distribution(name: str, version: str, license_text: str, *, expression: bool = False):
    metadata = Message()
    metadata["Name"] = name
    metadata["License-Expression" if expression else "License"] = license_text
    metadata["Classifier"] = "License :: OSI Approved :: BSD License"
    return SimpleNamespace(metadata=metadata, version=version)


class LicensePolicyTests(unittest.TestCase):
    def check_distribution(self, value) -> int:
        with (
            patch.object(policy.importlib.metadata, "distributions", return_value=[value]),
            contextlib.redirect_stderr(io.StringIO()),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            return policy.check_current_environment()

    def test_torch_composite_declaration(self):
        self.assertTrue(
            policy.is_allowed(
                "Apache-2.0 AND Apache-2.0 WITH LLVM-exception AND BSD-2-Clause "
                "AND BSD-3-Clause AND BSL-1.0 AND MIT"
            )
        )

    def test_reviewed_notice_preserving_licenses(self):
        for expression in ("MIT-CMU", "BSL-1.0", "BSD-3-Clause AND MIT"):
            with self.subTest(expression=expression):
                self.assertTrue(policy.is_allowed(expression))

    def test_operator_precedence_and_parentheses(self):
        self.assertTrue(policy.is_allowed("MIT OR GPL-3.0-only AND Apache-2.0"))
        self.assertFalse(policy.is_allowed("(MIT OR Apache-2.0) AND GPL-3.0-only"))
        self.assertTrue(policy.is_allowed("(MIT AND BSD-3-Clause) OR GPL-3.0-only"))

    def test_unknown_or_incompatible_required_terms_still_fail(self):
        for expression in (
            "BSD",
            "BSD-4-Clause",
            "LicenseRef-Unreviewed",
            "MIT AND GPL-3.0-only",
            "MIT AND (BSD-3-Clause OR LicenseRef-Unreviewed) AND GPL-3.0-only",
        ):
            with self.subTest(expression=expression):
                self.assertFalse(policy.is_allowed(expression))

    def test_exception_requires_reviewed_base_and_exact_exception(self):
        for expression in (
            "MIT WITH LLVM-exception",
            "Apache-2.0 WITH Classpath-exception-2.0",
            "Apache-2.0 WITH Unknown-exception",
            "LLVM-exception",
        ):
            with self.subTest(expression=expression):
                self.assertFalse(policy.is_allowed(expression))

    def test_malformed_expressions_fail_even_after_allowed_alternative(self):
        for expression in (
            "",
            "MIT OR",
            "MIT AND",
            "(MIT",
            "MIT)",
            "MIT OR (Apache-2.0 AND)",
            "MIT OR Apache-2.0 WITH",
            "MIT OR ???",
            "MIT OR MIT WITH WITH MIT",
            "MIT Apache-2.0",
            "MIT OR ()",
        ):
            with self.subTest(expression=expression):
                self.assertFalse(policy.is_allowed(expression))

    def test_ambiguous_bsd_is_only_corrected_for_reviewed_releases(self):
        for name, version in (("mpmath", "1.3.0"), ("sympy", "1.14.0"), ("torchvision", "0.29.0")):
            with self.subTest(name=name):
                self.assertEqual(self.check_distribution(distribution(name, version, "BSD")), 0)
                self.assertEqual(self.check_distribution(distribution(name, "99.0", "BSD")), 1)
                self.assertEqual(
                    self.check_distribution(distribution(name, version, "BSD-4-Clause")), 1
                )
        self.assertEqual(self.check_distribution(distribution("other-package", "1.0", "BSD")), 1)

    def test_classifiers_cannot_override_explicit_expression(self):
        for expression in ("GPL-3.0-only", "MIT AND Unknown-License", "MIT AND"):
            with self.subTest(expression=expression):
                self.assertEqual(
                    self.check_distribution(
                        distribution("example", "1", expression, expression=True)
                    ),
                    1,
                )

    def test_missing_metadata_override_never_masks_changed_declaration(self):
        self.assertEqual(
            self.check_distribution(distribution("annotated-types", "1", "GPL-3.0-only")), 1
        )

    def test_long_legacy_expression_cannot_fall_back_to_classifier(self):
        expression = " AND ".join(["MIT"] * 30 + ["GPL-3.0-only"])
        self.assertGreater(len(expression), 128)
        self.assertEqual(self.check_distribution(distribution("example", "1", expression)), 1)

    def test_full_legacy_license_prose_can_use_package_classifier(self):
        self.assertEqual(
            self.check_distribution(
                distribution("example", "1", "Copyright example\nRedistribution and use...")
            ),
            0,
        )


if __name__ == "__main__":
    unittest.main()
