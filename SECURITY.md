# Security Policy

ClipMill processes untrusted media and model output. The Rust daemon owns durable
state, workers authenticate over a local socket, and the desktop renderer uses
purpose-specific host commands for media, files, and processing operations.

Analysis is local by default. YouTube import and optional cloud editorial
processing introduce explicit network paths. The Local Lock badge reports
application activity; it does not impose an operating-system firewall. See
[Local Lock](docs/local-lock.md) for the current controls and limitations.

The [threat model](docs/threat-model.md) records trust boundaries, verification,
and unresolved risks. Pull requests that change a sensitive boundary must
complete the threat-review checklist; CI derives the required categories from
the diff.

## Reporting a vulnerability

Please report vulnerabilities **privately** via
[GitHub private vulnerability reporting](https://github.com/Macmilan24/clipmill/security/advisories/new).
Do not open a public issue.

You can expect an acknowledgment within 7 days. We follow coordinated disclosure
with a **90-day** window from report to publication, extended by mutual agreement
if a fix needs longer.

## Scope

Areas of particular interest include:

1. **Media parsing:** crafted files, decoder supervision, and container metadata.
2. **IPC and workers:** authentication, socket access, shared-memory validation,
   and artifact ownership.
3. **Network and credentials:** unauthorized cloud processing or downloads,
   credential exposure, and bypasses of consent or budget controls.
4. **Desktop host:** WebView escapes, unsafe file access, and capability bypasses.
5. **Publication and recovery:** path traversal, corrupt artifacts, and incomplete
   exports accepted as successful.

## Supported versions

While ClipMill is pre-release, security fixes target `main`. Once versioned
releases are available, the latest minor release receives security fixes.
