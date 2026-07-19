# Local Lock proof level

Local Lock is the product's zero-egress mode. Its final form combines an
exclusive network broker, OS enforcement, and a project-visible audit.

In Phase 0, CI enters a Linux network namespace with no interfaces, verifies an
egress canary cannot connect, and runs the Rust workspace tests plus W3 cache
and W4 durable-job hard-kill smoke drills plus the W5 pinned media/source smoke
suite offline. The daemon, scheduler, artifact store, and FFprobe sidecar have no
network-capable dependency or request path, and health reports
`local_lock=true`. This is a harness-level proof, not yet the final desktop
enforcement boundary: the broker, model registry, shell, and complete offline
import-to-export scenario land in later phases.

The claim must expand with the product. Any new offline-core process is added
to the namespace test before its workstream can exit.
