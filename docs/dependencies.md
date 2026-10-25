# Dependencies and reproducibility

Toolchains are pinned in `mise.toml`, `.nvmrc`, `rust-toolchain.toml`, and Docker build stages.
`scripts/check-repository.mjs` checks agreement, direct npm versions, lock roots, and source sizes.
CI uses `.nvmrc` and `rust-toolchain.toml` instead of repeating toolchain versions.

Cargo workspace dependencies are centralized in `Cargo.toml`; commit the resolved `Cargo.lock`
and use `--locked` for normal builds, tests, and container packaging. `scripts/gates.sh` fails
without a non-empty `Cargo.lock`. Do not handwrite checksums; generate and review them.

Both npm packages commit lockfiles and exact direct versions. Use `npm ci`, not a fresh
unreviewed resolution, in CI. The installed `openapi-typescript` 7.13.0 declares a
`typescript ^5.x` peer range while this repository pins TypeScript 6.0.3. The
`legacy-peer-deps=true` setting in `web/.npmrc` applies across the web workspace; remove it
when the upstream peer range supports the pinned compiler, after generation, type, and build
checks pass. The E2E workspace does not inherit this web project setting and needs no
configuration change.

The optional `local-server-parallel` prerequisite uses Python's standard library plus
separately installed `cargo-llvm-cov`/LLVM tools for its regression suite; Python is not an
application-runtime dependency. The supported local Python versions tested are 3.14 and 3.15.

For intentional updates: change the manifest/toolchain, resolve with the package manager,
review the dependency graph and checksums, regenerate contracts where necessary, and run the
relevant gates. Cargo checksums and npm integrity hashes detect changed artifacts; they are not
proof of publisher identity. Never accept unexpected checksum changes as routine maintenance.

CI actions use full commit SHAs inherited from Flow's verified workflow. Review action version
updates like code. Dependabot checks Cargo, both npm packages, GitHub Actions, and Docker weekly.
Cargo-deny covers Rust advisories, allowed licenses, and sources; npm audit covers both complete
npm trees, including development tools. Pin utility versions in CI and the matching test docs.

Dependabot groups routine Cargo patch updates, minor and patch updates separately for `web` and
`e2e`, and minor and patch GitHub Actions updates. These are semantic-version classifications,
not compatibility guarantees: in particular, a `0.x` minor update can still be breaking. Major
updates and anything outside a group remain individual proposals for explicit review. Groups
apply only to version updates, so security updates remain separate, visible pull requests. There
are no ignored dependency versions and no automatic merge policy.

`getrandom = "=0.3.4"` stays outside the Cargo group because it is the server's OS-entropy
boundary for ordinary session tokens, room identifiers and codes, resume capabilities, and
private round seeds. Its output is not itself a replay algorithm: journals capture the actual
round seed. An upgrade still needs individual review of supported targets, the fill API, and
failure handling at each credential and seed call site.

`rand_chacha = "=0.9.0"` is a replay-compatibility pin. Generator version one consumes the exact
`ChaCha20Rng` word stream from a 32-byte seed with stream and word position zero; private replay
records retain that seed, and golden tests pin generated layouts. Upgrade it only with an
explicit generator-version and replay-compatibility decision, not as routine dependency churn.

The Node, Rust, and Debian runtime images in the `Dockerfile` and the PostgreSQL image in
`docker-compose.yaml` are pinned by digest. Dependabot's Docker ecosystem refreshes the
`Dockerfile` images; update the Compose PostgreSQL digest manually. The
composed application is scanned with Flow's pinned Trivy image in the `browser-image` job, before
browser acceptance.

BuildKit keeps locked Cargo registry and Git caches plus a platform-isolated target cache between
container builds. The build touches every workspace crate root before compiling so restored target
entries cannot substitute stale workspace code, then copies the new server binary out of the cache
mount for the runtime image. Keep the touched-root list aligned when adding production Cargo targets.

Docker updates stay individual and are not blanket-ignored by major version. A Node or Rust
builder-image proposal must be coordinated with the repository's matching toolchain pins and
pass `scripts/gates.sh repository`; the image change alone is not a valid toolchain update.

Kubernetes validation uses kubeconform 0.8.0 and Kubernetes 1.33.0 schemas from a pinned
schema-repository commit. CI verifies the kubeconform archive checksum before execution;
local schema validation additionally needs those schemas reachable or cached.
