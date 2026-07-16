# Security Policy

ClipMill's security posture is central to its promise: media parsing is
sandboxed, the UI WebView has no filesystem/shell/network privileges, and
Local Lock means zero egress — enforced by CI, not by promise.

## Reporting a vulnerability

Please report vulnerabilities **privately** via
[GitHub private vulnerability reporting](https://github.com/Macmilan24/clipmill/security/advisories/new).
Do not open a public issue.

You can expect an acknowledgment within 7 days. We follow coordinated
disclosure with a **90-day** window from report to publication, extended by
mutual agreement if a fix legitimately needs longer.

## Scope

Highest-value targets, in order:

1. **Media parsing** — anything reachable through a crafted media file
   (FFmpeg supervision, probe, container metadata handling).
2. **IPC boundary** — the daemon's Unix-domain-socket surface, worker
   protocol, shared-memory descriptor validation.
3. **Local Lock / egress** — any way a Lock-mode process emits network
   traffic.
4. **Desktop shell boundary** — WebView escapes, Tauri capability bypasses.

## Supported versions

Pre-alpha: only `main` is supported. Once releases exist, the latest minor
release receives security fixes.
