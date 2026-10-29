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
independently reviewed candidates, then publish the corresponding prerelease tag.
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

## Current prerelease distribution

Current releases publish source tags and GitHub prerelease notes. Accepted image IDs in the
verification record identify locally tested artifacts; no production registry or hosted service
has been configured. A source prerelease does not claim a production deployment.

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

## v0.3.118 — 2026-10-29

Activates lairs and pursuing mobs for all supported two-to-four-player human/CPU arenas.
Creators can change the spawn interval (5–60 seconds) and living-mob cap (1–8 per lair),
with defaults of ten seconds and three mobs. New rooms enable all items, cloud and lairs;
feature presets and the first-start editing lock apply to the complete configuration.
Directional sprites, authoritative interpolation, private targets, reduced motion and EN/PL
controls accompany the shared simulation and finite CPU contact guards.

Corrects a terminal roster mismatch: a new transport loss after the final result retires that
seat without invalidating the surviving peers' publication. Reservations created before the
final result retain their original deadlines. Running reconnect still restores the same seat,
state and stable capability; retired terminal seats reject resume.

The paired endpoint requires gameplay16/rules8/CPU19. Versions 1–15 reject before credential
claim; archived adapters remain closed. Restart discards process-local matches and credentials.
Upgrade server/browser together and reload existing tabs. Use the retained v0.3.117 image for
rollback; a source prerelease does not claim a hosted production service or durable sessions.

The [lair guide](game/lairs.md), [settings](game/settings.md),
[reconnect policy](game/reconnect-server.md) and [artist handoff](../assets/art/README.md)
are the current behavioral references. The unchanged lair/mob artist bundle is published with
[v0.3.117](https://github.com/liveweird/grenadiers/releases/download/v0.3.117/grenadiers-lairs-art-v0.3.117.zip).

### Verification and source identity

Accepted source: `91c5e9280af2e61c212de6d52f1301bb81fbbaed`, tree
`be5408e1ff7f8c57c07c3b013d2f0ac26b0c013a`. Source PR
[#277](https://github.com/liveweird/grenadiers/pull/277) merged as
`b11ca40921d40c06b37151ffff0eb3c6bd93ed40` with the identical tree.

- Fresh Rust workspace debug/release, fmt, Clippy, build and independent coverage pass in
  311.032 seconds: 42,475/46,125 lines (**92.09%**), retaining the 90% floor.
  LCOV SHA-256: `f51d9e1945c9cf271b34fa957436076574c445dc97a8d32fd294d95a12a81f3e`.
- Only the unchanged historical ExpandedV5 exhaustive-mask execution reuses v0.3.116's
  all-4,194,304-mask debug/release/coverage acceptance. Its legacy lifecycle dependency closure
  excludes the corrected roster. Current coverage is fresh; no old profiles/counts are merged.
- Full web acceptance: **7,139 tests / 118 files**, 280.07 seconds; lines 93.90%, branches 88.80%,
  statements 90.86%, functions 89.91%. JS 358,943/400,000 bytes and CSS 15,226/20,000. Reused after
  byte-identical web/API/art/dependency input checks; later changes affect Rust lifecycle and
  browser tests only. Repository/tooling/E2E static gates pass.
- All 22 requested item suites, CPU Wall Pass and generation3 base upgrades pass on paired v16
  application inputs. The terminal correction changes none of their effect paths. The new
  generation3 pickup seed does not replace the archived generation1 trace.
- Corrected-image Compose: **19/19** in 3.8 minutes. Lairs: **2/2** in 26.3 seconds; reconnect:
  **3/3** in 44.0 seconds. Each fixture suite mounts its documented seed. Desktop EN and Polish 320px
  reduced-motion rendering pass. The terminal regression fails on the original candidate and
  passes on the corrected image; earlier failure/configuration logs remain separate evidence.
- Corrected-image Kubernetes: native authentication/turning **2/2** in 5.5 seconds, plus restricted
  security, exact image, resources/probes, zero restarts and scoped cleanup. The earlier 17 unchanged
  Kubernetes cases are retained; the affected terminal behavior is exercised freshly
  in Compose and runtime tests without repeating unchanged cases.
- Pinned Trivy 0.74.0: zero fixable HIGH/CRITICAL vulnerabilities. Dependency audits are reused
  after unchanged lockfile checks. Distinct [Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40969060566)
  pass on accepted source 91c5e928. Independent simulation/CPU/protocol/client/server/runtime
  reviews are clear after fixes.

### Local deployment and rollback

Promoted image:
`sha256:1aee8c309815b99c3ef9135e7373e803f48b0cf8727f7d9126b6cf8fb519f516`, labelled version 0.3.118
and source 91c5e928. Localhost:8085 passes exact-image/security/cgroup/health/readiness, CSP/cache,
and served HTML/JS/CSS/PNG equality with the accepted image. The non-root, read-only app retains
256 MiB memory/PID 128 limits. The immutable v0.3.117 override/image remains available for rollback.
Human difficulty, interaction and network/browser playtesting remain in [BACKLOG.md](../BACKLOG.md).

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
