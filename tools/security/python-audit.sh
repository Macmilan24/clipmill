#!/usr/bin/env bash
# Audit every locked Python dependency set against the live vulnerability DB.
set -euo pipefail
cd "$(dirname "$0")/../.."

audit_root="$(mktemp -d /tmp/clipmill-python-audit.XXXXXX)"
cleanup() {
  case "$audit_root" in
    /tmp/clipmill-python-audit.*) rm -rf -- "$audit_root" ;;
    *) echo "python-audit: refusing to remove unexpected path $audit_root" >&2 ;;
  esac
}
trap cleanup EXIT INT TERM

projects=()
while IFS= read -r lockfile; do
  projects+=("$(dirname "$lockfile")")
done < <(find . -name uv.lock -not -path './.git/*' -not -path './target/*' | sort)
if [ "${#projects[@]}" -eq 0 ]; then
  echo "python-audit: no uv.lock files found" >&2
  exit 1
fi
for project in "${projects[@]}"; do
  safe_name="${project#./}"
  safe_name="${safe_name//\//-}"
  requirements="$audit_root/$safe_name.txt"
  echo "==> pip-audit $project"
  uv export --frozen --all-groups --no-emit-local --project "$project" \
    --format requirements-txt >"$requirements"
  uvx --from pip-audit==2.10.1 pip-audit \
    --strict --progress-spinner off --requirement "$requirements"
done
echo "python-audit: OK (${#projects[@]} lockfiles)"
