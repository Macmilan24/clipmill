import pytest
from clipmill_worker_editorial import runtime


@pytest.fixture(autouse=True)
def no_resident_model():
    """Each test starts and ends with no model kept loaded by the worker."""
    runtime.RESIDENT.close()
    yield
    runtime.RESIDENT.close()
