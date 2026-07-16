"""The echo worker: reference implementation of the worker protocol.

Populated in workstream W6. It advertises a trivial capability, takes leased
tasks, streams heartbeats, and completes without doing any work — proving the
protocol end to end.
"""

__version__ = "0.0.1"
