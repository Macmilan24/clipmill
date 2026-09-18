"""Failure categories shared by local inference and the opt-in cloud adapter."""


class BudgetExceeded(RuntimeError):
    pass


class ProviderRequestError(RuntimeError):
    """A redacted provider failure with explicit retry semantics."""

    def __init__(self, detail: str, *, retryable: bool):
        super().__init__(detail)
        self.retryable = retryable
