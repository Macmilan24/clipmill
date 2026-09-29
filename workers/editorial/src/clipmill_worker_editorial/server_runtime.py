"""llama.cpp inference for GGUF models, through a private llama-server.

Windows and Linux have no MLX. There a GGUF model, its weights and the vision
projector llama.cpp calls mmproj, runs in llama.cpp's own server, pinned in
bom.toml and installed with this component (``clipmill_llama_server``). The
server listens on loopback, answers only the key this process made, serves no
web page, reaches no network and holds one request at a time. It stays in this
worker's process group, so whatever ends the worker ends it.

Each call keeps ``LocalModel``'s contract: greedy decoding under the call's JSON
schema with thinking off, at most 12,000 prompt tokens and frames of 448x448,
a ten-minute deadline, and cancellation that stops the generation.
"""

from __future__ import annotations

import base64
import http.client
import io
import json
import os
import secrets
import socket
import subprocess
import sys
import threading
import time
from contextlib import suppress
from pathlib import Path

PROMPT_TOKEN_LIMIT = 12_000
CONTEXT_TOKENS = 16_384
CALL_SECONDS = 600
# Loading a 6 GB model from a slow disk, and fitting it to the GPU's memory.
START_SECONDS = 600
FRAME_SIDE = 448
_POLL_SECONDS = 0.1
_CREATE_NO_WINDOW = 0x0800_0000


def is_gguf(model) -> bool:
    """Whether a verified model is one llama.cpp loads."""
    return any(name.lower().endswith(".gguf") for name in getattr(model, "files", ()))


def gguf_files(model) -> tuple[Path, Path]:
    """The weights and the vision projector a GGUF editorial model pins."""
    names = [name for name in model.files if name.lower().endswith(".gguf")]
    projectors = [name for name in names if Path(name).name.lower().startswith("mmproj")]
    weights = [name for name in names if name not in projectors]
    if len(weights) != 1 or len(projectors) != 1:
        raise ValueError("a GGUF editorial model pins one weights file and one mmproj projector")
    return model.path(weights[0]), model.path(projectors[0])


def server_executable() -> Path:
    """The pinned llama-server: installed with the component, or named by
    ``CLIPMILL_LLAMA_SERVER`` in a development checkout or a test."""
    explicit = os.environ.get("CLIPMILL_LLAMA_SERVER")
    if explicit:
        return Path(explicit)
    import clipmill_llama_server  # only the Windows and Linux components carry it

    return clipmill_llama_server.executable()


def server_build() -> str:
    """The llama.cpp build the runtime names in receipts and producers."""
    if os.environ.get("CLIPMILL_LLAMA_SERVER"):
        return "external"
    try:
        import clipmill_llama_server
    except ImportError:
        return "missing"
    return clipmill_llama_server.BUILD


class ServerModel:
    """One llama-server for one verified GGUF model, open until ``close()``."""

    def __init__(self, model, cancellation, *, executable=None, start_seconds=START_SECONDS):
        weights, projector = gguf_files(model)
        cancellation.raise_if_cancelled()
        self.cancellation = cancellation
        # What the server measured for the last reply, for the runtime check.
        self.last_timings: dict = {}
        self._key = secrets.token_hex(32)
        self.port = _free_port()
        command = [
            str(executable or server_executable()),
            "--model",
            str(weights),
            "--mmproj",
            str(projector),
            "--host",
            "127.0.0.1",
            "--port",
            str(self.port),
            "--ctx-size",
            str(CONTEXT_TOKENS),
            "--parallel",
            "1",
            "--seed",
            "0",
            "--chat-template-kwargs",
            json.dumps({"enable_thinking": False}),
            "--no-webui",
            "--no-slots",
            "--offline",
        ]
        # The key travels in the environment, not the command line, which
        # other processes on the machine can read.
        environment = {**os.environ, "LLAMA_API_KEY": self._key}
        options = {"creationflags": _CREATE_NO_WINDOW} if sys.platform == "win32" else {}
        self.process = subprocess.Popen(
            command, stdin=subprocess.DEVNULL, env=environment, **options
        )
        try:
            self._wait_until_ready(start_seconds)
        except BaseException:
            self.close()
            raise

    def generate(self, prompt, schema, max_tokens, images=None):
        if self._prompt_tokens(prompt) > PROMPT_TOKEN_LIMIT:
            raise ValueError("editorial context exceeds the 12,000-token limit")
        content = [_frame(image) for image in images or []]
        content.append({"type": "text", "text": prompt})
        body = {
            "messages": [{"role": "user", "content": content}],
            "temperature": 0.0,
            "seed": 0,
            "max_tokens": max_tokens,
            "stream": True,
            "stream_options": {"include_usage": True},
            "response_format": {
                "type": "json_schema",
                "json_schema": {"name": "answer", "strict": True, "schema": schema},
            },
        }
        text, usage, self.last_timings = self._stream(body)
        return text, {
            "input": int(usage.get("prompt_tokens", 0)),
            "output": int(usage.get("completion_tokens", 0)),
        }

    def close(self):
        process = getattr(self, "process", None)
        if process is None or process.poll() is not None:
            return
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

    def _wait_until_ready(self, seconds):
        deadline = time.monotonic() + seconds
        while True:
            self.cancellation.raise_if_cancelled()
            code = self.process.poll()
            if code is not None:
                raise RuntimeError(f"llama-server stopped while loading the model (exit {code})")
            try:
                status, _ = self._request("GET", "/health", timeout=2)
            except OSError:
                status = 0
            if status == 200:
                return
            if time.monotonic() > deadline:
                raise TimeoutError("llama-server did not load the model in time")
            time.sleep(0.25)

    def _prompt_tokens(self, prompt):
        """The prompt's length as the model reads it, template included."""
        messages = [{"role": "user", "content": prompt}]
        templated = self._json("POST", "/apply-template", {"messages": messages})
        return len(self._json("POST", "/tokenize", {"content": templated["prompt"]})["tokens"])

    def _json(self, method, path, body):
        status, answer = self._request(method, path, body)
        if status != 200:
            raise RuntimeError(f"llama-server answered {path} with {status}")
        return json.loads(answer)

    def _request(self, method, path, body=None, timeout=60):
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=timeout)
        try:
            payload = None if body is None else json.dumps(body).encode()
            connection.request(method, path, body=payload, headers=self._headers(payload))
            response = connection.getresponse()
            return response.status, response.read()
        finally:
            connection.close()

    def _headers(self, payload):
        headers = {"Authorization": f"Bearer {self._key}"}
        if payload is not None:
            headers["Content-Type"] = "application/json"
        return headers

    def _stream(self, body):
        """Send one streamed completion and collect it on a reader thread, so
        cancellation and the deadline are checked while the server is still
        reading a long prompt. Closing the connection ends the generation."""
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=CALL_SECONDS)
        # Connected here, so the socket is at hand to shut down whatever the
        # reader is doing: a streamed response takes it over from the
        # connection, which then no longer holds it.
        connection.connect()
        stream = connection.sock
        payload = json.dumps(body).encode()
        pieces: list[str] = []
        usage: dict = {}
        timings: dict = {}
        failure: list[BaseException] = []
        finished = threading.Event()

        def read():
            try:
                connection.request(
                    "POST", "/v1/chat/completions", body=payload, headers=self._headers(payload)
                )
                response = connection.getresponse()
                if response.status != 200:
                    detail = response.read(2000).decode("utf-8", "replace")
                    raise RuntimeError(
                        f"llama-server refused the request ({response.status}): {detail}"
                    )
                for line in response:
                    event = _event(line)
                    if event is None:
                        continue
                    if event == "[DONE]":
                        break
                    for choice in event.get("choices", []):
                        piece = (choice.get("delta") or {}).get("content")
                        if piece:
                            pieces.append(piece)
                    if event.get("usage"):
                        usage.update(event["usage"])
                    if event.get("timings"):
                        timings.update(event["timings"])
            except BaseException as error:  # handed to the caller below
                failure.append(error)
            finally:
                finished.set()

        reader = threading.Thread(target=read, name="llama-server-reply", daemon=True)
        reader.start()
        deadline = time.monotonic() + CALL_SECONDS
        try:
            while not finished.wait(_POLL_SECONDS):
                self.cancellation.raise_if_cancelled()
                if time.monotonic() > deadline:
                    raise TimeoutError("editorial call exceeded the 10-minute limit")
                if self.process.poll() is not None:
                    raise RuntimeError("llama-server stopped during the call")
        except BaseException:
            _abandon(stream)
            connection.close()
            reader.join(timeout=5)
            raise
        connection.close()
        if failure:
            raise failure[0]
        return "".join(pieces), usage, timings


def _abandon(stream: socket.socket) -> None:
    """End a request from another thread. Shutting the socket down wakes a
    reader blocked in it, which closing alone does not, and tells the server
    the client has gone, which stops the generation."""
    with suppress(OSError):
        stream.shutdown(socket.SHUT_RDWR)


def _event(line: bytes):
    """One server-sent event's data: a parsed chunk, "[DONE]", or None."""
    text = line.decode("utf-8").strip()
    if not text.startswith("data:"):
        return None
    data = text[5:].strip()
    if data == "[DONE]":
        return data
    event = json.loads(data)
    if "error" in event:
        raise RuntimeError(f"llama-server failed the request: {event['error']}")
    return event


def _frame(path) -> dict:
    """A frame as the server reads it: 448x448, as the MLX runtime sizes it."""
    from PIL import Image

    with Image.open(path) as image:
        resized = image.convert("RGB").resize((FRAME_SIDE, FRAME_SIDE), Image.Resampling.BICUBIC)
    encoded = io.BytesIO()
    resized.save(encoded, format="PNG")
    data = base64.b64encode(encoded.getvalue()).decode("ascii")
    return {"type": "image_url", "image_url": {"url": f"data:image/png;base64,{data}"}}


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]
