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

| Operation         | Scope  | Arguments                                                                                            | Result                                                                                                    |
| ----------------- | ------ | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `status`          | read   | none                                                                                                 | engine version, Local Lock, every operation and its scope                                                 |
| `list_projects`   | read   | none                                                                                                 | `projects`                                                                                                |
| `create_project`  | edit   | `name`                                                                                               | `project_id`                                                                                              |
| `add_source`      | edit   | `project_id`, `path` (absolute, on this computer)                                                    | `source_id`                                                                                               |
| `list_sources`    | read   | `project_id`                                                                                         | `sources`                                                                                                 |
| `start_analysis`  | edit   | `project_id`, `source_id`, optional `min_seconds` and `max_seconds`, `language`, `editorial: "none"` | the job                                                                                                   |
| `get_job`         | read   | `job_id`                                                                                             | state (`queued`, `running`, `succeeded`, `failed`, `cancelled`), progress, stages                         |
| `list_jobs`       | read   | `project_id`                                                                                         | `jobs`                                                                                                    |
| `cancel_job`      | edit   | `job_id`                                                                                             | the job                                                                                                   |
| `get_transcript`  | read   | `project_id`, `source_id`, optional `job_id`, `start_seconds`, `end_seconds`                         | `sentences` with unit ids, times, speakers and text                                                       |
| `memory_overview` | read   | `project_id`, `source_id`, optional `job_id`                                                         | Chapters, events, entities and links from the recording memory                                            |
| `find_moments`    | read   | same recording identifiers, `query`, optional `limit` (1–30) and `filters`                           | Timecoded transcript and memory matches, with the search methods that found each one                      |
| `list_clips`      | read   | `project_id`, `source_id`, optional `job_id`                                                         | `clips`: the mind's plans (`kind: "plan"`, with form and parts) and the ranked moments (`kind: "moment"`) |
| `describe_clip`   | read   | as `list_clips`, plus `clip_id`                                                                      | the clip with the words of each part                                                                      |
| `set_decision`    | edit   | as `describe_clip`, plus `decision`: `approved`, `kept`, `rejected`, `undecided`                     | the decision                                                                                              |
| `create_clip`     | edit   | as `describe_clip`, optional `shape` (`vertical`, `square`, `portrait`, `landscape`)                 | `doc_id` of the edit document                                                                             |
| `export_clip`     | render | `doc_id`, `destination_dir`, `rights`, optional `title`                                              | the export job                                                                                            |
| `list_models`     | read   | none                                                                                                 | `models`                                                                                                  |

Without a `job_id`, clip and transcript operations read the newest finished analysis of the source.

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

Analysis and exports can return a durable task when a
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
narrow the results. The HTTP operation and the MCP tool accept the same
filters in a `filters` object. Obtain chapter identifiers with
`clipmill call memory_overview`.

With the meaning-search worker and model installed, analysis indexes passages
and remembered events locally. Search also works with an existing transcript
index. The response's `meaning` field says whether this search used vectors.
Times are in seconds; unit identifiers refer to the source transcript.
