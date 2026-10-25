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

## v0.3.59 — 2026-10-25

[v0.3.59](https://github.com/liveweird/grenadiers/releases/tag/v0.3.59) generates browser item
catalogs from each v7–v11 schema's own `ItemId` enum. Live settings and validation reuse those
tuples; retained validators preserve their narrower activation subsets. V2–v6 generated output,
schema JSON, standalone validators, rules, generation, replay identities and dependencies are
unchanged. An E2E privacy assertion now checks the actual unique secret and redaction markers
rather than rejecting a harmless `/private/tmp` checkout path; sanitizer behavior is unchanged.

Application inputs match `473d1e6e889cfecd5c08178f3b881ca74d775fae`; later source adds only the
E2E test correction and release documentation. Accepted image is
`sha256:946a5f6fbab0c8b573c66badff34157b445cc57cc65798d519bc7fdc6a303fc0`.
Independent catalog review is clear. Fresh full web acceptance passes 6,799 tests / 88 files
in 314.30s; coverage is 90.94/89.49/89.85/93.30% above the unchanged floors. Production gzip
JavaScript/CSS sizes are 336,404/36,292 bytes. All retained generation/standalone drift checks,
API conventions, lint, unused-code, repository and corrected full E2E static checks pass.

Rust behavior is unchanged: reuse v0.3.58's complete 4,194,304-mask debug/release/official-coverage
acceptance (93.50% application lines), plus its shutdown and separate graphics diagnostics.
Dependency inputs are unchanged, so v0.3.57's fresh Cargo-deny and both zero-finding npm audits
remain applicable. The exact new image receives its own scan and runtime acceptance below.

Fresh pinned Trivy 0.74.0 acceptance reports zero fixable HIGH/CRITICAL vulnerabilities.
Native Chromium passes eight default cases through Compose (36.4s) and eight through Kubernetes
(40.3s), with unchanged assertions and no configured retries. The first Kubernetes setup used an
8085 origin allowlist with a browser on 18086; correcting only that isolated environment fixed
session admission. Its original failure log was overwritten; the verifier's command-output
excerpt is retained as a limitation. The initial scan lacked Docker socket access; the corrected
exact-image scan is authoritative. Health/readiness, runtime security and CSP/cache headers pass.

Localhost:8085 now serves that accepted image. Promotion verifies exact HTML/JavaScript/atlas
bytes against the local build and image, runtime security, headers and one additional native
realtime smoke. Temporary acceptance services, namespaces and forwards are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.59.yaml`; `rollback-v0.3.58.yaml` retains the previous
image. No migration or gameplay configuration change is introduced by this release.

<a id="v0358--2026-10-25"></a>
The [v0.3.58 record](history/releases.md#v0358--2026-10-25) is preserved in release history.

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
