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

## v0.3.52 — 2026-10-25

[v0.3.52](https://github.com/liveweird/grenadiers/releases/tag/v0.3.52) enforces a compatible
browser content policy using static Pixi implementations and precompiled validators.
Application inputs match `578f581be1593b56ab589c6a8da2dce8a8e4fc8f`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:9b5918458c30fd8dc8e877c8430607f09dd773abdec52f407bc139e5b58ddb85`.

Full source-bound Rust acceptance passes all 4,194,304 masks in debug, release and official
coverage at the 90% floor, with no retries. Application line coverage is 16,394/17,634 (92.97%).
The parallel gate phase completes in 7m23s while the web gate and image build also run. Web
passes 6,790 tests / 88 files with 90.95/89.49/89.78/93.32% coverage and unchanged floors;
compressed JavaScript/CSS are 336,413/36,292 bytes, within budgets. E2E static checks, three wired
relay/config regressions, the pinned image scan and the distinct Linux Rust unit job pass.
The image has zero fixable HIGH/CRITICAL findings under the existing scanner policy.

Before server delivery, enforced policy trials pass fourteen renderer/artist cases and controls,
audio and both reconnect cases with zero observed violations. Final acceptance uses the actual
server image without relay policy injection: eight native cases through Docker Compose, eight
through Kubernetes and six HTTP/WS + HTTPS/WSS CSP cases pass, with no retries. Real page scripts
prove execution/connection/worker blocking; the frame test isolates CSP from XFO and includes
an attachment control. An earlier relay reset crash and no-test selector are retained as failed
attempts, not passes. Independent review is clear.

Promotion health/readiness/index and a fresh native browser diagnostic/gameplay flow pass.
Served HTML/JavaScript/atlas bytes and runtime security identity match the accepted artifact;
CSP is enforced, including on API responses. Temporary acceptance services are removed.
Durable control is `.local-checkpoints/runtime/compose-v0.3.52.yaml`; v0.3.51 is retained for
rollback. Reload existing tabs; matches remain process-local. No migration is needed.

The style policy explicitly permits Mantine inline styles; scripts do not allow inline
execution or string compilation. Browser acceptance covers Chromium, not an untested wider
matrix. Peer-keyed hosting admission and the separate artist redraw remain future releases.

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
