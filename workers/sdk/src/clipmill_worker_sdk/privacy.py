"""Whether a file's permissions let anyone but its owner in.

Unix says so in mode bits, and the worker refuses an identity or a staging
directory that others may read. Windows has no such bits: Python reports every
file there as 0o666 and every directory as 0o777. There the daemon's data
folder, in the user's local application data, has an access list that admits
only its user, SYSTEM and administrators, and the daemon relies on it in the
same way (``platform::is_shared`` in ``crates/clipmilld``).
"""

from __future__ import annotations

import stat
import sys


def is_shared(mode: int) -> bool:
    """Whether ``mode`` lets anyone but the owner read, write or enter."""
    return sys.platform != "win32" and bool(stat.S_IMODE(mode) & 0o077)


__all__ = ["is_shared"]
