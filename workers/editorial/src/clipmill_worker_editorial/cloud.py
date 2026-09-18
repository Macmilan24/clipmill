"""One opt-in, transcript-only adapter. No provider SDK retries or remote media."""

from __future__ import annotations

import fcntl
import json
import os
import re
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

from clipmill_worker_sdk import LeaseCancelled

from .errors import BudgetExceeded, ProviderRequestError

MODEL = "claude-sonnet-4-6"
# https://platform.claude.com/docs/en/models/sonnet-4-6/overview
# USD millionths per token; no prompt caching, batch mode, or tools.
INPUT_RATE = 3
OUTPUT_RATE = 15


class Budget:
    """Reserve before sending; keep uncertain/crashed calls charged in full.

    Shared by both operations and all leases of this run. A separate lock file
    protects atomic state replacement, including across worker processes.
    """

    def __init__(self, root: Path, run_id: str, cap: int):
        if not re.fullmatch(r"job_[A-Za-z0-9]+", run_id):
            raise ValueError("missing run budget identity")
        if not 10_000 <= cap <= 100_000_000:
            raise ValueError("cloud budget must be $0.01-$100")
        root.mkdir(mode=0o700, parents=True, exist_ok=True)
        self.path = root / (run_id + ".json")
        self.lock = root / (run_id + ".lock")
        self.cap = cap
        self.spent = 0

    def change(self, delta):
        fd = os.open(self.lock, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "r+b") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            state = (
                json.loads(self.path.read_text())
                if self.path.exists()
                else {"cap": self.cap, "spent": 0}
            )
            if state["cap"] != self.cap:
                raise RuntimeError("run budget changed after submission")
            value = state["spent"] + delta
            if value > self.cap:
                raise BudgetExceeded(
                    "Cloud budget reached. Start a new run with a larger cap or choose Local; "
                    "no further request was sent."
                )
            if value < 0:
                raise RuntimeError("invalid budget refund")
            temporary = self.path.with_suffix(".pending")
            handle = os.open(
                temporary, os.O_CREAT | os.O_TRUNC | os.O_WRONLY | os.O_NOFOLLOW, 0o600
            )
            with os.fdopen(handle, "w") as f:
                json.dump({"cap": self.cap, "spent": value}, f)
                f.flush()
                os.fsync(f.fileno())
            os.replace(temporary, self.path)
            self.spent = value


def credential():
    # Stored by the user in the OS credential store, never a renderer field,
    # task payload, trace, subprocess argument, or environment variable.
    result = subprocess.run(
        [
            "/usr/bin/security",
            "find-generic-password",
            "-s",
            "dev.clipmill.anthropic",
            "-a",
            "clipmill",
            "-w",
        ],
        capture_output=True,
        text=True,
        timeout=15,
    )
    if result.returncode or not result.stdout.strip():
        raise ProviderRequestError(
            "Anthropic key unavailable in Keychain (service dev.clipmill.anthropic, "
            "account clipmill). Add it in Keychain Access, then retry or choose Local.",
            retryable=False,
        )
    return result.stdout.strip()


def compatible_schema(schema):
    """Keep the grammar structure; Pydantic enforces provider-unsupported bounds."""
    if isinstance(schema, list):
        return [compatible_schema(v) for v in schema]
    if not isinstance(schema, dict):
        return schema
    return {
        k: compatible_schema(v)
        for k, v in schema.items()
        if k
        not in {
            "minimum",
            "maximum",
            "exclusiveMinimum",
            "exclusiveMaximum",
            "minLength",
            "maxLength",
            "minItems",
            "maxItems",
            "pattern",
            "default",
            "title",
        }
    }


class CloudModel:
    def __init__(self, config, budget_root, run_id, cancellation, send=None, key=None):
        if not config.transcript_consent or config.model != MODEL:
            raise RuntimeError("This run has no valid transcript-only cloud consent")
        self.config = config
        self.cancellation = cancellation
        self.budget = Budget(budget_root, run_id, config.budget_micro_usd)
        self.send = send or self._send
        self.key = key or credential
        self.last_cost = 0
        self.disabled_error = None

    def generate(self, prompt, schema, max_tokens, images=None):
        self.last_cost = 0
        if self.disabled_error is not None:
            raise self.disabled_error
        if images:
            raise RuntimeError(
                "Cloud media scope is disabled; only transcript text may leave this device"
            )
        if not self.config.transcript_consent:
            raise RuntimeError("Cloud consent revoked")
        self.cancellation.raise_if_cancelled()
        # Verified against the provider's stable Messages structured-output API:
        # https://platform.claude.com/docs/en/build-with-claude/structured-outputs
        body = {
            "model": MODEL,
            "max_tokens": max_tokens,
            "temperature": 0,
            "messages": [{"role": "user", "content": prompt}],
            "output_config": {
                "format": {"type": "json_schema", "schema": compatible_schema(schema)}
            },
        }
        encoded = json.dumps(body).encode()
        # Byte count is a conservative token bound plus provider/schema overhead.
        reservation = (len(encoded) + 5000) * INPUT_RATE + max_tokens * OUTPUT_RATE
        key = self.key()
        self.cancellation.raise_if_cancelled()
        self.budget.change(reservation)
        self.last_cost = reservation
        try:
            self.cancellation.raise_if_cancelled()
        except LeaseCancelled:
            self.budget.change(-reservation)
            self.last_cost = 0
            raise
        try:
            result = self.send(encoded, key)
        except urllib.error.HTTPError as error:
            # Classify only the status; never read a provider body or headers
            # into diagnostics because either could echo private request data.
            detail, retryable = provider_failure(error.code)
            error.close()
            failure = ProviderRequestError(detail, retryable=retryable)
            if not retryable:
                self.disabled_error = failure
            raise failure from None
        except Exception:
            # Never serialize provider bodies or headers: either may echo credentials.
            raise RuntimeError(
                "Anthropic unavailable. The uncertain request remains reserved against this "
                "run budget. Retry explicitly or choose Local."
            ) from None
        usage = result.get("usage", {})
        incoming = usage.get("input_tokens")
        outgoing = usage.get("output_tokens")
        if (
            not isinstance(incoming, int)
            or not isinstance(outgoing, int)
            or incoming < 0
            or outgoing < 0
        ):
            raise RuntimeError(
                "Anthropic omitted usage; the request remains reserved against the budget"
            )
        cost = incoming * INPUT_RATE + outgoing * OUTPUT_RATE
        if cost > reservation:
            self.budget.change(self.budget.cap - self.budget.spent)
            raise RuntimeError(
                "Provider usage exceeded the reserved bound; "
                "no further calls are allowed for this run"
            )
        self.budget.change(cost - reservation)
        self.last_cost = cost
        self.cancellation.raise_if_cancelled()
        if result.get("stop_reason") != "end_turn":
            raise ValueError("Cloud response stopped before completing its JSON answer")
        text = "".join(
            b.get("text", "") for b in result.get("content", []) if b.get("type") == "text"
        )
        # Even a provider response echoing a header must not put a credential
        # into a raw trace or a validation exception.
        return text.replace(key, "[REDACTED]"), {"input": incoming, "output": outgoing}

    def _send(self, body, key):
        request = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "content-type": "application/json",
                "anthropic-version": "2023-06-01",
                "x-api-key": key,
            },
            method="POST",
        )

        # No redirect following: credentials are scoped to exactly this provider.
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, *args, **kwargs):
                return None

        opener = urllib.request.build_opener(NoRedirect)
        with opener.open(request, timeout=60) as response:
            return json.loads(response.read(2_000_000))

    def close(self):
        pass


def provider_failure(status):
    """Map documented HTTP statuses to actionable, redacted failure messages.

    https://platform.claude.com/docs/en/api/errors
    Uncertain usage remains reserved even when no usage record was returned.
    """
    details = {
        400: "Anthropic rejected the request. Check provider limits or update the cloud adapter.",
        401: (
            "Anthropic rejected the API key. Update the dev.clipmill.anthropic credential "
            "in Keychain Access, then start a new analysis or choose Local."
        ),
        402: "Anthropic billing needs attention. Check the provider account or choose Local.",
        403: (
            "The Anthropic API key does not have permission for this model. "
            "Check its workspace access or choose Local."
        ),
        404: "The configured Anthropic model or endpoint is unavailable. Update the cloud adapter.",
        413: (
            "The transcript request exceeds the provider's size limit. "
            "Use Local or a shorter source."
        ),
        429: "Anthropic rate or spend limit reached. Check the provider account before retrying.",
    }
    detail = details.get(
        status, f"Anthropic request failed (HTTP {status}). Try again or choose Local."
    )
    detail += " The request remains reserved against this run's budget."
    return detail, status in (408, 409, 429) or status >= 500
