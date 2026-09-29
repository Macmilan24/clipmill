"""The llama.cpp runtime against a stand-in for llama-server.

The stand-in speaks the part of the server's HTTP API the runtime uses, checks
the key the runtime made, and records every request, so these tests run
anywhere, with no model and no GPU.
"""

from __future__ import annotations

import base64
import io
import json
import os
import stat
import sys
import threading
import time
from pathlib import Path

import pytest
from clipmill_worker_editorial import server_runtime
from clipmill_worker_editorial.runtime import implementation_suffix, runtime_name
from clipmill_worker_editorial.server_runtime import ServerModel, gguf_files, is_gguf
from clipmill_worker_sdk import CancellationToken, LeaseCancelled
from clipmill_worker_sdk.weights import VerifiedModel

pytestmark = pytest.mark.skipif(sys.platform == "win32", reason="the stand-in is a script")

STAND_IN = r"""#!{python}
import json, os, sys, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

arguments = sys.argv[1:]
port = int(arguments[arguments.index("--port") + 1])
key = os.environ["LLAMA_API_KEY"]
behaviour = json.loads(os.environ.get("STAND_IN", "{{}}"))
record = os.environ["STAND_IN_RECORD"]
with open(record, "a") as out:
    out.write(json.dumps({{"path": "argv", "body": arguments}}) + "\n")
if behaviour.get("exit"):
    sys.exit(behaviour["exit"])


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def reply(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self.reply(200 if self.path == "/health" else 404, {{"status": "ok"}})

    def do_POST(self):
        if self.headers.get("Authorization") != "Bearer " + key:
            return self.reply(401, {{"error": "invalid key"}})
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        with open(record, "a") as out:
            out.write(json.dumps({{"path": self.path, "body": body}}) + "\n")
        if self.path == "/apply-template":
            prompt = "<user>" + body["messages"][0]["content"] + "</user>"
            return self.reply(200, {{"prompt": prompt}})
        if self.path == "/tokenize":
            return self.reply(200, {{"tokens": list(range(len(body["content"].split())))}})
        if self.path != "/v1/chat/completions":
            return self.reply(404, {{}})
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        for piece in behaviour.get("pieces", ['{{"ready"', ":true}}"]):
            chunk = {{"choices": [{{"delta": {{"content": piece}}}}]}}
            self.wfile.write(("data: " + json.dumps(chunk) + "\n\n").encode())
            self.wfile.flush()
            time.sleep(behaviour.get("delay", 0))
        final = {{
            "choices": [],
            "usage": {{"prompt_tokens": 7, "completion_tokens": 3}},
            "timings": {{"prompt_per_second": 100.0, "predicted_per_second": 20.0}},
        }}
        self.wfile.write(("data: " + json.dumps(final) + "\n\n").encode())
        self.wfile.write(b"data: [DONE]\n\n")


ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
"""

READY = {
    "type": "object",
    "properties": {"ready": {"const": True}},
    "required": ["ready"],
    "additionalProperties": False,
}


@pytest.fixture
def stand_in(tmp_path, monkeypatch):
    script = tmp_path / "llama-server"
    script.write_text(STAND_IN.format(python=sys.executable))
    script.chmod(script.stat().st_mode | stat.S_IXUSR)
    record = tmp_path / "requests.jsonl"
    record.touch()
    monkeypatch.setenv("STAND_IN_RECORD", str(record))
    monkeypatch.setenv("CLIPMILL_LLAMA_SERVER", str(script))

    def requests(path=None):
        entries = [json.loads(line) for line in record.read_text().splitlines()]
        return [entry["body"] for entry in entries if path is None or entry["path"] == path]

    return requests


def gguf_model(tmp_path, files=("Qwen3.5-9B-Q4_K_M.gguf", "mmproj-F16.gguf")):
    return VerifiedModel(
        name="qwen3-5-editorial-gguf",
        capability="editorial",
        digest="sha256:" + "0" * 64,
        root=tmp_path,
        files=tuple(files),
    )


def test_a_reply_streams_under_its_schema_with_the_workers_key(stand_in, tmp_path):
    runtime = ServerModel(gguf_model(tmp_path), CancellationToken())
    try:
        raw, tokens = runtime.generate("Reply with ready.", READY, 32)
    finally:
        runtime.close()
    assert json.loads(raw) == {"ready": True}
    assert tokens == {"input": 7, "output": 3}
    assert runtime.last_timings["predicted_per_second"] == 20.0
    (request,) = stand_in("/v1/chat/completions")
    assert request["temperature"] == 0.0
    assert request["max_tokens"] == 32
    assert request["stream"] is True
    assert request["response_format"]["json_schema"]["schema"] == READY
    assert request["messages"][0]["content"] == [{"type": "text", "text": "Reply with ready."}]


def test_the_server_loads_both_files_on_loopback_offline_with_thinking_off(stand_in, tmp_path):
    runtime = ServerModel(gguf_model(tmp_path), CancellationToken())
    runtime.close()
    (arguments,) = stand_in("argv")
    assert arguments[arguments.index("--model") + 1] == str(tmp_path / "Qwen3.5-9B-Q4_K_M.gguf")
    assert arguments[arguments.index("--mmproj") + 1] == str(tmp_path / "mmproj-F16.gguf")
    assert arguments[arguments.index("--host") + 1] == "127.0.0.1"
    assert json.loads(arguments[arguments.index("--chat-template-kwargs") + 1]) == {
        "enable_thinking": False
    }
    for flag in ("--offline", "--no-webui", "--no-slots"):
        assert flag in arguments
    # The key is in the server's environment, never on its command line.
    assert not any(len(argument) == 64 for argument in arguments)


def test_frames_arrive_as_448_pixel_pngs_before_the_text(stand_in, tmp_path):
    from PIL import Image

    frame = tmp_path / "frame.jpg"
    Image.new("RGB", (320, 180), (200, 30, 30)).save(frame)
    runtime = ServerModel(gguf_model(tmp_path), CancellationToken())
    try:
        runtime.generate("Look.", READY, 32, [str(frame)])
    finally:
        runtime.close()
    (request,) = stand_in("/v1/chat/completions")
    image, text = request["messages"][0]["content"]
    assert text == {"type": "text", "text": "Look."}
    url = image["image_url"]["url"]
    assert url.startswith("data:image/png;base64,")
    sent = Image.open(io.BytesIO(base64.b64decode(url.split(",", 1)[1])))
    assert sent.size == (448, 448)


def test_a_prompt_over_the_limit_is_refused_before_anything_generates(stand_in, tmp_path):
    runtime = ServerModel(gguf_model(tmp_path), CancellationToken())
    try:
        with pytest.raises(ValueError, match="12,000-token"):
            runtime.generate("word " * 12_001, READY, 32)
    finally:
        runtime.close()
    assert stand_in("/v1/chat/completions") == []


def test_cancelling_stops_a_reply_in_progress(stand_in, tmp_path, monkeypatch):
    monkeypatch.setenv("STAND_IN", json.dumps({"pieces": ["{"] * 50, "delay": 0.2}))
    cancellation = CancellationToken()
    runtime = ServerModel(gguf_model(tmp_path), cancellation)
    threading.Timer(0.5, cancellation.cancel).start()
    started = time.monotonic()
    try:
        with pytest.raises(LeaseCancelled):
            runtime.generate("Reply slowly.", READY, 2048)
    finally:
        runtime.close()
    assert time.monotonic() - started < 5


def test_close_ends_the_server(stand_in, tmp_path):
    runtime = ServerModel(gguf_model(tmp_path), CancellationToken())
    runtime.close()
    assert runtime.process.poll() is not None


def test_a_server_that_stops_while_loading_is_an_error(stand_in, tmp_path, monkeypatch):
    monkeypatch.setenv("STAND_IN", json.dumps({"exit": 3}))
    with pytest.raises(RuntimeError, match="stopped while loading"):
        ServerModel(gguf_model(tmp_path), CancellationToken(), start_seconds=30)


def test_a_gguf_model_names_one_weights_file_and_one_projector(tmp_path):
    weights, projector = gguf_files(gguf_model(tmp_path))
    assert (weights.name, projector.name) == ("Qwen3.5-9B-Q4_K_M.gguf", "mmproj-F16.gguf")
    for files in (("model.gguf",), ("a.gguf", "b.gguf", "mmproj.gguf"), ("mmproj.gguf",)):
        with pytest.raises(ValueError, match="one weights file and one mmproj"):
            gguf_files(gguf_model(tmp_path, files))


def test_gguf_models_run_on_llama_cpp_and_mlx_folders_keep_their_names(tmp_path, monkeypatch):
    monkeypatch.setenv("CLIPMILL_LLAMA_SERVER", "/nowhere/llama-server")
    gguf = gguf_model(tmp_path)
    mlx = gguf_model(tmp_path, ("config.json", "model.safetensors"))
    assert is_gguf(gguf) and not is_gguf(mlx)
    assert runtime_name(gguf) == "llama.cpp@external/clipmill-json-v1"
    assert runtime_name(mlx) == "mlx-vlm@0.7.1/clipmill-json-v2"
    assert implementation_suffix(gguf) == "-llama.cpp"
    assert implementation_suffix(mlx) == ""
    assert implementation_suffix(None) == ""


def test_the_packaged_server_is_used_when_nothing_names_another(monkeypatch):
    monkeypatch.delenv("CLIPMILL_LLAMA_SERVER", raising=False)
    fake = type(sys)("clipmill_llama_server")
    fake.BUILD = "b11255"
    fake.executable = lambda: Path("/app/llama-server")
    monkeypatch.setitem(sys.modules, "clipmill_llama_server", fake)
    assert server_runtime.server_executable() == Path("/app/llama-server")
    assert server_runtime.server_build() == "b11255"
    assert os.environ.get("CLIPMILL_LLAMA_SERVER") is None
