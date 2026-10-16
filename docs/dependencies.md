# Dependencies and reproducibility

Toolchains are pinned in `mise.toml`, `.nvmrc`, `rust-toolchain.toml`, and Docker build stages.
`scripts/check-repository.mjs` checks agreement, direct npm versions, lock roots, and source sizes.
CI uses `.nvmrc` and `rust-toolchain.toml` instead of repeating toolchain versions.

Cargo workspace dependencies are centralized in `Cargo.toml`; commit the resolved `Cargo.lock`
and use `--locked` for normal builds, tests, and container packaging. The scaffold was authored
without crates.io access or a Cargo cache, so the initial lock resolution is explicitly outstanding.
Do not handwrite checksums or claim a locked build passed before generating and reviewing it.

Both npm packages commit lockfiles and exact direct versions. Use `npm ci`, not a fresh
unreviewed resolution, in CI. If TypeScript 6 requires openapi-typescript's older peer range
exception, keep that compatibility setting explicit in `web/.npmrc` and verify generation/builds.

For intentional updates: change the manifest/toolchain, resolve with the package manager,
review the dependency graph and checksums, regenerate contracts where necessary, and run the
relevant gates. Cargo checksums and npm integrity hashes detect changed artifacts; they are not
proof of publisher identity. Never accept unexpected checksum changes as routine maintenance.

CI actions use full commit SHAs inherited from Flow's verified workflow. Review action version
updates like code. Dependabot checks Cargo, both npm packages, GitHub Actions, and Docker weekly.
Cargo-deny covers Rust advisories, allowed licenses, and sources; npm audit covers both complete
npm trees, including development tools. Pin utility versions in CI and the matching test docs.

Node and PostgreSQL image digests are inherited from Flow. Rust and runtime images currently
use exact release tags; their immutable digest baseline is deferred until registry access is
available. Do not invent digests. Review and pin them before a release. The composed application
is scanned with Flow's pinned Trivy image after the CI browser smoke test.

Kubernetes validation uses kubeconform 0.8.0 and Kubernetes 1.33.0 schemas from a pinned
schema-repository commit. CI verifies the kubeconform archive checksum before execution;
local schema validation additionally needs those schemas reachable or cached.
