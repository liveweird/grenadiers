# Releases

Planned release scope and work status live in the [backlog](../BACKLOG.md).
The [implementation plan](game/implementation-plan.md) defines the release checklist and future tracks.
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
Use patch releases for bounded engineering increments; historical milestone labels are preserved
in the [plan archive](history/implementation-plan.md). Keep PRs focused on one behavior and its tests. Merge passing,
independently reviewed candidates. Publish stable tags for accepted milestones and label previews
as prereleases.
The repository gate checks that both local Cargo packages and their dependency agree.

### Release preparation helper

Stage the candidate notes under `CHANGELOG.md`'s Unreleased heading. Set `NEXT_VERSION` to the
chosen newer product version and `RELEASE_DATE` to its `YYYY-MM-DD` date, then preview:

```sh
node scripts/prepare-release.mjs --version "$NEXT_VERSION" --date "$RELEASE_DATE"
```

The default is a dry run. Add `--write` only after reviewing the plan; it updates the Cargo
workspace and local-package lock versions, then moves the Unreleased notes into the requested
dated release while retaining an empty Unreleased section. It does not run checks or perform
Git, tag, publish, or network operations. Run the applicable release checks separately.

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

## Current release distribution

Current releases publish source tags and GitHub release notes. Version 1.0.0 is the first stable
game milestone. Accepted image IDs identify locally tested artifacts; no production registry
or hosted service has been configured. A stable source release does not claim a hosted deployment.

Evidence may name a clean candidate commit from before rebase or squash. Before merging,
compare its Docker build inputs with the final PR head, require the applicable local-first checks to pass,
and verify that the merged tree equals that head. Use the corresponding release tag for the
published source. Retain the accepted image when exact artifact reuse is required; a later
rebuild can receive different operating-system security updates and needs its own acceptance.

## Upgrade and rollback

The initial deployment uses one replica and `Recreate`. In-memory matches cannot survive
restart or rollback. Plan upgrades around active matches; a graceful stop publishes the final
interrupted state but cannot preserve process-local matches or credentials.
Keep the previous immutable image reference for rollback; database migration compatibility
must be designed when durable application data is introduced.

### Runtime operating-system refresh

Perform a controlled runtime refresh weekly and when a relevant Debian or runtime advisory
requires it. Keep the reviewed base digest and rebuild only the runtime stage without its cached
layer, following Buildx's
[`--no-cache-filter` contract](https://docs.docker.com/reference/cli/docker/buildx/build/#no-cache-filter):

```sh
docker buildx build --no-cache-filter runtime --load --tag grenadiers:candidate-os-refresh .
```

Record the resulting exact image ID, scan that ID with the pinned release vulnerability scanner,
and run the applicable runtime acceptance against that same ID. Promote only the accepted ID and
retain the previous immutable image reference for rollback. The runtime stage intentionally runs
rolling Debian package updates, so a later build may resolve newer packages and is not expected to
be byte-identical; treat it as a new candidate with new evidence rather than reusing acceptance.

## v1.3.0 — 2026-11-01

Solo and custom rooms now offer Easy, Standard and Hard, with Standard as the default and one
shared tier for every CPU seat. Hard uses range-aware firing positions and gives demonstrated
opponent pressure priority over routine farming while retaining valuable pickups and persistent
goals. A bounded geometric prefilter and legacy pursuit fallback keep distant rivals reachable
when the board clears. See [difficulty](game/single-player.md#difficulty) and
[CPU pressure](game/multiplayer-cpu.md#hard-pressure).

Hard has the same permitted observations, movement/damage rules, two-worker planner and 65,536-work
limit. Its optional pressure analysis is capped at 4,096 and one eighth of remaining work.
Planting still needs full escape, contact and exact-target usefulness proofs; fresh admission
withdraws usefulness when a rival leaves the blast. Pressure evaluates current visible positions.
Easy keeps its navigation pause; Easy/Standard retain CPU27, and Hard selects CPU28.

The [controls capture](screenshots/hard-difficulty.md) records the visible lobby changes.

### Paired upgrade

Gameplay19 is the live contract for every newly created room, including human-only matches.
Reload tabs after upgrading the paired server/web image. Restart ends process-local matches.
Archived wire14–18 and replay readers remain closed to Hard. Runtime publication identity is
explicit and independent of rules, difficulty and navigation. Private replay13 binds gameplay19,
rules10/navigation3 and the exact Easy/Standard27 or Hard28 pairing. Production replay recording
and export remain disabled. See [gameplay19](game/protocol-v19.md).

### Verification

Accepted image source: `3b6944d21267f82aebf5759fbe47e53ffc2f3b02`.
Independent protocol, client, tactics, server/activation and final regression reviews are clear.
Later release-record changes do not alter image build inputs.

- Full local parallel Rust gate passes on the frozen candidate in 696 seconds: 1,698 core and
  626 server tests, plus all 4,194,304 catalogue masks in debug, release and coverage. Four
  workers, 32 partitions and no retries; Clippy, formatting, build and repository checks pass.
  Fresh line coverage is 55,073/59,483 (92.59%), above the unchanged 90% floor.
- Authoritative fixtures execute useful range-aware planting, target destruction and survival
  with lairs enabled/disabled. Selector/guard fixtures cover pickup priority, moving/hidden/blocked
  rivals, commitment retention, low-work fallback and unchanged old policy output/work. A reproduced
  distant-rival Explore regression is fixed and retained with/without a competing Farm goal.
- Current profile episodes cover every board/lair combination with deterministic reruns and
  native two-worker scheduling. Each CPU moves and creates physical owned bombs; completed-job
  accounting and work limits hold. Cancellation/admission refusal remains diagnostic. Random
  episode outcomes are not a universal survival or planting-availability guarantee.
- Distinct Linux ARM64 release checks pass: 332 CPU, ten gameplay19, 15 replay10–13 and three
  current Hard episode tests. Archived readers reject consistently forged Hard provenance.
- Full web gate: 7,426 tests in 169 files. Coverage: 95.57% lines, 92.54% statements,
  91.73% functions and 88.95% branches. Types, lint, Knip, API/contracts/validator drift and
  120 documented browser scenarios pass. JavaScript is 399,742/400,000 gzip bytes;
  CSS is 15,474/20,000. Production loads only the v19 gameplay validator.
- Native renderer regression: 25 passed, 13 project-specific skips; all camera, death,
  billboard, terrain, fire, reduced-motion and recovery checks pass. Rendering logic,
  artwork and audio are unchanged.
- Exact-image Compose authentication/duel: two passed. A three-Hard-CPU all-features match
  moves every CPU, places bombs and renders diagnostics without browser errors. Three browser
  scenarios pass: all 27 board/count/tier combinations, two real countdown/round intros, and
  Polish keyboard Hard creation at 320 pixels. Enabled diagnostics pass both opt-in and ordinary
  URL cases, EN/PL rendering, controls and accessibility.
- Exact-image restricted Kubernetes authentication/duel and disabled-diagnostics checks pass.
  Three Hard CPUs also move and plant with every feature enabled under the pod's CPU/memory
  limits. Owned stacks/namespaces are removed. Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL
  vulnerabilities. All five image web files equal the verified build.

### Operation

Accepted local image:
`sha256:86541d79381f6436e5a9c1e4e0c60cbfde61d9526249dbe80c52757ee3119727`.
The exact accepted image is promoted to localhost:8085; v1.2.5 remains available for rollback.
This is a source/local-runtime release without a hosted service or production registry.
The unchanged artwork remains in the
[v1.2.2 artist ZIP](https://github.com/liveweird/grenadiers/releases/download/v1.2.2/grenadiers-artist-v1.2.2.zip).

<a id="v125--2026-11-01"></a>
The [v1.2.5 record](history/releases.md#v125--2026-11-01) is preserved in release history.

<a id="v124--2026-11-01"></a>
The [v1.2.4 record](history/releases.md#v124--2026-11-01) is preserved in release history.

<a id="v123--2026-11-01"></a>
The [v1.2.3 record](history/releases.md#v123--2026-11-01) is preserved in release history.

<a id="v122--2026-10-31"></a>
The [v1.2.2 record](history/releases.md#v122--2026-10-31) is preserved in release history.

<a id="v121--2026-10-31"></a>
The [v1.2.1 record](history/releases.md#v121--2026-10-31) is preserved in release history.

<a id="v120--2026-10-31"></a>
The [v1.2.0 record](history/releases.md#v120--2026-10-31) is preserved in release history.

<a id="v118--2026-10-31"></a>
The [v1.1.8 record](history/releases.md#v118--2026-10-31) is preserved in release history.

<a id="v117--2026-10-31"></a>
The [v1.1.7 record](history/releases.md#v117--2026-10-31) is preserved in release history.

<a id="v116--2026-10-31"></a>
The [v1.1.6 record](history/releases.md#v116--2026-10-31) is preserved in release history.

<a id="v115--2026-10-31"></a>
The [v1.1.5 record](history/releases.md#v115--2026-10-31) is preserved in release history.

<a id="v114--2026-10-31"></a>
The [v1.1.4 record](history/releases.md#v114--2026-10-31) is preserved in release history.

<a id="v113--2026-10-31"></a>
The [v1.1.3 record](history/releases.md#v113--2026-10-31) is preserved in release history.

<a id="v112--2026-10-31"></a>
The [v1.1.2 record](history/releases.md#v112--2026-10-31) is preserved in release history.

<a id="v111--2026-10-31"></a>
The [v1.1.1 record](history/releases.md#v111--2026-10-31) is preserved in release history.

<a id="v110--2026-10-31"></a>
The [v1.1.0 record](history/releases.md#v110--2026-10-31) is preserved in release history.

<a id="v106--2026-10-31"></a>
The [v1.0.6 record](history/releases.md#v106--2026-10-31) is preserved in release history.

<a id="v105--2026-10-31"></a>
The [v1.0.5 record](history/releases.md#v105--2026-10-31) is preserved in release history.

<a id="v104--2026-10-31"></a>
The [v1.0.4 record](history/releases.md#v104--2026-10-31) is preserved in release history.

<a id="v103--2026-10-31"></a>
The [v1.0.3 record](history/releases.md#v103--2026-10-31) is preserved in release history.

<a id="v102--2026-10-31"></a>
The [v1.0.2 record](history/releases.md#v102--2026-10-31) is preserved in release history.

<a id="v101--2026-10-30"></a>
The [v1.0.1 record](history/releases.md#v101--2026-10-30) is preserved in release history.

<a id="v100--2026-10-30"></a>
The [v1.0.0 record](history/releases.md#v100--2026-10-30) is preserved in release history.

<a id="v03118--2026-10-29"></a>
The [v0.3.118 record](history/releases.md#v03118--2026-10-29) is preserved in release history.

<a id="v03117--2026-10-29"></a>
The [v0.3.117 record](history/releases.md#v03117--2026-10-29) is preserved in release history.

<a id="v03116--2026-10-29"></a>
The [v0.3.116 record](history/releases.md#v03116--2026-10-29) is preserved in release history.

<a id="v03115--2026-10-29"></a>
The [v0.3.115 record](history/releases.md#v03115--2026-10-29) is preserved in release history.

<a id="v03114--2026-10-29"></a>
The [v0.3.114 record](history/releases.md#v03114--2026-10-29) is preserved in release history.

<a id="v03113--2026-10-29"></a>
The [v0.3.113 record](history/releases.md#v03113--2026-10-29) is preserved in release history.

<a id="v03112--2026-10-29"></a>
The [v0.3.112 record](history/releases.md#v03112--2026-10-29) is preserved in release history.

<a id="v03111--2026-10-28"></a>
The [v0.3.111 record](history/releases.md#v03111--2026-10-28) is preserved in release history.

<a id="v03110--2026-10-28"></a>
The [v0.3.110 record](history/releases.md#v03110--2026-10-28) is preserved in release history.

<a id="v03109--2026-10-28"></a>
The [v0.3.109 record](history/releases.md#v03109--2026-10-28) is preserved in release history.

<a id="v03108--2026-10-28"></a>
The [v0.3.108 record](history/releases.md#v03108--2026-10-28) is preserved in release history.

<a id="v03107--2026-10-28"></a>
The [v0.3.107 record](history/releases.md#v03107--2026-10-28) is preserved in release history.

<a id="v03106--2026-10-28"></a>
The [v0.3.106 record](history/releases.md#v03106--2026-10-28) is preserved in release history.

<a id="v03105--2026-10-28"></a>
The [v0.3.105 record](history/releases.md#v03105--2026-10-28) is preserved in release history.

<a id="v03104--2026-10-28"></a>
The [v0.3.104 record](history/releases.md#v03104--2026-10-28) is preserved in release history.

<a id="v03103--2026-10-28"></a>
The [v0.3.103 record](history/releases.md#v03103--2026-10-28) is preserved in release history.

<a id="v03102--2026-10-28"></a>
The [v0.3.102 record](history/releases.md#v03102--2026-10-28) is preserved in release history.

<a id="v03101--2026-10-28"></a>
The [v0.3.101 record](history/releases.md#v03101--2026-10-28) is preserved in release history.

<a id="v03100--2026-10-28"></a>
The [v0.3.100 record](history/releases.md#v03100--2026-10-28) is preserved in release history.

<a id="v0399--2026-10-28"></a>
The [v0.3.99 record](history/releases.md#v0399--2026-10-28) is preserved in release history.

<a id="v0398--2026-10-27"></a>
The [v0.3.98 record](history/releases.md#v0398--2026-10-27) is preserved in release history.

<a id="v0397--2026-10-27"></a>
The [v0.3.97 record](history/releases.md#v0397--2026-10-27) is preserved in release history.

<a id="v0396--2026-10-27"></a>
The [v0.3.96 record](history/releases.md#v0396--2026-10-27) is preserved in release history.

<a id="v0395--2026-10-27"></a>
The [v0.3.95 record](history/releases.md#v0395--2026-10-27) is preserved in release history.

<a id="v0394--2026-10-27"></a>
The [v0.3.94 record](history/releases.md#v0394--2026-10-27) is preserved in release history.

<a id="v0393--2026-10-27"></a>
The [v0.3.93 record](history/releases.md#v0393--2026-10-27) is preserved in release history.

<a id="v0392--2026-10-27"></a>
The [v0.3.92 record](history/releases.md#v0392--2026-10-27) is preserved in release history.

<a id="v0391--2026-10-27"></a>
The [v0.3.91 record](history/releases.md#v0391--2026-10-27) is preserved in release history.

<a id="v0390--2026-10-27"></a>
The [v0.3.90 record](history/releases.md#v0390--2026-10-27) is preserved in release history.

<a id="v0389--2026-10-27"></a>
The [v0.3.89 record](history/releases.md#v0389--2026-10-27) is preserved in release history.

<a id="v0388--2026-10-27"></a>
The [v0.3.88 record](history/releases.md#v0388--2026-10-27) is preserved in release history.

<a id="v0387--2026-10-27"></a>
The [v0.3.87 record](history/releases.md#v0387--2026-10-27) is preserved in release history.

<a id="v0386--2026-10-27"></a>
The [v0.3.86 record](history/releases.md#v0386--2026-10-27) is preserved in release history.

<a id="v0385--2026-10-27"></a>
The [v0.3.85 record](history/releases.md#v0385--2026-10-27) is preserved in release history.

<a id="v0384--2026-10-27"></a>
The [v0.3.84 record](history/releases.md#v0384--2026-10-27) is preserved in release history.

<a id="v0383--2026-10-27"></a>
The [v0.3.83 record](history/releases.md#v0383--2026-10-27) is preserved in release history.

<a id="v0382--2026-10-26"></a>
The [v0.3.82 record](history/releases.md#v0382--2026-10-26) is preserved in release history.

<a id="v0381--2026-10-26"></a>
The [v0.3.81 record](history/releases.md#v0381--2026-10-26) is preserved in release history.

<a id="v0380--2026-10-26"></a>
The [v0.3.80 record](history/releases.md#v0380--2026-10-26) is preserved in release history.

<a id="v0379--2026-10-26"></a>
The [v0.3.79 record](history/releases.md#v0379--2026-10-26) is preserved in release history.

<a id="v0378--2026-10-26"></a>
The [v0.3.78 record](history/releases.md#v0378--2026-10-26) is preserved in release history.

<a id="v0377--2026-10-26"></a>
The [v0.3.77 record](history/releases.md#v0377--2026-10-26) is preserved in release history.

<a id="v0376--2026-10-26"></a>
The [v0.3.76 record](history/releases.md#v0376--2026-10-26) is preserved in release history.

<a id="v0375--2026-10-26"></a>
The [v0.3.75 record](history/releases.md#v0375--2026-10-26) is preserved in release history.

<a id="v0374--2026-10-26"></a>
The [v0.3.74 record](history/releases.md#v0374--2026-10-26) is preserved in release history.

<a id="v0373--2026-10-26"></a>
The [v0.3.73 record](history/releases.md#v0373--2026-10-26) is preserved in release history.

<a id="v0372--2026-10-26"></a>
The [v0.3.72 record](history/releases.md#v0372--2026-10-26) is preserved in release history.

<a id="v0371--2026-10-26"></a>
The [v0.3.71 record](history/releases.md#v0371--2026-10-26) is preserved in release history.

<a id="v0370--2026-10-26"></a>
The [v0.3.70 record](history/releases.md#v0370--2026-10-26) is preserved in release history.

<a id="v0369--2026-10-26"></a>
The [v0.3.69 record](history/releases.md#v0369--2026-10-26) is preserved in release history.

<a id="v0368--2026-10-26"></a>
The [v0.3.68 record](history/releases.md#v0368--2026-10-26) is preserved in release history.

<a id="v0367--2026-10-26"></a>
The [v0.3.67 record](history/releases.md#v0367--2026-10-26) is preserved in release history.

<a id="v0366--2026-10-26"></a>
The [v0.3.66 record](history/releases.md#v0366--2026-10-26) is preserved in release history.

<a id="v0365--2026-10-26"></a>
The [v0.3.65 record](history/releases.md#v0365--2026-10-26) is preserved in release history.

<a id="v0364--2026-10-26"></a>
The [v0.3.64 record](history/releases.md#v0364--2026-10-26) is preserved in release history.

<a id="v0363--2026-10-25"></a>
The [v0.3.63 record](history/releases.md#v0363--2026-10-25) is preserved in release history.

<a id="v0362--2026-10-25"></a>
The [v0.3.62 record](history/releases.md#v0362--2026-10-25) is preserved in release history.

<a id="v0361--2026-10-25"></a>
The [v0.3.61 record](history/releases.md#v0361--2026-10-25) is preserved in release history.

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
