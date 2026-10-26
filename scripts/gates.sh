#!/usr/bin/env bash
# Same fail-fast gates locally and in CI; install dependencies before invoking.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
source scripts/ci-server-mask-gate-context.sh

require_lock() {
  if [ ! -s Cargo.lock ]; then
    echo 'Cargo.lock is missing. Run cargo generate-lockfile on a network-enabled machine, review, and commit it.' >&2
    exit 1
  fi
}
repository() {
  node scripts/check-repository.mjs
  node --test scripts/source-size.test.mjs
  node --test scripts/ci-workflow-contract.test.mjs
  node --test scripts/check-installed-dependencies.test.mjs
  node --test scripts/prepare-release.test.mjs
}
installed_dependencies() {
  local workspace
  for workspace in "$@"; do
    node scripts/check-installed-dependencies.mjs "$workspace"
  done
}
mask_profile() (
  profile="$1"
  shift
  server_mask_resolve_context "$root"
  source="$server_mask_source"
  unset GRENADIERS_SERVER_REPLAY_ITEM_MASK_SOURCE GRENADIERS_SERVER_REPLAY_ITEM_MASK_PROFILE
  if [ -n "$source" ]; then
    export GRENADIERS_SERVER_REPLAY_ITEM_MASK_SOURCE="$source"
  fi
  export GRENADIERS_SERVER_REPLAY_ITEM_MASK_PROFILE="$profile"
  if [ -z "${GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR:-}" ]; then
    "$@"
    exit
  fi
  mkdir -p "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR"
  log="$GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR/$profile.log"
  "$@" 2>&1 | tee "$log"
  node scripts/ci-server-mask-summary.mjs \
    "$log" "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN_FILE" "$source" "$profile" \
    "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX" \
    "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR/$profile.json"
  rm "$log"
)
server() {
  require_lock
  bash -n scripts/measure-game.sh
  node --test scripts/measure-game-artifact.test.mjs
  node --test scripts/ci-server-mask-contract.test.mjs
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  mask_profile debug cargo test --workspace --locked -- --show-output
  # Production images disable debug assertions; verify behavior in that profile too.
  mask_profile release cargo test --workspace --release --locked -- --show-output
  cargo build --workspace --locked
}
server_parallel() {
  require_lock
  # This suite exercises real cargo-llvm-cov discovery and profile isolation. Keep it
  # with the coverage runner, rather than requiring coverage tooling for every gate.
  python3 -I -B scripts/local-server-parallel.test.py
  python3 -B scripts/local-server-parallel.py \
    --workers "${GRENADIERS_LOCAL_SERVER_WORKERS:-4}"
}
web() {
  installed_dependencies web
  npm --prefix web run lint:api
  npm --prefix web run check:api
  npm --prefix web run check:realtime
  npm --prefix web run check:gameplay
  npm --prefix web run lint
  npm --prefix web run knip
  npm --prefix web run test:coverage -- --sequence.shuffle
  npm --prefix web run build
}
e2e() {
  installed_dependencies e2e
  npm --prefix e2e run lint
  npm --prefix e2e run knip
  npm --prefix e2e run typecheck
  npm --prefix e2e run check:scenarios
}
art_atlas() {
  installed_dependencies web e2e
  node e2e/scripts/art-atlas-tools.check.mjs
  npm --prefix e2e run test:art-atlas
}
k8s() {
  command -v kubectl >/dev/null || { echo 'kubectl is required to render k8s/' >&2; exit 1; }
  K8S_RENDERED_FILE="$(mktemp "${TMPDIR:-/tmp}/grenadiers-k8s.XXXXXX")"
  trap 'rm -f "$K8S_RENDERED_FILE"' EXIT
  trap 'rm -f "$K8S_RENDERED_FILE"; exit 129' HUP
  trap 'rm -f "$K8S_RENDERED_FILE"; exit 130' INT
  trap 'rm -f "$K8S_RENDERED_FILE"; exit 143' TERM

  kubectl kustomize k8s > "$K8S_RENDERED_FILE"
  node web/scripts/check-k8s.mjs "$K8S_RENDERED_FILE"
  command -v kubeconform >/dev/null || { echo 'kubeconform 0.8.0 is required; see docs/deployment.md' >&2; exit 1; }
  kubeconform -strict -summary -kubernetes-version 1.33.0 \
    -schema-location 'https://raw.githubusercontent.com/yannh/kubernetes-json-schema/8df8a883b68a24a104b4a9e43c1288090ae60b3b/{{.NormalizedKubernetesVersion}}-standalone{{.StrictSuffix}}/{{.ResourceKind}}{{.KindSuffix}}.json' \
    < "$K8S_RENDERED_FILE"
}
security() {
  require_lock
  cargo deny --locked check
  npm --prefix web audit --audit-level=high
  npm --prefix e2e audit --audit-level=high
}
coverage() {
  require_lock
  local -a coverage_exclusion_lines=()
  local coverage_exclusion_line coverage_exclusions
  while IFS= read -r coverage_exclusion_line || [ -n "$coverage_exclusion_line" ]; do
    coverage_exclusion_lines+=("$coverage_exclusion_line")
  done < scripts/coverage-exclusions.txt
  if [ "${#coverage_exclusion_lines[@]}" -ne 1 ] || [ -z "${coverage_exclusion_lines[0]}" ]; then
    echo 'scripts/coverage-exclusions.txt must contain exactly one non-empty regex' >&2
    return 1
  fi
  coverage_exclusions="${coverage_exclusion_lines[0]}"
  mkdir -p target
  mask_profile coverage cargo llvm-cov --workspace --locked --fail-under-lines 90 \
    --ignore-filename-regex "$coverage_exclusions" \
    --lcov --output-path target/coverage.lcov -- --show-output
}

case "${1:-all}" in
  all) repository; server; web; e2e ;;
  repository) repository ;;
  server) repository; server ;;
  server-parallel) repository; server_parallel ;;
  web) repository; web ;;
  e2e) repository; e2e ;;
  art-atlas) repository; art_atlas ;;
  atlas-gameplay) repository; installed_dependencies web e2e; npm --prefix e2e run test:atlas-gameplay ;;
  browser) installed_dependencies e2e; npm --prefix e2e test ;;
  k8s) k8s ;;
  security) security ;;
  coverage) coverage ;;
  *) echo 'usage: scripts/gates.sh [all|repository|server|server-parallel|web|e2e|art-atlas|atlas-gameplay|browser|k8s|security|coverage]' >&2; exit 2 ;;
esac
