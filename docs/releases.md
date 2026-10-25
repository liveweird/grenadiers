# Releases

Release labels and scope live in the [implementation plan](game/implementation-plan.md).
A completed milestone is verification evidence; a release exists only after its tested commit
is tagged and its artifacts are published intentionally.

## Release verification policy

For this single-maintainer project, local checks are as authoritative as equivalent remote
checks. Ship the tested source and reuse its verified artifacts. Remote checks are required only
when they add coverage not exercised locally; a tag pointing at an already verified commit does
not require another complete suite.

Record the version, tested commit, applicable checks and outcomes, runtime environment and image
digest in the release notes. Confirm the tag targets the intended commit and check health,
readiness and the served artifact after deployment. Repeat affected checks when inputs change;
do not require separate full push, PR, master and tag runs or elaborate new proof packages for
each feature. Historical release records retain their original evidence and requirements.

## Version ownership

Use Git tags `vMAJOR.MINOR.PATCH` for the combined server/web product. Cargo workspace package
versions track the next product candidate; keep the local
`grenadiers-core` dependency version aligned when changing them. Private npm package versions
remain `0.0.0`: they are build inputs and are not separately published or product release IDs.
The WebSocket `protocolVersion` is an independent compatibility contract, not a release number.

Each shipped feature PR bumps the product version and records its scope in `CHANGELOG.md`.
Use patch releases within the current milestone for bounded engineering increments.
`0.2.0` is the first playable duel preview; `0.3.0` adds classic match progression. Subsequent
`0.3.x` increments stage reconnect support and activate it at `0.3.9`; `0.4.0` remains the resilient playtest milestone. Keep PRs focused on one behavior and its tests. Merge passing,
independently reviewed candidates, then publish the corresponding prerelease tag.
The repository gate checks that both local Cargo packages and their dependency agree.

## Candidate checklist

1. Update package metadata, lockfiles, contracts, and affected documentation intentionally.
2. Run the complete [release checklist](game/implementation-plan.md#release-checklist), including
   coverage, dependency audits, the image scan, real browser scenarios, and Compose/Kubernetes
   runtime checks. Record commands, results, image identity, and limitations in
   [verification.md](verification.md).
3. Obtain independent review of substantive changes. Commit a focused candidate and rerun
   affected checks if the reviewed source changes.
4. When publishing is authorized, tag that tested commit and promote the exact tested image;
   avoid rebuilding an unverified replacement under the release tag. Include supported rules,
   preview limits, and upgrade notes in the release notes.

## Current prerelease distribution

Current releases publish source tags and GitHub prerelease notes. Accepted image IDs in the
verification record identify locally tested artifacts; no production registry or hosted service
has been configured. A source prerelease does not claim a production deployment.

Evidence may name a clean candidate commit from before rebase or squash. Before merging,
compare its Docker build inputs with the final PR head, require every configured push/PR check to pass,
and verify that the merged tree equals that head. Use the corresponding release tag for the
published source. Retain the accepted image when exact artifact reuse is required; a later
rebuild can receive different operating-system security updates and needs its own acceptance.

## Upgrade and rollback

The initial deployment uses one replica and `Recreate`. In-memory matches cannot survive
restart or rollback. Plan upgrades around active matches; a graceful stop publishes the final
interrupted state but cannot preserve process-local matches or credentials.
Keep the previous immutable image reference for rollback; database migration compatibility
must be designed when durable application data is introduced.

## v0.3.58 — 2026-10-25

[v0.3.58](https://github.com/liveweird/grenadiers/releases/tag/v0.3.58) bounds HTTP serving and
application drain with one five-second deadline measured from the shutdown signal. Incomplete
HTTP requests cannot postpone the game/session drain budget. Graceful final state, socket
closure and startup/I/O errors are preserved. On expiry, main returns and runtime teardown
releases remaining asynchronous I/O; this is an asynchronous budget, not a hard real-time bound.
The release also adopts the merged Napoleonic sheet and refreshed gameplay captures without
changing manifest slots, anchors, gameplay or texture budgets.

Application inputs match tested source `e758ad249a066854c35d200cf8e7ce866d8bda74`; the release
source adds documentation. Localhost:8085 serves accepted image
`sha256:cc8f5d0308b8e1d6b774f20dfcee20fdb49691c080e95ac95624d6c2bd3ce0af`.
Independent code, timing and artwork reviews are clear. Nineteen focused shutdown tests pass;
a deliberate nine-second mutant fails all three deadline cases. A real Linux container with
incomplete headers held until process exit terminates successfully in 5.127s with one deadline
warning. A graceful room/diagnostic probe exits in 0.115s, publishes `serverStopped` and closes
member, unseated and diagnostic sockets with code 1001. No frames or credentials are recorded.

Fresh source-bound Rust acceptance completes all 4,194,304 replay masks in debug, release and
official coverage in 433.085s, with zero retries. Each ordinary profile passes 930 core and
293 server tests plus two doctests; four existing performance tests remain ignored and the
heavy case runs separately across 32 complete partitions. Coverage is 17,278/18,480 lines
(93.50%) at the unchanged 90% floor. Rustfmt, Clippy, repository checks and the
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40228900957) pass.
The first runner-fixture setup failed from an inherited target-directory override; the unchanged
fixture suite passed with its isolated environment before the complete runner passed.

The fresh web gate passes 6,791 tests / 88 files with 90.95/89.49/89.80/93.32% coverage.
JavaScript/CSS gzip sizes are 336,430/36,292 bytes within their budgets; contract, standalone
validator, lint and unused-code checks pass. Dependency inputs are unchanged from v0.3.57,
whose fresh Cargo-deny and both zero-finding npm audits are reused. A fresh pinned Trivy scan
of this exact image reports zero fixable HIGH/CRITICAL findings under the existing policy.

Exact-image Chromium acceptance passes nine Compose cases (eight default plus Oil Barrel)
and eight Kubernetes cases. Separate diagnostics under the enforced browser policy pass
14 artist, terrain, atlas and isolated renderer-lifecycle cases. These are representative
checks, not a rerun of every configured browser suite. Post-promotion runtime/security,
health/readiness, CSP/cache, exact HTML/JavaScript/atlas equality and one additional browser
smoke pass. Temporary acceptance resources are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.58.yaml`; `rollback-v0.3.57.yaml` retains the previous
image. No migration is needed; restarts still end process-local matches. Keep external stop grace.

<a id="v0357--2026-10-25"></a>
The [v0.3.57 record](history/releases.md#v0357--2026-10-25) is preserved in release history.

<a id="v0356--2026-10-25"></a>
The [v0.3.56 record](history/releases.md#v0356--2026-10-25) is preserved in release history.

<a id="v0355--2026-10-25"></a>
The [v0.3.55 record](history/releases.md#v0355--2026-10-25) is preserved in release history.

<a id="v0354--2026-10-25"></a>
The [v0.3.54 record](history/releases.md#v0354--2026-10-25) is preserved in release history.

<a id="v0353--2026-10-25"></a>
The [v0.3.53 record](history/releases.md#v0353--2026-10-25) is preserved in release history.

<a id="v0352--2026-10-25"></a>
The [v0.3.52 record](history/releases.md#v0352--2026-10-25) is preserved in release history.

<a id="v0351--2026-10-25"></a>
The [v0.3.51 record](history/releases.md#v0351--2026-10-25), including corrected Docker/Compose
runtime attribution, is preserved in release history.

<a id="v0350--2026-10-25"></a>
The [v0.3.50 record](history/releases.md#v0350--2026-10-25) is preserved in release history.

<a id="v0349-published-prerelease--2026-10-25"></a>
[v0.3.49 published prerelease](history/releases.md#v0349-published-prerelease--2026-10-25)
is preserved in the release history.

<a id="v0342-published-prerelease--2026-10-23"></a>
Older release records, including [v0.3.42 published prerelease — 2026-10-23](history/releases.md#v0342-published-prerelease--2026-10-23),
are preserved in the [release history](history/releases.md).
