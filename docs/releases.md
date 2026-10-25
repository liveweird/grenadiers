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

## v0.3.60 — 2026-10-25

[v0.3.60](https://github.com/liveweird/grenadiers/releases/tag/v0.3.60) enables all 22 configurable
items in newly created live rooms, including curses and Oil. All on/All off edit only the item
draft; numeric settings, editing locks and explicit Apply remain intact. Restore match defaults
enables all available items; Classic disables optional items. Existing rooms, rematches and
historical replay defaults retain their settings. Live replay headers record explicit defaults;
no wire, rules, generator or replay format version changes.

A responsive own-player bar shows upgrade totals, equipment/weapon slots, every effect timer,
all Slow Foot stack timers, freeze and damage grace. Counts come from authoritative snapshots;
an opponent's Stopwatch freeze has no invented countdown. Recent own pickups, including basic
upgrades and Teleporter, are announced once, with reconnect/round/death boundaries respected.
Oil remains a world object. English/Polish text and icons reuse the current sheet. See
[settings](game/settings.md) and [browser presentation](game/browser.md).

Application inputs match `c5b668f9fddf8cf95a582c499192a92f658a0e95`; later source changes only E2E
assertions, artifact ignores and documentation. Accepted image is
`sha256:ab7cf594b833c7d5c9b3c7dec15a34292dc22854affa00a949b3a309b7df1e8b`.
Independent server, web and scenario reviews are clear. Corrections prove the active collector's
Polish 320px bar and expose remaining grace time in its accessible name. Own/rival numeric and
label-geometry assertions are preserved after moving the own bar.

Fresh Rust acceptance at clean source `5e4743d8d868ee86cf7a078aa88af150688d8d1b` has identical
Cargo/core/server inputs. All 4,194,304 replay masks pass in debug, release and official coverage
in 544.996s, with 32 complete partitions per profile and zero retries. Ordinary profiles pass
930 core and 296 server tests plus two doctests; four existing performance cases stay ignored.
Coverage is 17,290/18,492 lines (93.50%) at the 90% floor, with helper exclusions intact. The
initial sandbox denied local listener binds; the unchanged suite passed with socket permission.
Rustfmt, Clippy and the runner fixture checks pass.

The fresh web gate passes 6,812 tests / 91 files in 292.82s. Coverage is
91.10/89.47/90.12/93.47% above the existing floors; gzip JavaScript/CSS are 337,937/36,505 bytes.
API/schema/standalone drift, lint, unused-code, build, repository and full E2E static checks pass.
The pre-review coverage attempt was interrupted for the accessible-name fix; this final run
uses the corrected source. Dependency inputs are unchanged; reuse v0.3.57's fresh Cargo-deny
and both zero-finding npm audits. A fresh pinned Trivy scan of this exact image reports zero
fixable HIGH/CRITICAL findings under the existing policy.

Exact-image native Chromium passes 32 Compose cases: all 22 item scenarios, generated upgrades,
a complete classic match/rematch, and eight default cases. Eight more default cases pass
through Kubernetes (40 total), with zero configured retries. Collector-side Polish 320px Heart
capture, pickup feedback, replacement/consumption, expiry, privacy and own/rival geometry are
covered. The [distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40232475529)
pass. Two temporary Kubernetes helper preflights failed (resource path and omitted empty env
values) before browser execution; corrected helpers pass, and failed logs remain retained.

Localhost:8085 serves the accepted image. Promotion verifies runtime security, health/readiness,
CSP/cache and exact HTML/JavaScript/atlas equality with the image/local build; one additional
native configured-rules smoke verifies fresh defaults and EN/PL bulk controls. Temporary
Compose projects, namespaces and port-forwards are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.60.yaml`; `rollback-v0.3.59.yaml` retains the prior image.
No migration is required; restarts end process-local matches. The accepted CPU design is next;
this release does not implement single-player mode or finish every quality-review follow-up.

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
