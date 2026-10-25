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

## v0.3.56 — 2026-10-25

[v0.3.56](https://github.com/liveweird/grenadiers/releases/tag/v0.3.56) shares the common
Playwright configuration across 24 simple gameplay wrappers. Public config names and commands,
artifact paths, worker/retry limits and base browser/environment settings are unchanged.
Audio keeps its 60-second timeout; the other 23 keep 90 seconds. Trace capture remains disabled.
Specialized configurations and the pickup/matches/reconnect chain remain separate.
Application inputs match `d7849292fed30d728bffcf0c106aff7d4a1cf6f8`; the source tag adds release
documentation. Localhost:8085 serves accepted image
`sha256:5f1e0627b21d18162c6b12e53516a934ea4d370a0e1a285b75bff2d551a6e603`.

Independent review, fresh types/lint/unused-code/scenario/repository checks and the wired
configuration regression pass. Complete before/after exports match in all 96 comparisons
(24 wrappers across four simulated environment/platform profiles), using pinned Playwright
`defineConfig` and device defaults, including undefined/symbol keys. This source evaluation
preserves Linux ANGLE and capture-profile settings; it is not native Linux browser execution.

The exact image passes nineteen Chromium cases without retries: eight default cases through
Compose, eight through Kubernetes, plus controls, natural headed audio and a seeded Landmine
scenario through Compose. The pinned image scan reports zero fixable HIGH/CRITICAL findings.
Post-promotion checks pass image/runtime security, health/readiness, CSP/cache headers, exact
served HTML/JavaScript/atlas bytes and one additional native diagnostic/gameplay smoke.
Claude's redraw is unchanged. Temporary acceptance services are removed.

Production Rust, API, frontend, asset and deployment sources are unchanged; Cargo changes only
local package versions. Reuse ordinary Rust/Linux results from v0.3.54, frontend/dependency/
renderer evidence from v0.3.53 and exhaustive replay/coverage measured at `055d1d2`
(all 4,194,304 masks, 93.15% line coverage at the 90% floor). These results are not claimed as
freshly rerun for configuration reuse. The new production image is built and accepted separately.

Durable control is `.local-checkpoints/runtime/compose-v0.3.56.yaml`; `rollback-v0.3.55.yaml`
retains the previous accepted image. No migration is needed; restarts end process-local matches.
The testing index/archive cleanup is included; its historical labels do not impose current gates.

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
