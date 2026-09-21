## Summary

Describe the problem and the resulting change.

## Verification

- [ ] Relevant tests and checks pass locally.
- [ ] Generated contracts and the worktree are clean.
- [ ] Documentation and recovery claims match the implementation.

## Threat review

Check every category CI identifies as relevant to this change. A checked item
means the boundary was reviewed against `docs/threat-model.md`, with a
regression test or documented residual risk where appropriate.

- [ ] Hostile input and parsers
- [ ] IPC and worker authentication
- [ ] Filesystem publication and paths
- [ ] Subprocess and sandbox
- [ ] Secrets, logs, and credentials
- [ ] Network policy and egress
- [ ] Dependencies, licenses, and generated code
