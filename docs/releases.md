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

## v0.3.61 — 2026-10-25

[v0.3.61](https://github.com/liveweird/grenadiers/releases/tag/v0.3.61) centralizes production
startup in a typed configuration. Invalid address/origin settings fail with value-free errors;
OS-native paths, proxy → seed → lazy database validation, existing AppState constructors and
invalid-log fallback remain supported. An explicitly empty origin list still denies browser
access and now emits one warning. Docker logging matches Compose/Kubernetes; the
[defaults table](security.md#startup-configuration) distinguishes intentional development and
container values. No gameplay, protocol, replay or dependency changes are introduced.

Tested application source is `29093f3a50297f97cba0033a0f34b2244872c98b`; publication adds only
documentation. Localhost:8085 serves accepted image
`sha256:a22899d0d0e0c45063bf658c089ce55428ac02f37d6417a4270705cc02c42696`.
Independent review, 12 focused startup cases and nine origin cases pass. Fresh source-bound Rust
acceptance completes every 4,194,304 mask in debug, release and official coverage in 499.886s,
with zero retries. Ordinary profiles pass 930 core / 304 server tests and two doctests; four
existing performance cases remain ignored. Coverage is 17,388/18,598 lines (93.49%) at the 90%
floor. Rustfmt, Clippy, runner fixtures, repository checks and
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40234533129) pass.

The exact image passes eight native browser cases through Compose and eight through Kubernetes.
Five actual-image startup probes verify typed/redacted address, database, seed and proxy errors,
plus healthy startup/log fallback, one empty-origin warning and browser-session denial.
A fresh pinned Trivy scan reports zero fixable HIGH/CRITICAL findings. Web/contracts/dependency
inputs are unchanged; reuse v0.3.60 web acceptance and v0.3.57 Cargo/npm audits. Served HTML,
JavaScript and atlas bytes match the accepted v0.3.60 artifacts exactly.

Initial temporary checks needed corrections: Compose override setup, browser sandbox permission,
the absent host Trivy binary (use the pinned container), and a helper expecting API cache policy
for HTML. These were setup failures; final checks use documented policies and unchanged product
assertions. Failed outputs or tool excerpts are retained; no hidden retries were added.

Promotion passes image/runtime security, health/readiness, CSP/cache/artifact equality and one
additional native room-admission smoke. Temporary acceptance resources are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.61.yaml`; `rollback-v0.3.60.yaml` retains the prior image.
Restarts still end process-local matches. The review status now separates shipped fixes from
remaining agreed work; CPU implementation waits for that work to finish.

<a id="v0360--2026-10-25"></a>
The [v0.3.60 record](history/releases.md#v0360--2026-10-25) is preserved in release history.

<a id="v0359--2026-10-25"></a>
The [v0.3.59 record](history/releases.md#v0359--2026-10-25) is preserved in release history.

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
