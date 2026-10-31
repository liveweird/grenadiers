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

## v1.2.0 — 2026-10-31

Add playable contact-triggered spike pits: warning, raised damage and automatic retraction.
Pits start enabled with a 0.6s warning and 3s raised interval; creators can adjust both timings.
Shield, Heart and normal grace retain their usual behavior. CPU26 and lair mobs account for
pit contact and timing. The shared artist sheet includes three editable phase tiles.
See [spike pits](game/spike-pits.md) and [gameplay18](game/protocol-v18.md).

### Verification

Accepted source: `0f81f2271caef45dd60249d2d759ccdb8406f9a4`,
[PR #300](https://github.com/liveweird/grenadiers/pull/300).
Independent reviews cover physics, CPU forecasting/execution, transport, replay, mob navigation,
client validation and artwork. Resolved findings include mutable-layout reconstruction,
protection-expiry forecasting, room-setting agreement and immutable round-cell history.

The broader browser matrix exposed storm/pit terminal proofs exhausting CPU planting work.
The rules10-only fix advances stationary hazards by events and checks the complete interval;
per-tick execution guards, archived accounting and the 65,536 allowance remain unchanged.
Exact queued seed-2 reproduction now moves and places bombs; oracle tests cover protection,
future hazards, pickups and old-profile work parity.

- Core debug/release pass 1,650 tests plus two doctests; server passes 598 tests in both
  profiles. Fifteen existing server skips and the unchanged exhaustive item-mask acceptance
  are retained. Formatting, strict Clippy and the 400-line gate pass.
- Workspace line coverage is 90.43% against the unchanged 90% floor. Full web passes
  7,334 tests in 160 files; coverage is 95.18% lines, 91.98% statements, 91.20% functions
  and 88.48% branches. Contract/type/lint/Knip and 110 documented scenario checks pass.
- Distinct Linux debug/release workspace checks pass for the accepted source in
  [the platform run](https://github.com/liveweird/grenadiers/actions/runs/41231946287).
- Native Three checks pass 17 cases with seven intentional skips; artist workbench passes
  all 11 cases. The real seeded keyboard scenario covers activation, escape, retraction,
  death, reconnect and rematch. Broader unaffected human/multiplayer cases are reused.
- The exact image passes Compose duel/authentication, the full seeded spike cycle, all
  board/controller/difficulty CPU progress cases, three-CPU first-bomb escape and the Easy
  solo lifecycle. Kubernetes duel/authentication and restricted-runtime checks also pass;
  owned acceptance stacks and namespace are removed.
- The production bundle remains within the unchanged limits: JavaScript 397,583/400,000
  gzip bytes and CSS 15,430/20,000. Every image web file equals the verified build.
  Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL findings. Dependency inputs are
  unchanged; prior dependency acceptance is reused.

### Operation and artist handoff

Exact accepted local image:
`sha256:ffe445094e57c9241bcde91aa65c94b71eb3fcfb0d2512c99c65516c08901a75`,
version 1.2.0/source 0f81f227, available as `grenadiers:spike-pits-v120`.
This paired gameplay18/rules10/CPU26 upgrade ends in-memory matches; reload existing tabs.
Historical contracts and private replay formats remain closed. Production replay export and
hosted distribution remain outside this release.

The [v1.2.0 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.2.0/grenadiers-artist-v1.2.0.zip)
contains the canonical 1120×2128 PNG/JSON, separate guides, overview, 185 runtime slices and 3D
phase previews: 199 files total. The source has 189 assigned entries, four blank reservations
and one unassigned cell. New slots 186–188 preserve every previous sprite and mapping.

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
