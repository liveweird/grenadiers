# Releases

Release labels and scope live in the [implementation plan](game/implementation-plan.md).
A completed milestone is verification evidence; a release exists only after its tested commit
is tagged and its artifacts are published intentionally.

## v0.3.41 delivery recovery

Invisible Bombs was merged in PR 91, but post-merge workflow `39870771097` failed
while starting the headed visibility browser. v0.3.41 remains unpublished, and the
local deployment remains v0.3.40. The CI-only observer revision gathers bounded
process and Xvfb diagnostics without changing gameplay or test budgets; a passing
instrumented run does not establish the cause of the intermittent failure.

Roll forward from the unpublished merged commit `2fbc782`, keeping version 0.3.41.
Require fresh push and PR checks on the reviewed observer revision, exact-tree squash
merge, post-merge checks and tag checks before promoting the accepted native image.
Retain native proofs under their original source identity and require independently
reviewed input equality. Failed hosted runs contribute zero accepted cases. See the
[current investigation](verification.md#v0341-post-merge-startup-investigation--2026-10-23).

## v0.3.41 source-bound acceptance checkpoint

The native acceptance source is `eb0b756df43fdd896cf87573a5224e4eb5f659a3`, tree
`098f2a3beb766b4fefd2316b7c7ceaf37275a826`. Fresh source checks, a diagnostic-only headed
Chromium run and the full source-bound native matrix pass. Native root 40405 completed with
consumed exit zero across 53 Compose plus 53 Kubernetes cases, with zero retries or carry.
Independent audit is clear under SHA-256
`b0c72b1ff651257930f0db8df9db3cfc1274bd6882f142336e508e699c41097e`; result review is clear under
`0cff7c9dd3bd4895cfcb426f340eca146b8c9627a55bac04c01111819b9008be`. The evidence is durably
sealed under receipt SHA-256 `10db792e74f84a7f540e463f04fee5ebaae6a94f984bbb6d2a15317956cf99f1`,
and independent seal review is clear under SHA-256
`87666f73ca51ac78e2d726de48f74444059e1aca636864839ab437249f353ec8`.
With those source-bound records complete, the documentation commit and independent review come
next, followed by release-contract review. At this native verification checkpoint, no PR, merge, v0.3.41 tag,
publication or promotion was claimed, and localhost:8085 served the unchanged published v0.3.40
deployment. Subsequent delivery and deployment state is recorded by the release history.

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
