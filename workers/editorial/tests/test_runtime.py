import gc
import sys
import traceback
import weakref
from types import ModuleType, SimpleNamespace
from unittest.mock import Mock

import pytest
from clipmill_worker_editorial.runtime import LocalModel


@pytest.fixture
def mlx_runtime(monkeypatch):
    # No MLX import or model allocation: these tests also run on Linux.
    core = ModuleType("mlx.core")
    core.random = SimpleNamespace(seed=Mock())
    core.clear_cache = Mock()
    mlx = ModuleType("mlx")
    mlx.core = core
    vlm = ModuleType("mlx_vlm")
    vlm.load = Mock(return_value=(object(), object()))
    collect = Mock(wraps=gc.collect)
    monkeypatch.setitem(sys.modules, "mlx", mlx)
    monkeypatch.setitem(sys.modules, "mlx.core", core)
    monkeypatch.setitem(sys.modules, "mlx_vlm", vlm)
    monkeypatch.setattr("clipmill_worker_editorial.runtime.gc.collect", collect)
    monkeypatch.setenv("HF_HUB_OFFLINE", "1")
    monkeypatch.setenv("TRANSFORMERS_OFFLINE", "1")
    return SimpleNamespace(core=core, load=vlm.load, collect=collect)


@pytest.mark.parametrize("cleanup_failure", [None, "traceback", "collect", "cache"])
def test_failed_load_releases_memory_and_preserves_original_error(
    tmp_path, mlx_runtime, cleanup_failure, monkeypatch
):
    failure = MemoryError("model allocation failed")
    mlx_runtime.load.side_effect = failure
    if cleanup_failure == "traceback":
        monkeypatch.setattr(
            "clipmill_worker_editorial.runtime.traceback.clear_frames",
            Mock(side_effect=RuntimeError("frame cleanup failed")),
        )
    elif cleanup_failure == "collect":
        mlx_runtime.collect.side_effect = RuntimeError("collection failed")
    elif cleanup_failure == "cache":
        mlx_runtime.core.clear_cache.side_effect = RuntimeError("cache cleanup failed")
    model = LocalModel.__new__(LocalModel)

    with pytest.raises(MemoryError) as raised:
        model.__init__(tmp_path, SimpleNamespace(raise_if_cancelled=Mock()))

    assert raised.value is failure
    assert model.model is None
    assert model.processor is None
    mlx_runtime.collect.assert_called_once_with()
    mlx_runtime.core.clear_cache.assert_called_once_with()


def test_failed_load_releases_traceback_owned_allocation_before_clearing_cache(
    tmp_path, mlx_runtime
):
    class PartialModel:
        pass

    failure = MemoryError("model allocation failed")
    allocations = []
    original_stack = []
    released_before_cache_cleanup = []

    def failing_loader(*args, **kwargs):
        partial_model = PartialModel()
        partial_model.cycle = partial_model
        allocations.append(weakref.ref(partial_model))
        try:
            raise failure
        except MemoryError as error:
            original_stack.extend(traceback.extract_tb(error.__traceback__))
            raise

    mlx_runtime.load.side_effect = failing_loader

    def clear_cache():
        released_before_cache_cleanup.append(len(allocations) == 1 and allocations[0]() is None)

    mlx_runtime.core.clear_cache.side_effect = clear_cache
    with pytest.raises(MemoryError) as raised:
        LocalModel(tmp_path, SimpleNamespace(raise_if_cancelled=Mock()))

    assert raised.value is failure
    assert str(raised.value) == "model allocation failed"
    assert released_before_cache_cleanup == [True]
    assert traceback.extract_tb(failure.__traceback__)[-1] == original_stack[-1]
    mlx_runtime.core.clear_cache.assert_called_once_with()


def test_close_releases_loaded_model_before_collecting_and_clearing_cache(tmp_path, mlx_runtime):
    cancellation = SimpleNamespace(raise_if_cancelled=Mock())
    model = LocalModel(tmp_path, cancellation)
    assert model.model is mlx_runtime.load.return_value[0]
    assert model.processor is mlx_runtime.load.return_value[1]
    assert model.cancellation is cancellation
    mlx_runtime.load.assert_called_once_with(str(tmp_path), trust_remote_code=False)
    mlx_runtime.collect.assert_not_called()
    mlx_runtime.core.clear_cache.assert_not_called()
    released = []

    def collect():
        assert model.model is None
        assert model.processor is None
        released.append("collect")

    mlx_runtime.collect.side_effect = collect
    mlx_runtime.core.clear_cache.side_effect = lambda: released.append("cache")
    model.close()

    assert released == ["collect", "cache"]
