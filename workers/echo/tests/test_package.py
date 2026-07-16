import clipmill_worker_echo
import clipmill_worker_sdk


def test_sdk_dependency_resolves() -> None:
    assert clipmill_worker_echo.__version__ == clipmill_worker_sdk.__version__
