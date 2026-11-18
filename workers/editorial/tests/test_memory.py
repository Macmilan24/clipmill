import tomllib
from pathlib import Path

import clipmill_worker_editorial as editorial
import pytest
from clipmill_worker_sdk import resources

REGISTRY = Path(__file__).resolve().parents[3] / "models" / "registry"


def resident_bytes(name: str) -> int:
    manifest = tomllib.loads((REGISTRY / f"{name}.toml").read_text())
    return manifest["memory"]["weights_bytes"] + manifest["memory"]["runtime_overhead_bytes"]


@pytest.mark.parametrize(
    ("backend", "model"),
    [("mlx", "qwen3-5-4b-editorial-mlx"), ("llama.cpp", "qwen3-5-4b-editorial-gguf")],
)
def test_the_floor_is_the_smallest_model_the_manifest_declares(backend: str, model: str):
    assert editorial.SMALLEST_MODEL_BYTES[backend] == resident_bytes(model)


def test_an_8_gb_mac_offers_the_daemon_a_ceiling_it_admits(monkeypatch: pytest.MonkeyPatch):
    eight_gb = 8 * 1024**3
    monkeypatch.delenv(resources.OVERRIDE, raising=False)
    monkeypatch.setattr(resources, "machine_memory", lambda: eight_gb)
    ceiling = resources.memory_ceiling(editorial.SMALLEST_MODEL_BYTES["mlx"])
    # The daemon leases to a worker only within three quarters of memory.
    assert ceiling <= eight_gb * 3 // 4
    # The 4B fits under it; the 9B does not, and readiness says so instead.
    assert resident_bytes("qwen3-5-4b-editorial-mlx") <= ceiling
    assert resident_bytes("qwen3-5-editorial-mlx") > ceiling
