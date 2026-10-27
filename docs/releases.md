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

## v0.3.94 — 2026-10-27

[v0.3.94](https://github.com/liveweird/grenadiers/releases/tag/v0.3.94) fixes ordinary CPU
navigation that kept requesting a perpendicular turn outside configured tolerance. The CPU
first aligns along its current heading axis, retains the chosen next cell, then turns from a
fresh stopped pose. Existing pulse arithmetic bounds both residual endpoints within the center
window; an unjustified correction yields Stop. Normal movement, settings, planting and hazard
certification retain authority. UtilityV12 preserves V1–V11 accepted-input replay identities
and the unchanged v12 room shape.

Both recorded a0/c0 and a2/c0 queued-driver cases require physical movement and a new cell after
running tick 2200, survival, deterministic repetition and exact replay. Focused tests cover both
poses, signs/headings, supported speeds/tolerances and residual boundaries; an authoritative
alignment→Stop→turn sweep covers all five speeds and four tolerances. Independent review is
clear. Rust arithmetic tests sample boundary offsets; the separate exhaustive design calculation
covers the full offset domain. The authoritative sweep uses one axis/sign.

The unchanged opening and full-round matrices each replay all 16 episodes exactly. Full rounds
retain 15 timeout draws and the same CPU win, with no CPU loss or censoring. The four formerly
stalled retreat cases visit 33–39 cells rather than 6–7; retreat totals rise from 133 to 252 cells
and 114 to 208 manual actions. Full calls fall 4,118→4,010 while logical work rises
61,017,649→89,659,046 (+46.9%); the per-job maximum stays 65,536. Host elapsed rises
11.15→12.23s, with p95/max planner observations 3.724/4.951ms. Most added work belongs to the
restored cases, but a1/c0 retreat also costs more for 37→38 cells and the same safe draw. These
scripted workloads and timings do not establish tuned difficulty or performance thresholds.

One exhaustive local server gate passes on tested source
`63db5126b868db66f1473e280b6753f41319fad5` (tree
`4fa6e9663254f1f2ac26eb3dee0c489d4ab8491c`): 487.449s,
26,089/28,323 lines (**92.1124%**), all 4,194,304 item masks separately in debug,
release and official coverage, 32 partitions/four workers/zero hidden retries and 37 raw
profiles. Official LCOV was copied before releasing the source/target freeze. Unique Linux
[run 40546313960](https://github.com/liveweird/grenadiers/actions/runs/40546313960) passes.

Exact local image
`sha256:f26653fc40e26616639324e5dbc99f99928672485b54097fc09471def96227da`
passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings, Compose 10 cases/49.9s,
Kubernetes 10 cases/51.1s, CPU-item collection 1 case/16.5s, Polish 320px, served-asset identity
and runtime security/resources/probes. Temporary projects/namespaces were removed. Unchanged
frontend/API/art/E2E/tooling inputs preserve the accepted full-web 6,874 tests/97 files and
static/audit evidence; no duplicate full web run was required.

This is a source prerelease with a verified local image, not a container-registry publication.
Localhost uses this exact image; v0.3.93 is retained for rollback. Further CPU tactics, difficulty
and human validation remain in the [backlog](../BACKLOG.md#single-player).

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
