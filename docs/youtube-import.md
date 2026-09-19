# YouTube source import

YouTube import is an explicit network operation before the existing local media
pipeline. It is independent of channel connection and does not require Google
OAuth. The creator's permission is a user assertion; ClipMill cannot infer a
clipping licence from a public link or the download succeeding.

## User flow

Choose **New project → YouTube**, paste a video link, confirm permission to use
the footage, and import. ClipMill stores one original in its managed source
directory. The status shows actual downloaded bytes, then media preparation and
source inspection. A failed or interrupted attempt has an explicit Retry action.
Closing a window does not cancel the daemon's import. Restarting the daemon marks
unfinished work interrupted; it never silently resumes network activity.

Once source inspection succeeds, the same content profile, analysis, results,
editor and export workflow used for local files becomes available. Import does
not run editorial analysis without the user's next action. Existing rights
confirmation at export remains independent of the import assertion.

This version accepts a single HTTPS watch, share, Shorts, embed or recorded-live
video link. Tracking, start time and playlist context are removed; the whole
selected video is imported. Channels, playlists without a video, current live
broadcasts, upcoming videos, login-only content and protected media are refused.
Choose 1080p for normal work, 720p for a smaller source, or 360p for a lightweight
test. These are maximum heights, with no upscaling. Duration is capped at six hours
and downloaded media at 8 GiB. Retry keeps the original quality choice. Separate
streams are remuxed without re-encoding. Original source
quality beyond 1080p and authenticated downloads are not implemented.

## Runtime and setup

Run `just setup-youtube` after the ordinary repository setup. The importer uses
the locked Python environment in `integrations/youtube-import`, the repository's
pinned FFmpeg, and Node.js 22 or newer. `CLIPMILL_NODE` can select an absolute
runtime path for a desktop launch with a limited PATH. The daemon supports an
explicit `CLIPMILL_YOUTUBE_IMPORTER` helper path; the development default is the
repository's `tools/import-youtube.sh`. Packaged distribution must provision that
helper environment and the JS runtime; the source checkout setup is not an
installer for end users.

yt-dlp **2026.08.19** and its matching **yt-dlp-ejs 0.8.0** are pinned together.
The helper never installs packages, fetches remote solver components, reads
browser cookies or runs user plugins/configuration. Updates are reviewed through
the lockfile and dependency checks. YouTube changes can still require a future
importer update. See the upstream [release](https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19)
and [runtime requirements](https://github.com/yt-dlp/yt-dlp/wiki/EJS).

## Process and storage boundary

The daemon stores import identity, canonical video identity, permission,
attempt and outcome before starting work. Each attempt has a private directory.
The helper writes a fixed `source.mkv`, emits a bounded JSONL status protocol and
never writes SQLite. The daemon validates the final file and runs the existing
local source inspector before recording successful source registration. Failed
or cancelled work cannot win a late completion race.

Both observed download bytes and temporary directory usage are bounded; merging
may temporarily retain both streams and their combined output. A disk monitor
keeps a 1 GiB free-space reserve and checks during merging as well as downloading.
Monitoring is periodic, so it is not an atomic filesystem quota. Attempts also
have a two-hour transport deadline. Cancellation kills the attempt's entire
process group; a parent watcher stops the helper group after daemon death.
An advisory `.import.lock` allows recovery cleanup to avoid live attempts.

Imported originals are source data, not expendable analysis cache. Cleanup must
only target ClipMill's managed project directories, never a user's ordinary local
files. Source registration retains the same mutation detection and validation
as local import.

The Local Lock session indicator counts admitted imports as network operations.
It does not claim that YouTube acquisition is offline or that it is protected by
an OS firewall. After acquisition, local analysis does not send footage to
YouTube or enable optional cloud reasoning.

## Verification

The Python suite exercises URL and metadata rejection, missing final files,
aggregate unknown-size download limits, temporary disk accounting, runtime
readiness, incomplete-fragment refusal configuration and safe error messages.
The Rust transport suite exercises completion validation, output path rejection,
readiness failures and process-group cancellation. These run without YouTube
access in CI. Live provider compatibility and native UI verification are recorded
separately in the PR; fake transports are not live download evidence.

On 2026-09-19, the native desktop imported the user-selected video
`Zw5DkDfKw_c` at 360p: 640×360, 24 fps, 21:30.901, 40,174,549 bytes. A complete
FFmpeg video/audio decode passed. Relaunching the desktop restored that exact
completed import, its quality choice and inspected local source. Native
cancel/retry was also exercised before switching test videos. These checks
verify acquisition and recovery; they do not measure editorial selection quality.
