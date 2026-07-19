# Contract versioning

Compatibility rules are boring on purpose (design book, ch. 9):

- **Additive fields are backward-compatible** — a minor change. Optional
  fields, new enum values, new messages/schemas.
- **Removal or reinterpretation requires a major version** — a new
  `.v(N+1)` proto package or schema file. The old version keeps existing
  until nothing produces or consumes it.
- **Workers support the current and the previous minor** protocol version.
- **Unknown enum values are preserved, never silently defaulted.**
- A consumer may decline input it cannot handle, with a machine-readable
  reason — before touching media.

Mechanics:

- Protobuf: `buf breaking --against '.git#branch=main'` enforces wire
  compatibility on every PR. Package paths carry the major version
  (`clipmill.ipc.v1`).
- JSON Schema: file names carry the version
  (`clipmill.artifact.<kind>.v<N>.json`); every instance carries a
  `schema_version` const so artifacts are self-describing.
- Times are rational integers everywhere. A `number`-typed field matching
  `*time|start|end|duration|offset*` fails `tools/schema-lint` (D06).
- Generated code (Rust/Python/TypeScript) is committed; CI regenerates and
  fails on drift, so contracts and code can never disagree on `main`.
- Source-map v1 keeps its W1 shape readable. W5 producers always include the
  additive `mapping` extension; absence identifies legacy evidence and is not
  accepted as newly produced Phase 0 output.
