# Milestone 2 — complete moments from the selected Qwen model

Revised 18 September 2026. The user selected Qwen3.5, asked to skip M0,
and asked not to focus this implementation pass on benchmarking. We proceeded
with functional delivery using the propose → validate → review pipeline.
Editorial quality remains unverified: the original acceptance-gain objective
is retained, not attributed to an explicit waiver by the user.

## Scope and finish line

English podcasts/interviews, including two speakers, on the current
Apple-silicon desktop. Reuse Milestone 1's selected-run editor and immutable
export. Milestone 3 still owns improvements to framing, caption styling,
and audio polish; a semantic review does not certify the entire final video.

Functional completion means a real pinned Qwen generation works, a real
analysis reaches reviewable candidates, and one selected candidate goes
through document creation and decodable MP4/SRT/VTT export. Invalid references,
malformed output, cancellations, and failed calls cannot masquerade as
"no suitable moments." Returning fewer clips, including zero, is valid.
A synthetic talk is suitable for exercising this path. It does not establish
editorial superiority on real interviews. The quality exit remains open: human
keep/reject decisions and repair effort on development footage must show
whether this route improves on the heuristic baseline.

## Pipeline

| Stage    | Work                                                                                                                                  | Output                                           |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| Windows  | Deterministic overlapping sentence windows, adjacent context, cited outline                                                           | `editorial.windows.v1`                           |
| Propose  | Qwen chooses duration-valid full-sentence spans; includes title, hook/setup/payoff, reason and uncertainty                            | `editorial.proposals.v1`                         |
| Validate | Rust checks source, window identity, references, duration, boundary timing and duplicate overlap; derives exact word-based cuts       | `discovery.candidates.v1` and validation reasons |
| Review   | Qwen checks each candidate with neighboring transcript sentences for missing context, incomplete payoff, ads and misleading omissions | `editorial.judgments.v1`                         |
| Look     | Only visually dependent candidates: at most four frames from inside the cut, one bounded question                                     | `editorial.looks.v1`                             |
| Rank     | Exclude rejected clips, put accepted first, suppress substantially overlapping selections, allow shortfall                            | `ranking.set.v1` with review status/reasons      |

The validator retains the proposed semantic span. It may add at most 150 ms
of padding in a word-free gap, subject to the requested duration. Padding is clamped before any unavailable region, and minimum duration is
checked on the spoken span before padding. Its single
legal boundary pair stops the heuristic optimizer from changing a span after
review. The local/cloud proposal decoder enumerates legal sentence spans for the requested duration, presents compact allowed-end ranges, and maps the model’s chosen span back to the shared proposal contract. It never lengthens a too-short quote automatically. Longer answers can yield a self-contained part. Invalid
references and unavailable speech are recorded with reasons; an entirely invalid nonempty response
fails analysis rather than being described as a recording with no moments. Interpolated timing at a cut boundary is rejected. Interpolated words and short alignment-unavailable regions strictly inside a complete span remain usable, but deterministically mark the result Needs review with a caption-timing warning, regardless of the semantic verdict.

Repeated nominations with at least 85% interval overlap are merged while
retaining proposal IDs and uncertainties. The candidate keeps its title,
hook, setup, payoff and rationale; ranking carries the title to Results.

The heuristic route remains an explicitly named, separately selectable
baseline. It is not silently mixed into an editorial result or invoked when
the model fails. Old jobs without the new route fields keep their original
behavior. Existing accepted edit documents remain bound to their original run.

## Local runtime and provenance

`workers/editorial` uses `mlx-vlm==0.7.1`, locked torchvision/PyTorch image
preprocessing dependencies, and the registry-pinned
`mlx-community/Qwen3.5-9B-4bit` revision
`8b2b98c00a6b4d291155e4890773ca8f769aee53`. All files have recorded SHA-256 and
size pins. The weights are Apache-2.0; they are not included in video exports.
There is no second model server or duplicate model store.

One selected model serves proposal, review and visual checks. Temperature is
zero, seed zero, output is capped at 2,048 tokens, and JSON-schema constraints
are applied during decoding. Long-recording outlines are capped at 32 cited entries, mixing nearby and global topics. Input text is bounded to 12,000 tokens and each
call has a ten-minute token-loop deadline. Visual inputs are resized to a
bounded resolution. These settings, the model digest, prompt digest, source
and input artifacts participate in reproducible recipes. This does not promise
byte-identical inference across devices.

The worker releases model state after a stage, checks cancellation during
generation, and uses existing scheduler memory admission. A successful tiny
real generation writes `state/editorial-runtime.json`, bound to this hardware
and model digest plus the pinned runtime version. The inference subprocess exits before available memory is measured, and the scheduler retains the verified GPU capacity after consuming its update. Readiness also checks the model’s memory reservation. This establishes runtime readiness for Metal admission; it
is neither M0 nor an automatic model-selection benchmark. Other platforms and
GGUF runtimes are deferred until required; this change does not claim they work.

Calls retain private traces with prompt, response, route, model, token usage
and timing. A failed or malformed window does not stop later windows. Mixed
passes publish explicit per-window/per-candidate failures alongside usable
answers; no answered windows or no usable candidate reviews still fail the
stage. Missing answers are never interpreted as an editorial “none.” Failed
semantic reviews are excluded; unavailable visual checks remain Needs review.

Partial artifacts remain durable evidence, but are not reusable as complete
cache hits. A retry uses an artifact- and task-bound recovery recipe without
changing old artifacts. Successful earlier stages can still be reused. The
shared partial cache key remains partial, so subsequent runs can require
additional inference even after another run recovered; this is a conservative
cost tradeoff, not a promise of per-window cache reuse.

## Optional cloud route

The user must choose Cloud-assisted and consent to transcript sharing for
that run. Changing routes clears consent. Video, audio and sampled frames
remain local. The sole adapter is Anthropic `claude-sonnet-4-6`; proposal and
review use it, while any visual check stays with local Qwen. No automatic
cloud fallback exists. The default worker exposes only local capabilities.
Cloud processing uses a separate worker entry point, explicitly started with
`./tools/run-workers.sh --cloud-editorial` after trust-key enrollment; launching
that worker does not substitute for the per-analysis consent.

Credentials are read inside the worker from macOS Keychain, service
`dev.clipmill.anthropic`, account `clipmill`. They never enter renderer state,
job payloads, artifact recipes or traces. Add the password through Keychain
Access. Missing credentials produce a recovery message, not a hidden prompt
or a secret in a command argument.

The setup screen names the provider, scope, cost example, and a $0.01–$100
run cap. Rates are $3 per million input tokens and $15 per million output
tokens ([provider documentation](https://platform.claude.com/docs/en/models/sonnet-4-6/overview)).
The exact transcript/token count is not known before transcription, so the
screen does not invent a recording-specific price estimate. A durable,
locked per-run ledger reserves a conservative request bound before sending,
refunds the difference after verified usage, and keeps uncertain calls fully
reserved. There are no automatic provider retries. Errors offer explicit
reanalysis locally. Tests use a fake provider; a live paid request requires
explicit cloud enablement by the user.

The session's Local Lock disengages when a network-allowed task starts, including
a conservative cache hit, not merely because its implementation is installed.
It reports task admission rather than an OS-level proof of zero egress; see
[Local Lock](local-lock.md) and the [threat model](threat-model.md). Settings reports task-policy
counts; neither Settings nor progress claims to measure network bytes.

## UI contract

Local Qwen is the default for a new analysis. Readiness is filtered to the
chosen route; missing weights/workers have a recovery command. Results and
Inspector display **Ready to review** or **Needs review**, a summary, reasons,
and the route. They do not present the heuristic diagnostic score as an
editorial-quality percentage. Visual dependency stays visibly marked for
human inspection even if a sparse frame check answers positively.

An analysis failure is shown as a run failure with its reason and a local
reanalysis action. A fully assessed empty ranking says no suitable complete
moments were found. Partial runs show failed window/review/visual-check counts
even when the requested clip count was met. Empty partial results explicitly
say that some material was not assessed. Results → Inspector → Editor → Export continue to carry
the selected project, source, run, candidate, document and revision.

## Developer checks

Install the worker and pinned weights:

```sh
uv sync --project workers/editorial
./tools/fetch-models.sh qwen3-5-editorial-mlx
```

`just workers` performs the local runtime readiness check before enrolling
its capabilities with the running daemon (development trust keys must already
be enrolled before daemon startup). `just app` performs enrollment first.

`just gate-editorial` checks windows, contracts, invalid references, review
selection, worker failure/cancellation behavior, cloud consent/budget tests,
and the real local model-to-export smoke. `just gate-editorial-unit` runs the
checks that need no weights. `just gate-milestone-1` exercises the earlier
selected-clip workflow independently of optional editorial models.

The local smoke leaves its MP4, captions, ranking, result JSON and logs in the
reported temporary directory. It uses the shell/daemon contracts, not a
hand-clicked Tauri window. A native-window check must be reported separately.
