# Engine API v2

The engine API is how anything other than the Studio uses ClipMill: the `clipmill` command, agents through MCP, and scripts through the local gateway. It is one set of operations over the same engine the Studio drives (decision R72), so an agent can do what a person can do in the app, and nothing it does is decided differently.

## Calls

A call is JSON:

```json
{
    "op": "start_analysis",
    "args": { "project_id": "prj_…", "source_id": "src_…" },
    "request_id": "optional"
}
```

The answer is `{"ok": true, "result": …}` or `{"ok": false, "error": {"code", "message", "retryable"}}`. Messages are written for people. Times are in seconds. A `request_id`, when given, makes a retried call safe: the same id and the same call return the first answer instead of doing the work twice.

Over the daemon's socket a call travels in the v1 envelope as `EngineCallRequest { token, call_json }`.

## Who is calling

| Caller                                     | How                                                     | Scopes                   |
| ------------------------------------------ | ------------------------------------------------------- | ------------------------ |
| A process of this account on this computer | no token                                                | `read`, `edit`, `render` |
| A connected agent                          | the token created for it in Settings → Connected agents | what the person gave it  |

A token is shown once when it is created and stored only as its SHA-256. Revoking it stops it at once. Scopes are `read`, `edit`, `render`, `publish` and `delete`; publishing and deleting always need a token that was given them.

Every call is written to the audit log with the client, the operation, the ids it named and how it ended (`ok`, `refused`, `failed`). Settings shows the log.

## Operations

| Operation                   | Scope                    | Arguments                                                                                                                                        | Result                                                                                                                                      |
| --------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `whoami`                    | any authenticated client | none                                                                                                                                             | Client ID and granted scopes                                                                                                                |
| `audit_log`                 | read                     | optional `limit` (1–2000)                                                                                                                        | Recent calls from this client. Local Studio can inspect the full log.                                                                       |
| `status`                    | read                     | none                                                                                                                                             | engine version, Local Lock, every operation and its scope                                                                                   |
| `list_projects`             | read                     | none                                                                                                                                             | `projects`                                                                                                                                  |
| `create_project`            | edit                     | `name`                                                                                                                                           | `project_id`                                                                                                                                |
| `add_source`                | edit                     | `project_id`, `path` (absolute, on this computer)                                                                                                | `source_id`                                                                                                                                 |
| `list_sources`              | read                     | `project_id`                                                                                                                                     | `sources`                                                                                                                                   |
| `get_source`                | read                     | `source_id`, optional `project_id`                                                                                                               | Source map and recording details                                                                                                            |
| `get_timeline`              | read                     | `project_id`, `source_id`, optional `job_id`                                                                                                     | Artifact references for the available timeline layers                                                                                       |
| `job_events`                | read                     | `job_id`, optional `after` cursor and `limit` (1–500)                                                                                            | Task transitions in order, with a cursor for the next page                                                                                  |
| `propose_clips`, `reprompt` | edit                     | `project_id`, `source_id`, `prompt`, optional parent `job_id`, `count`, and minimum and maximum seconds                                          | A new job that reuses the recording's observations                                                                                          |
| `create_clip_from_plan`     | edit                     | Recording and run identifiers, plus a ranked `clip_id` or a full clip plan                                                                       | Edit document ID and revision                                                                                                               |
| `get_edit`, `edit_log`      | read                     | `doc_id`                                                                                                                                         | Current edit and revision, or the ordered command history                                                                                   |
| `apply_edits`               | edit                     | `doc_id`, `expected_revision`, `commands` (1–100)                                                                                                | New revision and inverse command                                                                                                            |
| `undo_edit`                 | edit                     | `doc_id`, `expected_revision`                                                                                                                    | New revision after applying the saved inverse                                                                                               |
| `render_preview`            | render                   | `doc_id`, `expected_revision`                                                                                                                    | Job for a private render of the frozen revision                                                                                             |
| `start_analysis`            | edit                     | `project_id`, `source_id`, optional `prompt` (1000 characters), `count` (1–30), `min_seconds` and `max_seconds`, `language`, `editorial: "none"` | the job                                                                                                                                     |
| `get_job`                   | read                     | `job_id`                                                                                                                                         | state (`queued`, `running`, `succeeded`, `failed`, `cancelled`), progress, stages                                                           |
| `list_jobs`                 | read                     | `project_id`                                                                                                                                     | `jobs`                                                                                                                                      |
| `cancel_job`                | edit                     | `job_id`                                                                                                                                         | the job                                                                                                                                     |
| `get_transcript`            | read                     | `project_id`, `source_id`, optional `job_id`, `start_seconds`, `end_seconds`                                                                     | Source fingerprint and sentences with unit IDs, exact tick boundaries, times, speakers and text                                             |
| `memory_overview`           | read                     | `project_id`, `source_id`, optional `job_id`                                                                                                     | Chapters, events, entities and links from the recording memory                                                                              |
| `find_moments`              | read                     | same recording identifiers, `query`, optional `limit` (1–30) and `filters`                                                                       | Timecoded transcript and memory matches, with the search methods that found each one                                                        |
| `list_clips`                | read                     | `project_id`, `source_id`, optional `job_id`                                                                                                     | `clips`: ranked plans with parts, joins, evidence, critic findings, keep scores and the saved decision. Built-in runs return ranked moments |
| `describe_clip`             | read                     | as `list_clips`, plus `clip_id`                                                                                                                  | the clip with the words of each part                                                                                                        |
| `set_decision`              | edit                     | as `describe_clip`, plus `decision`: `approved`, `kept`, `rejected`, `undecided`                                                                 | The saved decision and optional `reason` (up to 240 characters)                                                                             |
| `create_clip`               | edit                     | as `describe_clip`, optional `shape` (`vertical`, `square`, `portrait`, `landscape`)                                                             | `doc_id` of the edit document                                                                                                               |
| `export_clip`               | render                   | `doc_id`, `destination_dir`, `rights`, optional `title`                                                                                          | the export job                                                                                                                              |
| `list_models`               | read                     | none                                                                                                                                             | `models`                                                                                                                                    |

Without a `job_id`, clip and transcript operations read the newest finished analysis of the source.

A completed editorial ranking may contain zero clips. Each new brief creates its own run and leaves the earlier results available by job ID. The last twelve project-specific reviews and editing actions can inform subsequent briefs. The recording and the review history stay on this computer.

Use `clipmill reprompt PROJECT SOURCE "Find the clearest answer" --run JOB --count 2 --min 15 --max 40 --wait` to reuse a finished analysis from the shell. A working preview stays in the local artifact store. The preview job reports its output addresses. A final export also needs a destination and the user's rights statement.

`start_analysis` runs the local editorial model and the mind unless `editorial` is `"none"`; everything stays on this computer.

`export_clip` needs `rights`: a rendered file states whether the person owns or may use the recording, and there is no honest default for that. An agent must ask the person and pass what they said.

## Errors

| Code                  | Meaning                                                                 |
| --------------------- | ----------------------------------------------------------------------- |
| `invalid_argument`    | The call is malformed or an argument is missing; the message says which |
| `not_found`           | No such project, source, job or clip                                    |
| `out_of_scope`        | The client was not given the scope this operation needs                 |
| `unauthenticated`     | The token is unknown or revoked                                         |
| `conflict`            | The thing changed since it was read                                     |
| `unavailable`, `busy` | Try again shortly (`retryable` is true)                                 |
| `policy_denied`       | The Local Lock or a setting forbids it                                  |
| `internal`            | A fault in ClipMill                                                     |

## MCP tools and background jobs

Run `clipmill mcp` for stdio or post to `/mcp` on the loopback gateway. HTTP
requires a connected-client bearer token. The 2026-07-28 protocol supports
stateless discovery with `server/discover`. Clients may also send a tool request
directly. Include the protocol version and client capabilities in each request's
`params._meta`. Earlier clients continue to use `initialize`, `tools/list`,
`tools/call` and the ordinary job and wait tools.

Analysis, proposals, re-prompts and exports can return a durable task when a
client declares `io.modelcontextprotocol/tasks` in its per-request extension
capabilities. A `resultType` of `task` carries the engine job ID, original
creation time and suggested polling interval. Use `tasks/get` with that ID
after a reconnect or daemon restart. A completed task includes the final tool
result, including an `isError` result when a job fails. A successfully requested
cancellation returns an empty acknowledgement; poll until the engine reports a
terminal state.

Task polling verifies the submitting identity and its current read scope.
Cancellation requires the original edit or render scope. A client without
read access receives the ordinary tool result. The local account is the caller
when stdio runs without a connected-client token. These jobs never ask for
mid-flight input. `tasks/update` therefore authenticates and acknowledges
valid input-response maps without changing a job. Progress is available by
polling. See the [Tasks extension](https://modelcontextprotocol.io/extensions/tasks/overview)
and [Streamable HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http).

When a modern client supports form elicitation, `clipmill_export_clip` first
asks the person for a rights statement on the original tool call. The engine
checks the render grant, caller, exact arguments and five-minute expiry when
the client returns the form. Declining or cancelling finishes without creating
a job. After a daemon restart, a client can begin a fresh confirmation.
Other clients pass the person's statement through the existing `rights` argument.

Tool results that create an edit include a `resource_link` to its document.
Read it through `resources/read` with the read scope, using the returned
`clipmill://edits/{doc_id}` URI. The payload contains the current revision and
edit document. The resource template is discoverable through
`resources/templates/list`.

A retried write can carry the same `Idempotency-Key` header on either HTTP
endpoint. Over stdio, provide the key as `params._meta["io.clipmill/requestId"]`
on a tool call. Keys accept 1 to 255 printable ASCII bytes. Reuse a key only
with the same operation and arguments. The engine scopes it to the submitting
identity. MCP's JSON-RPC message ID identifies one exchange.

Run `just gate-mcp` with the pinned FFmpeg tools installed to verify both client
generations and both transports. To check an inline result from a completed
analysis, install the shots and faces workers and the YuNet model, then run
`uv run --locked --project eval/harness python tools/drills/mcp-tasks/drive.py --with-workers`.

## Search a recording

Run `clipmill find PROJECT SOURCE "where do they talk about pricing"`. Pass
`--speaker A`, `--mood funny`, `--chapter c2` or `--modality laughter` to
narrow the results. Use `--modality emphasis` for passages with a sustained rise in
voiced pitch or level relative to nearby speech. The HTTP operation and the MCP tool accept the same
filters in a `filters` object. Obtain chapter identifiers with
`clipmill call memory_overview`.

With the meaning-search worker and model installed, analysis indexes passages
and remembered events locally. Search also works with an existing transcript
index. The response's `meaning` field says whether this search used vectors.
Search times are in seconds. For an edit, use the transcript’s exact integer tick boundaries and its source fingerprint. One second contains 90,000 ticks.
