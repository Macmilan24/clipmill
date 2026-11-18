# Connect an MCP client

Open ClipMill, then open **Settings → Connected agents**. Create a connection
for one client, choose its scopes and copy the token when it appears. Use
`read` to search recordings and check jobs, `edit` to create a project or
analyze a source, and `render` to export. Use a separate connection for each
client so its activity and revocation stay independent.

## Start a connection

Configure the MCP host to run the bundled `clipmill` executable with the
argument `mcp`. Settings shows the command path for this installation. Put
`CLIPMILL_TOKEN` in the host's private environment or credential store. Avoid
putting the token in a project file, command history or shared configuration.

For an HTTP client, run `clipmill serve` while ClipMill is open. Set the MCP
endpoint to `http://127.0.0.1:7428/mcp` and supply the connected-client token
as an Authorization bearer credential. The gateway runs on this computer.
Stopping the gateway ends network access to that endpoint.

## Analyze and review a recording

1. Call `clipmill_status` to check the connection.
2. Call `clipmill_create_project` and `clipmill_add_source`. Use the project's
   returned ID and the absolute path of a recording accessible to ClipMill.
3. Call `clipmill_analyze` with the project and source IDs. A client using the
   Tasks extension can keep the task ID, reconnect and poll for completion.
   Other clients can call `clipmill_wait_for_job` with the returned job ID.
4. Use `clipmill_find_moments` or `clipmill_list_clips`. Open a candidate with
   `clipmill_describe_clip` to inspect its text and timing before creating an
   edit with `clipmill_create_clip`.
5. A modern client can open the returned edit resource. For export, supply the
   edit ID and a local destination directory. A client with form elicitation
   asks the person for a rights statement before it submits the job. Other
   clients pass the person's statement in the `rights` argument.

Settings shows the client's activity and lets you revoke access. After
revocation its token can no longer poll tasks or read edits, including ones it
opened earlier. The engine reference describes the supported methods,
request metadata and retry behavior. [Engine API](engine-api.md)

## Work with editorial plans

Use the project, source and job IDs of a finished local editorial analysis to
keep every call on the same recording and run.

- **Ask ClipMill to edit.** Call `clipmill_propose_clips` with the IDs and a
  `prompt` such as “Find the explanation of the monthly bill and keep the
  numbers together.” Follow the returned job, then use its ID in
  `clipmill_list_clips`. Inspect a proposal with `clipmill_describe_clip`
  and open it with `clipmill_create_clip_from_plan` using its `clip_id`.
- **Supply an edit.** Inspect `clipmill_memory_overview` and
  `clipmill_find_moments`, then read `clipmill_get_transcript`. Use its
  `source_fingerprint` and the exact `start_ticks` and `end_ticks`
  from the sentences you want to keep. Pass a complete
  `clipmill.mind.clip_plan.v1` document to `clipmill_create_clip_from_plan`
  in the `plan` argument. ClipMill checks its source fingerprint and cut
  boundaries. The reply includes a readable edit resource and the revision
  to use for subsequent commands. [Clip plan schema](../contracts/schemas/clipmill.mind.clip_plan.v1.json)
- **Revise the brief.** Call `clipmill_reprompt` with the previous job ID,
  a new prompt, and optional `min_seconds`, `max_seconds` and `count`.
  The new job reuses the recording's observations. To record feedback on
  one of its clips, pass its run and clip IDs, your decision, and an optional
  reason to `clipmill_set_decision`.
