# The model library

Models is where ClipMill's models are downloaded, chosen and removed. A fresh
installation has the pinned manifests and no weights; nothing is fetched until
a person asks.

## What is listed

Every model a ClipMill worker can run, grouped by the job it does: Editorial AI,
Transcription, Word timing, Speech detection and Face tracking. The catalog is
the model registry — the bundled manifests in `models/registry/`, plus any model
the person pinned in the app — so a model appears only if a worker knows how to
run it.

Every model is listed on every computer. Each row states the download size, the
memory the model needs (weights plus the runtime's allowance), where it runs and
its licence. Two conditions are warnings rather than filters:

- **Memory.** A model needing more than analysis may use on this computer (75%
  of physical memory on macOS; three quarters of available memory elsewhere) is
  marked as tight; one needing more than the computer has is marked as unlikely
  to run. Either can still be downloaded.
- **Platform.** MLX models run only on Apple silicon Macs, and GGUF editorial
  models only on Windows and Linux PCs, in the llama.cpp server the editorial
  component carries. Models lists only what this computer can run, and a model
  of another platform only while some of it is on disk, so it can be removed.

The recommended set — marked in each manifest's `[catalog]` table — is what a
fresh installation is offered in one download: the portable models every job
needs, and the editorial model: the MLX build on Apple silicon, the GGUF build
on Windows and Linux.

## Downloading

A download is an explicit network operation, counted by the
[Local Lock](local-lock.md). It fetches each pinned file from Hugging Face at the
pinned commit, over HTTPS, following at most five redirects and only onto HTTPS,
and sends no credential. Every byte is hashed on the way in; a file becomes
visible under its final name only when its size and SHA-256 match the pin.
Workers hash every file again immediately before loading it.

Downloads run one at a time. A cancelled or interrupted download keeps what
arrived under `<weights>/.partial/<model>/` and resumes from there; the kept
bytes are hashed from disk before they are trusted, and a kept prefix that turns
out not to match is discarded and fetched again. Nothing resumes automatically
after a restart.

**Check files** re-hashes every installed file and removes any that no longer
matches its pin, so the next download repairs exactly that file. **Remove**
deletes a model's weights; it is refused while an unfinished analysis task was
planned with the model.

## Which model does a job

Per job, the next analysis plans:

1. the model the person chose, when its weights are installed;
2. otherwise what the signed device profile bound — the benchmark's choice
   (D19), or the portable model for an unmeasured device — or, for editorial,
   the bundled model, when installed;
3. otherwise any installed candidate, the portable one first;
4. otherwise the default, which readiness then reports as not installed.

The choice is resolved when a job is planned and written into its tasks, so a
change never alters an analysis already running. Removing a chosen model returns
its job to automatic. Choices live in `state/model-choices.json`.

Some candidates are _opt-in_: Whisper Large v3 Turbo, and every model a person
pins. A benchmark ranks speed, and these exist for other reasons — accuracy, or
the person's own preference — so they are never ranked, never written into the
signed profile, and run only when chosen (or as a stand-in when nothing else is
installed).

## Adding a model from Hugging Face

Two jobs accept a person's own model, because their workers load whatever model
the lease binds:

- **Transcription:** a whisper.cpp GGML `.bin` file.
- **Editorial AI:** on Apple silicon, an MLX vision-language model folder; on
  Windows and Linux, a GGUF model file chosen from a repository (the 4-bit
  `Q4_K_M` build is offered first), pinned with the repository's vision
  projector (`mmproj`, F16 where there is one). A GGUF repository without a
  projector is refused, since the editorial model looks at frames. ClipMill's
  editorial prompts are tuned for Qwen3.5 9B; another model may not follow the
  format, and an analysis says so if it fails.

Looking a repository up is a network operation, counted like a download. The
branch, tag or commit resolves to one commit and only that commit is pinned.
Weights carry the SHA-256 recorded by the repository's large-file storage; small
files kept in git (configs, tokenizers) are fetched whole at the pinned commit
and the digest of what arrived becomes the pin. The licence must be one the
bundled registry allows (MIT, Apache-2.0, BSD-2/3-Clause, CC0-1.0, ISC), because
model output ends up in what creators publish and sell. Gated and private
repositories are refused rather than signed in to.

Pinned manifests are kept as `state/models/<name>.json` and load with the
bundled registry at startup; a broken one is skipped and logged. A name can
never replace one already registered. Removing a model the person added
forgets it entirely.

## Which worker runs a model

Each model is run by one worker family: the Whisper models by the whisper.cpp
worker, the English aligner by the word-timing worker, the Qwen3 speech models by
the MLX speech worker, and the editorial model by the editorial worker. A task
is handed only to a worker of the family its model needs, so two families can
serve one job side by side — `just workers` starts the MLX speech worker beside
the portable ones on Apple silicon — and a model chosen in Models runs without
restarting anything. When the worker a job's model needs is not connected,
readiness names it under the job, and Models says so beside another model before
it is chosen.

## Workers and memory

The transcription and editorial workers declare a memory ceiling sized from the
computer they run on — 70% of physical memory on macOS, of available memory
elsewhere — and never less than the ceiling they had before. The daemon never
hands a worker a task above its ceiling, and never leases to a worker declaring
more than the device's processing budget, so readiness says when either would
leave a stage waiting. `CLIPMILL_WORKER_MEMORY_BYTES` overrides the measured
ceiling.

The editorial runtime check proves the chosen editorial model generates on this
machine before its worker is admitted: on the Mac's GPU for MLX, or in the
llama.cpp server on Windows and Linux. There it also measures one
representative reply, a prompt of about 1,500 tokens and an answer of about 80,
and Models shows how fast the model read and wrote on this machine. When a
typical editorial step (about 6,000 tokens read and 800 written) would take
more than two minutes, Models says so and that a smaller model is faster; it
never refuses. A packaged app runs the check once its component and model are
installed; a
development checkout runs it with `just workers`
(`tools/editorial-runtime-check.py`). After choosing a different editorial
model, restart the workers so the check runs against it.

## Where models live

Weights are installed under the daemon's weights directory: `.cache/models` in a
development checkout, or `CLIPMILL_WEIGHTS_DIR`. A packaged build must point it
at a writable directory in the user's data folder. `tools/fetch-models.sh`
installs the same pins, verified the same way, for scripted setups and CI.

## Storage clean-up

Settings → Storage acts on what nothing uses:

- **Generated media** — collects generated files no project, source, task or
  system root reaches, without waiting out the retention period. The collection
  re-verifies every reachable manifest and payload and keeps whatever a reader
  holds; objects younger than 15 minutes are kept, covering the moment between
  publishing an object and rooting it. The report estimates what would be freed
  beforehand.
- **Database backups** — deletes all but the newest backup taken before a
  schema migration.
- **Temporary files** — removes scratch nothing has written to for an hour, and
  interrupted downloads that are not being resumed.

Freeing generated media runs through the same loop as the scheduled collection,
so the two never interrupt each other, and it answers only once its pass has
finished. Verification reads every file a project uses — seconds for a small
library, minutes for a large one — and yields whenever the engine has other
work. A pass that yielded resumes rather than restarts: it reads the roots and
reader pins afresh, but skips the objects it already verified and, inside an
object of many files such as a filmstrip, continues from the file and byte it
had reached. Every pass still verifies every reachable byte once. Settings waits up to 30 minutes for the answer, and the clean-up finishes
whether or not anyone is still waiting.

Model weights are removed from Models, where a removal can say which job loses
its model. Project state and imported originals belong to their projects.
