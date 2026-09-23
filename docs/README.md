# Documentation

Start with the [project README](../README.md) to install and run ClipMill, or
[Contributing](../CONTRIBUTING.md) to work on the code.

## Features and operation

- [YouTube import](youtube-import.md): download a permitted source video.
- [Local Lock](local-lock.md): local processing, optional cloud consent, and the
  network indicator's limits.
- [Inspector](inspector.md): inspect candidates and record editorial decisions.
- [Reframing](reframing.md): face tracking, crop paths, and layout constraints.
- [Captions](captions.md): cue construction, word corrections, and sidecars.
- [Preview and render parity](preview-parity.md): timeline and presentation rules.
- [Export](export.md): validation, output naming, and delivered files.
- [Workers](../workers/README.md): setup, authentication, and model families.

## Architecture and development

- [Desktop shell](shell.md): host boundary, routing, media access, and recovery.
- [Desktop workspace](frontend-workspace.md): layout conventions and browser preview.
- [Daemon](daemon.md): state ownership and job scheduling.
- [Artifact store](artifact-store.md): content addressing and publication.
- [Evaluation](evaluation.md): fixtures, signed evidence, and quality checks.
- [Threat model](threat-model.md): security boundaries and known limitations.
- [Decision record](decisions.md): architectural choices and their rationale.

## Planning and historical verification

The `phase*`, `milestone*`, and dated audit documents record development plans and
checks at particular revisions. Their status statements are historical, not a
list of current features or prerequisites for contributing. CI recipe names that
refer to these plans remain stable because scripts depend on them.

The [YouTube channel setup guide](youtube-channel-setup.md) describes planned
publishing support, which is separate from the source import available on `main`.
