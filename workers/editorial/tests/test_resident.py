from types import SimpleNamespace

import pytest
from clipmill_worker_editorial.runtime import ResidentModel
from clipmill_worker_sdk import CancellationToken, LeaseCancelled


class Clock:
    def __init__(self):
        self.now = 0.0

    def __call__(self):
        return self.now


class Runtime:
    def __init__(self, model, cancellation, opened):
        self.model = model
        self.cancellation = cancellation
        self.closed = False
        opened.append(self)

    def generate(self, prompt, *args):
        if prompt == "fail":
            raise RuntimeError("model failed")
        return "{}", {"input": 1, "output": 1}

    def close(self):
        self.closed = True


def model(name="qwen", digest="sha256:a"):
    return SimpleNamespace(name=name, digest=digest)


@pytest.fixture
def pool():
    opened = []
    clock = Clock()
    resident = ResidentModel(
        lambda model, cancellation: Runtime(model, cancellation, opened),
        keep_warm=90,
        clock=clock,
    )
    yield resident, opened, clock
    resident.close()


def test_the_next_task_for_the_same_model_reuses_it(pool):
    resident, opened, _ = pool
    first, second = CancellationToken(), CancellationToken()
    runtime = resident.acquire(model(), first)
    runtime.generate("propose")
    runtime.close()
    runtime = resident.acquire(model(), second)
    assert len(opened) == 1
    assert opened[0].cancellation is second, "each task cancels only its own calls"
    runtime.close()


def test_another_model_unloads_the_resident_one_first(pool):
    resident, opened, _ = pool
    resident.acquire(model(digest="sha256:a"), CancellationToken()).close()
    resident.acquire(model(digest="sha256:b"), CancellationToken()).close()
    assert [runtime.closed for runtime in opened] == [True, False]
    assert resident.resident == ("qwen", "sha256:b")


def test_a_model_whose_call_failed_is_unloaded(pool):
    resident, opened, _ = pool
    runtime = resident.acquire(model(), CancellationToken())
    with pytest.raises(RuntimeError):
        runtime.generate("fail")
    runtime.close()
    assert opened[0].closed and resident.resident is None


def test_an_idle_model_is_unloaded_after_the_keep_warm_period(pool):
    resident, opened, clock = pool
    resident.acquire(model(), CancellationToken()).close()
    clock.now = 89
    resident.expire()
    assert not opened[0].closed
    clock.now = 90
    resident.expire()
    assert opened[0].closed and resident.resident is None


def test_keep_warm_zero_unloads_when_the_task_ends():
    opened = []
    resident = ResidentModel(
        lambda model, cancellation: Runtime(model, cancellation, opened), keep_warm=0
    )
    resident.acquire(model(), CancellationToken()).close()
    assert opened[0].closed and resident.resident is None


def test_a_borrowed_model_is_never_lent_twice(pool):
    resident, _, _ = pool
    held = resident.acquire(model(), CancellationToken())
    with pytest.raises(RuntimeError):
        resident.acquire(model(), CancellationToken())
    held.close()
    held.close()  # closing twice returns it once


def test_a_server_that_exited_is_started_again(pool):
    resident, opened, _ = pool
    runtime = resident.acquire(model(), CancellationToken())
    opened[0].process = SimpleNamespace(poll=lambda: 1)
    runtime.close()
    resident.acquire(model(), CancellationToken()).close()
    assert len(opened) == 2 and opened[0].closed


def test_a_cancelled_task_loads_nothing(pool):
    resident, opened, _ = pool
    token = CancellationToken()
    token.cancel()
    with pytest.raises(LeaseCancelled):
        resident.acquire(model(), token)
    assert opened == []
    resident.acquire(model(), CancellationToken()).close()


def test_stopping_the_worker_unloads_the_model(pool):
    resident, opened, _ = pool
    resident.acquire(model(), CancellationToken()).close()
    resident.close()
    assert opened[0].closed and resident.resident is None
