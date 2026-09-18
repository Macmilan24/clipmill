# Milestone 2 — the system finds complete, worthwhile moments

The plan (`upload-ready-v1-plan.md`) says what Milestone 2 must do. This is how
it is built on this codebase: which stages, which contracts, which worker,
which models, in what order, and what "done" is measured against. Written
after Milestone 1 landed (PR #67, #68), so it names the mechanisms that now
exist rather than ones that would have to be invented.

## What changes, in one paragraph

One editorial model per run does two bounded jobs over the transcript —
**propose** complete moments and **review** them — behind the same
task/worker layer that runs speech today. Its proposals become candidates in
the schema the director and the ranking already read, so the lattice, the
boundary optimizer, the Inspector and the editor keep working unchanged. The
review's verdicts replace the uncalibrated "99" with a status and a reason.
A visual model is asked only when the review says the picture matters. Local
by default; a narrow cloud route is opt-in per run, transcript-only first,
with the provider, the data scope and a hard budget on screen before it runs.

## What exists and is reused

| Need                    | Already here                                                                                                                                |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Isolation and leasing   | Worker families in their own `uv` environments, Ed25519 identity, leased tasks, deterministic-vs-transient failure classes, bounded retries |
| Pinned models           | `models/registry/*.toml` with per-file digests, licence class, `[memory]` for admission; `tools/fetch-models.sh`; `GetReadiness`            |
| Per-device choice       | The measured-binding mechanism (D19): a benchmark decides which implementation serves a capability on this machine                          |
| Offline by construction | `NetworkPolicy` per stage; the Local Lock derived from the registry and a counter of what actually started                                  |
| Caching and provenance  | Content-addressed artifacts keyed by recipe (inputs + model digest + implementation); producers recorded in every document                  |
| Stable references       | Word ids on every transcript word (Milestone 1), sentence and utterance indexes in `index.transcript.v1`                                    |
| Boundary arithmetic     | The lattice and boundary optimizer in `clipmill-discovery`; the director reads `discovery.candidates.v1` + `ranking.set.v1`                 |
| Human decisions         | `clip_decisions` (kept / rejected / approved) per project, source and candidate — the raw material of an acceptance measurement             |
| Evaluation harness      | `eval/harness` with a daemon client, recall scoring, signed reports                                                                         |

## Stages

New recipes in `crates/clipmilld/src/recipes.rs`, in the analysis DAG after
`index-transcript`. Every stage below the editorial ones consumes the same
kinds it does today.

| Stage                | Executor | Capability          | Reads                                         | Publishes                   |
| -------------------- | -------- | ------------------- | --------------------------------------------- | --------------------------- |
| `editorial-windows`  | builtin  | —                   | `speech.transcript.v1`, `index.transcript.v1` | `editorial.windows.v1`      |
| `editorial-propose`  | worker   | `editorial-propose` | `editorial.windows.v1`                        | `editorial.proposals.v1`    |
| `editorial-validate` | builtin  | —                   | proposals, transcript, index, `speech.vad.v1` | `discovery.candidates.v1`   |
| `editorial-review`   | worker   | `editorial-review`  | candidates, windows, transcript               | `editorial.judgments.v1`    |
| `editorial-look`     | worker   | `editorial-look`    | judgments (visual flags), `media.frames.v1`   | `editorial.looks.v1`        |
| `rank-candidates`    | builtin  | —                   | candidates **+ judgments (+ looks)**          | `ranking.set.v1` (extended) |

Notes on each:

- **Windows** are deterministic and cheap: overlapping spans that keep
  sentence boundaries, a paragraph of context on either side, and the word
  and sentence ids of everything inside. A recording past a threshold also
  gets a short outline, one line per topic, each line citing its sentence
  range — context for the model, never a substitute for the words. This is
  Rust, in `clipmill-discovery`, with goldens on the synthesized talk.
- **Propose** is the model's first job. Per window it answers in ids: which
  sentences (or words, at the edges) make a moment with a hook, the setup it
  needs, and a payoff; a title; a one-sentence source-grounded reason; what it
  is unsure of. It may answer "none". Long answers are marked as splittable
  into subspans rather than thrown away for length. Timestamps are never
  asked for.
- **Validate** is Rust and refuses before it trusts: every id must exist,
  every span must fall inside the window, duplicates across overlapping
  windows are merged, durations are checked against the run's target, and
  endpoints are snapped to word timings and nearby silence with the existing
  boundary lattice. What survives is written as `discovery.candidates.v1`
  with proposer `editorial` — the same schema the heuristic proposers write,
  so nothing downstream changes shape. The heuristic proposers keep running
  beside it during this milestone, marked as such, until the measurement in
  the exit says they can go.
- **Review** is the model's second job, over each candidate with its
  neighbouring context: unresolved references, a question that was never
  asked, a payoff that is not there, an ad read, an irrelevant intro, an
  omission that changes the meaning, and whether understanding it depends on
  what is on screen. The answer is a status — `accepted`, `needs_review`,
  `rejected` — with reasons a person can read. A malformed reply or a model
  failure is recorded as exactly that, distinct from "no good moment".
- **Look** runs only for candidates the review flagged as visual-dependent:
  a bounded number of frames from the candidate's span, one question per
  frame set ("is what is being talked about visible?"), answered by a local
  visual model. Not a frame-by-frame viewer.
- **Rank** consumes the judgments: only accepted and needs-review candidates
  are ranked; diversity still applies; fewer are published when the review
  says so. `display_score` is replaced by `review.status` and `review.reasons`.
  Any number that remains is characterized in the document (what it measures,
  how it was validated) or it is not shown.

## Contracts

New schemas in `contracts/schemas/`, generated into all three languages as
every contract is. Each carries `source_fingerprint`, `inputs` (artifact ids
it read), and a `producer` with the implementation, the model digest, the
route, the prompt version and the decoding configuration.

- `editorial.windows.v1` — windows with sentence ranges, word id ranges,
  context ranges, the optional outline with its citations.
- `editorial.proposals.v1` — per window: the model's proposals in ids,
  titles, reasons, uncertainties, splittability; per window a status
  (`answered`, `none`, `malformed`, `failed`) with the failure class.
- `editorial.judgments.v1` — per candidate: status, reasons, visual
  dependency, the context window it was judged in.
- `editorial.looks.v1` — per visual check: frames examined, the answer, the
  confidence the model reported.
- `editorial.trace.v1` — the raw request and response text of every model
  call, credentials redacted, stored locally for diagnosis and **not on the
  shell's readable list**. Route, model, prompt digest and token counts are
  here too; this is where the cloud cost is accounted.
- `ranking.set.v1` — extended, compatibly: `review` per ranked entry, and a
  `route` on the producer. The renderer's `results/model.ts` and the board
  read the status instead of the score.

The proposal and judgment shapes are validated deterministically (ids exist,
enums are known, spans lie inside windows) before anything semantic is
judged; the semantic judgement is the review stage's, and the evaluation's.

## The worker family

`workers/editorial`, one family, three capabilities (`editorial-propose`,
`editorial-review`, `editorial-look`), the runtime inside the worker like
every family today — no separate model server, no Ollama (a separate daemon
with its own model store and TCP port would break digest pinning, the Lock,
and reproducible recipe keys).

- **Runtimes.** On Apple silicon, `mlx-lm` for the text model and `mlx-vlm`
  for the visual one, the way `speech-mlx` runs Qwen3 ASR today. On Linux and
  Windows, `llama-cpp-python` over GGUF weights, the way `asr-whispercpp`
  runs whisper. Both are registry-pinned by digest.
- **Determinism.** Temperature zero, a fixed seed, and a JSON schema
  enforced at decode time (grammar-constrained sampling) so the reply is
  structurally valid or the stage says `malformed` — never a parse of prose.
  The recipe key includes the prompt version and the decoding configuration,
  so a changed prompt is a different result and an unchanged one is a cache
  hit. Cross-device byte identity is not claimed and not required; the
  quality claim is made by the evaluation, not by hashing.
- **Prompts** are versioned files inside the worker package, one per job
  (propose, review, look), each with a rubric the reply must follow. The
  rubric's digest is part of the producer record.
- **Cancellation and budget.** Every model call checks the lease's
  cancellation first; the cloud implementation also checks the run's spend
  against its budget before each request and stops with a named reason.
- **Memory.** The registry's `[memory]` declarations drive admission: an
  8B model at 4-bit is ~5 GB resident plus overhead, and admission will not
  lease it beside the aligner on a 16 GB machine; the stages run in turn.
  This is the existing rule, exercised by a bigger model.

**Models to benchmark**, chosen by the measured-binding mechanism rather than
by preference, on the development recordings and the synthesized talk. The
floor that matters is memory: the model runs beside the aligner and the
recogniser, admission adds their `[memory]` declarations, and the development
machine has 24 GB. Sizes below are the 4-bit MLX builds as published in
September 2026.

| Job                   | Candidate                                                               | 4-bit resident | Fits 24 GB beside speech | Notes                                                                                                         |
| --------------------- | ----------------------------------------------------------------------- | -------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------- |
| all three jobs        | **Qwen3.5-9B** (natively multimodal; `mlx-community` 4-bit via mlx-vlm) | ~5–6 GB        | yes                      | The first candidate on this machine: one model for propose, review and look, 25–35 tok/s, room to spare       |
| propose/review        | **Gemma 4 12B** (`mlx-community`, Apache-2.0)                           | ~8 GB          | yes                      | The head-to-head for the text jobs: dense, structured output                                                  |
| propose/review        | Gemma 4 26B-A4B (MoE, 3.8B active)                                      | ~18 GB         | tight                    | Fast per token; admission decides whether it can sit beside the speech models                                 |
| propose/review + look | **Qwen3.8-27B** (dense, natively multimodal, Apache-2.0)                | ~15–18 GB      | no, not beside speech    | The strongest local model and one model for all three jobs — on 32 GB+ machines, or run after speech finishes |
| look                  | Qwen3.5-9B (same model), or Qwen3.8-27B where it fits                   | —              | —                        | Frame sets, short answers; Qwen3.5-4B as the small-machine fallback                                           |

GLM-5.3-Flash (320B-A18B) and DeepSeek V4.1-Flash (~100B MoE) are not
laptop-local — the community MLX ports need 48 GB with SSD streaming or a
256–512 GB Mac Studio — so they belong to the cloud route or a workstation
tier, not to this milestone's default. No 4B–14B Qwen3.8 exists; a smaller
Qwen means an earlier generation, which is why Gemma 4 leads the local list.

The registry's licence class is enforced per model; every candidate above is
Apache-2.0, and the class rule is extended in this milestone to say that an
editorial model's weights never ship inside an export — a decision to record,
not to assume.

## The cloud route

One more implementation of the same two capabilities, not a platform:
`editorial-cloud`, a worker that speaks to one provider's API (Anthropic
first — the structured-output and long-context needs fit, and one adapter is
the plan's whole ask). It lives behind the existing network policy:

- The stage variants that reach the network are `NetworkPolicy::NetworkAllowed`.
  The Local Lock is then _disengaged for that run_ and the health badge and
  Settings say so, by the mechanism that already counts network-allowed
  stages — no new claim, the existing one made false honestly.
- **Credentials** live in the OS keychain (macOS Keychain via the host;
  `keyring` on the others). The daemon reads the key at task start and hands
  it to the worker inside the lease over the authenticated socket. It is never
  in the payload (which is hashed into the recipe), the trace, a log, a
  document, or the renderer.
- **Scope switches**, per run, in the analysis setup: transcript only
  (default for cloud); frames or short audiovisual samples are a second
  switch that is off and says what leaves the machine when turned on.
- **Budget**: an estimated cost is shown before the run from the transcript's
  token count and the provider's published prices; a hard cap per run stops
  the stage with a named reason when reached, and the trace carries the
  actual spend.
- **Failure**: an unavailable provider is a distinct failure class; the run
  offers retry or the local fallback, and never silently substitutes one
  route's result for the other's. Both routes' reports are kept apart.

## Screens

- **Analysis setup** (New Project): a Local / Cloud-assisted choice, local
  default. Cloud names the provider, the scope, the estimated cost and the
  budget; the readiness card lists the editorial model like every other
  stage, with the fetch command when the weights are missing.
- **Analysis progress**: the editorial stages appear as stages, with the wait
  reasons the readiness mechanism already writes.
- **Results**: the score badge becomes a status (`Ready`, `Needs review`,
  with reasons on hover and in the Inspector); the model's hook/setup/payoff
  reading is shown as the "why"; a recording with no worthwhile moment says
  so rather than padding; a malformed or failed model reply is shown as a run
  problem, not as an empty result. Accepted edits survive a re-analysis that
  proposes a new set: documents are keyed by candidate and run already
  (Milestone 1), and the board shows which candidates of an older run carry
  an edit.

## Evaluation and the exit

The exit is a measurement, so the measurement is built first with the stage
that produces it, not last.

- **Acceptance protocol.** For the development recordings (the dogfood
  episode and the ones already analysed here, plus the synthesized talk as a
  smoke), both routes' top-K are laid out in the Results board; a person
  marks each kept or rejected with the existing decisions. The harness gains
  `acceptance`: it reads the decisions per candidate and per producer and
  reports acceptance rate, counts, and the failure cases, per route,
  heuristic baseline beside editorial. Counts are reported with the
  percentages; a route with too few items says so.
- **Rejection tests.** Contract tests over fixtures: a proposal naming a
  sentence that does not exist, a span outside its window, a duration past
  the target, a duplicate across windows, a malformed reply — each is
  rejected with its reason, never silently dropped.
- **Zero is an answer.** A recording built to have no complete moment (the
  synthesized talk cut mid-sentence throughout) returns none, and the board
  says so.
- **Gate.** `just gate-editorial`: the window goldens, the validator's
  refusals, a real propose/review over the synthesized talk with the pinned
  local model (local-only, like `gate-milestone-1`), and the acceptance
  report over whatever development decisions exist.

**Exit, as the plan states it:** the new pipeline produces watchable clips
through the Milestone 1 UI and measurably improves human acceptance over the
heuristic baseline on development footage; incorrect references and
unsupported output are rejected; failure reasons are visible; a poor
recording can produce zero suggestions.

## Order of work

Each step lands as a reviewable change with its tests, and the pipeline stays
runnable end to end after every one.

1. **Windows and contracts.** `editorial.windows.v1`, the builtin stage, the
   schema set for proposals/judgments/looks/trace; goldens on the talk.
2. **The worker, local propose.** `workers/editorial` with the MLX and
   llama.cpp runtimes, two registry entries (one text model per runtime),
   prompts v1, constrained decoding, the trace artifact, readiness. Propose
   runs over the talk and the dogfood episode.
3. **Validate into candidates.** The Rust validator writing
   `discovery.candidates.v1`; the whole existing pipeline — ranking,
   director, Inspector, editor, export — runs on editorial candidates with no
   other change. This is the first watchable clip from the model.
4. **Review, and the ranking that reads it.** Judgments, the extended
   ranking, Results and Inspector showing status and reasons, the
   zero-result and failed-reply states.
5. **The measurement.** The harness's `acceptance`, the decisions recorded on
   the development recordings for both routes, the first report — and the
   model benchmark that picks the local model per device.
6. **Look.** The visual model for flagged candidates, registry entry, gate
   coverage.
7. **The cloud route.** Keychain, budget, scope switches, the setup screen,
   the Lock's honest badge, the provider-unavailable path, the same
   acceptance report for the cloud route kept apart from the local one.
8. **Retire what the measurement retires.** If the editorial route beats the
   heuristic proposers on the development set, the heuristic proposers become
   a fallback that is named as one; if it does not, the milestone is not done
   and the failure category is fixed before more footage is added.

## Decisions to take before step 2

- **Which local model to start with.** Benchmark Qwen3.5-9B against Gemma 4
  12B on this machine (and Gemma 4 26B-A4B if admission allows it; Qwen3.8-27B
  on a machine that holds it) for structured-output reliability, latency per
  window, resident memory beside the speech models, and — for Qwen3.5-9B —
  whether one model serving all three jobs beats two; record the choice as a
  decision with the numbers. The registry entry is written for whichever
  wins, by digest, licence verified at that point.
- **Licence class for editorial weights.** Extend the class rule
  (editorial weights never ship in an export) or restrict to permissive
  models; decide and record.
- **Where the heuristic proposers stand during the milestone.** Proposed:
  they keep running beside the editorial one, labelled, until step 8.
- **Cloud provider for the narrow adapter.** Proposed: Anthropic first, one
  adapter, no abstraction layer until a second provider is actually wanted.
  DeepSeek V4.1-Flash and GLM-5.3-Flash are the obvious second and third,
  as API routes, once the adapter has earned a second one.

## What this milestone does not do

Stated so it is not discovered: no general connector platform; no fleet of
judges (one model, two jobs); no frame-by-frame visual controller (Milestone
3's layout work uses a visual model to classify, never to steer); no cloud
audio or frames until the scope switch exists and is off by default; no
numeric quality score on the board until one has been characterized and
validated against human acceptance.
