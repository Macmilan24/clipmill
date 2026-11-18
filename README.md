# ClipMill

**A local-first video clipping studio.** Turn long recordings into short clips:
find moments, review the suggestions, edit the framing and captions, and export
video ready to upload.

ClipMill runs on **macOS 14 or later on Apple silicon**, on **64-bit Linux**,
and, as a beta, on **64-bit Windows 10 and 11**. The import-to-export workflow is
complete. A local Qwen model plans clips, reviews each edit against the source,
and ranks the cuts that pass. It runs through MLX on Apple silicon and through
llama.cpp on Linux and Windows, on the graphics card or the processor. A
built-in picker is available on machines without an editorial model.

## Install

Download ClipMill for macOS, Linux or Windows (beta) from the
[latest release](https://github.com/Macmilan24/clipmill/releases/latest), then
follow [Installing ClipMill](docs/install.md). On first run, **Set up ClipMill**
installs the app's components and the recommended models in one step; after
that, analysis works without a connection.

## What you can do

- Import a local video or download a single YouTube video you have permission to
  use, with a choice of download quality.
- Analyze English podcasts, interviews, and scripted scenes. Give the local
  editor a brief for a topic or a montage. It can tighten a passage, lift an
  opening, or join a question to its answer elsewhere in the recording.
- Review ranked clips, inspect the source at each join, and keep or reject
  suggestions. A new brief reuses the same recording analysis.
- Edit trims, caption wording and line breaks, framing, crop movement,
  transitions, and audio. Saved edits support undo and redo within an editing
  session.
- Export rendered video with burned-in captions and SRT/VTT sidecars, individually
  or in batches. Export jobs and saved document revisions survive restarts.

For a local editorial run, ClipMill checks the complete played transcript and the
original lines around each cut. A failed check gets one repair attempt. The
ranker compares eligible clips in both presentation orders and balances chapters
and forms. Results explains when fewer clips meet the brief than you requested.

YouTube **channel connection and publishing are still in development**. YouTube
source import is available; publishing is not part of the merged application yet.

## Privacy and network use

Analysis runs locally by default. Installing dependencies and model weights,
and importing a YouTube video, require network access. An optional cloud
editorial route sends transcript context only after explicit per-run consent;
source frames and visual checks remain local. There is no automatic cloud
fallback.

The Local Lock badge reports network operations started by the application in
the current daemon session. It is **not an operating-system firewall**. See
[Local Lock](docs/local-lock.md) for the exact guarantees and limitations.

## Run from source

To work on ClipMill itself, run it from a checkout. You need Rust (the version in `rust-toolchain.toml`), Node.js 22+, pnpm (the version
in `package.json`), Python 3.12, `uv`, `just`, and `ripgrep`. On macOS, install the
Xcode Command Line Tools. Linux desktop development also needs WebKitGTK 4.1,
GTK 3, libsoup 3, librsvg, and patchelf development packages.

From the repository root:

```sh
just setup

# Speech recognition, alignment, voice activity, and face detection
./tools/fetch-models.sh silero-vad whisper-base wav2vec2-ctc-en yunet-face ced-mini-audio-events

# Local editorial model (about 6.5 GB): MLX on Apple silicon, and GGUF on
# Linux, where the worker runs it in the llama.cpp server CLIPMILL_LLAMA_SERVER
# names (a llama.cpp release build)
./tools/fetch-models.sh qwen3-5-editorial-mlx   # Linux: qwen3-5-editorial-gguf
```

Launch the application, then start its workers in a second terminal:

```sh
# Terminal 1: enroll workers, build the daemon, and launch the desktop app
uv run --offline --no-sync --project workers/sdk just app

# Terminal 2, from the same repository root: start processing workers
uv run --offline --no-sync --project workers/sdk just workers
```

The SDK environment supplies the Python dependencies used during worker
enrollment. Keep both terminals running. **Models** lists every model with its
download size and the memory it needs, can download the recommended set in place
of the commands above, and shows which model each job uses; analysis readiness
explains missing requirements. Every file is checked against its pinned SHA-256
before it is used. See [the model library](docs/model-library.md). Local
inference needs several gigabytes of memory beyond the downloaded weights, so
available memory affects which jobs can run.

For YouTube downloads, `just setup-youtube` verifies the importer installation.
See [YouTube import](docs/youtube-import.md) for supported URLs and limits.

## Documentation and contributing

- [Installing ClipMill](docs/install.md) — the released app, first run, and where it keeps things
- [Changelog](CHANGELOG.md) — what each release changed
- [Documentation index](docs/README.md) — feature guides and architecture
- [Releasing](docs/releasing.md) — how a release is built, signed and published
- [Contributing](CONTRIBUTING.md) — development workflow and checks
- [Worker setup](workers/README.md) — model processes and authentication
- [Security policy](SECURITY.md) — reporting vulnerabilities

The desktop application uses Tauri and React, a Rust daemon owns project state
and jobs, and Python workers run the models. Shared schemas define the contracts
between them. Bug reports, documentation improvements, and focused pull requests
are welcome.

## License

[AGPL-3.0-only](LICENSE). See the license for the terms of use, modification, and
redistribution. Bundled dependencies and model weights retain their own licenses.
