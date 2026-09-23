"""How much memory a worker offers to take tasks up to.

The ceiling a worker declares is a promise to the scheduler: it is never handed
a task whose model needs more. A constant sized for one model is a promise
about one model, so a worker that loads whatever model its lease binds — a
person may choose a larger one, or pin their own — sizes its ceiling from the
machine it is running on instead.

The daemon refuses to lease to a worker that declares more than the device's
processing budget (three quarters of memory), so the share here stays under
it. The floor keeps a small machine at the ceiling the worker always had.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

#: The share of memory offered. Below the daemon's 75% processing budget, so
#: a declaration sized this way is one the daemon admits.
SHARE = 0.70

#: An explicit ceiling in bytes, for a machine where the measured one is wrong.
OVERRIDE = "CLIPMILL_WORKER_MEMORY_BYTES"


def memory_ceiling(floor: int) -> int:
    """The larger of `floor` and this machine's share, unless overridden."""

    override = os.environ.get(OVERRIDE, "").strip()
    if override:
        try:
            value = int(override)
        except ValueError:
            value = 0
        if value > 0:
            return value
    measured = machine_memory()
    if measured <= 0:
        return floor
    return max(floor, int(measured * SHARE))


def machine_memory() -> int:
    """Physical memory on macOS; memory available now elsewhere.

    The same readings the daemon budgets from: macOS reclaims and compresses,
    so its budget follows physical memory, while Linux follows `MemAvailable`.
    Zero when neither can be read.
    """

    if sys.platform == "darwin":
        try:
            return os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES")
        except (OSError, ValueError):
            return 0
    try:
        for line in Path("/proc/meminfo").read_text(encoding="ascii").splitlines():
            if line.startswith("MemAvailable:"):
                return int(line.split()[1]) * 1024
    except (OSError, ValueError, IndexError):
        return 0
    return 0


__all__ = ["OVERRIDE", "SHARE", "machine_memory", "memory_ceiling"]
