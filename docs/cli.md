# Use ClipMill from a shell

Start the app and finish installing the local components and models for your
recording. Settings → Connected agents shows the bundled command's absolute
path. Run that executable for the commands below, or add its directory to
your shell's search path. In a development checkout, build the command with
`cargo build -p clipmill-cli`.

For the portable Linux AppImage, run `./ClipMill.AppImage --cli status` and
pass later commands after `--cli` in the same way. Use the image's filename on
your computer. `--print-cli-path` prints the path Settings gives MCP clients.

## Create and analyze

```sh
clipmill status
clipmill project create "Interview"
clipmill add PROJECT_ID "/absolute/path/to/interview.mp4"
clipmill analyze PROJECT_ID SOURCE_ID --language en --wait
```

Replace the IDs with the values printed by the preceding commands. Omit
`--language` for automatic language selection. To keep working while an
analysis runs, omit `--wait` and later run `clipmill job JOB_ID --wait`. When a
component is missing, open Settings → Models and check the recording's analysis
readiness in the app.

## Search, inspect and export

```sh
clipmill find PROJECT_ID SOURCE_ID "how did you get started"
clipmill clips PROJECT_ID SOURCE_ID
clipmill clip PROJECT_ID SOURCE_ID CLIP_ID
clipmill edit PROJECT_ID SOURCE_ID CLIP_ID
clipmill export DOC_ID --to "/absolute/path/to/exports" --rights "YOUR RIGHTS STATEMENT"
```

Choose the clip after reading its words and times. For the export, use an
existing directory and state why you may use the recording in your own words.
The command returns a job ID and destination information. Use
`clipmill job JOB_ID --wait` to follow rendering.

Pass `--json` for structured output. A command running under the local account
has read, edit and render scopes by default. A script can instead use a
revocable connected-client token from its environment. The engine guide lists
operations, scopes and structured errors. [Engine API](engine-api.md)
