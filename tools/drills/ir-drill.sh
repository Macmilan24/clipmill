#!/usr/bin/env bash
# W12 Edit IR gate.
#
# Proves the editing spine: every typed command inverts to the same canonical
# bytes, a batch undoes as one transactional step, replaying a durable command
# log reproduces the live document exactly, commands acknowledged over the
# control socket survive a SIGKILLed daemon, and the render snapshot is
# content-addressed without the rationale that explains the edit.
set -euo pipefail
cd "$(dirname "$0")/../.."

ITERATIONS="${1:-1}"
case "$ITERATIONS" in
  ''|*[!0-9]*)
    echo "ir-drill: iterations must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$ITERATIONS" -lt 1 ]; then
  echo "ir-drill: iterations must be at least 1" >&2
  exit 2
fi

echo "==> edit IR invertibility and durability ($ITERATIONS iterations)"
for iteration in $(seq 1 "$ITERATIONS"); do
  echo "ir-drill: iteration $iteration/$ITERATIONS"
  cargo test -p clipmill-edit-ir
  cargo test -p clipmilld --lib -- db::tests::replaying_the_command_log
  cargo test -p clipmilld --lib -- db::tests::applying_against_a_stale_revision
  cargo test -p clipmilld --test edit_documents
done
echo "ir-drill: OK ($ITERATIONS iterations; inverses, batches, log replay, kill survival, snapshots)"
