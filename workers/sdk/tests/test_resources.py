"""A worker's memory ceiling follows the machine, never below its old floor."""

from __future__ import annotations

import pytest
from clipmill_worker_sdk import resources


def test_the_ceiling_is_the_machines_share_above_the_floor(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.delenv(resources.OVERRIDE, raising=False)
    monkeypatch.setattr(resources, "machine_memory", lambda: 32 * 1024**3)
    assert resources.memory_ceiling(768 * 1024**2) == int(32 * 1024**3 * resources.SHARE)


def test_a_small_machine_keeps_the_floor(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.delenv(resources.OVERRIDE, raising=False)
    monkeypatch.setattr(resources, "machine_memory", lambda: 1024**3)
    assert resources.memory_ceiling(768 * 1024**2) == 768 * 1024**2


def test_an_unreadable_machine_keeps_the_floor(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.delenv(resources.OVERRIDE, raising=False)
    monkeypatch.setattr(resources, "machine_memory", lambda: 0)
    assert resources.memory_ceiling(512) == 512


def test_an_explicit_override_wins_and_nonsense_is_ignored(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setattr(resources, "machine_memory", lambda: 32 * 1024**3)
    monkeypatch.setenv(resources.OVERRIDE, str(3 * 1024**3))
    assert resources.memory_ceiling(512) == 3 * 1024**3
    monkeypatch.setenv(resources.OVERRIDE, "lots")
    assert resources.memory_ceiling(512) == int(32 * 1024**3 * resources.SHARE)


def test_the_share_stays_under_the_daemons_budget():
    # The daemon leases only to workers declaring at most 75% of memory.
    assert resources.SHARE < 0.75


def test_this_machine_reports_some_memory():
    assert resources.machine_memory() > 0
