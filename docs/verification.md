# Scaffold verification

Rechecked 2026-10-16. This records observed results, not a claim that the full stack is validated.

## Passed

- Both npm packages installed from their own repository lockfiles with actual offline `npm ci`
  under Node 24.21.0, using the available npm cache.
- `scripts/gates.sh repository`: toolchain agreement, exact npm dependency pins, lock roots,
  and source-size limits.
- `scripts/gates.sh web`: Spectral, HTTP/realtime generated-type drift, ESLint, Knip,
  shuffled coverage tests, and production build.
- Frontend: **4 test files, 10 tests**. Coverage: **95.34% statements, 90% branches,
  100% functions, 96.05% lines**; every configured floor is 85%.
- `scripts/gates.sh e2e`: lint, dead-code checks, TypeScript, and scenario/spec parity
  for the one documented two-browser smoke scenario.
- `bash -n scripts/gates.sh` and `docker compose config --quiet`.
- `kubectl kustomize k8s` renders the Namespace, Service, and single-replica Deployment.
  This validates composition, not Kubernetes schemas or live execution.
- `cargo +stable fmt --all -- --check` and
  `cargo +stable metadata --offline --no-deps --format-version 1`.
  The installed stable compiler reports Rust 1.99.1, matching the intended toolchain pin;
  installing/selecting a separately named 1.99.1 toolchain requires unavailable network access.

## Current overall result

`scripts/gates.sh all` **fails** after repository checks because `Cargo.lock` is missing.
The coverage and security targets fail at the same prerequisite. No claim that all tests or
all guardrails pass is warranted. The latest direct offline Rust build also exits 101 at
missing `serde`, before compilation.

## Blocked or not executed

- crates.io DNS is unavailable and the Cargo registry cache is empty. Offline tests stop at
  unresolved `serde`; **Rust compilation, Clippy, tests, and coverage are unverified**.
- No `Cargo.lock` was fabricated. `cargo generate-lockfile` must run with registry access;
  review and commit the result before the normal locked gates or Docker build can succeed.
  CI intentionally fails its initial lockfile check until that happens.
- The original `AGENTS.md` is outdated (empty-workspace instructions); it was preserved under
  the initial explicit do-not-modify instruction. Current conventions live in `CONTRIBUTING.md`.
- The OrbStack Kubernetes API at `127.0.0.1:26443` rejects access with `operation not permitted`.
  Neither a live apply nor rollout/probe/WebSocket validation ran. `scripts/gates.sh k8s`
  additionally requires unavailable kubeconform 0.8.0 and access to pinned schema resources;
  its strict schema check is configured in CI but not verified locally.
- Docker daemon access is denied. Container builds/startup and the real Playwright scenario
  were not executed. Compose syntax/configuration validation does not validate runtime behavior.
- cargo-deny, npm online audits, and the Trivy image scan were not executed in this environment.
- Rust/runtime image digest resolution remains outstanding. Node/PostgreSQL and CI/scanner
  pins were inherited from the local Flow reference, not newly resolved here.

## Complete validation with network and Docker access

```sh
mise install
cargo generate-lockfile
# Review and commit Cargo.lock before running the normal locked build.
npm --prefix web ci
npm --prefix e2e ci
scripts/gates.sh all
cargo install cargo-llvm-cov --version 0.9.1 --locked
rustup component add llvm-tools-preview
scripts/gates.sh coverage
cargo install cargo-deny --version 0.20.2 --locked
scripts/gates.sh security
scripts/gates.sh k8s  # Requires kubectl, kubeconform 0.8.0, and pinned schema access.
docker compose up --build -d
npm --prefix e2e run install:browsers
scripts/gates.sh browser
docker compose down
```

Resolve any first-compilation, measured Rust coverage, dependency-policy, or browser failures
before treating this scaffold as a release baseline. The CI workflow contains the image scan.

The repository has since been initialized on `master`, with `origin` configured as
`git@github.com:liveweird/grenadiers.git`. Use `git log` and `git status` for current
commit and push status; initialization does not resolve the validation limits above.
Local Kubernetes build/apply commands are in [deployment.md](deployment.md).
