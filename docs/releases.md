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

## v0.3.51 — 2026-10-25

[v0.3.51](https://github.com/liveweird/grenadiers/releases/tag/v0.3.51) precompiles the live
browser validators, preserving schemas, complete errors and semantic/privacy checks.
Application inputs match `a7ac65b36592ad8871a1a78f64f4e9adcc24eb36`; the source tag includes
release documentation. Localhost:8085 serves the accepted image
`sha256:cb7be95e9b5c6c291d8c828579b3de5facff9efb466d5a61e4dcdcadd2c7c76f`.

The full web gate passes 6,789 tests / 88 files at `31219c0`, with coverage
90.95/89.49/89.78/93.32% (statements/branches/functions/lines). Later portability corrections
change generated comments only; 13 focused tests, lint, Knip, drift, TypeScript and fresh
production builds pass on the final generator. Exact artifacts match across checkout depths,
invocation directories and pinned Linux. Compressed JavaScript falls from 357,324 to 333,442
bytes; CSS remains 36,292 bytes, within unchanged budgets. Independent review is clear.

Actual Chromium development and production flows validate live diagnostic welcome/pong and
gameplay admission. Sixteen native cases pass without retries: the default eight each in
Compose and Kubernetes on the same image. Kubernetes retains base security, probes and resource
limits. Strict schema checks and the pinned image scan pass, with zero fixable HIGH/CRITICAL
findings under the existing scanner policy. Isolated acceptance services are removed.

Rust source, wire contracts and external dependencies are unchanged from v0.3.50. Reuse its
full exhaustive/coverage acceptance; Linux debug/release units additionally pass at `b366958`,
whose Rust inputs equal the final candidate. Do not treat these reused results as a new full
Rust run. Docker's clean dependency install reports zero npm vulnerabilities.

Promotion health/readiness/index are 200. Served HTML/JavaScript/atlas match the image; the
atlas still matches canonical source. Runtime hardening remains intact. Durable deployment
control is `.local-checkpoints/runtime/compose-v0.3.51.yaml`; v0.3.50 is retained for rollback.
Reload browser tabs after upgrading; matches remain process-local. No database migration is
needed. CSP enforcement and the separate artist redraw remain pending.

<a id="v0350--2026-10-25"></a>
The [v0.3.50 record](history/releases.md#v0350--2026-10-25) is preserved in release history.

<a id="v0349-published-prerelease--2026-10-25"></a>
[v0.3.49 published prerelease](history/releases.md#v0349-published-prerelease--2026-10-25)
is preserved in the release history.

<a id="v0342-published-prerelease--2026-10-23"></a>
Older release records, including [v0.3.42 published prerelease — 2026-10-23](history/releases.md#v0342-published-prerelease--2026-10-23),
are preserved in the [release history](history/releases.md).
