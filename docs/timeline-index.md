# Timeline Index

Every analysis that produces a transcript also publishes a **Timeline Index**
(`timeline.index.v1`, decision R74): one SQLite file, `timeline.sqlite`, holding
what the recording contains in time order. It is derived, never edited. The
`index-timeline` stage builds it in the daemon from the recording's published
evidence, so it caches, invalidates and recovers like every other stage, and
readers open it read-only.

## What it holds

| Table           | Rows                                                                                                                 |
| --------------- | -------------------------------------------------------------------------------------------------------------------- |
| `meta`          | `schema_version` (`clipmill.timeline.index.v1`), `source_fingerprint`, `language`                                    |
| `words`         | each recognised word, its times, its sentence and speaker                                                            |
| `sentences`     | each sentence of the transcript index, its times, text and the voice that speaks most of it                          |
| `sentence_text` | an FTS5 full-text index over sentence text (case- and accent-insensitive)                                            |
| `utterances`    | speech between pauses, with the pause before and after                                                               |
| `topics`        | lexical topic segments and their keywords                                                                            |
| `turns`         | speaker turns, with voices labelled `S1`, `S2`... in order of first speech                                           |
| `shots`         | shots between cuts, when the recording has video                                                                     |
| `events`        | timed events: today, pauses of at least 0.7 s; later stages add laughter, applause, music, emphasis and visual notes |

Times are integer ticks at 1/90000 s. The inputs are `speech.transcript.v1`
and `index.transcript.v1`, plus `speech.speakers.v1` and `evidence.shots.v1`
when the plan ran them. A recording with no audio has no transcript, so the
stage is skipped with the reason `no_audio`.

## Guarantees

- **Reproducible.** The same evidence builds the same bytes: tables are filled
  in a fixed order in one transaction and the file is compacted before it is
  published. Nothing else is written beside it.
- **Versioned.** A reader refuses a file whose `schema_version` it does not
  know. Changing the tables is a new version.
- **Safe to query with typed text.** `clipmill-timeline` turns a search into an
  FTS5 query in which every word and quoted phrase is literal text, so input
  from a person or an agent cannot use FTS5 operators or cause a syntax error.

The `clipmill-timeline` crate builds and reads the file. Nothing in the
analysis plan reads it yet; the editorial mind, library search and the agent
tools will.
