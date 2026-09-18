**ClipMill architecture and Phase 1 review — 17 September 2026**

The design contains sound engineering ideas, but its scope and implementation order are too ambitious for reaching a useful clipping product quickly. The code contains substantial working infrastructure alongside several serious integration defects. Editorial quality remains unmeasured. Keep the useful foundations, repair the user workflow, and bring a small, measurable semantic clipping pipeline forward before implementing the rest of the book.

This is a targeted architecture and implementation review, not a complete security audit or an OpusClip quality benchmark. I inspected the design book's LaTeX sources in the neighboring `creator-os` project, the Phase 1 documentation, discovery/ranking, analysis planning, directing, editing, preview, export, and evaluation code. I ran existing tests and a small caption/trim reproduction. I did not manually operate the desktop UI or evaluate a representative collection of real recordings.

**What is sound in the design**

- Keeping source media local, using SQLite for durable state, and storing reusable derived artifacts are reasonable choices for a local desktop product.
- Separating model inference from media rendering gives model failures a smaller impact and allows model replacements.
- Keeping timestamps precise, preserving source provenance, and defining one edit document for export are valuable. The implementation must still correctly distinguish source time, clip time, and proxy coordinates.
- Separating candidate discovery, boundary refinement, and final selection is a useful conceptual model. These do not all need independent services or elaborate inference pipelines.
- Human correction, recovery, and an actual rendered deliverable belong in an early version.

The existing Rust/Python/Tauri split is defensible. Rewriting it into another stack would add work without directly improving clip judgment. The immediate simplification should be in product scope and the intelligence pipeline.

**Where the plan overreaches**

The roadmap explicitly assumes two systems/media engineers, three applied-ML engineers, two editor/frontend engineers, one designer, and one evaluation/data engineer, plus contracted editors and shared release/security work. That is roughly nine core contributors. See [the staffing assumptions](/Users/sami/Personal/YT/creator-os/clipper-system-design/chapters/24-roadmap.tex:85).

The intended product spans dialogue clipping, gameplay, sports, visual events, a general editor, evidence graphs, hierarchical memory, many proposers, judge ensembles, tracking, B-roll, publishing, and multiple inference backends. Those are plausible long-term capabilities. Treating their infrastructure as prerequisites for testing useful clip selection delays the project's central experiment: can this system find moments a creator wants to publish?

Phase 1 deliberately specifies a deterministic ranking baseline; the lack of an LLM judge is therefore consistent with that part of the plan. The problem is the plan's sequencing relative to your goal. A dependable export proves delivery, while useful moment discovery needs separate evidence. The current project has invested much more heavily in the first.

Three design assumptions should remain experiments:

1. **More local model calls do not automatically compensate for weaker reasoning.** Repeated votes can repeat the same mistake, and consume latency, memory bandwidth, battery, and thermal headroom. A recent [judge-panel study](https://arxiv.org/abs/2605.29800) found strongly correlated errors in its tested tasks; that is a reason to measure ensemble value, not proof about video clipping. Start with one capable judge and add votes only when a held-out comparison shows a worthwhile gain.
2. **Evidence references provide traceability, not automatic semantic correctness.** A deterministic checker can verify that a word or frame exists and that timestamps are ordered. It cannot establish arbitrary claims such as “the answer resolves the question” merely by validating a citation. Separate structural validation from semantic judgment.
3. **A graph and several levels of summaries may help, but their benefit must justify their maintenance cost.** First compare simpler overlapping transcript windows, neighboring context, and source references. Research such as [Lost in the Middle](https://arxiv.org/abs/2307.03172) supports testing long-context reliability; it does not establish that this project's full graph is required, or predict the performance of every current model.

**What the implemented intelligence currently does**

The active discovery and ranking path is heuristic. Speech recognition uses models, but editorial discovery does not currently use an LLM or VLM to understand whether a clip has an interesting, complete story.

| Component            | Current behavior                                                                    | Consequence                                                                                             |
| -------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Narrative proposer   | Uses lexical topic spans and a novelty-based closing sentence                       | Topic continuity does not establish a setup, reveal, and payoff                                         |
| Insight proposer     | Weights vocabulary novelty, numbers/capitalization, a claim-word list, and emphasis | Distinctive wording can be mistaken for valuable content                                                |
| Q&A proposer         | Uses question marks/English question openings and following sentences               | Does not know who asked, whether the question is rhetorical, or whether the answer actually resolves it |
| Ranker               | Hand-set feature weights, uncertainty penalties, and overlap diversity              | No demonstrated calibration against editorial preferences                                               |
| Display score        | Percentile within the recording; a singleton receives 99                            | A high number does not demonstrate publishable quality                                                  |
| Visual understanding | Shot detection; face detector exists separately                                     | No active semantic visual discovery or VLM verification                                                 |

Sources: [proposers](/Users/sami/Personal/YT/clipmill/crates/clipmill-discovery/src/proposers.rs:71), [scorecard](/Users/sami/Personal/YT/clipmill/crates/clipmill-discovery/src/scorecard.rs:37), [percentiles](/Users/sami/Personal/YT/clipmill/crates/clipmill-discovery/src/scorecard.rs:381).

The selector stops when it reaches the requested count or exhausts eligible nonduplicates. It has no separate acceptance threshold for editorial usefulness. Returning fewer because candidates overlap is different from recognizing that nothing is worth publishing. See [selection](/Users/sami/Personal/YT/clipmill/crates/clipmill-discovery/src/ranking.rs:314).

A further recall limitation: the boundary lattice must contain the entire seed. If a narrative topic or Q&A seed exceeds the requested maximum duration, that proposer cannot shorten it into a useful excerpt; it produces no legal interval. Other proposers may still find something there. Use smaller anchors or semantic subspans before expansion. See [seed expansion](/Users/sami/Personal/YT/clipmill/crates/clipmill-discovery/src/lattice.rs:112).

OpusClip's documented multimodal product includes visual/audio cues and low-dialogue footage. That is a broader target than the current speech-led baseline; its public documentation does not reveal its internal architecture. See [OpusClip's ClipAnything description](https://help.opus.pro/docs/article/9947095-clip-anything).

**Concrete implementation findings, in repair order**

1. **P1 — Editor and Export lose the user's selected clip.** Both load the newest project and its newest edit document. Approving a clip in an older project can therefore open/export another project's clip, or show an empty editor if the newest project has no document. Batch approval creates multiple documents but these screens provide no explicit way to select among them. Carry `projectId`, `sourceId`, and `docId` through navigation and resolve the proxy belonging to the document's source. Sources: [editor loading](/Users/sami/Personal/YT/clipmill/apps/desktop/src/editor/useEditor.ts:55), [export loading](/Users/sami/Personal/YT/clipmill/apps/desktop/src/screens/ExportScreen.tsx:56). Confirmed by tracing the active code paths.

2. **P1 — Editor preview plays the full-source proxy without the clip's time mapping.** The video element receives the original proxy URL. Playback time and frame stepping are interpreted directly as program time; the preview plan contains no source interval mapping. A clip selected from 10:00–10:30 therefore has no mechanism here to seek to 10:00 when clip time is zero. Playback also ends at the proxy's end rather than the clip's end. Add an explicit source/program mapping and enforce the segment boundaries in playback. Sources: [stepping](/Users/sami/Personal/YT/clipmill/apps/desktop/src/screens/Editor.tsx:80), [video playback](/Users/sami/Personal/YT/clipmill/apps/desktop/src/screens/Editor.tsx:320), [preview contract](/Users/sami/Personal/YT/clipmill/crates/clipmill-render/src/preview.rs:82). Confirmed by code tracing, not a manual UI playback test.

3. **P1 — Caption editing addresses the wrong representation.** Preview exposes `captions.burn_in` when present, while edit commands locate and modify only `captions.cues`. Distinct IDs cause an unknown-cue error; overlapping IDs can modify a different reading cue and leave the visible word untouched. Actual caption generation numbers both groupings independently as `cue_1`, `cue_2`, etc. Preserve common word identities/corrections across both presentations, with explicit intent for grouping edits. Sources: [preview cues](/Users/sami/Personal/YT/clipmill/crates/clipmill-render/src/preview.rs:179), [edit command](/Users/sami/Personal/YT/clipmill/crates/clipmill-edit-ir/src/command.rs:201), [cue lookup](/Users/sami/Personal/YT/clipmill/crates/clipmill-edit-ir/src/document.rs:416), [cue generation](/Users/sami/Personal/YT/clipmill/crates/clipmill-captions/src/document.rs:329). Reproduced against the compiled command engine.

4. **P1 — Trimming mixes time domains and fails to preserve caption synchronization.** The UI sends clip-relative ticks to `Trim`, whose fields are absolute source in/out points. Independently, the command handles a shorter segment as a deletion at its tail even when the user advances its start. Caption splicing only touches the reading cues; the burned-in cues remain unchanged. Correct source-coordinate commands, explicitly account for head and tail changes, and retime both caption presentations and automation consistently. Sources: [trim controls](/Users/sami/Personal/YT/clipmill/apps/desktop/src/screens/Editor.tsx:219), [trim implementation](/Users/sami/Personal/YT/clipmill/crates/clipmill-edit-ir/src/command.rs:308), [caption splicing](/Users/sami/Personal/YT/clipmill/crates/clipmill-edit-ir/src/document.rs:448). Command-level caption failures reproduced; UI time-domain mismatch confirmed by code tracing.

5. **P2 — Face tracking is not connected to normal analysis.** `analyze_source` plans speech, shots, index, discovery, and ranking, but no face task; the analysis manifest also has no face stage. Consequently, a normal newly analyzed source directs clips with fit layout. Connect the existing detector to the relevant analysis/directing path and retain fit as a legitimate fallback. Sources: [analysis planning](/Users/sami/Personal/YT/clipmill/crates/clipmilld/src/jobs.rs:1341), [manifest stages](/Users/sami/Personal/YT/clipmill/crates/clipmilld/src/analysis.rs:46). This gap is already acknowledged in [Phase 1 notes](/Users/sami/Personal/YT/clipmill/docs/phase1.md:55). Connecting detection alone would not provide active-speaker understanding for multiple faces.

6. **P2 — The crop re-solve path also needs coordinate repair.** It asks for tracks from zero to the clip duration instead of the clip's source interval, and converts normalized detections using output dimensions rather than source dimensions. Wiring the detector alone will expose these errors for nonzero clip starts and landscape sources. Reuse the director's source-aware conversion and map source timestamps to segment-local keyframes. Source: [re-solve](/Users/sami/Personal/YT/clipmill/apps/desktop/src/screens/EditorScreen.tsx:68).

7. **P1 for release evidence — The render-speed gate does not measure rendering.** It times `cargo test -p clipmill-render --test compilation`, whose tests compile render plans without encoding a video. It then searches all of `target` and reads a preexisting render manifest. The claimed ratio can therefore combine a previous clip's duration with compiler-test runtime. Time an actual fresh export and read the exact manifest produced by that run. Sources: [benchmark](/Users/sami/Personal/YT/clipmill/tools/drills/render-slo.sh:31), [compiler tests](/Users/sami/Personal/YT/clipmill/crates/clipmill-render/tests/compilation.rs:35). The recorded 1.587× figure is not trustworthy throughput evidence from this gate.

The project's “exact preview” claim also exceeds its tests. The parity suite principally compares Rust-produced plans and subtitle arithmetic. It does not exercise the actual browser's source-time playback, crop transform, caption styling, and audio behavior against a decoded export. Its own [parity document](/Users/sami/Personal/YT/clipmill/docs/preview-parity.md:65) acknowledges part of that missing connection.

**Evidence from this review**

- Frontend: all 219 tests in 19 files passed.
- Rust: the existing suites passed for discovery, director, evidence, reframe, captions, edit IR, render, and export.
- Daemon: both edit-document integration tests passed, including acknowledged-edit recovery after killing the daemon. The initial sandboxed attempt could not bind Unix sockets; a permitted rerun passed. That first failure was environmental.
- Actual export: explicitly ran the normally ignored `the_first_slice_renders_to_a_playable_vertical_clip` test. It passed and verified a real rendered file. This establishes a working fixture export, not full product correctness or a throughput SLO.
- The three analysis integration tests were reported as ignored in the standard invocation. I did not run the entire speech/model fleet or a private-corpus quality evaluation.
- A standalone reproduction using the operational edit IR confirmed: editing a displayed distinct-ID burn-in cue fails; editing a reading cue leaves burned text unchanged; colliding cue IDs can change the wrong representation; trimming leaves stale burned-in timing; advancing the source start leaves the original opening caption.

The reproduction source and output are retained in [the audit artifacts](/Users/sami/Personal/YT/clipmill/target/audit-2026-09-16/caption_probe.rs) and [its output](/Users/sami/Personal/YT/clipmill/target/audit-2026-09-16/caption_probe.txt). They use the published two-caption-intents fixture; the overlapping-ID variant models the independent numbering used by the caption generator.

Passing component tests and a fixture render are useful evidence. They do not overturn the failures above: the missing assertions concern how those pieces connect.

**A smaller path to the intended product**

Start with one declared content category, such as English podcasts/interviews, and one reference machine. Keep the current infrastructure, but freeze expansion of it while establishing this loop:

```text
Local video → transcript with word times → contextual transcript windows
→ one LLM proposes complete moments using source sentence/word IDs
→ validate references, refine boundaries, deduplicate, judge usefulness
→ optional targeted visual checks → face-aware layout or fit
→ editable captions → correctly mapped preview → FFmpeg export
```

Use overlapping windows and neighboring context so a question and answer are not separated arbitrarily. Let the model identify the hook, necessary setup, and payoff, and allow it to return no clip. Map its selections to measured source timestamps rather than trusting invented timestamps. Retain the current heuristics as a baseline and possibly additional proposals, while measuring what they contribute.

For speech-led content, use a VLM when the candidate depends on something shown, when the layout is unclear, or when visual context could change the editorial decision. Give it timestamped samples or short sequences sufficient for the question. If the next target is sports/gameplay or low-dialogue content, add visual/audio proposal coverage across the source; checking only transcript finalists would miss the very moments those genres need.

Begin with approximately 10–20 representative videos and a separate held-out subset. Annotate acceptable moments and alternate boundaries. Compare the existing baseline with the simple LLM pipeline using recall@k, how many top clips are actually usable, context completeness, boundary repair effort, duplicate rate, time to first useful clip, and runtime/memory. Include recordings with no worthwhile moments. Existing recall and annotation code can support this; a signed synthetic fixture is not a substitute for editorial labels.

Only add an evidence graph, additional proposers, pairwise tournaments, multiple judges, or extra runtimes when an experiment identifies a failure that the addition resolves. An optional API comparison can be useful if you permit that data flow; local-first does not require it.

YouTube URL acquisition is a separate unfinished capability: the import screen currently chooses a local file, and source registration explicitly refuses URLs. A deliberate network import step can precede a local analysis pipeline. See [source validation](/Users/sami/Personal/YT/clipmill/crates/clipmilld/src/sources.rs:200).

The next milestone should be concrete: choose a clip from an older project, preview the correct source interval, correct a word, trim either edge, export it, and verify the resulting media and captions; then demonstrate that a small semantic selector finds useful clips on held-out recordings. That would justify further sophistication far more directly than implementing another large chapter of the architecture.
