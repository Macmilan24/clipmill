"""A release may delegate a test only to a workflow covered by the same policy."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

CHECK = Path(__file__).resolve().parents[1] / "check-actions.py"
HEADER = "permissions:\n  contents: read\nconcurrency:\n  cancel-in-progress: true\njobs:\n"


class WorkflowCallPolicy(unittest.TestCase):
    def check(self, caller: str, callee: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as root:
            folder = Path(root) / ".github/workflows"
            folder.mkdir(parents=True)
            (folder / "release.yml").write_text(HEADER + caller)
            (folder / "smoke.yml").write_text(HEADER + callee)
            return subprocess.run([sys.executable, CHECK], cwd=root, capture_output=True, text=True)

    def test_callee_enforces_the_timeout_at_the_runnable_job(self):
        caller = "  smoke:\n    uses: ./.github/workflows/smoke.yml\n"
        job = "  exercise:\n    runs-on: windows-2025\n"
        self.assertEqual(self.check(caller, job + "    timeout-minutes: 10\n").returncode, 0)
        self.assertIn("smoke.yml:exercise: explicit timeout", self.check(caller, job).stderr)

    def test_unchecked_workflow_and_delegated_write_are_rejected(self):
        local = "    uses: ./.github/workflows/smoke.yml\n"
        job = "  exercise:\n    runs-on: windows-2025\n    timeout-minutes: 10\n"
        external = "  smoke:\n    uses: someone/other/.github/workflows/smoke.yml@main\n"
        self.assertIn("called workflow must be local", self.check(external, job).stderr)
        writable = "  publish:\n    permissions:\n      contents: write\n" + local
        self.assertIn("write permission", self.check(writable, job).stderr)
