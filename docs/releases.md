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

## v0.3.55 — 2026-10-25

[v0.3.55](https://github.com/liveweird/grenadiers/releases/tag/v0.3.55) shares native responsive
browser-context setup across 26 gameplay scenarios. Two independent contexts still start
concurrently and return in desktop/mobile order. Viewports, mobile reduced motion, pickup's
390px override, privacy observations, assertions and caller-owned cleanup remain unchanged.
Application inputs match `8b7bacb82de085988e1592a280c0119ba3a41bfa`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:a053abd96387ebe50f85b68aeecffcaa48160ccacb70a2f26f7808c1d7cee6d2`.

Independent review and fresh E2E types, lint, unused-code, repository and scenario checks pass.
The scenario gate includes the new exact-options, concurrency, reverse-resolution ordering and
native-identity regression. All 46 spec title inventories are unchanged; every migrated spec
is byte-identical outside its new import and equivalent setup block. Seventy-four mapped
scenarios remain registered; this static count is not a claim that all ran natively.

The exact image passes twenty Chromium cases without retries: eight default cases through
Compose, eight through Kubernetes, and four seeded Compose cases covering Landmine, 390px
pickups, recipient-private Cloak/reconnect/expiry and Invisible Bombs' live pixel/privacy window.
The pinned image scan reports zero fixable HIGH/CRITICAL findings. Post-promotion checks pass
health/readiness, runtime security, CSP/cache headers, exact served HTML/JavaScript/atlas bytes
and one fresh real-browser diagnostic/gameplay smoke. Claude's graphics are preserved.

Production Rust, API, frontend and asset sources are unchanged from v0.3.54; Cargo changes only
local package versions. Reuse v0.3.54 ordinary Rust/Linux results and v0.3.53 frontend/security/
renderer evidence. Exhaustive replay/coverage remains attributed to measured `055d1d2`
(all 4,194,304 masks, 93.15% line coverage at the 90% floor); none is claimed as freshly rerun
for test setup reuse. The production image is rebuilt and accepted independently.

Temporary acceptance services are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.55.yaml`, with `rollback-v0.3.54.yaml` retaining the
previous accepted image. No migration is needed; restarting still ends process-local matches.
Configuration factories and broader fixture sharing remain separate work.

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
