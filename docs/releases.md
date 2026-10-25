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

## v0.3.57 — 2026-10-25

[v0.3.57](https://github.com/liveweird/grenadiers/releases/tag/v0.3.57) gives shared Rust item/
effect DTOs, common item projections and bounded-number input neutral protocol ownership.
V7–V11 retain their public names and shared type identity. Reserved bits, activation subsets,
validation stages/errors, generation order, schema JSON and replay identities are unchanged.
Four independent regressions pin identity, retained schemas, serialization and numeric boundaries.
Application inputs match `a94aaddea8211e262c1d0acf3cf3103db797db05`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:170e5d071feeabf6d62c55dcb1dce6b6191d7bcdbeccee511e1c9a1c8a72b9bf`.

Independent review is clear; moved bodies and mappings match baseline after path relocation.
Fresh source-bound Rust acceptance passes all 4,194,304 replay masks in debug, release and
reported coverage in 8m06s, without retries. Each ordinary profile passes 930 core and 287
server tests plus two doctests; four existing performance tests remain ignored, and the heavy
case runs separately across 32 complete partitions. Official line coverage is 17,248/18,445
(93.51%) at the unchanged 90% floor, with helper exclusions intact. Clippy/rustfmt, repository
checks and the [distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40222285097) pass.

The fresh web gate passes 6,791 tests / 88 files with 90.95/89.49/89.80/93.32% coverage.
JavaScript/CSS gzip sizes remain 336,437/36,292 bytes within their budgets. All retained
contract/standalone drift checks pass. Fresh Cargo-deny, both zero-finding npm audits and the
pinned exact-image scan pass; the latter reports zero fixable HIGH/CRITICAL findings.

The exact image passes twenty native Chromium cases without retries: eight default cases
through Compose, eight through Kubernetes, and seeded Landmine, Heart, Cloak/reconnect/expiry
and Oil Barrel scenarios through Compose. Post-promotion checks pass runtime/image/security,
health/readiness, enforced CSP/cache headers, exact served HTML/JavaScript/atlas equality and
one additional native diagnostic/gameplay smoke. Claude's redraw is unchanged. Temporary
acceptance services and port-forwards are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.57.yaml`; `rollback-v0.3.56.yaml` retains the prior image.
No migration is needed; restarts still end process-local matches.

A separate quiet probe of the unchanged shutdown path in v0.3.56 confirms a follow-up issue:
an incomplete HTTP request keeps the process running beyond seven seconds after SIGTERM,
before the application drain timer is reached. This release does not fix that ordering;
bounding total shutdown is the next isolated fix. Keep the configured external stop grace.

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
