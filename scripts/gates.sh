#!/usr/bin/env bash
# Same fail-fast gates locally and in CI; install dependencies before invoking.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

require_lock() {
  if [ ! -s Cargo.lock ]; then
    echo 'Cargo.lock is missing. Run cargo generate-lockfile on a network-enabled machine, review, and commit it.' >&2
    exit 1
  fi
}
repository() {
  node scripts/check-repository.mjs
}
server() {
  require_lock
  bash -n scripts/measure-game.sh
  node --test scripts/measure-game-artifact.test.mjs
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  cargo build --workspace --locked
}
web() {
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
  npm --prefix e2e run lint
  npm --prefix e2e run knip
  npm --prefix e2e run typecheck
  npm --prefix e2e run check:scenarios
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
  cargo llvm-cov --workspace --locked --fail-under-lines 80 --lcov --output-path target/coverage.lcov
}

case "${1:-all}" in
  all) repository; server; web; e2e ;;
  repository) repository ;;
  server) repository; server ;;
  web) repository; web ;;
  e2e) repository; e2e ;;
  browser) npm --prefix e2e test ;;
  k8s) k8s ;;
  security) security ;;
  coverage) coverage ;;
  *) echo 'usage: scripts/gates.sh [all|repository|server|web|e2e|browser|k8s|security|coverage]' >&2; exit 2 ;;
esac
