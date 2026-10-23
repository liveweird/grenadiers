# Releases

Release labels and scope live in the [implementation plan](game/implementation-plan.md).
A completed milestone is verification evidence; a release exists only after its tested commit
is tagged and its artifacts are published intentionally.

## v0.3.41 published prerelease — 2026-10-23

Commit `f5c4adf66f8af9a5ef66b48f9230d17aea14f1d5`, tree
`a274fb2fe828f3a18a624614f1addf5bf5df4d99`, is published as prerelease tag `v0.3.41`.
Post-merge run `39879618476` and distinct tag run `39881306987` each pass all nine jobs,
53 browser cases and 65,536 settings masks. The published source retains the accepted application,
E2E and native inputs, while the CI-only browser readiness and observer changes retain their own
reviewed identity. The sealed native image
`sha256:d063462c3130e3c8653734da72e8dfd6785b0c036e9579d9046cee884ccd9f82` is promoted
locally with consumed exit zero. Sixteen of 22 requested item types are published; Teleporter is
the seventeenth source candidate in v0.3.42, whose fresh acceptance and delivery remain pending.

## v0.3.41 browser file readiness checkpoint — 2026-10-23

The observability-only PR 92 was closed after its required PR workflow failed
while starting Chromium; its parallel successful push run supplies no acceptance
for a later source. A bounded pre-read of the installed browser files targets the
observed kernel page waits. Additional fault, I/O and pressure counters preserve
useful evidence without asserting an exact file or storage cause.

At this historical checkpoint v0.3.41 was unpublished and localhost:8085 served v0.3.40.
The application, E2E and native inputs retain H's accepted bytes; the native
106-case execution remains labeled H. Require independently reviewed input
identity and fresh push, PR, post-merge and tag checks for the meaningful CI
revision before promoting H's sealed image. See the [verification record](verification.md).

## v0.3.41 observability checkpoint — 2026-10-23

Invisible Bombs was merged in PR 91, but post-merge workflow `39870771097` failed
while starting the headed visibility browser. At that recovery checkpoint, v0.3.41
was unpublished and the local deployment served v0.3.40. The CI-only observer revision gathers bounded
process and Xvfb diagnostics without changing gameplay or test budgets; a passing
instrumented run does not establish the cause of the intermittent failure.

The recovery rolled forward from merged commit `2fbc782` while keeping version 0.3.41.
Native proofs retain their original source identity under independently reviewed input equality.
Failed hosted runs contribute zero accepted cases. See the
[historical investigation](verification.md#v0341-post-merge-startup-investigation--2026-10-23).

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
