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

## v0.3.45 published prerelease — 2026-10-24

Source `5be42456cee80e06e2f138bdfd458ef821bb34c2` is published as
[v0.3.45](https://github.com/liveweird/grenadiers/releases/tag/v0.3.45) through annotated tag object
`0e6224643ce85bf8f0a77894dcb929ab2655e397`. It publishes Punch as the twentieth requested item.
Localhost serves the accepted image
`sha256:3f13b26f7a4a41ba2e90b8cc9261c9e95a60eceeddf49504341e65258eb7d37d`.
Bracelet v0.3.46 is the current prepared source candidate; its native acceptance and publication
remain pending.

## v0.3.44 published prerelease — 2026-10-24

Source `f32fd86d8f0c7c4bebba8fa60d1c66c65d1a174a`, tree
`a0f2addb9204c40b4b30be2830fde4df368f019c`, is published as
[v0.3.44](https://github.com/liveweird/grenadiers/releases/tag/v0.3.44) through annotated tag object
`c5b1ab6d4cad94ba2f16614103903067745c5179`. It publishes Kick as the nineteenth requested item.
Localhost serves the accepted image
`sha256:8ff3005861127f8ac57cc32f7e6c72ce5fa996fe628ad48426dbb8cf133e1af9`.
This is the published parent recorded for the historical Punch candidate below.

## Historical v0.3.44 Kick native-acceptance checkpoint — 2026-10-24

Kick candidate source `cb70cba891ed972380f30f52133622ce36fd9ee8`, tree
`5be27219ba749d2811396d6f84af7d31f528a692`, passes fresh source-bound Rust, web, E2E and security
gates. Every Rust profile passes 763 core and 210 server tests with four documented ignores and
enumerates all 524,288 settings masks; debug and release each pass two doctests. Rust line coverage
is 15,233/16,429 (92.720190%). The web gate passes 1,059 tests across 66 files with
95.26/90.9/96/97.67% coverage and 57 static scenarios. Configured security retains three affected
moderate packages represented by six GHSA advisories, with no high or critical findings; E2E audit
reports zero vulnerabilities. Gate receipt
`903736dda1001d60f83204fa10f58940796c87716a5d9eea151b58e62fd97153` binds actual-zero roots
`31818` and `5290`.

Fresh image `sha256:8ff3005861127f8ac57cc32f7e6c72ce5fa996fe628ad48426dbb8cf133e1af9`
passes all 56 Compose plus 56 Kubernetes cases with zero retries or carried results. Root `97948`
finishes with consumed actual exit zero; the matrix retains 54 reports, 126 PNGs, 44 runtime
records, 22 distinct Kubernetes identities, 21 fixtures and 42 seed records. Each target passes 22
latency cases and retains 80 game observations under the unchanged 250 ms budget. The earlier
failed attempt's 55 cases are not carried.

Facts receipt `00542824d775035473b8a2c6bc15ea54d176dbf990c8a2e5004dd28643746915`
records this completed execution, and cleanup receipt
`c69c1d93de5be0c4e4c4b9dfe6214175fa5b63041aa7e393ce5b5d2a479cff5a`
records owned-resource removal. Audit `58e03d36c7dc967e71342e0e70e1fb65ac045e0f5a2883800d984d83bfe4654c`,
result review `8f9f1e7d520a3da7baf52344c551545f6903310af458fdfe6b90465c6d406594`
and proof-binding review `24edf45899b2402c86204eec7516e3a31b39bba8b2704df0a6d2d3b303466dea`
are clear. Final manifest `a5d0c54f699629ea732c3f641bad02bd322ebb8c0f1a4a0efd1df293c4126620`
is durably bound by seal `94c60dac44c639f7e9a214061e9daf9475628126d15526346ed7be2ce4c9a947`;
independent post-seal review `97c1b19ff6694746b73ac9d427f30f4840f14300967e4333fed9d634fb05fd51`
is clear.

At this dated acceptance checkpoint, candidate/master/tag CI, publication and local promotion have
not run. v0.3.43 remains the verified published/deployed baseline for this checkpoint; consult
[GitHub Releases](https://github.com/liveweird/grenadiers/releases) for authoritative publication
status after it.

## v0.3.43 published prerelease — 2026-10-24

Published source `c086f719ea470b9fe24963adef84b5e3bdf3e17a`, tree
`d74fad8a1c870f2834fca48f80ea4e38f0c512d9`, is available as
[v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43) through annotated tag object
`c8395fd7dcc5701842644967802e61c76211931c`. Candidate push/PR runs `40051959641` and
`40051963999`, master run `40053637247`, and tag run `40055400664` each pass all ten jobs, 55
browser cases and all 262,144 masks in debug, release and coverage on their first attempts.

Localhost serves accepted image
`sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01` from container
`d7f68cc89bfc87d06c202fbedae78c1892f54ea3ccdc6571fee0d1c5ea2f4240`; health, readiness and
index return 200 and the served script SHA-256 is
`61d6722543d93559ad980b2f6577897ddbdfcc80b4137d598d7aecf75c6ea637`. The v0.3.42 image is retained
as the rollback parent. Delivery contract
`f5696ccb5aba2633f3ecbf7a12353aa5073d61218cf0f717807e516bd69bbeb4`, independent release review
`da546c5038b3dc514b03a1b3fef658f828b27557c629082d95f2ccd368ba9889`, delivery binding
`40578e13d8a44a3cdb22bc222ac71eedd0005b08f3fb5f0748d07f7e333cf5d1`, and binding review
`21f6b544ee9d8e906c056671242c6be73ba742d1bcecb55f39c804ae78d9ec54` are clear.

Native execution remains attributed to source `11d0dc7e39e8e2cafcf70c1548be339f52c740ab`, seal
`31fe5eef2900a0229d002e45b5a310888f3915829db0aaf23bc3535a8eea5e2c`, and final manifest
`3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c`. Artifact-equivalence receipt
`81bd78943fa8e80bc839b06bcd68c7bb26872f4cf600c2fcc80e2269c7c94209` and review
`42bbbcb68e8a306d9ed611513e186189ab3430f17bf545191edace5bdb6e3b75` connect corrected source
`5dbcd58` to the published source through eight documentation-only changes. They do not claim
unchanged Docker inputs, a new Rust compilation, or new native execution. Eighteen of 22 requested
items are published at this checkpoint. Kick is the v0.3.44 native-accepted candidate; hosted CI
and publication follow.

## Historical v0.3.43 Wall Pass native checkpoint — 2026-10-24 01:00 UTC

Wall Pass is the eighteenth requested item and is native verified at source
`11d0dc7e39e8e2cafcf70c1548be339f52c740ab`, tree
`655c7d8a80b4dd3fdd3b232534c2cbaa45b054cc`, on exact image
`sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`.
Fresh gates pass 711 core and 209 server tests with four historical ignores in each Rust profile,
797 web tests across 64 files, 56 scenarios and all 262,144 masks in debug, release and coverage.
Rust line coverage is 92.358954%; web statement/branch/function/line coverage is
95.24/90.88/96/97.66%. The configured security commands pass; the web inventory retains two
moderate development advisories and no high or critical advisories.

Native root `45615` completes with consumed exit zero: all 55 Compose plus 55 Kubernetes cases,
52 reports, 122 PNGs, 42 runtime records, 21 Kubernetes identities, 20 fixtures, 40 seed records
and 80 latency observations below 250 ms pass with zero retries or carried results. Independent
audit `25e48638872f7b61fd64c28af30abf5e44910cf29e3e8356246236991819380e` and result review
`c8652f4e67852002db9e49e2766481e6c58b071d3e93bbb7866bfd6c5194848d` are clear. Binding review
`7c6eda746a703f599ced928798750f05478b78dbf5fc387d03fbe5adac551d46` is clear. Sealer root
`96118` completes with consumed exit zero; final manifest
`3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c` authenticates 2,665
payloads, 2,666 files and 284 directories. Seal
`31fe5eef2900a0229d002e45b5a310888f3915829db0aaf23bc3535a8eea5e2c` is durable; independent
post-seal review `5d253559ba95a9d30fc83f16fd0786a6d283356d892f26abed3611771588d33f`
is clear.

The source includes the reviewed Zod 4.6.6 lockfile correction and parallel independent English
match-result inspections without changing the 43 assertions, three screenshots, fixtures or
budgets. The descendant release candidate adds CI-only procfs short-read and external-target
coverage-parent fixes in commit `dee44b5ca81d2556d775a54e47948da77b9dbaa3`. Its production and
Docker build inputs remain byte-identical to the sealed native source. Those fixes improve
diagnostics and gate setup without claiming that a particular Chrome I/O cause was fixed.

Latest corrected candidate `5dbcd58e920f852a8036c4bf741e26517793b072` pins Happy DOM 21.0.4 to
resolve GHSA-834v-8h66-3h79. Fresh local verification passes the full 797-test, 64-file web gate
with unchanged 95.24/90.88/96/97.66% coverage and the complete configured security gate: cargo
deny plus both npm audits exit zero, with three moderate development advisories
(`@vitest/mocker`, `vitest` and `fast-uri`) and zero high or critical findings. Receipt
`0a8c605c65bdceff9d04d74752aaafc3ee5e9ee2378e60805a7fecb8cc216ce0` binds the web check, and
`e0c7363348d4f4092bc0afd3037068e183b78bc99c0969a6a2ef7b6451fdceb6` binds the full security
result. The two web manifests are the only changed Docker inputs: 554 of 556 entries remain equal.
The complete host distribution still matches the accepted image byte-for-byte across 17 entries
(15 files and two directories), using retained server binary
`26bccf1696b75b33b741b43305429df92cd09870d02d1080b725e05665e3966d`; a fresh scan of that exact
image passes current policy. Equivalence receipt
`81bd78943fa8e80bc839b06bcd68c7bb26872f4cf600c2fcc80e2269c7c94209` and independent review
`42bbbcb68e8a306d9ed611513e186189ab3430f17bf545191edace5bdb6e3b75` are clear. This is retained
artifact equality, not unchanged build inputs, a new Rust compilation, or Docker-build
reproducibility; the 110 native cases remain attributed to source `11d0dc7` and accepted image
`577abb3`.

At this checkpoint, the first PR #104 head `98dffe5` failed security in push run `40050321682` and PR run
`40050338206`; other jobs were still finishing at that checkpoint. Corrected head `5dbcd58` has
not yet been pushed. Those then-pending steps were later completed by the published delivery above.

This section is the dated native checkpoint, not the current release status. The published record
above supersedes its pending statements. Kick, Punch, Bracelet and Oil Barrel remain before
ART-ATLAS.

The reviewed Wall Pass captures are [English desktop](screenshots/wall-pass-desktop.png) and
[Polish 320px reduced motion](screenshots/wall-pass-mobile-pl.png), with SHA-256 values
`d46367b7eeee7c55e94dbcfba6de7de50b27b96c6761ad6a9ff69db2b9332c08` and
`84843190d2be59a89f5c614c3d17bd78e969dcb7cf692d0e06243b2d3978d72f`.

## v0.3.42 published prerelease — 2026-10-23

Merge `0ff1512c5a8fdbd1e1b16583ae7791e668bf070b`, tree
`b4201abe9546d22b26b3231506d1e2ab6b1d3613`, is published as
[v0.3.42](https://github.com/liveweird/grenadiers/releases/tag/v0.3.42) through annotated tag object
`ea3930a3490fb483de46692e1009743e20c61648`. Master run `39982027955` and distinct tag run
`39984896509` each pass all nine jobs, 54 browser cases and 131,072 settings masks on the first
attempt, with independent reviews clear. Promotion root `84311` completes with consumed exit zero;
container `0a3bcbac08bc2c2cf46365917a3719d3c5e522c4555234da8e1f661f6edcf214` serves exact native image
`sha256:8ae9818d27c4345bceec6dad77f02e21ee7148626efded48c421d86421285477`. Seventeen of 22 requested
item types are published. Wall Pass was the next source candidate at this delivery checkpoint; its
later native acceptance is recorded above without changing v0.3.42's published history.

### Historical native-accepted source checkpoint — `2026-10-23 04:37 UTC`

Source `880fff7c67269f2886746b45239cd888576832f4`, tree
`c1da58099bb00bb0c99b4bd00de38e9c0c6f87d6`, and image
`sha256:8ae9818d27c4345bceec6dad77f02e21ee7148626efded48c421d86421285477` pass 54 Compose plus 54 Kubernetes cases with zero retries or
carried passes. Independent audit `0dbb15b27d5a23eae680ca9cfa2d9766a7e64e173f5f8652fd753b4611135521` and result review
`cf3380353ca06f36cc61f67c8898ed766d6263bdc6108e003a0a9ac17da4923c` are clear; durable manifest `d81c0923f5a3d0a50208fe43a12412f60ca09e2fddfc1c6aabfb793779676b7f`
authenticates the sealed evidence. Source gates cover 689 core, 208 release server and 657 web
tests, 55 static scenarios and all 131,072 settings masks in debug, release and coverage profiles.
At this checkpoint, hosted candidate, post-merge, tag, publication and local promotion remained
pending. The delivery record above documents their later completion.

The reviewed English desktop and Polish 320px reduced-motion Teleporter captures hash to
`e262534f6c157678f762519aa03cc2ba7896139f04d9002c1347f68bc12cff63` and `30f9e9cd099a2b01ae41c9c64096cb15b50b4850b5198cea54ad8b2eaddd63ba`.

## v0.3.41 published prerelease — 2026-10-23

Commit `f5c4adf66f8af9a5ef66b48f9230d17aea14f1d5`, tree
`a274fb2fe828f3a18a624614f1addf5bf5df4d99`, is published as prerelease tag `v0.3.41`.
Post-merge run `39879618476` and distinct tag run `39881306987` each pass all nine jobs,
53 browser cases and 65,536 settings masks. The published source retains the accepted application,
E2E and native inputs, while the CI-only browser readiness and observer changes retain their own
reviewed identity. The sealed native image
`sha256:d063462c3130e3c8653734da72e8dfd6785b0c036e9579d9046cee884ccd9f82` is promoted
locally with consumed exit zero. At that release checkpoint, sixteen of 22 requested item types
were published; the v0.3.42 delivery record above documents Teleporter's later publication.

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
