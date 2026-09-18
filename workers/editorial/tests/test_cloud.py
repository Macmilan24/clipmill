import io
import json
import urllib.error
from types import SimpleNamespace

import pytest
from clipmill_worker_editorial.cloud import Budget, BudgetExceeded, CloudModel
from clipmill_worker_sdk import CancellationToken, LeaseCancelled


def config(**kw):
    return SimpleNamespace(
        transcript_consent=kw.get("consent", True),
        model="claude-sonnet-4-6",
        budget_micro_usd=kw.get("cap", 1_000_000),
    )


def reply():
    return {
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 100, "output_tokens": 20},
        "content": [{"type": "text", "text": "{}"}],
    }


def test_permission_images_and_cancellation_stop_before_network(tmp_path):
    calls = []
    with pytest.raises(RuntimeError):
        CloudModel(config(consent=False), tmp_path, "job_TEST", CancellationToken())
    c = CancellationToken()
    m = CloudModel(
        config(), tmp_path, "job_TEST", c, send=lambda *a: calls.append(a), key=lambda: "secret"
    )
    with pytest.raises(RuntimeError):
        m.generate("private text", {}, 20, ["frame.jpg"])
    c.cancel()
    with pytest.raises(LeaseCancelled):
        m.generate("private text", {}, 20)
    assert calls == []


def test_budget_is_shared_across_workers_and_restarts(tmp_path):
    a = Budget(tmp_path, "job_TEST", 20_000)
    a.change(15_000)
    b = Budget(tmp_path, "job_TEST", 20_000)
    with pytest.raises(BudgetExceeded):
        b.change(6_000)
    b.change(5_000)
    assert b.spent == 20_000
    with pytest.raises(BudgetExceeded):
        Budget(tmp_path, "job_TEST", 20_000).change(1)


def test_actual_usage_refunds_reservation_and_never_writes_key(tmp_path):
    m = CloudModel(
        config(),
        tmp_path,
        "job_TEST",
        CancellationToken(),
        send=lambda *a: reply(),
        key=lambda: "secret-never-store",
    )
    assert m.generate("private text", {}, 2048) == ("{}", {"input": 100, "output": 20})
    assert m.budget.spent == 600 and m.last_cost == 600
    assert "secret-never-store" not in (tmp_path / "job_TEST.json").read_text()


def test_uncertain_network_request_stays_reserved_and_error_is_redacted(tmp_path):
    def fail(*a):
        raise RuntimeError("secret-never-store")

    m = CloudModel(
        config(),
        tmp_path,
        "job_TEST",
        CancellationToken(),
        send=fail,
        key=lambda: "secret-never-store",
    )
    with pytest.raises(RuntimeError, match="Anthropic unavailable") as e:
        m.generate("private text", {}, 2048)
    assert "secret-never-store" not in str(e.value)
    assert m.budget.spent > 0


def test_no_request_when_cap_is_too_small(tmp_path):
    calls = []
    m = CloudModel(
        config(cap=10_000),
        tmp_path,
        "job_TEST",
        CancellationToken(),
        send=lambda *a: calls.append(a),
        key=lambda: "secret",
    )
    with pytest.raises(BudgetExceeded):
        m.generate("text", {}, 2048)
    assert calls == []


def test_provider_response_cannot_echo_the_key_into_a_trace(tmp_path):
    result = reply()
    result["content"][0]["text"] = '{"detail":"secret-never-store"}'
    model = CloudModel(
        config(),
        tmp_path,
        "job_TEST",
        CancellationToken(),
        send=lambda *args: result,
        key=lambda: "secret-never-store",
    )
    text, _ = model.generate("text", {}, 2048)
    assert "secret-never-store" not in text and "[REDACTED]" in text


def test_cancellation_after_reservation_refunds_without_sending(tmp_path):
    cancel = CancellationToken()
    calls = []
    model = CloudModel(
        config(),
        tmp_path,
        "job_TEST",
        cancel,
        send=lambda *args: calls.append(args),
        key=lambda: "secret",
    )
    original = model.budget.change

    def reserve_and_cancel(delta):
        original(delta)
        if delta > 0:
            cancel.cancel()

    model.budget.change = reserve_and_cancel
    with pytest.raises(LeaseCancelled):
        model.generate("text", {}, 2048)
    assert calls == [] and model.budget.spent == 0 and model.last_cost == 0


def test_request_uses_documented_messages_structured_output_shape(tmp_path, monkeypatch):
    from clipmill_worker_editorial.inference import ReviewReply

    requests = []

    class Opener:
        def open(self, request, timeout):
            requests.append((request, timeout))
            return io.BytesIO(json.dumps(reply()).encode())

    monkeypatch.setattr("urllib.request.build_opener", lambda *args: Opener())
    model = CloudModel(config(), tmp_path, "job_TEST", CancellationToken(), key=lambda: "secret")
    model.generate("transcript text", ReviewReply.model_json_schema(), 2048)
    request, timeout = requests[0]
    body = json.loads(request.data)
    assert request.full_url == "https://api.anthropic.com/v1/messages"
    assert request.method == "POST" and timeout == 60
    assert request.get_header("Anthropic-version") == "2023-06-01"
    assert request.get_header("X-api-key") == "secret"
    assert body["model"] == "claude-sonnet-4-6"
    assert body["messages"] == [{"role": "user", "content": "transcript text"}]
    assert body["output_config"]["format"]["type"] == "json_schema"
    schema = body["output_config"]["format"]["schema"]
    assert schema["additionalProperties"] is False
    assert schema["properties"]["status"]["enum"] == ["accepted", "needs_review", "rejected"]
    assert "maxLength" not in schema["properties"]["summary"]
    assert "secret" not in request.data.decode()


@pytest.mark.parametrize(
    ("status", "message", "retryable"),
    [
        (401, "rejected the API key", False),
        (403, "does not have permission", False),
        (429, "rate or spend limit", True),
        (529, "HTTP 529", True),
    ],
)
def test_provider_errors_are_actionable_redacted_and_classified(
    tmp_path, status, message, retryable
):
    from clipmill_worker_editorial.errors import ProviderRequestError

    calls = []

    def fail(*args):
        calls.append(args)
        raise urllib.error.HTTPError(
            "https://api.anthropic.com/v1/messages",
            status,
            "secret-never-store",
            {},
            io.BytesIO(b'{"error":"secret-never-store"}'),
        )

    model = CloudModel(
        config(),
        tmp_path,
        "job_TEST",
        CancellationToken(),
        send=fail,
        key=lambda: "secret-never-store",
    )
    with pytest.raises(ProviderRequestError, match=message) as error:
        model.generate("text", {}, 2048)
    assert error.value.retryable is retryable
    assert "secret-never-store" not in str(error.value)
    assert model.budget.spent > 0
    if not retryable:
        # Each subsequent window still gets a failure record, without sending
        # the same invalid credentials again or reserving more budget.
        spent = model.budget.spent
        with pytest.raises(ProviderRequestError, match=message):
            model.generate("next window", {}, 2048)
        assert len(calls) == 1 and model.budget.spent == spent and model.last_cost == 0
