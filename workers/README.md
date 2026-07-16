# Workers

Python model/media workers. Each worker **family** gets its own uv project and
its own locked virtual environment, so a dependency conflict (CUDA, PyTorch,
Paddle, …) is contained to one pool and can never take down another family or
the daemon. Workers are strictly stateless: they take leased tasks over the
worker protocol, stream heartbeats, and return artifacts — all durable state
belongs to `clipmilld`.

- `sdk/` — `clipmill_worker_sdk`: the worker protocol client (handshake with
  capability descriptor, lease/heartbeat/complete/decline) and generated
  contract types. Every worker family depends on it.
- `echo/` — the null worker: exercises the full protocol without doing any
  work. It is the protocol's reference implementation and test target.

Future families (`asr/`, `vision/`, `judge/`, …) follow the same shape: own
`pyproject.toml`, own venv, `clipmill-worker-sdk` as a path dependency.
