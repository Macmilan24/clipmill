"""The runtime check proves a model runs here, and says how fast."""

from __future__ import annotations

import json
import os

import pytest
from clipmill.worker.v1 import worker_pb2
from clipmill_worker_editorial import runtime_check
from clipmill_worker_sdk.weights import VerifiedModel


class Runtime:
    """Answers the readiness prompt, then the measurement, as a model would."""

    def __init__(self, *, measured: bool):
        self.prompts: list[str] = []
        if measured:
            self.last_timings: dict = {}

    def generate(self, prompt, schema, max_tokens, images=None):
        self.prompts.append(prompt)
        if prompt == runtime_check.MEASURE_PROMPT:
            self.last_timings = {"prompt_per_second": 912.5, "predicted_per_second": 41.0}
            return json.dumps({"words": ["word"] * 20}), {"input": 1560, "output": 80}
        if hasattr(self, "last_timings"):
            self.last_timings = {"prompt_per_second": 48.0, "predicted_per_second": 60.0}
        return '{"ready":true}', {"input": 22, "output": 6}

    def close(self):
        pass


def prove(tmp_path, monkeypatch, runtime, files):
    model = VerifiedModel(
        name="qwen",
        capability="editorial",
        digest="sha256:" + "1" * 64,
        root=tmp_path,
        files=files,
    )
    monkeypatch.setenv("CLIPMILL_LLAMA_SERVER", "/pinned/llama-server")
    monkeypatch.setattr(runtime_check, "verify_model", lambda _binding: model)
    monkeypatch.setattr(runtime_check, "open_runtime", lambda _model, _cancellation: runtime)
    receipt = tmp_path / "editorial-runtime.json"
    document = runtime_check.prove(worker_pb2.ModelBinding(), receipt, "fingerprint")
    assert json.loads(receipt.read_text()) == document
    return document


def test_a_llama_cpp_proof_measures_a_representative_reply(tmp_path, monkeypatch):
    runtime = Runtime(measured=True)
    document = prove(tmp_path, monkeypatch, runtime, ("Qwen3.5-9B-Q4_K_M.gguf", "mmproj-F16.gguf"))
    assert runtime.prompts == [runtime.prompts[0], runtime_check.MEASURE_PROMPT]
    assert document["runtime"] == "llama.cpp@external/clipmill-json-v1"
    # The speeds of the long reply, not of the one-line readiness answer.
    assert document["prompt_tokens_per_second"] == 912.5
    assert document["output_tokens_per_second"] == 41.0
    assert document["validated"] is True


def test_the_receipt_is_written_where_there_is_no_o_nofollow(tmp_path, monkeypatch):
    # Windows has no O_NOFOLLOW.
    monkeypatch.delattr(os, "O_NOFOLLOW", raising=False)
    document = prove(
        tmp_path,
        monkeypatch,
        Runtime(measured=True),
        ("Qwen3.5-9B-Q4_K_M.gguf", "mmproj-F16.gguf"),
    )
    assert document["hardware_fingerprint"] == "fingerprint"


def test_an_mlx_proof_takes_one_reply_and_reports_no_speed(tmp_path, monkeypatch):
    runtime = Runtime(measured=False)
    document = prove(tmp_path, monkeypatch, runtime, ("config.json", "model.safetensors"))
    assert len(runtime.prompts) == 1
    assert document["runtime"] == runtime_check.RUNTIME
    assert "output_tokens_per_second" not in document


@pytest.mark.parametrize("schema", [runtime_check.READY_SCHEMA, runtime_check.MEASURE_SCHEMA])
def test_the_check_asks_only_for_closed_objects(schema):
    assert schema["additionalProperties"] is False
