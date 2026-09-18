"""Separate, explicitly launched transcript-only network worker."""

from . import execute_stage, run_worker
from .cloud import CloudModel

CAPABILITIES = ("editorial-propose-cloud", "editorial-review-cloud")


def execute(context):
    return execute_stage(context, CAPABILITIES, CloudModel)


def main() -> int:
    return run_worker(
        execute,
        capabilities=CAPABILITIES,
        description="ClipMill opt-in transcript-only cloud editorial worker",
        backend="cloud",
        max_memory_bytes=128 * 1024**2,
    )
