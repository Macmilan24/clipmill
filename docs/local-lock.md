# Local Lock: current guarantees and proof level

The default analysis route processes media and model inference locally. The
standard worker launcher starts a local editorial process serving propose,
review, and visual checks. It does not register cloud capabilities or import the
cloud adapter. Model acquisition is a separate, explicitly invoked operation.

Cloud editorial processing is optional. `./tools/run-workers.sh --cloud-editorial`
(or `CLIPMILL_EDITORIAL_CLOUD=1`) starts a separate cloud worker with its own
identity. On its first use, enroll that identity with
`./tools/run-workers.sh --cloud-editorial --enrol-only` before starting the
daemon, which reads its trust store at startup. Starting that worker is not
permission to process an analysis: the
analysis request must still opt in, choose the supported provider/model, and
supply a bounded budget. No automatic local-to-cloud fallback exists. Cloud
propose/review send transcript context; visual checks and source frames remain
local.

## What the badge measures

`engaged=true` means no network-allowed task has started in the current daemon
session. The registry can contain optional cloud recipes while the badge is
engaged. Once a cloud task starts, the badge remains disengaged until the daemon
restarts. The separately displayed registry count shows how many stages can use
the network.

The IPC field `egress_attempts` is a historical name. It counts network-allowed
task starts, including a task satisfied from cache. It is not a packet counter,
a byte meter, or proof that a provider request completed. The UI labels it
“Cloud tasks started this session.” Restart resets this session counter; durable
job history and the cloud budget ledger are separate records.

## Enforcement in the application

- The recipe registry is the authority for every task kind's network policy.
  A plan with a mismatched policy is rejected. Lease admission checks the same
  registry through bound SQL parameters, including for older persisted tasks.
- A local worker advertises only local capabilities and rejects cloud work at
  its entry point. A separately enabled cloud worker advertises only cloud
  propose/review and refuses local work. Authenticated worker capabilities and
  resource admission still apply.
- Cloud processing needs explicit per-analysis consent. The worker validates
  provider, model, consent, and budget before sending. The Keychain credential
  is read only by the cloud adapter and is never placed in a task, renderer
  field, command argument, environment variable, or trace.
- A locked per-run ledger reserves the maximum charge before a provider call.
  Known usage refunds the unused reserve; ambiguous failures and interrupted
  calls retain the reservation. Provider output is untrusted and passes the
  same reference, timing, and completeness checks as local output.

These are application controls, not an OS network sandbox. A compromised worker
or another process running as the same OS user is outside this claim. The
current desktop launcher does not impose a firewall on its Python processes.

## Independent offline proof

CI's denied-network gate enters a Linux namespace without network interfaces,
checks that an outbound canary cannot connect, and runs the offline suite.
That establishes that the exercised local paths work without egress. It does
not contain arbitrary desktop processes or exercise a real paid provider call.
New local worker paths must be covered by offline tests; a passing unit test
with a fake provider is not a claim of live provider compatibility.

The intended stronger zero-egress mode requires OS enforcement, an exclusive
network broker, and a durable project-visible audit. Those release controls are
not implemented by this badge. See [the threat model](threat-model.md).
