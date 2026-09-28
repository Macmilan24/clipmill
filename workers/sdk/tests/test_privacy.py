"""Who may read a worker's identity and staging, on each system."""

from __future__ import annotations

import pytest
from clipmill_worker_sdk import privacy


def test_mode_bits_decide_on_unix(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(privacy.sys, "platform", "linux")
    assert privacy.is_shared(0o100644)
    assert privacy.is_shared(0o040755)
    assert not privacy.is_shared(0o100600)
    assert not privacy.is_shared(0o040700)


def test_the_data_folder_decides_on_windows(monkeypatch: pytest.MonkeyPatch) -> None:
    # Python reports every Windows file as 0o666 and directory as 0o777.
    monkeypatch.setattr(privacy.sys, "platform", "win32")
    assert not privacy.is_shared(0o100666)
    assert not privacy.is_shared(0o040777)
