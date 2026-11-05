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

## v1.3.25 — 2026-11-05

Hard66 can prefer a visibly contested capacity/range upgrade over an equally useful, equally
close uncontested alternative when choosing a fresh goal. Existing commitments remain stable.
The comparison uses bounded public-state distances; movement and planting proofs retain their
existing authority. Easy/Standard62, generation7, pit-layout2, gameplay21/rules10 and ContactV2
remain unchanged; private replay35 closes the pickup family.

The paired queued-owner scene proves actual contested collection, an available alternative and
subsequent cell progress. Six generated reference cases record identical Hard65/Hard66 metrics;
the demonstrated improvement is this tie decision, not a general strength claim. See the
[physical evidence](history/cpu-pickup-contention-2026-11-05.md) and
[contract](game/cpu-contracts.md#hard-pickup-contention).

### Verification

- Frozen source `a2b0ea04bc5f96f13082275069e5e89af6cfa8fe` passes full local Rust checks:
  2,017 core tests and two doctests, 826 ordinary server cases and four isolated timing cases
  per debug/release/coverage profile. All 4,194,304 item masks pass per profile with 32 partitions,
  four workers and no retries. Coverage is 81,313/87,318 lines (93.12%), above the unchanged 90%
  floor, using 40 raw profiles. Wall time is 1,044.19 seconds.
- The separate native Hard66 pacing test passes with lairs off/on in 34.64 seconds. Independent
  review findings are resolved; formatting, strict Clippy, repository and source-size checks pass.
  [Linux debug/release checks](https://github.com/liveweird/grenadiers/actions/runs/41936538403)
  pass on the same source.
- Web/API/assets/E2E/build/deployment configuration and third-party dependency resolution are
  unchanged from v1.3.24. Reuse its web/static/tooling/schema/dependency-policy and npm-audit
  results; only the two local Cargo package versions changed in the lockfile. Exact served-file
  equivalence and fresh runtime/image acceptance are recorded below.

- The initial five-scenario Chromium run passed four cases; the duel's three-second result UI
  assertion missed its window despite agreeing authoritative results. An isolated rerun of that
  unchanged scenario passed in 24.1 seconds against the same image. Short-lived browser lifecycle
  assertions now explicitly share the quiet-host rule in contributor guidance.
- The first Kubernetes smoke helper expected diagnostics while its test overlay disabled them.
  Correcting the disposable test overlay enables the existing diagnostic assertions; production
  deployment defaults and application code remain unchanged. Failed-run logs are retained.
- The completed Compose and restricted OrbStack Kubernetes smoke checks pass with three moving/
  bombing Hard CPUs, two moving mobs, rendered canvas and configured spikes. Exact-image asset,
  CSP, cache, readiness, probe and security checks pass. Temporary resources are removed.
  Pinned Trivy 0.74.0 reports zero fixable high/critical findings.

The accepted image is `sha256:06b8a4cf4c7e8e5a51bad445eb12a5ad7a7a209e5fbe22c827b26d73b9cbe302`, labelled v1.3.25 and source
`a2b0ea04bc5f96f13082275069e5e89af6cfa8fe`. Later release-document changes preserve the verified
code. Promote this exact image and retain v1.3.24 for rollback.

## v1.3.24 — 2026-11-04

Hard65 can conditionally plant beside visible stationary rival bombs and clear a forward wall
while retaining its opponent goal. After interruption it can prove a fresh nonmanual OwnEscape
route through its owned bombs' complete flame lifetime. New contradictory publications still
revoke authority. These are visible-state predictions, not guarantees against unseen actions or
loot. Easy/Standard62, generation7, pit-layout2, gameplay21/rules10, navigation3 and ContactV2
remain unchanged; private replay34 closes the new CPU family.

The controlled queued match has two useful surviving plants. An actual safe pose correctly uses
stationary recovery; a separately labelled unsafe-position counterfactual proves a directional
escape. The fixed comparison is mixed, with slightly fewer plants and more stalled-objective
switches despite fewer ever-eliminated seats and more completed-objective switches. It does not
establish a general strength increase. See the [dated evidence](history/reactive-hard-cpu-2026-11-04.md)
and [planning contract](game/cpu-contracts.md#reactive-hard-planning).

### Verification

- Full local Rust verification passes on `a5fced0840c4cce4f505a49fb076faaa862b79a3`:
  2,004 core tests and two doctests, 818 ordinary server cases and four isolated timing cases
  in each debug/release/coverage profile. All 4,194,304 item masks pass per profile with
  32 partitions, four workers and no retries. Coverage is 80,756/86,740 lines (93.10%), above
  the unchanged 90% floor, using 40 raw profiles. Wall time is 1,033.65 seconds.
- The separate native Hard65 production-pacing test passes with lairs off/on in 31.74 seconds.
  Independent review findings are resolved; strict Clippy, formatting, repository and size gates pass.
  [Linux debug/release verification](https://github.com/liveweird/grenadiers/actions/runs/41926894973)
  passes on the same verification source.
- Web coverage passes 172 files/7,435 tests, with 95.51% lines, 92.54% statements, 91.77% functions
  and 88.90% branches. API/type/validator drift, lint, Knip and E2E static checks pass,
  including 122 scenario-documentation matches.
  Compressed JS is 399,060/400,000 bytes; CSS is 16,228/20,000. All nine exact-image web files
  are byte-identical to v1.3.23.
- Five Chromium scenarios pass. Compose and restricted OrbStack Kubernetes verify the same image,
  three moving/bombing Hard CPUs, two spawned/moving mobs, probes and security settings. Temporary
  stacks/namespaces are removed. Rust dependency policies and high-severity npm audits pass;
  existing moderate findings remain 10 web/11 E2E. Pinned Trivy reports zero fixable high/critical findings.

The accepted image is `sha256:724b6f06bf3f37900caea7eae9006a4e7028a31bff7f1a1cb7dd9d51466875b7`,
labelled v1.3.24 and build source `14e5392482af02c3c427a0abb5e71852a94d2acf`. The later verification
commit corrects only two `#[cfg(test)]` fixture modules and a history document. Independent review
confirmed that production compilation excludes those changes; Docker build contexts are not
claimed byte-identical. Image labels retain the actual build source. Later release-document
changes leave the verified code unchanged. Promote this exact image and retain v1.3.23 for rollback.

## v1.3.23 — 2026-11-04

New matches reserve a connected floor ring around each lair and keep spike pits away from its
spawn exits. Weighted generator7 retains the accepted ×1–×10 Bomb Capacity selection setting;
new live rooms still select ×5. Construction consumes no additional randomness, preserves
uncarved contents and spawn reservations, and uses lair-aware pit-layout2 on every board.

CPU62 on Easy/Standard and CPU64 on Hard retain certified escape authority through the full
own-bomb flame horizon and accept crossed route anchors. Hard64 can reuse a complete ineffective
attack proof for useful visible-block demolition at the same certified origin. The work ceiling,
contact/Teleporter guards, collision rules and recipient privacy remain unchanged.

Gameplay21/rules10/navigation3/ContactV2 remain paired. Private replay33 binds the new
construction, accepted settings and CPU family. Published generator6/replay32/CPU60–61 and
archived policies retain their exact semantics. Production replay recording/export stays disabled.
See [generation](game/generation.md), [CPU behavior](game/multiplayer-cpu.md#recovery-and-initial-demolition)
and the [live arena example](screenshots/lair-clearance.png).

### Verification

- Frozen source `5a18d0ebfdd3d70cbfb06151e598662416d418a3` passes the full local Rust gate:
  1,995 core tests and two doctests, 804 ordinary server cases and four isolated native cases
  in each debug/release/coverage profile. All 4,194,304 item masks pass per profile in 32 partitions
  with four workers and no retries. Coverage is 78,525/84,383 lines (93.06%), above the unchanged
  90% floor, from 40 raw profiles. The gate took 952.002 seconds. Formatting, strict Clippy,
  repository and source-size checks pass. Two independent reviews found no actionable defects.
- The actual queued generator7/×5 Hard witness places all three CPU bombs at tick190,
  detonates them at370 and confirms all three owners alive after complete finite-flame clearance
  at400. Five blocks are destroyed and lair mobs spawn/move. The all-board/difficulty/lair-mode
  queued matrix passes. The new native Hard64 test is also run separately with two real workers
  at production pacing, with lairs off/on; its ordinary-suite ignore prevents timing contention.
- Distinct Linux debug/release workspace tests and doctests pass in
  [41900316646](https://github.com/liveweird/grenadiers/actions/runs/41900316646).
- Full web coverage passes 172 files/7,435 tests: lines 95.51%, statements 92.54%, functions
  91.77%, branches 88.90%. API/type/validator drift, lint, Knip and E2E static/scenario checks pass.
  Compressed JS is 399,060/400,000 bytes and CSS 16,228/20,000. The exact image's nine served
  web files, including the atlas and branding, are byte-identical to v1.3.22.
- Five exact-image Chromium cases pass: authentication, keyboard duel/Axe, shared settings,
  responsive lobby and rendered four-player HUD. Compose and restricted Kubernetes both pass
  with three moving, bombing Hard CPUs and two spawned, moving mobs; no browser errors occur.
  Pit/lair separation, image identity, probes and container security assertions pass. Temporary
  stacks/namespaces are removed. Dependency audits pass with 10 web/11 E2E moderate findings
  and no high/critical findings; pinned Trivy reports zero fixable high/critical vulnerabilities.

The accepted image is `sha256:b707492ab49e08e53b9355f23611c1d0591b08b12e2ee57789ab412dfd6065e8`,
labelled v1.3.23 and the frozen source above. Later documentation commits leave its build inputs
unchanged. Retain the accepted v1.3.22 image for rollback. This release makes no claim that every
combat episode is survivable or that conservative unsupported planning cases are solved; those
remain explicit CPU design boundaries and backlog work.

## v1.3.22 — 2026-11-02

Bomb Capacity drop weight is configurable from ×1 to ×10, defaulting to ×5 for new games.
Restore selects 5, Classic selects 1 and All on/off preserve the number. The value changes only
selection weight: concealment stays 30%, item switches and Oil suppression remain unchanged.
With all optional items enabled, the Bomb Capacity share rises from 9.09% to 33.33%; with them
disabled, from 40% to 76.92%. It is not a fivefold absolute probability.

Gameplay21 carries the checked multiplier through settings, revisions, reconnect, rounds,
rematches and CPU observations. Generator6 remains deterministic for complete accepted inputs.
Private replay32 records weighted construction and updates; historical formats reject nonlegacy
multiplier provenance before execution. Rules10, navigation3, ContactV2 and CPU60/61 remain.

### Verification

- Frozen Rust source passes debug/release/coverage: 1,980 Core tests and two doctests, 789 ordinary
  server cases plus four isolated native worker cases per profile. Every one of 4,194,304 item
  masks passes in each profile with 32 partitions/four workers and no retries. Coverage is
  75,716/81,360 lines (93.06%), above the unchanged 90% floor; 40 raw profiles feed the report.
  The full gate took 1,021.693 seconds. Current CPU opening helpers assert gameplay21/generator6/
  weight 5 before driving the existing seeded progress cases. Formatting, strict Clippy and
  source-size checks pass; independent domain, contract, server and web reviews are clear.
- Distinct Linux debug/release workspace units and doctests pass in run
  [41392820007](https://github.com/liveweird/grenadiers/actions/runs/41392820007).
- Fresh shuffled web coverage passes 172 files/7,435 tests: lines 95.51%, statements 92.54%,
  functions 91.77%, branches 88.90%, above unchanged floors. Focused regressions pass 173 cases.
  API/contracts, lint, Knip, validator parity, E2E static checks and 122 scenario mappings pass.
  Compressed JS is 399,060/400,000 bytes; CSS 16,228/20,000. Compact v21 validator output omits
  unused error prose while retaining all constraints, all-errors collection, paths and keywords.
- Five exact-image Chromium scenarios pass: authentication, native keyboard duel/Axe, shared
  settings, lobby containment and painted four-player play/dense HUD across supported sizes.
  The settings flow verifies ×5 defaults, ×1 Classic, bulk retention, a shared ×10 update,
  readiness reset, reconnect persistence and locking. English and Polish 320px controls are
  visually checked: [English](screenshots/bomb-drop-weight-en.png),
  [Polish 320px](screenshots/bomb-drop-weight-pl-320.png).
- Compose and restricted Kubernetes pass with three moving, bomb-using CPUs and no browser errors.
  Dependency policies and high-severity audits pass; existing moderate findings remain 20 web/12
  E2E. Pinned Trivy reports zero fixable high/critical findings. Temporary stacks/namespaces are
  removed. All five arena/branding images retain their accepted bytes.

### Artifact, upgrade and rollback

Rust verification source: `7a011e7be68257426df0f59e014d9fef5b3daa2f`.
Accepted application/image source: `06d4a5a91123fdedf53d29a18d027ed37e5d8279`.
The difference is limited to reviewed browser fixture isolation and accessible inventory naming;
Rust and API inputs are unchanged; the full web and exact-image checks cover those browser changes.
Accepted image: `sha256:edb4dcbc5aec8da0764a5bad1ff2bcb5b151cfe13545e5cb2b7b00f7a73930d8`.

The exact image runs on localhost:8085. Labels, restricted container settings, health/readiness,
headers/CSP/cache and all nine served files pass, plus fresh three-Hard-CPU play at weight 5.
Reload browser tabs after deploying the paired server/web image: older wire versions are rejected
before credential claim. The v1.3.21 immutable image/override remain for rollback. In-memory
matches do not survive restart. No hosted service or production registry is configured.

<a id="v1321--2026-11-02"></a>
The [v1.3.21 record](history/releases.md#v1321--2026-11-02) is preserved in release history.

<a id="v1320--2026-11-02"></a>
The [v1.3.20 record](history/releases.md#v1320--2026-11-02) is preserved in release history.

<a id="v1319--2026-11-02"></a>
The [v1.3.19 record](history/releases.md#v1319--2026-11-02) is preserved in release history.

<a id="v1318--2026-11-02"></a>
The [v1.3.18 record](history/releases.md#v1318--2026-11-02) is preserved in release history.

<a id="v1317--2026-11-02"></a>
The [v1.3.17 record](history/releases.md#v1317--2026-11-02) is preserved in release history.

<a id="v1316--2026-11-02"></a>
The [v1.3.16 record](history/releases.md#v1316--2026-11-02) is preserved in release history.

<a id="v1315--2026-11-01"></a>
The [v1.3.15 record](history/releases.md#v1315--2026-11-01) is preserved in release history.

<a id="v1314--2026-11-01"></a>
The [v1.3.14 record](history/releases.md#v1314--2026-11-01) is preserved in release history.

<a id="v1313--2026-11-01"></a>
The [v1.3.13 record](history/releases.md#v1313--2026-11-01) is preserved in release history.

<a id="v1312--2026-11-01"></a>
The [v1.3.12 record](history/releases.md#v1312--2026-11-01) is preserved in release history.

<a id="v1311--2026-11-01"></a>
The [v1.3.11 record](history/releases.md#v1311--2026-11-01) is preserved in release history.

<a id="v1310--2026-11-01"></a>
The [v1.3.10 record](history/releases.md#v1310--2026-11-01) is preserved in release history.

<a id="v139--2026-11-01"></a>
The [v1.3.9 record](history/releases.md#v139--2026-11-01) is preserved in release history.

<a id="v138--2026-11-01"></a>
The [v1.3.8 record](history/releases.md#v138--2026-11-01) is preserved in release history.

<a id="v137--2026-11-01"></a>
The [v1.3.7 record](history/releases.md#v137--2026-11-01) is preserved in release history.

<a id="v136--2026-11-01"></a>
The [v1.3.6 record](history/releases.md#v136--2026-11-01) is preserved in release history.

<a id="v135--2026-11-01"></a>
The [v1.3.5 record](history/releases.md#v135--2026-11-01) is preserved in release history.

<a id="v134--2026-11-01"></a>
The [v1.3.4 record](history/releases.md#v134--2026-11-01) is preserved in release history.

<a id="v133--2026-11-01"></a>
The [v1.3.3 record](history/releases.md#v133--2026-11-01) is preserved in release history.

<a id="v132--2026-11-01"></a>
The [v1.3.2 record](history/releases.md#v132--2026-11-01) is preserved in release history.

<a id="v131--2026-11-01"></a>
The [v1.3.1 record](history/releases.md#v131--2026-11-01) is preserved in release history.

<a id="v130--2026-11-01"></a>
The [v1.3.0 record](history/releases.md#v130--2026-11-01) is preserved in release history.

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
