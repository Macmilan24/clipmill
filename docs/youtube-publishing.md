# YouTube channel connection and publishing

Publishing is separate from source import and local analysis. It requires the
user's own Google Desktop OAuth client and browser consent; follow the
[setup guide](youtube-channel-setup.md). The initial credential-store integration
uses macOS Keychain. Other platforms report publishing unavailable rather than
writing tokens to files or pretending to connect.

## User flow

In Settings, select the Desktop client JSON through the native file picker, then
connect in the system browser. ClipMill displays the actual YouTube channel
returned by the API. Configuration, callback codes, tokens and resumable session
URLs never enter the renderer, project archives, command arguments or logs.

Finish a normal export first. The publishing panel binds to its exact saved
document revision and immutable rendered artifact. Review and edit the suggested
title, description and tags, confirm the audience and synthetic-media disclosure,
and approve a private upload to the displayed channel. Suggestions are grounded
in the clip's stored evidence; they are editable and never silently replace the
user's edits.

Upload progress counts bytes acknowledged by YouTube. Pause and resume retain
the same operation. After transfer, review the private video in YouTube Studio;
processing can continue after all bytes have arrived. A separate explicit
Publish action makes that same video public. A successful private upload does
not establish that Google permits public publishing for this API project.

## Recovery and identity

The daemon's SQLite actor owns the operation ledger. Publishing is not an
analysis worker task and cannot complete from an artifact-cache hit. Admission
resolves a succeeded export to a verified render artifact, digest, byte length,
revision, metadata and actual channel. A changed file in the export folder cannot
change the bytes uploaded. Recoverable operations retain their artifact roots.

The provider's resumable session is protected in Keychain before any media is
sent. On interruption, query that session and use the server's acknowledged
offset. Persist the intent to send the final range before sending it. If the
final response is lost and the session later expires, remote completion may be
unknown: do not automatically start another upload. Show the uncertainty and
direct the user to YouTube Studio. Local deduplication cannot prove that a remote
video does not exist.

If an expired session never reached its final range, an explicit Resume can
restart the incomplete transfer. The ledger records this reset before removing
the old session. This recovery is refused whenever the final range may have
been sent.

Pause is cooperative: an in-flight response may arrive before the operation
stops, and a successful remote receipt must still be saved. A reconnect is bound
to the verified channel, not merely the Google account's display name. Deleting a
local project must not erase knowledge of a remotely created video.

Settings keeps upload history across projects, including receipts from removed
projects. Re-exporting the same saved clip reopens its existing operation for
that channel. History preserves visibility changes made in YouTube Studio,
including unlisted videos, without inventing a local Publish request.

Publish operates on the saved video ID. It reads the current resource first,
checks the channel and processing state, and changes privacy while preserving
mutable audience, license and embedding settings. The resource ETag prevents a
concurrent change in Studio from being overwritten. A lost reply is reconciled
against that same video, never by creating another insert.

## Network and credentials

The desktop OAuth flow uses a bound loopback listener, random state and PKCE
S256, with bounded callback parsing and expiry. Desktop configuration cannot
override Google's fixed endpoints. The requested `youtube.force-ssl` scope
supports both private insertion and the separately requested visibility update;
`youtube.upload` alone does not authorize the latter. Granted scopes are checked
on initial authorization and refresh. Tokens are kept with their originating
client configuration, so selecting a new client does not reinterpret an old
connection. Disconnect is serialized with refresh to prevent credential revival.

The HTTPS client disables redirects and ambient proxies. A resumable Location
must use the exact approved Google host, port and upload path before it receives
credentials or media. Provider bodies and URLs are not copied into user-facing
errors. The Local Lock indicator counts explicit publishing network operations;
it remains a session activity indicator, not an OS firewall.

The transport is based on Google's [desktop OAuth flow](https://developers.google.com/identity/protocols/oauth2/native-app),
[resumable upload protocol](https://developers.google.com/youtube/v3/guides/using_resumable_upload_protocol),
[insertion requirements](https://developers.google.com/youtube/v3/docs/videos/insert)
and [update semantics](https://developers.google.com/youtube/v3/docs/videos/update).

## Verification boundary

Deterministic protocol tests use synthetic credentials and local fixtures. They
exercise private metadata, request ranges, lost final responses, refused hosts
and redirects, callback state, OAuth refusals, response bounds, channel identity,
ETag conflicts and idempotent Publish reconciliation. Daemon and UI tests cover
the durable coordinator and presentation. These do not substitute for a real
account authorization or an explicitly approved private upload. Record live
account verification separately; never infer it from protocol fixtures.

The September 19, 2026 development verification passed 875 workspace Rust tests
(38 environment-dependent tests ignored), 522 frontend tests, strict workspace
Clippy, type checking, lint and dependency policy. Protocol generation reproduced
all 115 generated files. The macOS app and daemon were rebuilt and launched;
the real store migrated from schema 13 to 14 with all 19 saved document hashes
unchanged, including revision 24 of the open edit. Native Settings showed the
unconfigured channel, disabled Connect action, empty history and a working,
cancellable client-file picker. The previously imported 360p test source remained
available. Real Google sign-in and private uploading remain unverified until the
user supplies their Desktop client and completes browser consent.
