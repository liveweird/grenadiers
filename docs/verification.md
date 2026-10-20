# Repository verification

## Piercing Bomb fresh native acceptance — 2026-10-20 — 94/94; independently reviewed and sealed

Final raw Slow Foot source `f499f680ac68bc8eed4803e4d3c06152cb89cf64` is integrated,
including the accepted startup-readiness baseline, stable Long Fuse cursor, corrected predecessor
browser drivers, final v0.3.33 native evidence and captures. The v0.3.34 version, Piercing Bomb
behavior, weapon HUD coverage and canonical ten-item catalog remain. The Piercing browser driver
now returns from the lower-left escape cell through the open (1,1) waypoint instead of attempting
to cross the hard pillar at (2,2).

Reviewed integrated source `1461f04d1a1b18ade191f282962e677d5ed2a43f` preserves product,
core, server, API, web, Cargo, lock and deployment inputs byte-for-byte from reviewed Piercing
source `d1b635fb501336495fa2819392fcc8bc246f25ea`. Using the existing exact locked developer tools,
the repository check, E2E lint, Knip, type checking and 48-scenario parity pass with actual exit
zero. Those checks precede frozen native source `dc6266e79ea78b3ea314aa16cd3c31a22a17d7c6`,
tree `63ea5ed005a2302d32a63eab126ea4c5bd27fa81`.

The single fresh same-image wrapper completes at `2026-10-20T21:01:06Z` with actual exit zero:
**47/47 Compose and 47/47 Kubernetes**, 36 groups/reports, all ten optional-item scenarios and
zero retries or carried results. Each target runs 37 baseline cases plus ten feature cases. Image
`sha256:5d0bc94c7febc6764b9f875f1273f4d3814077c8d22c2b56b4a2d80fbebbe0e8`
comes from source archive SHA-256
`90d2285112c64631c5709fb32a96d2672d02df9b3db80c67e370defdc6c52ece`;
the image, dependency and source-tree tar SHA-256 values are respectively
`abf66f4fda7e72da4b80f27205850a7b4fc0cd80cbf83f8cc5b67b7f2e753cfc`,
`db12f2e9df38f1af341bc3554957610970d60451cb223a6d86857c37cd830567` and
`ca9055ada8da11cdfd1d7f007bdc4f86aed63d099066807139bfb2602326dd8b`.
The source inventory has 710 files, 80 directories, no symlinks and 31 tracked PNGs. Dependencies
have 8,186 files, 1,305 directories and 13 symlinks; 132 packages are installed, all 42 omissions
are optional, and no required package, version or metadata mismatch is reported. Runtime evidence
contains 90 PNGs, 26 runtime image/security pairs, 13 Kubernetes identities, 12 fixtures and 24
seed-mode records. The 1,038-entry preparation manifest hashes to
`884cf6ba4fe5f616f8c04738bebc707b40b2c5840aeccfe72444ea798782b03a`.

The Piercing scenario proves ordinary-key collection and placement, an armed blast continuing
through breakable blocks `(4,1)` and `(5,1)`, one-charge consumption, and a following fresh
Standard bomb on both peers. The reviewed Compose [English desktop](screenshots/piercing-bomb-desktop.png)
and [Polish 320px](screenshots/piercing-bomb-polish-320.png) captures have no observed overlap.
The compact English four-column HUD wraps “Piercing bomb” inside the word, a nonblocking cosmetic
preview limitation; Kubernetes captures were retained but not separately viewed.

All four real latency profiles retain 20 observations, and all 22 latency cases per target pass.
The fixed 250 ms budget, startup-readiness causality checks and zero-retry policy remain unchanged.

| Target | Profile | Samples | Maximum | Exceedances |
| --- | --- | ---: | ---: | ---: |
| Compose | local | 20 | 56.044429 ms | 0 |
| Compose | delay-50ms | 20 | 142.791575 ms | 0 |
| Kubernetes | local | 20 | 51.750443 ms | 0 |
| Kubernetes | delay-50ms | 20 | 190.529203 ms | 0 |

Compose CPU usage advances from 702559 to 956078 microseconds; Kubernetes advances from 600100
to 797327 microseconds. Both throttle-count deltas are zero. Compose retains an unlimited CPU
quota and Kubernetes retains `50000 100000`. Coordinator-observed worker tasks remained completed
through measurement; this is not an acknowledgement-only hold or a globally idle-host claim.
Observed unrelated load included JVM 243.3% and OrbStack 19.7% at Compose capture, and OrbStack
30.3% at Kubernetes capture; the earlier JVM was not among the Kubernetes top-CPU entries.
Unrelated processes were not held, and all measurements were retained.

Independent raw evidence review is clear. The final manifest covers 1,987 payloads and hashes to
`67445cc105afe69d7894b550a70733e6634d1b05d478165c110fbc673a3d5ff7`.
All 1,988 files including the seal, plus the immutable source archive, have byte-verified durable
local checkpoint copies. Owned native resources are cleaned. Exact-head CI, PR, publication and
promotion are pending. The local default on port 8085 still serves the published v0.3.33 image;
the v0.3.34 image has not been promoted or published.

The earlier Piercing source passed 1,400 web tests across 47 files with
95.47/91.35/95.61/97.79% coverage, together with contract checks, lint, Knip and the production
build. Its E2E lint, Knip, type checking, 37-scenario parity, repository and diff checks also
passed under Node 24.21.0. Those full-suite and coverage results certify their named source only;
no new local Rust, web, full-coverage or production-build run is claimed for `1461f04`.

## Slow Foot source-prerelease publication — 2026-10-20

[PR 79](https://github.com/liveweird/grenadiers/pull/79) normalized accepted raw source
`f499f680ac68bc8eed4803e4d3c06152cb89cf64` to candidate head
`f3a26f3552902219d3139960a2fe4345113243ce`. Candidate
[PR run 39487844647](https://github.com/liveweird/grenadiers/actions/runs/39487844647) and
[push run 39487815667](https://github.com/liveweird/grenadiers/actions/runs/39487815667) each pass
five jobs and 46 browser cases. Merge head `27dcf225f3a1f3ef67ee3cf8c52c9d583e7318fd`
has tree `4457ec14280d240ce54b4d4c6064fffa769e9ec6`, byte-equal to the accepted raw tree; post-merge
[run 39490252393](https://github.com/liveweird/grenadiers/actions/runs/39490252393) also passes five
jobs and 46 browser cases. All three retained CI records contain no failure, flaky, skipped, retry
or timeout result markers.

[v0.3.33](https://github.com/liveweird/grenadiers/releases/tag/v0.3.33) was published as a source
prerelease at `2026-10-20T20:29:15Z` from that merged head and accepted tree. At publication, the
additional tag-triggered CI run had not completed and supplied no publication evidence. It later
completed successfully: [run 39492377407](https://github.com/liveweird/grenadiers/actions/runs/39492377407)
passes five jobs and 46 browser cases. The exact
accepted image `sha256:6608df856d8d1b0c69b5552ae1ee7d39ff08b0a1fcc4df804fb65806007f54be`
served `http://localhost:8085` at the local deployment checkpoint: readiness, health and
index probes returned 200, and the served index and JavaScript bytes equalled the container files.
The deployment reused the accepted image;
it did not rebuild it.

## Slow Foot fresh native acceptance — 2026-10-20 — 92/92

Frozen source `dbaa7f84f6013a620ec3f9bea12c02ad0e29d904`, archive SHA-256
`34db83f3d9a2663797743fa6d905cf00e07a72cb9cc801a4808ff94909d0be07`, builds the logged
no-cache image `sha256:6608df856d8d1b0c69b5552ae1ee7d39ff08b0a1fcc4df804fb65806007f54be`.
Image archive SHA-256 is `ce5f2c879aa1f60ebca4246f151e089e454a9caed5bd097558490921f245ed4a`.
The single complete wrapper invocation exits **0**: **46/46 Compose and 46/46 Kubernetes**,
34 groups/reports, all nine optional item scenarios, one worker and zero retries or carried results.
The Slow Foot scenario proves natural collection, one-stack half-rate movement in matching public
frames, exact wall stop and 600-tick expiry, restored base rate, peer equality and key-release stop.
The corrected driver uses the open top corridor and direction-matched, unclamped measurement
windows; product/API/Cargo/web inputs remain byte-identical to the earlier reviewed Slow Foot source.

All four real latency profiles retain twenty observations and complete readiness diagnostics.
Root independently checks full ingress → dispatch → page causality and the fixed watermark,
matching endpoint facts, accepted completion, and that every measured key starts afterward.
All 22 latency cases per target pass; the shaped budget remains 250 ms and retries remain zero.

| Target | Profile | Samples | Startup / fresh publication baseline | Maximum | Exceedances |
| --- | --- | ---: | ---: | ---: | ---: |
| Compose | local | 20 | 4253.027 / 108.991 ms | 49.566 ms | 0 |
| Compose | delay-50ms | 20 | 4891.056 / 186.425 ms | 167.692 ms | 0 |
| Kubernetes | local | 20 | 4195.146 / 128.683 ms | 49.484 ms | 0 |
| Kubernetes | delay-50ms | 20 | 4804.694 / 186.585 ms | 178.862 ms | 0 |

Startup includes context creation, joining, countdown and rendering; it has no 250 ms claim.
These are twenty-observation reference experiments, not Internet-tail or pixel-presentation
guarantees. Compose has unlimited CPU; Kubernetes retains `50000 100000` (half a CPU).
Both have zero observed throttle-counter deltas. Three delegated workers hold idle through both
latency groups. Before Compose measurement, root briefly holds the manual gate for unrelated
Flow E2E/browser load, preserving the initial snapshot. Measurement begins with that work still
active and lower observed load: OrbStack 15.7%, Flow E2E app 34.64%. The Kubernetes snapshot
later has no observed Flow E2E activity and OrbStack 11.5%. Unrelated processes remain untouched;
no globally idle host or unloaded benchmark is claimed. No measured sample is discarded.

Fresh dependencies contain 8186 files, 1305 directories and 13 symlinks. The installed proof has
132 packages, 42 optional omissions and zero required omissions or metadata/version mismatches.
Their tar hashes to `713d056c6a7ca5c7f3064287cf054b7aca1024b8fe21e5c9e2f9f48de89ed07a`.
The 694 source files, 79 directories, three locks, file modes, symlinks, source/dependency tar
bytes, semantic proof, image IDs and 1021-entry preparation seal match before and after execution.
Source-tree tar SHA-256 is `265921a4f1b12a722a2cc0c6127a31c105e8451207fd77f6569003158b33086c`;
preparation seal SHA-256 is `e589b26a26808dd39e46d14838ed6b51326eee41fe8a2a02c550ad107845dba1`.
Strict Node/YAML output and all three strict Kubernetes resources pass; the pinned image scan
reports zero fixable HIGH/CRITICAL findings. The wrapper verifies 86 runtime PNGs and 24 runtime
image/security records, 12 distinct Kubernetes pod/forward identities and 22 seed-mode records
across eleven mode-0444 fixtures. Independent live cleanup passes for owned resources, forwards
and ports. At this native-acceptance checkpoint, default localhost:8085 remained the separately
published v0.3.32 image; the later v0.3.33 deployment is recorded above.

Root actually viewed both fresh Compose captures: English desktop 1440×2295, SHA-256
`cc7e4d803ac8078bc7dbf13e0fbe7bfad67eff5e90b4c7d59313bbd98cb315c3`, and Polish 320×3065
reduced motion, `e53ca7b0979402155738f00d77c992f02a2fbda841a8a633560160514f4ddda6`.
Both show the exposed pickup before collection, with no active curse/timer. Layouts are readable
without observed overlap; the Polish browser scenario asserts horizontal overflow is absent.
Kubernetes captures are not separately viewed. The frozen source's 29 tracked PNGs are distinct
from the 86 runtime captures. Earlier failed and unexecuted preparations remain immutable history.
Independent native and documentation reviews are clear. The 1916-entry `FINAL.sha256` seal
hashes to `b4f287107099c3cbf17940493bb0a550c50336fffa9c751e79d9ece72b34906d`.
All 1917 evidence files and the source archive are copied and byte-verified outside `/tmp` in the
private checkpoint, excluded from Git and Docker. Exact-head CI, merge, post-merge CI and
publication remain separate release gates.

## Slow-foot accepted-parent integration — 2026-10-20 — before native acceptance

Final raw Diarrhea source `a5787b297d611c51c31523401823a1211470dd13` is integrated,
including the startup-readiness correction, stable Long Fuse cursor, robust predecessor drivers,
accepted Long Fuse, Reverse Controls and Diarrhea records, final v0.3.32 evidence and captures,
and the private checkpoint Docker exclusion. The v0.3.33 version, Slow-foot behavior and canonical
nine-item catalog remain. Every predecessor item driver matches the reviewed parent except for
that catalog extension. Core, server, API, Cargo and product web inputs remain byte-unchanged from
the reviewed Slow-foot tree `c5a7bdeebe677fa9ec0979ace950623dc1a290f3`.
All 882 web tests across 45 files pass with 95.43/91.25/95.59/97.77% coverage, together
with contract checks, lint, Knip and the production build. E2E lint, Knip, type checking,
47-scenario parity, 22-case latency discovery, repository and diff checks pass under Node 24.21.0.
Independent integration review of the earlier source is clear. Fresh whole 46+46 Slow-foot native
acceptance (**92 total**) and exact-head CI remain pending. Diarrhea's accepted matrix supplies no carried
Slow-foot result. The accepted Diarrhea parent is published separately as v0.3.32.

## Slow Foot first native attempt — 2026-10-20 — failed route

Frozen source `5b6845d014fc40a781227611ef8bf257259f5a91` builds image
`sha256:ae1e5a23bd282a202a076484137f24e21f2fb7cccfd6df1a7e0e7eb085f670fd`.
The whole wrapper exits **1**: **45 Compose cases pass, Slow Foot fails, and all 46 Kubernetes
cases are unattempted**. Sixteen groups pass, including all 22 latency cases. Source, dependency,
lock, mode, tar, preparation-seal and image checks match after execution; owned cleanup passes.

Independent review confirms a test-route defect: the actor escapes to `(1,2)`, then the default
X-first movement helper tries to reach `(3,1)` by walking into permanent pillar `(2,2)`.
The east command was observed, both failed captures show the actor at the escape tile with the
pickup uncollected, and the position wait times out. The failure precedes collection and every
Slow Foot speed/expiry assertion. The correction adds the ordinary-key waypoint `(1,1)` before
moving east, preserving product code, helpers, timing, retries and all later assertions.

The failed attempt is sealed with 1458 payload entries; `FINAL.sha256` hashes to
`76581d3c4d958c06fcdd42a602b4d65806c32ff4ac8721472f3ba9455ded815f`.
All 1459 files and the source archive are byte-verified outside `/tmp` in the private checkpoint.
Nothing is reused as acceptance: a new reviewed source, archive, dependency install and image
must pass the complete **46 Compose plus 46 Kubernetes** matrix with zero retries.

## Slow Foot prepared source blocked before execution — 2026-10-20

The route-corrected source `d9f213a281ea03fe07021b3c0e7b1729725385f6` completes fresh
preparation, but independent source review blocks browser launch. Its west-wall predicate
requires `x <= 384`, while the occupied border cell plus the actor footprint clamps the center
at `x = 1344`. Rate windows can also include frames preceding the intended movement direction.
The correction requires the exact reachable wall stop and direction-matched, unclamped public
positions while preserving every 12-tick rate, peer, timer and key-release assertion.

No browser acceptance or native matrix runs against this source. The 1025-entry preparation
closure seal hashes to `e2c18f776134c3c498e5f6b4e10362b6f4be8b11e9bd150e3591cf4f885eca35`;
all 1026 files and the source archive are copied and byte-verified outside `/tmp`. This preserves
an unexecuted preparation, separate from the earlier failed native attempt; neither supplies
acceptance for the corrected source.

## Diarrhea publication — 2026-10-20 — v0.3.32 published

[v0.3.32](https://github.com/liveweird/grenadiers/releases/tag/v0.3.32) was published as a
prerelease at `2026-10-20T18:55:48Z` from merged head
[`5b82b9e72d5aef37546d33ffc82162dbe1bc3063`](https://github.com/liveweird/grenadiers/commit/5b82b9e72d5aef37546d33ffc82162dbe1bc3063).
Candidate runs [39474746135](https://github.com/liveweird/grenadiers/actions/runs/39474746135)
and [39474740382](https://github.com/liveweird/grenadiers/actions/runs/39474740382) each pass
five jobs and 45 browser cases. Post-merge run
[39477257121](https://github.com/liveweird/grenadiers/actions/runs/39477257121) likewise passes
five jobs and 45 browser cases; all three runs have zero retry and failure markers. The release
tree `4e4f0b210f7965ff7040f5db21bce695580fe39e` exactly matches both the merged-head tree and the
normalized accepted raw-source tree from `a5787b297d611c51c31523401823a1211470dd13`.

## Diarrhea native acceptance after startup readiness correction — 2026-10-20 — 90/90

Frozen source `96e8be6c3d9c3214ab4fe7a15a883c07cae03845`, archive SHA-256
`1e875463977616fd6418d179f197a4baf101c26f15855d2d67331b0f8d682f54`, builds the logged
no-cache image `sha256:e37b5708604d5554aab1615cc8c98f6fe978068ed6bbcf80bb7f45c6e168c839`.
Its saved image archive hashes to
`e70f64d9b2defb18adc9c7172bd0fcdcfb944576ae0909c3f8543bf9c1f93e6e`.
The single full wrapper invocation exits **0**: **45/45 Compose and 45/45 Kubernetes** cases,
including all eight optional item scenarios, one worker and zero retries or carried results.
The scope adds eleven focused readiness regressions; each target retains two real latency
profiles with all twenty movement/release observations. All 32 reports and 82 runtime PNGs verify.

Startup now explicitly requires the running phase, both visible canvases and actor focus,
followed once by two fresh, matching, stationary public revisions above a fixed ingress watermark.
Accepted witness and head facts prove endpoint-specific ingress → dispatch → page causality;
all measured key starts follow readiness completion. Startup stages, failures, current heads and
accepted completion remain independently diagnosable. A later profile failure cannot overwrite
successful readiness. The movement/release sampler, bomb checks, 250 ms shaped limit, retries and
timeouts are unchanged. Fresh static checks pass **46 scenario titles** and **22 latency cases**.
Product/API/Cargo inputs are byte-unchanged from the earlier locally tested Diarrhea source;
new exact-head CI must still rerun the complete server, web, security and browser gates.

| Target | Profile | Samples | Startup / fresh publication baseline | Maximum | Exceedances |
| --- | --- | ---: | ---: | ---: | ---: |
| Compose | local | 20 | 4189.946 / 81.428 ms | 55.767 ms | 0 |
| Compose | delay-50ms | 20 | 4750.198 / 145.458 ms | 164.019 ms | 0 |
| Kubernetes | local | 20 | 4148.685 / 80.641 ms | 55.409 ms | 0 |
| Kubernetes | delay-50ms | 20 | 4776.686 / 126.462 ms | 160.842 ms | 0 |

Startup includes context creation, joining, countdown and first rendering, and has no 250 ms
performance claim. These are twenty-observation reference experiments, not Internet tail-latency
or pixel-presentation guarantees. Compose has unlimited CPU; Kubernetes retains its half-CPU
quota. Both have zero observed throttle-counter deltas. Root verifies current image and, for
Kubernetes, the live pod UID and forward PID before releasing each latency gate. Three task
workers hold idle; observed unrelated host processes remain untouched. No globally idle host
or unloaded hardware benchmark is claimed.

Fresh dependencies contain **8186 files, 1305 directories and 13 symlinks**, without reused caches.
The exact installed proof has 132 installed packages, 42 optional omissions, zero missing required
packages and zero metadata/version mismatches. Dependency tar SHA-256 is
`520934df359487083f4e9d51b107a004bd362c8a465389a81161f81e4c35a3ea`.
The 682 source files, 78 directories, three locks, files/modes/symlinks, source/dependency tar bytes,
semantic proof, image IDs and 1008-entry preparation seal match before and after execution.
The preparation seal hashes to `974c9c7cf0fe809e8d099aa1596b75e306eb99d9cf194dbb7db2985005b9b12a`;
the sole runner hashes to `918359baf2302f3847b87268b631edb635c0606e7237afa767f68a1bf8006002`.
Strict Node/YAML success output and all three strict Kubernetes resources pass; the pinned image
scan reports zero fixable HIGH/CRITICAL findings. All 22 runtime records retain UID/GID65532,
zero effective capabilities, no privilege escalation and read-only roots. Twenty seeded records
use ten matching mode-0444 fixtures; eleven Kubernetes pod/forward identities are distinct.
Owned containers, network, namespace, forwards and ports are removed; independent live cleanup passes.

The first preparation of this source stopped before build or browser execution: npm11 treated
macOS's `/tmp` alias as an extra linked package absent from its otherwise matching lockfile.
An offline reproduction proves the false entry; using the physical path fixes it. This completed
attempt regenerates the archive and installs/builds into entirely fresh paths with canonical
npm prefix/cache, without changing application lockfiles. The failed setup is durably preserved
with manifest SHA-256 `4adbd58ead0f14fa3a824dd31df8acb1a5615e96a1b512b78603536da919fb8b`.
The original 408.754 ms CI counterexample and original 68/68 native source remain separate history.

Root actually viewed and copied both fresh Compose captures: English desktop 1440×2209,
SHA-256 `cd2609aa7b08d40eadaf90f52cf71e2205eb071849e74830fe67f6765b3eb3a4`, and Polish
320×2980 reduced motion, `fe08792a273d8cf804d8769eb8acecaba42f10e8dd843210f9b42da6a3b6861e`.
Both show the exposed pickup before collection, with no active curse/countdown. Layout is readable
without observed overlap; browser assertions check horizontal overflow. Automatic placement and
death/reset are assertion evidence, not visible in these stills. The frozen source's 29 tracked
PNGs are separate from the 82 current runtime captures. Independent native evidence review is
clear. The 1836-entry `FINAL.sha256` seal hashes to
`f4c1a79ec36f7cc0e8d962d9883d57011f7f37ff40549d38687e44c38fb9b811`.
The sealed 1837-file evidence copy and source archive are byte-verified outside `/tmp` under the
private `.local-checkpoints/2026-10-20/` directory, excluded from Git and Docker. Candidate CI,
merge, post-merge CI and publication remain separate gates.

## Diarrhea candidate CI counterexample — 2026-10-20 — release blocked

[PR 78](https://github.com/liveweird/grenadiers/pull/78) initially tests normalized head
`23b033df6422634cecf61f0c89ca9ad46393802b`, whose tree equals raw final source
`1bf95c6479c87607c6f81c2864a1ca4ba13186ce`. Push run
[39462992507](https://github.com/liveweird/grenadiers/actions/runs/39462992507) passes four
non-browser jobs but fails one of twenty shaped latency observations: **408.754 ms > 250 ms**.
Its browser scope ends **30 passed, one failed, three unattempted**; the later image scan is
unattempted. Separate PR run
[39462993184](https://github.com/liveweird/grenadiers/actions/runs/39462993184) passes all five
jobs and 34 cases. That independent pass does not supersede the failed push run.

The peer publication backlog starts before the first measured key. Node event-loop delay,
shaper timers, queue limits and clock arithmetic do not explain the excess; saved evidence
cannot distinguish per-socket delivery from browser/runner scheduling. No underlying product
or platform cause is proven. The 23-entry private failed-evidence manifest hashes to
`93f1dccca18173a6044d833cf82a4c834af6f0d6c712d6874b753792a61795d1`.

Independent architecture review identifies a setup gap: round text can precede asynchronous
first arena rendering, and the round number also appears during countdown. The revised experiment
requires the actual running phase, both painted canvases and a fresh synchronized public baseline
once before any measured input, with separately reported startup stage/times and full sanitized
witness facts on success and failure. Eleven focused readiness cases are added: latency discovery
is 22 cases and static scenario parity is 46. Focused helper tests and the E2E static gate pass.
All twenty observations, the 250 ms limit, zero retries and existing timeouts remain.
Fresh whole native acceptance and new exact-head CI are required for the revised harness;
the following 68/68 result remains historical evidence for its recorded source only.

## Diarrhea fresh native acceptance — 2026-10-20 — 68/68

Frozen source `bdf33cee9aaf4e5d43bc7fbc663970043eabd255`, source archive SHA-256
`609b64b312a0eb9f31906e1f58851cc2504d14c38faf7b43286451d381aab346`, builds the logged
no-cache image `sha256:df3ec56ebb73ee246df4cbe38255843efcc920e2d5df636b47e49d49748245bf`.
Its saved image archive hashes to
`ab3ceb25228e9deeddb2c09a2c6cd25f740444c60e6b62ed4570197df1cae9c6`.
The first and only final wrapper invocation exits **0**, passing **34/34 Compose and 34/34
Kubernetes** cases with all eight optional scenarios, one worker and zero retries, reruns or
carried browser results. All 32 reports and 82 configured runtime PNGs verify. Frozen source
also contains 27 tracked PNGs; those are not counted as current runtime captures.

Fresh Node 24.21.0 installation supplies **8,186 files, 13 symlinks and 1,305 directories**,
without copying or padding caches. The actual npm-generated hidden lock verifies 132 exact
installed packages against the frozen lock, with 42 optional omissions and no missing required
packages or metadata/version mismatches. Pre/post files, directory modes, symlinks, dependency
tar, source files, all three relative lock records, image IDs and preparation seals match.
The dependency tar SHA-256 is
`f4aefcce0df9b15da23cf99158277ea981a64e7f6b06a8f41c196d29fb2c24a7`.
The pinned browser retains Playwright 1.63.0 and Chrome 153.0.8010.12. Its reviewed harness
disables jiti filesystem caching and invokes the installed Playwright CLI directly.

An initial preparation attempt stopped before any browser ran: the Node/YAML CLI entry guard
skipped validation through the `/tmp` alias, and required success output was absent. That
attempt remains at `/tmp/grenadiers-v0332-oct20-native-setup-failure-1`. The completed source
archive, fresh dependency installation and logged no-cache image are explicitly reused as
pre-browser inputs; no browser result is carried. Final preparation invokes the actual checker
through `/private/tmp`, requires its exact success output and validates all three resources
with checksum-verified kubeconform 0.8.0, strict Kubernetes 1.33 schemas. The pinned Trivy 0.74
archive-only scan reports zero fixable HIGH/CRITICAL findings.

The immutable base, reviewed-wrapper addendum and final-wrapper seals are respectively
`303a59a2601a3aaf561394d465f1b2abf0f6e8dab96b3adcd1fa58e3b6ff5716`,
`9c5ae140f22dde8349a92bab110d4defd6e6446f21a7e34da9a7a2c79dc9e3a2` and
`ccbfec6f4ff9842692313302ce5acb6aa0d7b6ca2aa5461eb480ec8bcbc93f6d`.
Only `run-v0332-native-final.sh` executes the matrix; its SHA-256 is
`1f9d3a387889a4ff44b973acf24867ccaa2f41666638776614b798f331e5679a`.
Earlier sealed wrappers remain unchanged as preparation history.

All 22 runtime records use the accepted image with UID/GID 65532, zero effective capabilities,
no privilege escalation and a read-only root. Ten external mode-0444 fixtures match frozen
source; 20 seeded runtime records retain that mode. Kubernetes records 11 unique pod UIDs
and 11 serial port-forwards with checked liveness and complete termination. Owned containers,
network, namespace, listeners and forwards are removed; independent live cleanup checks pass.

All four latency profiles retain 20 samples and zero exceedances. Compose local/delay-50ms
maxima are **65.882/178.928 ms**; Kubernetes maxima are **48.430/144.079 ms**. Coordinator
approvals record fresh scoped worker holds and the current Kubernetes pod/forward identity.
Unrelated `webbench` load was observed and left intact; these records claim task-owned quiet
coordination, not a globally idle host or a controlled hardware benchmark.

Root actually viewed and copied the fresh [English desktop](screenshots/diarrhea-desktop.png)
and [Polish 320px reduced-motion](screenshots/diarrhea-polish-320.png) images. Both show the
exposed pickup **before collection**, with no active curse/timer; layout was readable without
observed overlap, and the browser asserts no horizontal overflow. Automatic placement and
death/reset are proved by assertions rather than these stills. Their SHA-256 values are
`1fc5dd81a5f54958422b6ea85bf92ca03dafa94e38bd4122ef806cbb0b511a68` and
`f93728569d91879175ac7e9f36ff380b103e271318324d349a61427915c7e4af`.
Product and test inputs remain unchanged after the accepted source; only documentation and
these copied captures change. Independent final evidence, visual and documentation review is clear.
The 1,848-entry `FINAL.sha256` seal at `/tmp/grenadiers-v0332-oct20-native-acceptance` hashes to
`f5943f7e78d22b4b7f7131d16eb012262ef2b0963c4b1da9e95231422993f102`.
Byte-verified copies of the sealed evidence, initial setup attempt and source archive are retained
outside `/tmp` under `.local-checkpoints/2026-10-20/`, privately excluded from Git and Docker.
Exact-head CI, merge and publication remain separate gates.

## Diarrhea final-parent integration checkpoint — 2026-10-20

Final raw Reverse Controls source `7cf5a5e86f190789103529e7a810bc7586d6e7d8` is integrated,
including the stable round/tick placement cursor, final Long Fuse and Reverse Controls native
records, guide corrections, and all four accepted captures. The v0.3.32 version and canonical
eight-item catalog remain. Product sources, API, Cargo inputs and every other test input are
unchanged from `d3c1774`; predecessor scenarios match the reviewed parent apart from the
canonical catalog. At source freeze, fresh whole 34+34 Diarrhea native acceptance and exact-head
CI remained required; predecessor acceptance supplied no carried Diarrhea result.

Integrated source `bdf33cee9aaf4e5d43bc7fbc663970043eabd255` passes the current local gates:
495 core tests, 188 server tests and two doctests in each Rust profile, Rust line coverage
91.58%, 617 web tests with unchanged coverage floors, 35 static scenarios, both dependency
audits with zero vulnerabilities, cargo-deny and strict validation of three Kubernetes resources.
An initial restricted server execution could not bind local sockets; the unchanged gate passes
with the required permissions. Initial missing-tool setup and the socket failure are retained
separately. Independent integration review is clear. Fresh same-image native acceptance
subsequently passes as recorded above; final exact-head CI remains pending.

## Predecessor publication status — verified 2026-10-20

Reverse Controls is published as [v0.3.31](https://github.com/liveweird/grenadiers/releases/tag/v0.3.31)
from `2b464744822dd08582e3769f5611fecef43e2c0e` through [PR 77](https://github.com/liveweird/grenadiers/pull/77).
Candidate runs `39120943388` and `39120940237` pass all ten jobs; post-merge run `39122032780`
passes all five. The additional tag run `39123070723` also completes successfully: exact release
head, five passing jobs, 33 browser cases, no failure, flaky, skipped, retry or timeout result
markers. Long Fuse v0.3.30 is also published; its additional tag run `39120430011` passes.
Pending statements in older source-specific sections describe their recorded checkpoint.

## Reverse Controls fresh native acceptance — 2026-10-18 — 66/66

Frozen source `ba4aa61a16488d3f39d5ee0bd69d2b3956f68f3d`, archive SHA-256
`8a46016482bf36ff903aeaf5f76af5a720622861ac990a2d9cbf35ad2d6c08e6`, builds fresh no-cache
image `sha256:7f44bad5be71802e4c35336f0ab53736b75099351e974b34d15b1c54f807bd2e`.
Its saved archive hashes to
`928044de2017ec1ce25d119dd6a2f4e5124bac0ee65127bc2a5f49f531010a9d`.
The first and only wrapper invocation exits **0**, passing **33/33 Compose and 33/33 Kubernetes**
cases, including all seven optional item scenarios. All 30 HTML reports and 78 configured PNGs
verify, with no failure screenshots, retries, reruns or carried results. E2E cursor assertions run
from separately mounted frozen source; the OCI image contains compiled product assets.

The pinned browser runs Node 24.21.0, Playwright 1.63.0 and Chrome 153.0.8010.12. Twenty hardened
runtime records verify UID/GID 65532, dropped capabilities, no privilege escalation and read-only
roots. Nine external mode-0444 seeds drive ten fixture identities per target; Kubernetes retains
ten unique pod UIDs and ten serial direct-pod port-forward lifecycles. Strict 200/JSON/no-store
probes, pinned archive-only Trivy 0.74 with zero fixable HIGH/CRITICAL findings, the frozen
Node/YAML checker with required success output, and strict Kubernetes 1.33 validation of all three
resources pass. Final containers/network/namespace/process/port cleanup passes.

All four latency profiles retain 20 samples, zero exceedances and zero throttling. Compose
local/delay-50ms maxima are **54.896/164.940 ms**; Kubernetes maxima are **55.388/151.280 ms**.
Root retained fresh scoped worker acknowledgments and actual current port-forward checks,
releasing each quiet hold only after the respective eleven-case latency result passed.

Actual pre/post dependency snapshots match: 8,215 files, 13 symlinks, 132 exact installed lock
entries and 42 optional omissions. Of these inputs, **8,186 package files came from fresh Node 24
installation and 29 jiti cache inputs were reused from the previously accepted pinned dependency
input**. All current hashes, tar snapshots, metadata and proofs were generated afresh; no browser
result was carried. Discarded Node 26 setup, replacement of an uncaptured build by the final
logged build, and the corrected browser probe are retained as setup history. Frozen source,
relative source locks, image IDs and the 1,015-entry preparation manifest match after execution.
Its SHA-256 is `eca5a6fd567aa19fab939845f6bbf67783f0c810c8742f66382007c93fa25e57`.

Sealed evidence at `/tmp/grenadiers-v0331-event-cursor-native-acceptance` includes RESULT,
root visual review, quiet coordination and independent guard records. Its 1,799-entry final
manifest hashes to `fcd6be180127f42976e333734354e33dc90367c40dab30624c24579b55d43e76`.
The original RESULT overstates the still-image review by mentioning an active effect/timer.
A separate seven-entry visual erratum at `/tmp/grenadiers-v0331-visual-record-correction`,
manifest SHA-256 `ddb5943ae194dcfb75383b8fb5599b48a1d25c657af2684e939a364d38cd937e`,
corrects that wording without modifying the original seal or claiming a rerun.

Root actually viewed and copied the fresh [desktop uncollected pickup](screenshots/reverse-controls-desktop.png)
and [Polish 320px after expiry/release](screenshots/reverse-controls-polish-320.png). Layout and
localized settings were readable without observed overlap/clipping; exact inversion and expiry
are established by browser/domain assertions. The capture SHA-256 values are
`b933ce3d3356433954c0a5332dfe3eb501432ad75a5aebc006e2d8d52eaddffa` and
`d1328d33e130935218e534fba78516f8f0ff734f13ddbe16e81fbfd9906db3da`.
Product and test inputs are unchanged after the accepted source; subsequent edits are only
this documentation and copied captures. Existing local gates remain 479 core, 186 server and
two doctests in each Rust profile, 482 web tests, 34 static scenarios and unchanged coverage
floors. Independent final evidence, erratum, capture and documentation review is clear;
exact-head CI remains pending.

The preceding Long Fuse release is now published as
[v0.3.30](https://github.com/liveweird/grenadiers/releases/tag/v0.3.30), merged in
[PR 76](https://github.com/liveweird/grenadiers/pull/76). Candidate runs `39118087194` and
`39118084669` passed all ten jobs; post-merge run `39119339836` passed all five jobs against
`894d6fe45a33aada83ca0862c265817220f31db1`, each browser run passing 32 cases. Older pending
sections below are chronological records for their named source, not current release status.

## Reverse Controls stable-cursor parent acceptance integration — 2026-10-18 — native pending

The final accepted Long Fuse integrated parent `6cd809fea81a9eceef06b2f0a2e7ff7ebeb6b9ef`
is integrated with its corrected-source 64/64 evidence and actual captures. This preserves
v0.3.31, the canonical seven-item catalog and the complete stable-cursor driver. Product,
API, Cargo, version and every test input are byte-unchanged from reviewed `1393950`; only
documentation and copied parent captures change. Existing 482-web-test and 34-static-scenario
gates remain applicable. Repository, diff and local document links pass; independent integration
review is clear. Long Fuse exact-head CI/publication remains independently pending. Reverse
Controls requires fresh whole 33+33 same-image native acceptance and exact-head CI. Its earlier
15-case interrupted record supplies no carried results.

## Long Fuse stable-cursor native acceptance — 2026-10-18 — fresh 64/64

Corrected source `e0ffd635f9aa0b25c1f2dc00a5d20d8fc00be831`, archive SHA-256
`932bb85f796d87502dfa235f25b06fbd02a3735080eb0cfd915067b86a85f146`, builds fresh no-cache
image `sha256:55aa95c71be6c3373fb119100df3e8d5cb03e9e0738e948c53e7ed2dc4d692f4`.
Its saved archive hashes to
`97ee74a408d244807faf7dc1c0ddae4e43d76b568e835d64b308f1f58490b026`.
The first and only wrapper invocation exits **0**, passing **32/32 Compose and 32/32 Kubernetes**
cases, including all six optional item scenarios. All 28 reports and 74 configured PNGs verify,
with no failure screenshots, retries, reruns or carried results. The stable placement cursor
and valid-zero assertion run from separately mounted frozen E2E source; the application image
contains the compiled product assets, including the current Gear label and Long Fuse artwork.

The pinned browser is Node 24.21.0, Playwright 1.63.0 and Chrome 153.0.8010.12. Eighteen runtime
records verify UID/GID 65532, dropped capabilities, no privilege escalation and read-only roots.
Eight external mode-0444 seeds drive nine fixture identities per target; Kubernetes retains nine
unique pod UIDs and nine serial direct-pod port-forward lifecycles. Strict 200/JSON/no-store
probes, archive-only pinned Trivy 0.74 with zero fixable HIGH/CRITICAL findings, the frozen
Node/YAML checker with required success output, and strict Kubernetes 1.33 validation of all
three resources pass. Final containers/network/namespace/process/port cleanup passes.

All four latency profiles retain 20 samples, zero exceedances and zero throttling. Compose
local/delay-50ms maxima are **60.805/159.221 ms**; Kubernetes maxima are **110.890/155.845 ms**.
Root coordination retains separate fresh scoped worker acknowledgments and actual current
port-forward checks. Heavy holds end only after the respective eleven-case latency result is
zero; acknowledgments concern cooperating workers, without claiming global host idleness.

Actual pre/post dependency snapshots match: 8,215 files, 13 symlinks, 132 exact installed lock
entries and 42 optional omissions. Both dependency tar snapshots hash to
`daccf3269e949a85682437e8ffdf9694ef4d63c8b465f12c2ce8a595c0afcf4b`.
Frozen source, source archive, all three identically relative lock records, image IDs and the
1,001-entry preparation manifest match after execution. This wrapper needs no lock-path repair.
The preparation manifest SHA-256 is
`2ba05392949480f5904b4e4a96679b19748eb8ab906f9ac9995a199a4c287516`.
Sealed evidence at `/tmp/grenadiers-v0330-event-cursor-native-acceptance` includes RESULT,
root visual review and quiet coordination; its 1,727-entry final manifest hashes to
`bba883f044653dccc4bb23e2dceda52bf7c3cd9fc6403c9ea6294c740e421f40`.
Earlier failures, interrupted runs and bookkeeping/provenance exceptions remain separately
recorded below and supply no cases or captures to this acceptance.

Root visually reviewed and copied the fresh [desktop pickup](screenshots/long-fuse-desktop.png)
and [Polish 320px active curse](screenshots/long-fuse-polish-320.png). Their SHA-256 values are
`8340bf832e232e427bc3aac7545ab2c042cd4f39ee9ce0d72fe7bcbd2a732640` and
`47558bfd4482048979c48d2160088322d58b53cba0189cf53f09b57837840cbb`.
Product/test inputs remain unchanged after this accepted source; later changes are documentation
and copied captures. The local baseline remains 466 core, 184 server and two doctests in each
Rust profile, 409 web tests and 33 static scenarios, with unchanged coverage floors.
Independent final evidence review is clear; exact-head CI remains pending. Publication is recorded in
[release notes](https://github.com/liveweird/grenadiers/releases).

## Reverse Controls bounded-history integration — 2026-10-18 — native acceptance pending

The reviewed Long Fuse cursor correction `e0ffd635f9aa0b25c1f2dc00a5d20d8fc00be831` is
integrated. The inherited Long Fuse scenario matches that source except for the canonical
seven-item catalog. Product sources, API, Cargo inputs, version and all other test inputs are
byte-unchanged from `6ac0fc2`; the existing product checks remain applicable. E2E lint, Knip,
type checking, 34-scenario parity, repository and diff checks pass. Independent integration
review is clear. Long Fuse requires its corrected fresh native acceptance and final-head CI;
Reverse Controls then requires that final parent and a new whole 33+33 run. The interrupted
15-case record below supplies no carried acceptance.

## Long Fuse bounded-history cursor correction — 2026-10-18 — fresh acceptance pending

Candidate `fbc8593088de89dd60ed2457af508240994f83fa` failed both first hosted CI runs:
PR run `39114204134` and push run `39114201841`. Each passed four non-browser jobs and
14 browser cases, then failed Long Fuse; the later 17 classic-match/reconnect/latency/control/
headed/audio cases and built-image scan were unattempted. Neither run was rerun.

Both retained public attachments prove the third manual action placed bomb ID 4 and both peers
observed its exact 120-tick detonation (PR ticks 2174→2294; push 1958→2078). The observer shifts
its bounded 512-frame array, while the driver saved its length as a fixed slice index. Once full,
`slice(512)` stays empty even when the placement event is retained. Placement now selects events
strictly after the preceding physical tick in the same round, independently of array rotation.
Manual IDs, valid-zero IDs, captured fuses, escapes, reconnect, expiry, timeouts, retries and
remaining assertions are unchanged. Failure diagnostics also record the retained frame count.

A focused proof executes the actual old/new helpers against a rotated 512-frame history: the
old lookup misses a retained ID 0, the corrected lookup finds it, and prior-tick/different-round
events remain excluded. E2E lint, Knip, type checking, repository checks and 33-scenario parity
pass. The sealed hosted evidence at `/tmp/grenadiers-v0330-ci-failure` has 37 entries and manifest
SHA-256 `9610fa8e390f24c1461b6ffc1e42b7b8916ed5bc8bf5d855d7c1b371087ed79c`.
Fresh source, image, whole 32+32 native acceptance and final-head CI are required. The earlier
64-case record below certifies only its named source, not this driver correction.

The inherited Reverse Controls source `6ac0fc2` was stopped at its Compose latency gate before
approving measurements or starting optional/Kubernetes cases. Fifteen Compose cases passed and
none failed; the directed interruption returned 130. Its isolated evidence at
`/tmp/grenadiers-v0331-native-acceptance` seals 1,207 entries with manifest SHA-256
`2aeca61865b4e86fcbac13142518c7d1fdc6af4db2ace5fe035e13a7c4659156`.
All post-run integrity and cleanup checks pass, but it has no accepted or carryable matrix.
Reverse Controls requires the corrected final parent and a fresh whole 33+33 run.

## Reverse Controls final-parent integration — 2026-10-18 — native acceptance pending

The final audited Long Fuse parent `2ffb85fcd476bce0fdf1c5fbc979fd78234afc8b` is integrated,
including all twelve accepted predecessor screenshots and the complete corrected timing drivers.
The v0.3.31 version and canonical seven-item catalog remain. Product sources, API, Cargo inputs
and every test input are byte-unchanged from the reviewed integration `9df6c3f`; its 482-test web
and 34-scenario static gates therefore apply without rerunning them. Repository, diff and local
document links pass. Long Fuse PR #76 final CI/publication is pending independently. Fresh
33+33 same-image native acceptance and final-head CI are required for Reverse Controls.

## Diarrhea predecessor-driver integration — 2026-10-18 — acceptance pending

Reviewed Reverse Controls predecessor source `9df6c3fe7026a5a03acc081e1988cde6697a7ef9`
is integrated, including published Short Flame parent assets and corrected predecessor timing
scenarios. The v0.3.32 version and exact eight-item catalog remain. Parent native acceptance and
publication are still pending; their separate results cannot certify Diarrhea. Fresh 34+34 native
acceptance and final-head CI remain required after integrating the final Reverse Controls parent.
Integration checks pass under Node 24.21.0: all **617 web tests** across 43 files, coverage
**95.24/91.03/95.53/97.63%** (statement/branch/function/line), contracts, lint, Knip and
production build, plus repository checks and E2E lint, Knip, type checking and **35-scenario
parity**. Core, server, API and Cargo inputs remain byte-unchanged from the reviewed Diarrhea
candidate. All seven predecessor item scenarios exactly match the reviewed parent apart from
the canonical eight-item available catalog. These checks establish no native acceptance for
Diarrhea.

Records below are frozen against their named sources. Publication and later CI status are
recorded in [release notes](https://github.com/liveweird/grenadiers/releases).

## Power renderer and responsive-HUD integration — 2026-10-18 — native acceptance pending

The reviewed terrain cache, diagnostics and responsive HUD are integrated into the Power Bomb
candidate. Core, server, Cargo, API, client semantics/locales and the dedicated Power scenario are
unchanged. The scene-suite Pixi mock moves verbatim to a test-support module to keep the scene
suite below 400 lines without removing its Power and Piercing inventory-cue assertions. Updated
web, E2E static, diagnostic-build and production-fixture results are recorded below. Fresh native
acceptance and exact-head CI remain required.

## Power Bomb candidate — 2026-10-18 — local checks complete

The v0.3.35 candidate activates an opt-in one-charge Power Bomb. Successful manual placement
captures every floor cell in its Manhattan range, treating hard walls as intact, unburned holes
without blocking eligible cells beyond them. Automatic placements remain ordinary, failed
placements retain the charge, and existing bombs remain immutable. Catalog bit 7 and the final
generator weight preserve all 1024 earlier paths while supporting all 2048 subsets. The repeating
byte-31 fixture places the pickup at (3,1), a hard wall at (2,2), and a breakable block at (3,2).

Focused core and protocol regressions cover exact footprints, immutable mixed chains, obstruction
snapshots, ID permutations, weapon replacement and consumption, transactional failures, both
recipient streams, event-only records, replay and the ordinary seed route. Web regressions cover
the enabled capability guard, reserved triggers, localized settings and distinct pickup, held and
placed-bomb cues.

Debug and release suites pass **558 core tests, 191 server tests and two doctests**; four dedicated
performance profiles remain ignored. Rust line coverage is **91.84%** (12,544/13,659), above the
unchanged 80% floor. All **2,429 web tests** across 49 files pass with statement/branch/function/
line coverage of **95.49/91.39/95.61/97.81%**. Formatting, all-target Clippy with warnings denied,
repository limits, generated contract checks, lint, Knip, production and renderer-diagnostic builds,
E2E static checks and **38-scenario parity** pass. Cargo-deny and both cached offline npm audits pass
with zero npm vulnerabilities; strict Kubernetes schemas validate all three resources. Independent
backend and frontend/E2E reviews are clear.
Native browser acceptance, candidate-image checks and final-head CI remain pending.

## Piercing Bomb candidate — 2026-10-18 — local checks complete

The v0.3.34 candidate activates an opt-in one-charge Piercing Bomb. Successful manual placement
captures a blast that crosses breakable blocks; automatic placements remain ordinary, failed
placements retain the charge, and existing bombs remain immutable. Catalog bit 6 and a final
generator weight preserve all 512 earlier paths while supporting all 1024 subsets. The repeating
byte-31 fixture places the pickup at (3,1) with consecutive breakable blocks at (4,1) and (5,1).

Debug and release suites pass **534 core tests, 190 server tests and two doctests**; four dedicated
performance profiles remain ignored. Rust line coverage is **91.74%** (12,432/13,551), above the
unchanged 80% floor. All **1,399 web tests** across 47 files pass with statement/branch/function/
line coverage of **95.44/91.36/95.59/97.78%**. Formatting, Clippy, repository limits, generated
contract checks, lint, Knip, production builds, E2E static checks and **36-scenario parity** pass.
Cargo-deny, both cached offline npm audits with zero vulnerabilities, and strict Kubernetes
schemas for all three resources pass. Dependency graphs remain unchanged except for the two
workspace package versions in `Cargo.lock`. Independent backend/frontend review is clear; the native Piercing Bomb
playthrough and screenshots, candidate-image acceptance, and final-head CI remain pending.

Review regressions reject Remote, Contact and paused Timed bombs consistently in live state and
placement/detonation events. They also cover a manual Piercing request conflicting with an
automatic Diarrhea request, event-only Piercing detonation, and a live Piercing bomb whose dead
owner has an empty inventory. The browser scenario verifies both target blocks before planting,
compares both peers' captured Piercing record, and fences historical placement IDs before proving
the later ordinary bomb. All 22 focused Piercing Rust tests, the focused web regression, the full
web gate, and E2E static checks pass after these review fixes; the full Rust counts above are the
unchanged production baseline from before these test-only Rust additions.

## Slow-foot renderer and responsive-HUD integration — 2026-10-18 — native acceptance pending

The reviewed terrain cache, diagnostic fixtures and two-column narrow HUD are integrated.
Core, server, Cargo, API, Slow-foot client/protocol/locales and dedicated item scenario are
unchanged. Shared scene setup and the Slow-foot scene test now use the renderer resolution.
All 882 web tests across 45 files pass, with coverage 95.43/91.25/95.59/97.77%, plus
contracts, lint, Knip, production and diagnostic-fixture builds, and E2E static parity for
36 scenarios. Independent cache/responsive integration review is clear. Initial integration
checks caught the 400-line guard and the new scene test's missing cache mock; both were corrected
without changing assertions or limits. Fresh native acceptance and exact-head CI remain
required; inherited parent evidence does not certify Slow-foot.

## Slow foot candidate — 2026-10-18 — local checks complete

The v0.3.33 candidate activates opt-in Slow foot as up to six independent 600-tick stacks. Each
stack halves the exact current movement rate. Collection affects the following tick, expiry
precedes movement, and capped actors are excluded before pickup distance and tie arbitration.
Catalog bit 15 and a final generator weight preserve all 256 earlier mask paths while supporting
all 512 subsets. Protocol, replay, localized presentation and a deterministic browser scenario
are included.

Debug and release suites pass **511 core tests, 189 server tests and two doctests**; four dedicated
performance profiles remain ignored. Rust line coverage is **91.60%** (12,337/13,468), above the
unchanged 80% floor. All **880 web tests** across 45 files pass with statement/branch/function/line
coverage of **95.40/91.26/95.57/97.76%**. Formatting, Clippy, repository limits, generated
contract checks, lint, Knip, production builds, E2E static checks and **35-scenario parity** pass.
Cargo-deny, both cached offline npm audits with zero vulnerabilities, and strict Kubernetes
schemas for all three resources pass. Dependency graphs remain unchanged except for the two
workspace package versions in `Cargo.lock`. Independent backend and frontend reviews are clear;
native browser acceptance, candidate-image checks and final-head CI remain pending.

Review fixes require both recipient streams to agree at
identical physical-tick endpoints for slowed and restored movement, and verify all six independent
timers in the eight-field English and Polish HUD. All 21 focused web tests and E2E static
checks pass after those fixes. Component tests verify the six-stack presentation structure,
but an actual six-stack 320px native playthrough is not run;
the production-seed native scenario intentionally collects one stack and remains pending.

## Diarrhea terrain cache integration — 2026-10-18 — refreshed native acceptance pending
The current terrain-cache/shared-diagnostics integration passes all 617 web tests across 43 files, with coverage
of 95.24/91.03/95.53/97.63% above the unchanged 85% floors, production and diagnostic-fixture builds, and E2E
static checks with 35 documented scenarios. Core, server, version and API content are unchanged
by this integration. Diarrhea has no completed native baseline. The inherited Heart-only record
below certifies source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`; it includes neither
Diarrhea, Reverse Controls, Long Fuse, Short Flame, Short Fuse, Golden Flame, Shield nor the terrain cache. Fresh native acceptance and final-head CI remain required for this tree.

## Diarrhea candidate — 2026-10-18 — local checks complete

The v0.3.32 candidate activates opt-in Diarrhea for exactly 600 physical ticks. Each seat selects
one retained manual request or one automatic ordinary intent, and both seats share the existing
eligibility and conflict resolution. Collection begins automatic placement on the following tick;
expiry suppresses it at the exact boundary. Catalog bit 14 and a final generator weight of 15
preserve all 128 earlier item-mask paths while supporting all 256 subsets.

Debug and release suites pass **495 core tests, 188 server tests and two doctests**; four
dedicated performance profiles remain ignored. Rust line coverage is **91.58%**
(12,312/13,444), above the unchanged 80% floor. All **616 web tests** across 43 files pass with
statement/branch/function/line coverage of **95.21/91.03/95.51/97.62%**. Formatting, Clippy,
repository limits, generated contract checks, lint, Knip, production builds, E2E static checks
and **34-scenario parity** pass. Cargo-deny, both cached offline npm audits with zero
vulnerabilities, and strict Kubernetes schemas for all three resources pass. Dependency lock
graphs remain unchanged except for the two workspace package versions in `Cargo.lock`.
Independent backend and frontend/browser-scenario reviews are clear. Review regressions verify
the same automatic bomb and exact natural detonation on both peers without extra manual actions,
then clear the curse on death. Focused settings and E2E static checks pass after the copy/assertion
fixes. Native browser acceptance, candidate-image checks and final-head CI remain pending.

## Reverse Controls predecessor-driver integration — 2026-10-18 — acceptance pending

The reviewed source `6b818f1e7b3663fa05a23ad6d77950f7d677bef7` is integrated, including
published Short Flame parent assets, corrected predecessor timing scenarios and the valid-zero
Long Fuse placement assertion. Long Fuse native acceptance is still running against its separate
source/image; no result from that run certifies Reverse Controls. The v0.3.31 version and exact
seven-item catalog remain. Fresh 33+33 native acceptance and final-head CI remain required after
integrating the final Long Fuse release parent.

Integration checks pass under Node 24.21.0: all **482 web tests** across 41 files, coverage
**95.20/90.94/95.51/97.61%** (statement/branch/function/line), contracts, lint, Knip and
production build, plus repository checks and E2E lint, Knip, type checking and **34-scenario
parity**. Core, server, API and Cargo inputs are byte-unchanged from the reviewed Reverse
Controls candidate; all six predecessor item scenarios match the reviewed Long Fuse source
apart from the exact seven-item catalog. Independent integration review is clear. These
checks establish no native acceptance for Reverse Controls.

## Reverse Controls terrain cache integration — 2026-10-18 — refreshed native acceptance pending
Records below are frozen against their named sources. Publication and later CI status are
recorded in [release notes](https://github.com/liveweird/grenadiers/releases).

The current terrain-cache/shared-diagnostics integration passes all 482 web tests across 41 files, with coverage
of 95.20/90.94/95.51/97.61% above the unchanged 85% floors, production and diagnostic-fixture builds, and E2E
static checks with 34 documented scenarios. Core, server, version and API content are unchanged
by this integration. Reverse Controls has no completed native baseline. The inherited Heart-only record
below certifies source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`; it includes neither
Reverse Controls, Long Fuse, Short Flame, Short Fuse, Golden Flame, Shield nor the terrain cache. Fresh native acceptance and final-head CI remain required for this tree.

## Reverse Controls candidate — 2026-10-18 — local checks complete

The v0.3.31 candidate activates opt-in Reverse Controls for exactly 600 physical ticks. Raw held
directions and recency remain authoritative while simulation movement uses their opposite. The
collection tick moves normally, the following tick starts inversion, and the exact expiry tick
returns to ordinary movement. Pending turn assistance clears only when inversion state changes.

Catalog bit 18 and a final weight of 15 preserve the previous 64 generation paths and extend the
supported configuration set to all 128 subsets. The deterministic seed fixture places Reverse
Controls at cell (3,1). Protocol, replay, web and browser-scenario coverage are included.

Debug and release suites pass **479 core tests, 186 server tests and two doctests**; four
dedicated performance profiles remain ignored. Rust line coverage is **91.56%**, above the
unchanged 80% floor. All **481 web tests** across 41 files pass with statement/branch/function/
line coverage of **95.17/90.95/95.49/97.60%**. Formatting, Clippy, repository limits, generated
contract checks, lint, Knip, production builds, E2E static checks and **33-scenario parity** pass.
Cargo-deny, both cached offline npm audits with zero vulnerabilities, and strict Kubernetes
schemas for all three resources pass. Dependency lock graphs are unchanged except for the two
workspace package versions in Cargo.lock. Independent backend and frontend/browser-scenario
reviews are clear. Review fixes prove every held command remains raw east and key release stops
both peers across three later snapshots spanning six physical ticks; focused Pixi/cue tests pass.
The native Reverse Controls playthrough and screenshots, candidate-image acceptance, and
final-head CI remain pending.

## Long Fuse native acceptance — 2026-10-18 — fresh 64/64

Clean source `6b818f1e7b3663fa05a23ad6d77950f7d677bef7`, archive SHA-256
`4de08782230881d6e9d8d6d25dfaf949bf68fc12529e8d04e0d3ec42ae0bac7e`, builds the fresh
no-cache image `sha256:89a1158f3bbd726ecee2625353e2c1b03cd269b8e341089afc15a1ae228d16bd`.
Its saved archive hashes to
`1b85fdb53c229d719c82036e30289edb17e8ad2a8f713315df1e0b666c853e00`.
That exact image passes **32/32 Compose and 32/32 Kubernetes** browser cases, first attempts
with one worker and zero retries, reruns or carried passes. All 28 group exits are zero;
28 reports retain exactly 74 configured PNGs and no failure captures. The six optional cases
retain all predecessor coverage and add the ordinary generated Long Fuse pickup route.

The run uses pinned Node 24.21.0, Playwright 1.63.0 and Chrome 153.0.8010.12. Eighteen runtime
records verify UID/GID 65532, dropped capabilities, no privilege escalation and read-only roots.
Eight external mode-0444 seeds drive nine fixture identities per target. Kubernetes has nine
unique pod UIDs and nine direct-pod port-forward lifecycles, stopped and waited before each
rollout. Strict probes, archive-only Trivy 0.74 scanning with zero fixable HIGH/CRITICAL findings,
the frozen Node/YAML checker with its required success output, and strict Kubernetes 1.33
validation for all three resources pass. The production image contains the current web assets;
the corrected defined-ID assertion belongs to the separately mounted frozen E2E source.

All four latency profiles retain 20 samples, zero exceedances and zero CPU throttling. Compose
local/delay-50ms maxima are **55.800/145.273 ms**; Kubernetes maxima are **53.308/155.414 ms**.
Separate root coordination records retain fresh scoped worker acknowledgments for each window.
The initial sandboxed root process-existence probe was denied before writing any gate; the
subsequent permitted read-only probe confirmed the current port-forward before K approval.

Isolated read-only browser dependencies have pre-run file, symlink and archive snapshots:
8,215 files, 13 symlinks, 132 exact lock entries and 42 optional entries. The pre-run tar hashes
to `daccf3269e949a85682437e8ffdf9694ef4d63c8b465f12c2ce8a595c0afcf4b`; post-run files,
symlinks and tar bytes match exactly. Source files, lock contents and image IDs also match.
The original wrapper exited **1 after every case passed**, because pre/post lock records used
relative versus absolute paths. Both raw records, original mismatch, unmodified wrapper and
actual PTY exit remain preserved. A separate authorized comparison normalizes only the exact
evidence prefix, verifies the three unique ordered paths and equal digests, and recomputes the
actual lock hashes. Remaining image, preparation and cleanup checks then pass separately;
no browser or harness case was rerun and the wrapper exit is not reported as zero.

Evidence at `/tmp/grenadiers-v0330-zero-id-native-acceptance` seals 1,734 entries with final
manifest SHA-256 `a9a3085c2371c1b75c3fa20585a46daba4a72ad711728b0f90e9a9eebabc47d6`.
The unchanged 997-entry preparation manifest hashes to
`b9b01d0bdace64e5d7e698f0d8f85424f2a18f6660a2c76f63097c178b93e40b`.
A separate six-entry provenance correction at `/tmp/grenadiers-v0330-native-record-correction`
hashes to `022b8ae3c46b730ec9ef49558004bbb669f52fb08b826935cd2125a15022e5f3` and corrects
one original RESULT sentence that attributed the E2E assertion to the production image. The
original sealed bundle remains unchanged. A preparation-only `google-chrome` command-name
probe failed before any case and was corrected to the pinned browser's bundled executable;
its raw failure remains retained. Earlier failed native attempts remain separate and uncarried.

Root visually reviewed and copied the actual Compose [desktop pickup](screenshots/long-fuse-desktop.png)
and [Polish 320px active curse](screenshots/long-fuse-polish-320.png). Their hashes are
`1fadf789361cc3365cf745cc0a7f75f113de2bedd1d5afaec1181cbf720770a2` and
`cdcf7266b724aa917dfc2690e044d6e887ffbd24f994eda0d1a7d9a334580f7a`.
Independent final evidence review, including the separate provenance correction, is clear;
final-head CI remains pending. Later changes in this
increment are documentation and copied captures only; publication is recorded in release notes.

Records below are frozen against their named sources. Publication and later CI status are
recorded in [release notes](https://github.com/liveweird/grenadiers/releases).

## Long Fuse native scenario observation correction — 2026-10-18 — fresh acceptance pending

The fresh whole-matrix attempt against source `39e764a9a20fc0c25e663fd4921491c5268c8358`
passed 31 Compose cases and stopped on the Long Fuse case; Kubernetes was not started and no
result is carried forward. Its failure-safe public attachment conclusively records manual action
1, bomb ID `0` placed at tick 304 in cell (2,1) with a 120-tick fuse, and exact detonation at tick
424 on both peers. The actor page remained focused on the courtyard, both actors remained alive,
and the same round was running. The driver had observed the valid numeric ID `0` but tested it
with a truthiness assertion, so the poll expired after ten seconds with `Received: 0`. The
correction accepts any defined bomb ID, including zero; timers, timeouts, retries, product code,
and the remaining assertions are unchanged.

This attempt retains 31 passed and one failed Compose case, zero Kubernetes cases, 14 HTML
reports, and 37 PNGs including two failure captures. The sealed evidence is at
`/tmp/grenadiers-v0330-ready-native-acceptance`: its unchanged 988-entry preparation manifest
hashes to `9e5b2ddc43d44f8fac51d78d071093c56ba93de5388673e5626636b61243bb95`, and its
1,380-entry final manifest hashes to
`0c782387e843cf3c9e8e40a22e9c767e35741c91a8ee8cba5dd3dd21a4aaad86`. Source,
locks, external dependencies and image IDs matched before and after the run; cleanup passed.
This is a confirmed driver defect, not a product failure, and fresh full 32+32 native acceptance
from a new source archive and image remains required.

The first whole native attempt against source `04b89a6257bd965ac93346c662f9e1c05f19af62`
passed 31 Compose cases and then stopped at the first Long Fuse bomb placement poll; Kubernetes
was not started. The report retained no command/frame summary, so it cannot distinguish a Space
command that was not emitted or accepted from a 120-tick bomb that left the latest live state
before the poll observed it. The final screenshots do not resolve that ambiguity. The failure is
not attributed to a product focus defect, and none of its passes may be carried forward. The
later valid-zero-ID diagnosis does not retroactively prove the cause of this earlier attempt;
its historical cause remains unproven.

The corrected scenario keeps the 120-tick base, captured 240-tick Long Fuse, 600-tick effect,
timeouts, zero retries and two intended screenshots per target. Each placement now proves the
exact next manual bomb action and obtains its bomb ID from retained `bombPlaced` history. It also
proves that the first two escapes leave the actor alive in the same running round, while retaining
the captured-fuse, detonation, expiry, reconnect, action-floor and final lethal-blast assertions.
A failure-safe attachment records only whitelisted public command summaries, current public
actors and bombs, bomb placement/detonation events, round/tick facts and focused DOM identity;
credentials and private admission frames are excluded. Fresh full 32+32 native acceptance from a
new source archive and image remains required.

## Long Fuse final-parent integration — 2026-10-18 — native acceptance pending

The final audited Short Flame parent is integrated, including all accepted Heart, Shield,
Golden Flame, Short Fuse and Short Flame screenshots and corrected native timing drivers.
The English Gear label, matching web assertions and actor-stat E2E support are retained.
Core, server, API and Cargo inputs remain unchanged from the reviewed Long Fuse candidate.
The five predecessor item scenarios remain byte-identical to the accepted parent apart from
the exact six-item available catalog required by Long Fuse. The Long Fuse scenario retains its
120-to-240-tick capture proof, exact expiry, ordinary timing restoration, two screenshots per
target, strict assertions, existing timeouts and zero retries.

The merged tree retains the v0.3.30 six-item catalog. Fresh 32+32 same-image native acceptance
and final-head CI remain required; no earlier image result certifies Long Fuse.

The full web gate passes **409/409 tests** across 39 files, with statement/branch/function/line
coverage of **95.17/90.88/95.50/97.59%**, plus contract drift checks, API and application lint,
Knip, and the production build. Repository guards and E2E lint, Knip, type checking and static
parity pass all **33 documented scenarios** under the required Node 24.21.0. An initial command
selected ambient Node 26.11.1 and was rejected by the repository pin before any web task ran;
the retained successful gate uses the required mise toolchain.

## Long Fuse terrain cache integration — 2026-10-18 — refreshed native acceptance pending
Records below are frozen against their named sources. Publication and later CI status are
recorded in [release notes](https://github.com/liveweird/grenadiers/releases).

The current terrain-cache/shared-diagnostics integration passes all 409 web tests across 39 files, with coverage
of 95.17/90.88/95.50/97.59% above the unchanged 85% floors, production and diagnostic-fixture builds, and E2E
static checks with 33 documented scenarios. Core, server, version and API content are unchanged
by this integration. Long Fuse has no completed native baseline. The inherited Heart-only record
below certifies source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`; it includes neither
Long Fuse, Short Flame, Short Fuse, Golden Flame, Shield nor the terrain cache. Fresh native acceptance and final-head CI remain required for this tree.


## Long Fuse candidate — 2026-10-18 — local checks complete

The v0.3.30 candidate prepares the opt-in Long Fuse curse. Collection refreshes a 600-physical-
tick effect that doubles the configured fuse captured by newly placed ordinary bombs. Short Fuse
and Long Fuse cancel for new placements while both independent timers remain active. Historical
bomb schedules remain immutable across expiry, death, reconnect, and round transitions.

The new catalog bit is 17 and the conditional generation weight is 15. All 64 supported masks
retain the 32 previous generation paths, Heart-only defaults, and canonical item ordering. The
ordinary single-item seed fixture pins Long Fuse at cell (3,1). Protocol, server replay, web, and
production-seed browser coverage are included locally.

The repository/server gate passes formatting, Clippy with warnings denied, workspace build, and
both debug and release profiles with **466 core tests, 184 server tests, and two doctests**;
four dedicated performance profiles remain ignored. Rust coverage passes the unchanged 80% line
floor. All **408 web tests** across 39 files pass with statement/branch/function/line coverage
of **95.14/90.89/95.48/97.58%**, plus contracts, lint, Knip, and the production build. E2E lint,
Knip, type checking, and **32-scenario parity** pass. Cargo-deny,
both cached npm dependency audits, and strict Kubernetes schemas for all three resources pass.
Independent backend and frontend/protocol/browser-scenario review is clear after expiry,
resume, malformed-effect and screenshot-order regressions. The native Long Fuse playthrough
and screenshots, final candidate-image acceptance, and final-head CI remain pending. No release or shipped status is claimed.


## Long Fuse responsive-HUD integration — 2026-10-18 — native acceptance pending

The inherited two-column narrow HUD and real text-geometry regression are integrated. Core,
server, Cargo, API, Long Fuse client/protocol/locales and dedicated item scenarios remain
unchanged from the reviewed Long Fuse candidate. Its unit baseline remains unchanged;
the production build and E2E static checks pass for this responsive integration.
Fresh native Compose/Kubernetes acceptance and exact-head CI remain pending.


## Short Flame native acceptance — 2026-10-18 — fresh 62/62

Clean source `f3e7bac200115a453cf6819c220b0c4add389583`, archive SHA-256
`a429ed9ba5dc9a3b1dec2c87eb6a76e132191ed1c0083ffa3f52e90aa824e909`, builds image
`sha256:85beca611ec288e00e82952d49cde91d285965e9af6cce18b155d7122db5f5d7`.
The saved image archive hashes to
`4c0222a4477698c418373f07f84e805d99d8943dd8a69b6770b6acd21a61c25a`.
Fresh acceptance passes **31/31 Compose and 31/31 Kubernetes**, first test attempts, one
worker, zero retries, reruns or carried results. Each target retains 26 baseline cases plus
Heart, Shield, Golden Flame, Short Fuse and Short Flame. The new case proves generated
collection, real radius-one flames with the ordinary fuse, reconnect, exact physical expiry
and later base-range damage. Its time-sensitive bomb proof precedes active screenshots.

Four latency profiles retain 20 samples each, zero exceedances and zero CPU throttling.
Local/shaped maxima are **55.871/159.265ms Compose** and **50.420/181.861ms Kubernetes**.
Fresh root quiet acknowledgements concern named collaborating workers, not global host
idleness. All 26 reports and 70 configured PNGs verify, without failure screenshots.
Sixteen hardened runtime records use the same image: UID/GID 65532, no effective capabilities,
no-new-privileges and read-only roots. Seven external mode-0444 seeds, eight distinct Kubernetes
fixture UIDs and eight serial direct-pod forwarding lifecycles verify. Strict exact-200 JSON/
no-store health/readiness probes and final cleanup of project, network, namespace and ports
18176/18177 pass. Initial transport probes may repeat; browser test cases have zero retries.

The frozen Kubernetes checker actually runs with Node 24.21 and locked YAML, emitting its
required success message. Strict Kubernetes 1.33 validates all three resources; pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL issues. An early unmounted toolchain probe selected
Playwright 1.64 and failed Chrome lookup before any browser case; the corrected isolated mount
proves Node 24.21/Playwright 1.63/Chrome 153. Kubeconform's initial missing PATH entry was corrected
with its checksum-pinned 0.8 archive. The first unmounted probe is described in the sealed
correction narrative, but its raw command/stdout was not retained; its exact failure cannot be
independently reconstructed. The corrected mounted toolchain and pre-run dependency proof
are independently verifiable.

Browser dependencies use an isolated read-only mount outside the writable evidence bind.
The pre-run archive hashes to
`af88d6f20f82b7e4f4c230d0dae4a72cc95fa8b553fe4083dc05fc245f283e00`.
All 8,215 file hashes, 13 symlink targets, source files and lockfiles match before and after
execution. All 132 installed dependency packages match frozen lock versions/resolved locations/
integrity; 42 omitted packages are optional. This run records pre-execution dependency bytes,
unlike the explicitly limited earlier post-run corroboration.

The runtime probe overwrote preparation kubectl version JSON with equivalent plain output.
The exact overwritten bytes remain separately preserved; reproducing the approved JSON yielded
its original hash, and restoration is explicitly recorded. The unchanged 984-entry preparation
manifest verifies, SHA-256
`8016265ee0ee767ed6c6818a196fa4c41706c4139d24b029c2a3987d11e9e955`.
No case, source, image, dependency, assertion or timeout changed, and no test was rerun.
Future preparation uses collision-free `prep-*` version filenames.

Root visually inspected the exposed desktop pickup and active Polish 320px view: clear blue
pickup, contained Gear label and readable Krótki płomień (0:07), without overlap. Committed
screenshots are exact accepted copies. Independent final source/image/runtime/evidence review is clear.
The sealed 1,636-entry final manifest includes RESULT and hashes to
`bccadc22fba458943fd14a3964124fba287190ad62790455056658a7e9bad90e` at
`/tmp/grenadiers-v0329-native-acceptance`. Final-head CI remains required.

## Short Flame final-parent integration — 2026-10-18 — native acceptance pending

The final audited Short Fuse parent is integrated, including accepted Heart, Shield, Golden
Flame and Short Fuse screenshots, the corrected predecessor native timing proofs, and the
English Gear label and matching tests. Core, server, API and Cargo inputs are unchanged by
this integration. The four predecessor item scenarios remain byte-identical to the accepted
parent apart from the exact five-item available catalog required by Short Flame.

The complete web gate passes **368/368 tests** across 39 files, with statement/branch/function/
line coverage of **95.14/90.82/95.49/97.57%** above the unchanged floors. Contract drift,
API lint, application lint, Knip and the production build pass. Repository guards and E2E
lint, Knip, type checking and static parity pass all **32 documented scenarios** under the
required Node 24.21.0.

The Short Flame timing hardening remains in place: its natural range-one blast is proved before
the active screenshot, while exact expiry, reconnect and later base-range damage remain strict.
Fresh 31+31 same-image native acceptance and final-head CI remain required. Future native
acceptance must hash external E2E dependencies before and after execution, use pinned Node
24.21, fail fast on static checks, and assert the repository Kubernetes checker's required
success output. No preceding image result certifies Short Flame.

## Short Flame native timing hardening — 2026-10-18 — fresh acceptance pending

A confirmed native Shield failure showed that language switching and full-page screenshots can
consume several seconds of a ten-second effect window. Short Flame now captures the narrow Polish
pickup layout before collection, then plants, proves the captured range, escapes, and observes the
natural one-cell blast before checking the effect is still active and taking the active screenshot.
Exact expiry, reconnect, later base-range damage, timeouts, and zero retries remain unchanged.
E2E lint, Knip, type checking, repository guards, and 32-scenario static parity pass; the
hardened native scenario remains pending and no native pass is claimed.

## Short Flame terrain cache integration — 2026-10-18 — refreshed native acceptance pending

The terrain-cache/shared-diagnostics integration passes all 368 web tests, with coverage of
95.14/90.82/95.49/97.57% above the unchanged 85% floors, production and diagnostic-fixture
builds, and E2E static checks with 32 documented scenarios. Core, server, version and API
content are unchanged by that integration. Short Flame has no completed native baseline.

## Short Flame candidate — 2026-10-18 — native acceptance pending

The v0.3.29 candidate activates the opt-in Short Flame curse. Collection refreshes
600 physical ticks of range-one captures for new bombs, with precedence over Golden Flame
and independent effect timers. Existing bombs and base upgrades retain their values. The new
weight is 15; all 32 supported masks preserve the sixteen previous generation paths and
Heart-only defaults. The ordinary single-item seed fixture pins pickup cell (3,1).

Independent domain and integration reviews are clear. Both profiles pass **457 core tests,
183 server tests and two doctests**, with four dedicated performance profiles ignored.
Formatting, Clippy and production build pass. Rust line coverage is **91.46% (12,203/13,342)**.
All **367 web tests** across 39 files pass; statement/branch/function/line coverage is
**95.11/90.83/95.47/97.56%**. Contract checks, lint, Knip, dependency audits, strict Kubernetes
schemas 3/3 and E2E static parity **31 scenarios** pass. The first server-gate invocation was
rejected by the Node pin guard; the complete gate passes with the required Node 24.21.0.
An initial protocol test compile lacked its Tick import; the corrected focused and full suites
pass. The frontend's first lint run found excessive settings-panel complexity; rendering its
canonical item list resolves it, with the complete web gate passing afterward.

Tests cover both Golden Flame expiry orders, fixed old/new bomb ranges, base upgrades,
Short Fuse independence, blocker/chain geometry, all-layer transactional overflow and ordinary
generated collection. Server replay compares full state while a cursed bomb is live before
and after effect expiry, then again after round reset. The registered production-seed browser
scenario checks actual one-cell fire reach, unchanged fuse timing on both peers, reconnect,
exact expiry and later base-range damage. Its native playthrough, screenshots, same-image
Compose/Kubernetes **31+31** acceptance and final-head CI remain pending. No release is claimed.

## Short Flame responsive-HUD integration — 2026-10-18 — native acceptance pending

The inherited two-column narrow HUD and real text-geometry regression are integrated. Core,
server, Cargo, API, Short Flame client/protocol/locales and dedicated item scenarios remain
unchanged from the reviewed Short Flame candidate. Its unit baseline remains unchanged;
the production build and E2E static checks pass for this responsive integration.
Fresh native Compose/Kubernetes acceptance and exact-head CI remain pending.

## Short Fuse native acceptance — 2026-10-18 — fresh 60/60

Clean source `05e0427f22a1337e10232485a981513f76f77ef6`, archive SHA-256
`fa2005b453f1510a3860383c6c0a3af61cbcb4424abab40fe62f6bb60a3abc0a`, builds image
`sha256:9ab5888eac85289562af5b9d31b8b073c2782b84fd3a6719c69f1c6c82c21db0`.
The saved image archive hashes to
`68f32981dfc5d34570de95540ec4dedba85f133013f892cd051a85662de5df58`.
Fresh acceptance passes **30/30 Compose and 30/30 Kubernetes**, first test attempts, one
worker, zero retries and no carried results. Both targets retain all 26 preceding baseline cases
plus Heart, Shield, Golden Flame and Short Fuse. The Short Fuse case proves generated pickup,
captured 150-tick timing, real natural detonation, reconnect, exact physical expiry and a later
ordinary 300-tick bomb. Screenshots follow the time-sensitive bomb proof.

Four latency profiles retain 20 samples each, zero exceedances and zero CPU throttling.
Local/shaped maxima are **48.106/163.375ms Compose** and **52.593/159.301ms Kubernetes**.
Root's separate quiet acknowledgements cover named collaborating workers, not global host
idleness. Strict exact-200 JSON/no-store probes, hardened runtime, external mode-0444 fixtures,
seven fresh pod UIDs, serial direct-pod forwarding and complete cleanup pass. Initial transport
readiness probes may repeat; the no-retry claim concerns browser test cases.
Pinned archive-only Trivy finds zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes
1.33 validates all three resources. All 24 reports and 64 configured PNGs verify, without failure
captures. The accepted source includes the final audited Golden Flame parent and timing fixes.

Root visually inspected the exposed desktop pickup and active Polish 320px view. The icon,
Short Fuse timer and contained stat cells are readable; Gear now fits the desktop inventory
label. Committed screenshots are exact accepted copies. Independent final evidence review, including the corrective bundles below, is clear. The sealed 1,271-entry final manifest hashes to
`8509a86187966a50e9975a272235ba1ad897895b56e0a5e863f3919b39d6e8bd`. Preparation manifest retains 692 entries, SHA-256
`ec7d66f3076c5c887f867d65ebb958f30560c8fb2414ef42c8feef57620ebdf3`.
Evidence: `/tmp/grenadiers-v0328-native-acceptance`.

The original sealed RESULT overclaimed the repository-specific Kubernetes checker: its log
records a missing YAML dependency, while only kubeconform passed. A separate corrective
addendum runs the exact frozen checker and original rendered input successfully with Node
24.21 and its locked YAML dependency; a fresh frozen-source render is byte-identical. The
checker emits the required single-owner-invariant success message. An initial `/tmp` invocation
silently skipped its entry guard; that empty attempt is retained, followed by a successful
canonical `/private/tmp` invocation. Original browser results and seals remain intact.
The correction contains 251 manifest entries, SHA-256
`ba0ee3bfa8326da7e8d6a66cc29a6bf32da1eff0f56d463bb01f8f2d2e03325b`, at
`/tmp/grenadiers-v0328-k8s-check-correction`.

The original external E2E dependency mount was read-only but lacked a pre-execution content
hash. The addendum retains a post-run archive and 8,215 file hashes: all 132 installed packages
match frozen lock versions, resolved locations and integrity; 42 omissions are optional. File
mtimes predate the first case. This corroborates the dependency record without proving historical
bytes. Future acceptance requires dependency hashes before and after execution, pinned Node,
fail-fast static checks and an explicit checker success-output assertion.

The same retrospective static-gate correction applies to accepted v0.3.25–v0.3.27 inputs:
their complete checker, dependency lock and canonical rendered manifest are byte-identical
to the successful correction. v0.3.26/v0.3.27 retained the same missing-dependency error;
v0.3.25 did not retain a checker success line, so its original execution is unproven. The
11-entry identity bundle at `/tmp/grenadiers-v0325-v0327-k8s-identity-correction` hashes to
`cbd223f21aeecb0ee735c1b4a49ff67070213bf3d51a941d41182ff36348141a`. Earlier native
cases, source/image seals, release tags and independent CI results remain unchanged.

## Short Fuse final-parent and HUD integration — 2026-10-18 — native acceptance pending

The final audited Golden Flame parent is integrated, including the corrected Shield/Golden
native timing proofs and accepted Heart/Shield/Golden screenshots. Production/core/server/API/
Cargo inputs are unchanged by this parent merge. Its three predecessor item scenarios are
byte-identical to the accepted parent apart from the exact four-item available catalog.
E2E lint, Knip, type checking, repository guards and 31-scenario static parity pass.

The English HUD label now reads Gear to avoid splitting Equipment's final letter in narrow
desktop stat cells; Polish terminology and inventory behavior are unchanged. All 339 web tests
and the production build pass, with coverage 95.11/90.80/95.50/97.56%. The existing inventory
and native label assertions remain intact with the updated copy. Fresh 30+30 same-image native
acceptance and final-head CI remain required; no preceding image result certifies Short Fuse.

## Short Fuse native timing hardening — 2026-10-18 — fresh acceptance pending

A confirmed native Shield failure showed that language switching and full-page screenshots can
consume several seconds of a ten-second effect window. Short Fuse now captures the narrow Polish
pickup layout before collection, then plants and proves the captured 150-tick fuse, escapes, and
observes natural detonation before checking the effect is still active and taking the active
screenshot. Exact expiry, reconnect, later 300-tick bomb proof, timeouts, and zero retries remain
unchanged. E2E lint, Knip, type checking, repository guards, and 31-scenario static parity pass;
the hardened native scenario remains pending and no native pass is claimed.

## Short Fuse terrain cache integration — 2026-10-18 — refreshed native acceptance pending

The current terrain-cache/shared-diagnostics integration passes all 339 web tests, with coverage
of 95.11/90.80/95.50/97.56% above the unchanged 85% floors, production and diagnostic-fixture builds, and E2E
static checks with 31 documented scenarios. Core, server, version and API content are unchanged
by this integration. Short Fuse has no completed native baseline. The inherited Heart-only record
below certifies source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`; it includes neither
Short Fuse, Golden Flame, Shield nor the terrain cache. Fresh native acceptance and final-head CI remain required for this tree.

## Short Fuse candidate — 2026-10-18 — native acceptance pending

The v0.3.28 candidate adds opt-in Short Fuse: a refreshable 600-physical-tick effect that
captures the ceiling-half of the configured fuse for new bombs. Defaults remain Heart-only;
all 16 supported masks preserve canonical ordering and the eight prior generation paths.
The new weight is 15. Published v0.3.21 still has no optional pickups.

Independent domain, frontend/protocol and server/browser scenario reviews are clear. Both debug/release profiles pass **442 core tests, 182 server tests
and two doctests**, with four dedicated performance profiles ignored. Formatting and Clippy
pass. Rust line coverage is **91.42% (12,172/13,314)**. All **338 web tests** across 39 files pass;
statement/branch/function/line coverage is **95.08/90.80/95.48/97.54%**. Contracts, lint, Knip,
production build, both dependency audits and strict Kubernetes schemas 3/3 pass. E2E static
checks and documented parity pass **30 scenarios**.

Ordinary generated collection, connected replacement, disconnected restore, a shortened bomb
that outlives effect expiry, a later ordinary-fuse bomb, round reset and exact journal replay
(including checkpoints while a shortened bomb is live, before and after buff expiry) are exercised by server tests. Projection regressions cover odd half-fuses, effect-independent
captured timing, invalid/disabled timing and overdue deadlines before remaining-time clamping.
The first server test compile attempted crate-private effect observation; the final test uses
public wire state without expanding APIs. The new browser scenario covers captured deadlines,
actual natural detonation, reconnect and exact expiry. Its native playthrough, screenshots,
fresh same-image Compose/Kubernetes 30+30 acceptance and final-head CI remain pending.


## Golden Flame corrected native acceptance — 2026-10-18 — fresh 58/58

Clean source `b39f1b915aa4dc2b37707136af03224802d4347a`, archive SHA-256
`86ed52ab0c4e6f5a7dc8a15ddf4eb9f1102a0b5fe14424599a5626295216fd52`, builds image
`sha256:673f674f97f529a1dce143935c65d5aa8f7e525ff3f2ea9ae6f50e3154a124b0`.
The saved image archive hashes to
`806359661d891fc87a647145754b2c0b34824055234c8358e1be1ae7b7f6ad46`.
Fresh acceptance passes **29/29 Compose and 29/29 Kubernetes**, first test attempts, one
worker, zero retries and no carried results. Both targets retain all previous 26 cases plus
Heart, Shield and Golden Flame. The Golden case proves ordinary generated collection,
captured radius eight, natural detonation, reconnect, exact physical expiry and later base range.

Four latency profiles retain 20 samples each, zero exceedances and zero CPU throttling.
Local/shaped maxima are **49.836/167.982ms Compose** and **48.635/166.754ms Kubernetes**.
Strict exact-200 JSON/no-store readiness probes, hardened runtime, external mode-0444 fixtures,
six fresh pod UIDs, serial direct-pod forwarding and complete cleanup pass. Bounded initial
transport-readiness probes retry until ready; the no-retry claim concerns browser test cases.
Pinned archive-only Trivy finds zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes
1.33 validates all three resources. Scoped quiet acknowledgements concern named workers.

All 22 reports and 58 configured PNGs verify, with no failure captures. Root visually checked
the exposed desktop flame pickup and active Polish 320px view: clear icon and timer, contained
stat labels and no overlap. The minor English desktop Equipment final-letter wrap remains a
readability follow-up. Committed Golden screenshots are exact accepted copies. Independent
final source/image/runtime/report/evidence audit is clear. The sealed 1,205-entry manifest
hashes to `a3f3c3b31bc067e55c6863059d1d88d15d0b414cff66b14365a272071eb4dbff` under
`/tmp/grenadiers-v0327-corrected-native-acceptance`; its 685-entry preparation manifest is intact.

Local baseline remains **426 core, 181 server and two doctests** in each Rust profile, **91.39%**
Rust line coverage; **320 web tests**, coverage **95.07/90.72/95.47/97.53%**, production/fixture
builds and 30-scenario static parity pass. Independent implementation/integration reviews are
clear. Exact-head CI and publication remain required. This post-acceptance update changes only
documentation and screenshot copies. The failed catalog attempt below remains separate.

## Golden Flame shared catalog correction — 2026-10-18 — fresh acceptance pending

The first native attempt against source `8cec8e89702e903f15c9a2d160a9d9543743ced9`
passed seven default Compose cases and failed the eighth: the shared round scenario still
expected the earlier Heart/Shield catalog, while the server correctly advertised Golden Flame
as well. The dedicated Golden Flame scenario already expected the correct three-item catalog.
The corrected shared scenario retains an exact ordered catalog assertion and the independent
empty enabled-item settings assertion. No production behavior, timeout or retry policy changes. E2E lint, Knip, type checking,
repository guards and 30-scenario static parity pass.

No later Compose group, Kubernetes or latency case ran. The failed first attempt remains sealed
at `/tmp/grenadiers-v0327-native-acceptance`, 742-entry manifest SHA-256
`fb3bfe8eed1669168f18792a470aa1b33af80fd95c48ca0caa8f055b40963714`. Its seven passes
are not carried forward. A complete fresh 58-case native matrix is required for the correction.

## Golden Flame native timing hardening — 2026-10-18 — fresh acceptance pending

A confirmed native Shield failure showed that language switching and full-page screenshots can
consume several seconds of a ten-second effect window. Golden Flame now captures the narrow
Polish pickup layout before collection, then plants, verifies the captured range, escapes, and
observes the natural blast before checking the effect is still active and taking the active
screenshot. Exact expiry, reconnect, later base-range damage, timeouts, and zero retries remain
unchanged. E2E lint, Knip, type checking, repository guards, and 30-scenario static parity pass;
the hardened native scenario remains pending and no native pass is claimed.

## Golden Flame terrain cache integration — 2026-10-18 — refreshed native acceptance pending

The current terrain-cache/shared-diagnostics integration passes all 320 web tests, with coverage
of 95.07/90.72/95.47/97.53% above the unchanged 85% floors, production and diagnostic-fixture builds, and E2E
static checks with 30 documented scenarios. Core, server, version and API content are unchanged
by this integration. Golden Flame has no completed native baseline. The inherited Heart-only record
below certifies source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`; it includes neither
Golden Flame, Shield nor the terrain cache. Fresh native acceptance and final-head CI remain required for this tree.


## Golden Flame candidate — 2026-10-18 — native acceptance pending

The v0.3.27 candidate implements opt-in Golden Flame with a 600-physical-tick refreshable
timer and captured radius eight on newly placed bombs. Defaults remain Heart-only, Classic
clears optional items, and all eight supported pools retain canonical ordering. Existing four
item-pool generation paths stay byte-for-byte stable; the new enabled weight is 15. Repeated
seed byte31 gives the ordinary generated pickup at(3,1).

Independent domain and frontend/protocol/server/E2E reviews are clear. Core debug/release
passes **426 unit tests and two doctests** each; server debug/release passes **181 tests** each
(four dedicated performance profiles ignored). Workspace clippy and formatting pass. The first
server gate exposed a stale two-item admission expectation; the updated three-item expectation
and both complete server suites pass. Generated collection, captured range, connected replacement,
disconnected restore, exact expiry, base range restoration, next-round settings and journal replay
are exercised through ordinary runtime actions.

All **319 web tests** pass across 39 files, with statement/branch/function/line coverage
**95.04/90.73/95.45/97.52%**. Contracts, lint, Knip, builds, both dependency audits and strict
Kubernetes schemas3/3 pass. E2E static parity passes **29 scenarios**. Rust line coverage is
**91.39% (12,142/13,286)**. Coverage tests passed initially; report output failed because an
external target directory left the local output parent absent. Creating that parent and reporting
the existing data succeeds without rerunning tests.

A production-seed browser scenario is registered for range-eight placement, ordinary escape,
reconnect action floor and unchanged effect deadline, exact expiry and later radius-two death.
Its actual browser playthrough, screenshots, fresh same-image Compose/Kubernetes29+29 acceptance
and final-head CI remain pending. Prior candidate image results do not certify this item.

## Golden Flame responsive-HUD integration — 2026-10-18 — native acceptance pending

The inherited two-column narrow HUD and real text-geometry regression are integrated. Core,
server, Cargo, API, Golden Flame client/protocol/locales and dedicated item scenarios remain
unchanged from the reviewed Golden Flame candidate. Its unit baseline remains unchanged;
the production build and E2E static checks pass for this responsive integration.
Fresh native Compose/Kubernetes acceptance and exact-head CI remain pending.

## Shield corrected native acceptance — 2026-10-18 — fresh 56/56

Clean source `be44b8aeae88249b5635ed4424d3f04f7fa33f84`, archive SHA-256
`62daff016257a425b67340aa0598a4818ab0960d73a4fb6a3f2501995f40d60f`, builds image
`sha256:3751bcef474c7ca1d09ba30d46846309cfb72f783db36bce5e95be05c24391fb`.
Its saved archive hashes to `6b3654c71aa2e1117ab5f2828645c9561615b72b031e1277b576233bb50d2890`.
Fresh acceptance passes **28/28 Compose and 28/28 Kubernetes**, first attempts, one worker,
zero retries and no carried results. Both targets include all previous 26 cases, Heart and Shield.
The corrected test proves blast immunity, exact event-derived expiry and later lethal damage.

All four latency profiles retain 20 samples, zero exceedances and zero CPU throttling.
Local/shaped maxima are **54.531/141.873ms Compose** and **61.247/156.193ms Kubernetes**.
Strict exact-200 JSON/no-store probes, hardened runtime, external mode-0444 fixtures, five
fresh pod UIDs, serial direct-pod forwarding and complete cleanup pass. Pinned archive-only
Trivy finds zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes 1.33 validates 3/3
resources. Quiet acknowledgements cover named workers, without a global host-idle claim.

All 20 reports pass, with 52 fresh PNG checkpoints. Normal failure capture remains enabled;
no failure PNGs are present. Root visually checked the exposed desktop and active Polish 320px
Shield views: readable two-column narrow HUD, distinct glyph and timer, no overlap. The minor
English desktop Equipment final-letter wrap remains a readability follow-up. Committed Shield
screenshots are exact accepted copies. Independent final source/image/report/evidence audit is clear.
The sealed 1,127-entry manifest hashes to
`cc5b06b926398bd0b92c5a89a4c4c3b24675f51ab37d28e10aa53d6d7c3e4a50` under
`/tmp/grenadiers-v0326-corrected-native-acceptance`; its preparation manifest remains intact.

Local baseline: **413 core, 180 server and two doctests** in each Rust profile, **91.23%**
Rust line coverage; **304 web tests**, coverage **95.03/90.68/95.43/97.51%**, production/fixture
builds and 29-scenario static parity pass. Independent implementation and integration reviews
are clear. Exact-head CI and publication remain required. This post-acceptance change contains
only documentation and screenshot copies. The earlier failed attempt below remains separate.

## Shield native scenario correction — 2026-10-18 — fresh acceptance pending

The first responsive native attempt passed 27 Compose cases, then failed the Shield scenario;
Kubernetes did not run. Both retained public streams establish derived collection tick 656, expiry 956,
second-bomb placement 859 and detonation 1039. The player correctly died after Shield expired:
language/screenshot work had consumed 203 ticks before placement. This is a scenario timing
defect; no production simulation change is made. The corrected scenario captures exposed layouts
before collection, plants immediately, checks detonation precedes the actual Shield deadline,
and retains the active screenshot after both protection proofs. Existing expiry, survival,
later death, assertions, timeouts and zero-retry policy remain intact. E2E lint, Knip, typecheck
and 29-scenario static parity pass after the correction.

Failed evidence remains sealed at `/tmp/grenadiers-v0326-responsive-native-acceptance`,
886-entry manifest SHA-256 `51ca4c7ef6c1deb724a806335156b6c29b16aaf5832ac57bee3200177ee90d65`.
No prior result counts toward the required complete fresh 56-case acceptance.

## Shield responsive-HUD integration — 2026-10-18 — native acceptance pending

The parent two-column narrow HUD and real text-geometry regression are integrated. Core,
server, Cargo, API, Shield client/protocol/locales and dedicated item scenarios are unchanged
from the reviewed Shield cache candidate. Its 304 web tests and coverage below remain the
unit baseline. Production build and E2E lint, Knip, typecheck and 29-scenario static parity pass for this layout.
No parent or historical Heart evidence certifies Shield. Fresh same-image Compose/Kubernetes
acceptance and exact-head CI remain required.

## Shield terrain cache integration — 2026-10-18 — refreshed native acceptance pending

The current terrain-cache/shared-diagnostics integration passes all 304 web tests, with coverage
of 95.03/90.68/95.43/97.51% above the unchanged 85% floors, production and diagnostic-fixture builds, and E2E
static checks with 29 documented scenarios. Core, server, version and API content are unchanged
by this integration. Shield has no completed native baseline. The inherited Heart-only record
below certifies source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`; it includes neither
Shield nor the terrain cache. Fresh native acceptance and final-head CI remain required for this tree.

## Shield activation candidate — 2026-10-18 — native acceptance pending

The v0.3.26 working candidate passes **413 core tests, 180 server tests and two doctests in each
debug/release profile**, with four dedicated performance profiles intentionally ignored.
Rust line coverage is **91.23%** (12,084/13,245 lines); all **303 web tests** pass with
statement/branch/function/line coverage of **95.00/90.68/95.41/97.50%**. Contracts, lint,
Knip, builds, dependency audits, strict Kubernetes
schemas (3/3), E2E static checks and **28-scenario parity** pass. Independent domain/protocol
review is clear. Browser review identified an expiry-assertion gap; acceptance now derives the
deadline from the real collection event and checks each retained authoritative timer against
T+300, with observations before and after expiry. Coalesced delivery is allowed.

Regression coverage includes opt-in masks, ordinary collection, refresh without stacking,
Heart preservation, exact pre-damage expiry, both fire checks, one prevention event per player
per tick, atomic overflow, legacy rejection and frozen prior seeded outputs. A seeded server
scenario collects Shield, retains it through connected replacement and disconnected restore,
survives an actual planted bomb, expires, dies to a later bomb and replays exactly. A final
focused run verifies the strengthened bomb-placement assertion in both build profiles.

No v0.3.26 native image, full browser scenario run or release is claimed. Native Compose and
Kubernetes acceptance, visual inspection and exact final-head CI remain required.

## Heart responsive-HUD integration — 2026-10-18 — fresh native 54/54

Clean source `2f86fc96f4f562871ed962052c1f0f1b74d53881`, archive SHA-256
`7fa83c1139e1e0487b91ef2328db7d05d932146f3ff55b39d4f1a47123c95579`, builds the new image
`sha256:e8de9c77597e5c94fe5c406994d55ab7e776c0696b5ec3333f9f46273166ea09`.
Its saved archive hashes to `93c50da1b8f0a5d25b351778aa346b83abce89fb2235df1ee20f45f9f37e79de`.
Fresh acceptance passes **27/27 Compose and 27/27 Kubernetes**, first attempts, one worker,
retries zero and no carried results. Both runtimes include all previous 26 cases plus Heart.

Four latency profiles retain 20 samples each, zero exceedances and zero CPU throttling.
Local/shaped maxima are **51.386/160.103ms Compose** and **52.320/160.583ms Kubernetes**.
Strict exact-200 JSON/no-store probes, hardened runtime, external mode-0444 fixtures, four
fresh pod UIDs, serial direct-pod forwarding and complete cleanup pass. Pinned archive-only
Trivy finds zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes 1.33 validates three
resources. Quiet acknowledgements cover named local workers, without a global host-idle claim.

All 18 reports pass and 46 fresh PNGs remain. Capture profiling is unset and normal failure
capture is preserved. Root visually checked the readable Polish 320px equipment/protection HUD
and English desktop pickup. English desktop Equipment wraps its last letter; this minor label
readability follow-up does not cause overlap. Committed Heart screenshots are exact accepted
PNG copies. Independent final source/image/report/evidence audit is clear. The sealed 1,062-entry manifest hashes
to `fd296568303671e0c44736e0ba480e9334414ed4660a892a71845c70695185d5` under
`/tmp/grenadiers-v0325-fresh-responsive-acceptance`.

Core/server/Cargo/API and dedicated item scenarios remain unchanged from the reviewed Heart
cache candidate: **401 core, 179 server and two doctests** in each Rust profile, line coverage
**90.94%**. Its **287 web tests**, coverage **94.80/90.36/94.72/97.31%**, production/fixture
builds and 28-scenario static checks pass. Fresh responsive production build/static checks pass.
Exact-head CI and publication remain required. The post-acceptance update changes documentation
and screenshot copies only.

An earlier attempt paired the new source archive with the historical `35bf3678` image by
mistake. It is **invalid**, with no candidate passes or failures counted. Its observed match
geometry failure and cleanup are preserved separately in
`/tmp/grenadiers-v0325-responsive-native-acceptance`, 698-entry partial manifest
`f6b5ec3a6cc0c2dabd62f816e19c39bd549f05fd294afbbb3085eb66b5d2f75f`.
The complete run above builds and uses the correct new image throughout. Setup-only corrections
before that browser launch are retained in its evidence; no valid browser case was rerun.

## Heart activation candidate — 2026-10-18 — native baseline 54/54

Clean source `00d76ac9cc08e7bb4627caf7897e02c3f15a43d2`, source archive SHA-256
`cb60d03dd51e9793463191657448529ec119694656a4fca7f8ab0a943814aa61`, passes all 54 native
cases: 27 Compose/27 Kubernetes, each first attempt with zero retries. The single built image is
`sha256:35bf3678b5940149dd82acde55ada77f091d1bed70563db7f77b3dcbaafb0293`, saved archive
SHA-256 `72d0303d00c25cc7dff53ab501a2f0695364c5bc2c21be2f225b7f2a830a1d52`.
Pinned reference Chromium/Node 24.21/Playwright 1.63 runs every browser group. Heart itself passes
in 30.2s Compose/30.6s Kubernetes with external mode-0444 `arena-v2-heart`, ordinary controls and unchanged
assertions. All 18 reports have no failures or retries.

Each of four latency profiles retains 20 samples with zero exceedances; maxima are
51.860/172.306ms Compose local/shaped and 53.304/158.520ms Kubernetes. Throttling deltas are zero.
Strict probes, UID/GID 65532, zero capabilities, no-new-privileges, read-only roots, RuntimeDefault,
Kubernetes 500m/256Mi and direct fresh-pod port-forward liveness/stop/wait/cleanup pass. Four fixture
rollouts use distinct pod UIDs. Owned Compose containers/network and Kubernetes namespace are gone.

Independent evidence review confirms all 54 results and the frozen 1030-file final manifest
`6e8587290fc441f1310711391b2147d2486f61d5dd94ba87161782d0bcaeea5b` under
`/tmp/grenadiers-v0325-acceptance`. A preparation count-file path had become stale after its
recorded recheck; a sealed addendum preserves historical bytes and replays the original 630
preparation entries via the preserved count copy. Its ten-file audit manifest is
`8fd0932bd5d8e7eab1057a41169a9ffa97cb241a5b30f989481fc380aafe75a1`; independent re-review is clear.
This native baseline excludes the separately evaluated upstream terrain-cache candidate.
Final-head CI, integration and publication remain outstanding.

The v0.3.25 candidate passes **401 core tests, 179 server tests and two doctests** in both
debug and release profiles; four dedicated performance profiles remain intentionally ignored. Rust line
coverage is **90.94%** (11,950/13,140 lines), above the unchanged 80% floor. All **286 web tests**
pass with statement/branch/function/line coverage of **94.76/90.36/94.70/97.30%**. Contracts,
lint, Knip, production build, E2E static checks and **27-scenario parity** pass.
Dependency audits and strict pinned Kubernetes schemas pass. Independent gameplay/browser
and backend reviews are clear after the forged legacy replay-mask and socket-version issues
were fixed.

Regression evidence covers generated Heart reveal/collection through ordinary controls,
one-charge absorption, overlapping hazard cells and both damage passes, exact grace expiry,
damage before collection, replacement without stacking, overflow, disabled/Classic rejection,
real owner replacement/restore, quiet baselines and complete deterministic replay. Archived
contracts remain byte-unchanged; journal records retain their 64-byte and allocation bounds.

The inherited connected-takeover release-build defect is corrected; full debug and release gates
pass. Native acceptance requires actual movement from the replacement binding and now passes
at the source/image recorded above. Final-head CI remains required before publication.

## Parent responsive live-v7 HUD — 2026-10-18 — native 52/52, final CI pending

Clean source `695157bed0852835a16b3a9e9ee5dbcc5df61ad6`, archive SHA-256
`071a0927fa4c2087af918bbf84e2d848d38118bc02a9eaa7af1c42774cdeeef5`, builds one image:
`sha256:d68d1becb3d8ee9e710e0bf99af6429161b39160511d251f221c6f8af5fb644f`.
Its saved archive hashes to `4be7f6f969f98c10e53df0459d104701b1eb4e943367afdcf64906ac8dd9b8b9`.
Complete fresh acceptance passes **26/26 Compose and 26/26 Kubernetes**, first attempts,
one worker, retries zero, with no carried cases. All ordinary budgets and prior assertions
remain intact. Capture profiling is unset; normal failure capture and 42 screenshots remain.

Both runtimes pass the real match's text-geometry regression: both seats retain eight visible
labels with positive text fragments, four columns on English desktop and two on Polish 320px,
no overflow or cross-cell overlap. Root visual review confirms intact readable Polish words.
The committed desktop/narrow screenshots are byte-identical copies from this accepted run.

Four latency profiles have 20 samples each, zero exceedances and zero CPU throttling.
Local/shaped maxima are **51.492/161.999ms Compose** and **57.489/162.215ms Kubernetes**.
Strict exact-200 JSON/no-store probes, runtime hardening, mode-0444 external seeds, three fresh
Kubernetes pod UIDs, serial direct-pod forwarding in one recorded harness invocation, and complete
cleanup pass. Pinned archive-only Trivy reports zero fixable HIGH/CRITICAL findings; strict
Kubernetes 1.33 validates three resources. Quiet acknowledgements cover named local workers,
without a global ambient-host-idleness claim.

All **275 web tests** pass with coverage **95.08/90.80/94.68/97.56%**, plus production builds,
contracts, lint, Knip and E2E static parity 27. Backend verification remains **375 core tests,
174 server tests and two doctests in each debug/release profile**, Rust line coverage **93.27%**.
Independent implementation/layout and complete final native-evidence reviews are clear;
exact-head CI and publication remain required. Live rooms use v7 with optional pickups disabled.
The final evidence/screenshot update changes documentation only after the accepted source.

Evidence: `/tmp/grenadiers-v0324-responsive-hud-native-acceptance`, 949-file manifest SHA-256
`4b86be3b665e748fa640b715cf13932bae6b71e0b7d0c10bbd265f058ff14436`.

## Earlier HUD correction records

Visual review of the pre-layout baseline exposed overlapping Polish labels at 320px. The
intermediate wrap-only source `7dd97f2` passed **15/15 Compose functional cases**, including
text-containment assertions. Visual review still found four-to-five-line word splitting, so
that run stopped at the Compose latency gate. No latency or Kubernetes case ran; no result
carries to the final layout. Its interrupted cleanup initially left an owned container/network;
explicit cleanup removed them and final residue checks are empty. Partial evidence:
`/tmp/grenadiers-v0324-hud-wrap-native-acceptance`, 750-file manifest
`6f3ddef9a74c770d9ccba560ef2608075aa917b6a1b1e4d5254d9d371761186d`.

## Pre-layout live-v7 terrain-cache baseline — 2026-10-18 — native 52/52

Clean source `b1d0340e66300057e11de1a3dbcf57950850a793`, archive SHA-256
`1b752f0619592091bf54ed88dfa4b636625714ea028ee4355df0de0e7ee638ea`, builds one image:
`sha256:8f420bbf1cd7a1c6a6632d6dbb79b0eae7d0b2a91c670d1ff3e08bb3e6098e23`.
Its saved archive hashes to `e7125cea23b212ecc5a39a94645a5f72eb4f9f8aba8815482d2a4d3041e91d5d`.
Fresh ordinary acceptance passes **26/26 Compose and 26/26 Kubernetes**, all first attempts,
retries zero, one worker. This is a complete fresh run, with no carried cases. All eight HUD
fields, live-v7 bindings, original browser assertions and budgets are preserved. Capture
profiling is unset; normal failure capture and all 42 configured screenshots are retained.

Each target passes 15 functional and 11 latency cases. Four profiles have 20 samples each,
zero exceedances and zero CPU throttling. Local/shaped maxima are **47.241/161.882ms Compose**
and **126.458/162.159ms Kubernetes**. Strict exact-200 JSON/no-store probes, runtime hardening,
external mode-0444 seed fixtures, three fresh Kubernetes pod UIDs, serial direct-pod forwarding
in one recorded harness invocation and complete cleanup pass. Pinned archive-only Trivy finds
zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes 1.33 validates all three resources.
Initial forwarding-not-ready probes remain preserved before successful strict probes; no
browser scenario was retried. Quiet acknowledgements cover the named local workers and do not
claim global host idleness.

All **275 web tests** pass with coverage **95.08/90.80/94.68/97.56%**; production/diagnostic
builds, contracts, lint, Knip and E2E static parity 27 pass. Backend checks remain **375 core,
174 server and two doctests in each debug/release profile**, with Rust line coverage **93.27%**.
Independent implementation/integration and complete native-evidence reviews are clear.
Final-head CI and publication remain required. Live rooms use v7; optional pickups
remain disabled. The final evidence update changes documentation only after the accepted source.

Evidence: `/tmp/grenadiers-v0324-terrain-native-acceptance`, 949-file manifest SHA-256
`1068113a6564ebb80c2b72463c7d5ee35dab4deb8e821cd308304dfd5adc9482`.

## Earlier live-v7 verification records

## Live-v7 HUD acceptance — 2026-10-18 — native verified, publication pending

Clean source `0dde9142375ab495970f0031ef88fc10a25da499`, archive SHA-256
`91b0d98f37812b2f0d344e969ae5e07d53fba15eafb24cf5918bcba87b372412`, builds production image
`sha256:d4624da23bd07f0f28fad2807973ae7fe81b15b2725a95fa91e46fcd9d233daa`.
Pinned Trivy reports zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes schemas
validate3/3. The first Compose default group passes7 cases and fails the rendered duel:
its old four-value HUD assertion finds eight values because the live-v7 HUD now also shows
equipment, bomb type, effects and damage grace. That initial attempt leaves44 cases unrun, with zero retries and no
latency samples. Runtime hardening, strict probes and cleanup pass. Original failure artifacts
remain sealed in `/tmp/grenadiers-v0324-acceptance`; no Kubernetes runtime was started.

The corrected duel and generated-upgrade assertions check all eight named fields, including
the original four upgrade values and empty inventory/effects/grace. Complete label/value
arrays preserve field association and avoid adding many serial browser calls during a live
fuse. Product sources, screenshot options, deadlines, retry policy and gameplay are unchanged.
Independent implementation review and all E2E static gates/parity26 pass. This corrects the
HUD probe and does not explain the separate v0.3.22 hosted screenshot timeout.

Clean corrected source `893c150fcfcfdcd065418deb1349cb7852106df0`, archive SHA-256
`4019fd20ffdc6e2e6a99f8d8b54549a1e068d129c06ee75eef36420638431ef3`, has identical production
inputs to the image source above. The continuation passes **19/19 fresh Compose and26/26 fresh
Kubernetes cases**, all first attempts with zero retries. Seven default Compose cases retain
byte-identical transitive inputs, closing the explicit composite at **26/26 per target,52/52**.
Four latency profiles have20 samples each, zero exceedances and zero CPU throttling; local/
shaped maxima are48.491/133.364 ms Compose and48.906/160.071 ms Kubernetes. All32 strict
pre-browser probe pairs return200 JSON/no-store. Runtime hardening, external/projected0444
fixtures, fresh pod identities, direct-pod forward stop/wait ordering and cleanup pass.

Independent evidence review is clear. Source blobs, all16 embedded reports, retained test
inputs, immutable image/scan/schema references and both evidence manifests verify. Evidence:
`/tmp/grenadiers-v0324-hud-acceptance`; its374-file manifest SHA-256 is
`dbcc7b920f90fbc04bcb58afe947b6deb34f669f90ffdc2c1b50b0924b089d70`.
The preserved initial-failure90-file manifest SHA-256 is
`143eb7a3b7f0e3c847a3697d7059da822ba3b485c8716f0095af77d9df729a9f`.
Saved process records establish PID ordering/liveness and waited cleanup, without independently
proving literal PTY ownership or ambient quiet. Exact publication-head CI and upstream release
integration remain pending; this candidate is not yet merged or released.

## Live gameplay-v7 candidate — 2026-10-18 — local checks only

The v0.3.24 integration passes **375 core tests, 174 server tests and two doctests in each debug and release profile**;
four dedicated performance profiles remain intentionally ignored. Rust line coverage is
**93.27%** (10,568/11,331 lines), above the unchanged 80% floor. All **274 web tests** pass
with statement/branch/function/line coverage of **95.05/90.81/94.66/97.55%**. Contracts,
lint, Knip, builds and E2E static checks pass, with **26 documented scenarios**.
Dependency audits and strict pinned Kubernetes schemas pass. Independent backend and
frontend reviews are clear.

The server and browser now use gameplay-v7 together. Admission and both recipient frames
are encoded before transactional state changes. Routing tests use a real Seat Two membership
and cover publication, retained closing and server shutdown. Regressions also cover invalid
room provenance, failed admission rollback, previous-version rejection before credential claim,
and unsupported equipment actions without consuming the manual-action identifier.

This candidate retains Classic rules and empty optional-item availability. The inventory and
effect presentation is infrastructure; no new pickup is playable. These results describe the
initial local verification. The later image and native acceptance are recorded above; exact-head
acceptance for the terrain-cache integration remains pending.

## Parent canonical projections with terrain cache — 2026-10-18 — native 52/52, final CI pending

Clean source `20793428554458e69fe7014232d8d08e7ea089bc`, archive SHA-256
`4e490088c7a5f8eaadecdd6414642c4c9c453286772b2fcf00c35750a97f6303`, builds one image:
`sha256:2b4384501ffbd7ac990c7be0f7f609ac9278da61c2afd04f8415a9833cc83258`.
Its saved archive hashes to `39067add856a3f383e658b98d1544f413a95258ff9abd453c1c96661346cfe42`.
Fresh acceptance passes **26/26 Compose and 26/26 Kubernetes**, first attempts, retries zero,
one worker. All ordinary assertions and budgets remain unchanged. The optional capture profiler
is unset; normal failure capture and all 42 configured screenshots are retained.

Each runtime passes 15 functional and 11 latency cases. Four latency profiles retain 20 samples
each, zero exceedances and zero CPU throttling. Local/shaped maxima are **43.964/163.239ms Compose**
and **56.008/163.934ms Kubernetes**. Runtime security, strict exact-200 JSON/no-store probes,
mode-0444 external fixtures, three fresh Kubernetes pod UIDs, direct forwarding in one long-lived
runner and complete cleanup pass. Pinned Trivy archive-only scanning reports zero fixable
HIGH/CRITICAL findings; strict Kubernetes 1.33 validation passes three resources. A setup-only
schema invocation omitted Node from PATH and exited 127 before any scenario; its correction and
initial forwarding-not-ready probes remain preserved. No substantive test was retried.

All **260 web tests** pass; coverage is **94.85/90.08/94.92/97.44%**, above unchanged floors.
Production and diagnostic-fixture builds, contract checks, lint, Knip and E2E static parity 27
pass. This final verification update changes documentation only after the accepted source.
Independent code/integration and complete native-evidence reviews are clear. Final-head hosted
CI and publication remain required. Live rooms still use v6; optional pickups remain disabled.

Evidence: `/tmp/grenadiers-v0323-terrain-native-acceptance`, 945-file final manifest SHA-256
`c68bc0f99c5568e1edf90c9e3e16fe38072703d0222d61c0cdf35bb44f9b4b92`.

## Canonical projection history — 2026-10-18 — native verified

The v0.3.23 candidate passes **373 core tests, 170 server tests and two doctests in each debug and release profile**, with four
dedicated performance profiles intentionally ignored. Rust line coverage is **92.25%**
(10,207/11,064 lines), above the unchanged 80% floor. All **259 web tests**, contract checks,
ESLint, Knip, builds, E2E static checks, dependency audits and strict pinned Kubernetes schemas
pass. The application still uses gameplay-v6; optional items remain disabled. The reviewed renderer
lifetime correction is integrated; web statement/branch/function/line coverage is
**94.81/90.08/94.90/97.42%**, and all 26 E2E static scenarios match.

Independent review is clear. Regression tests establish complete-world validation before
filtering, hidden record and ownership checks, exact event/world fuse consistency, paused
Stopwatch provenance, eliminated actor cleanup, historical pickup geometry, captured audiences,
recipient-local indexes, bounded reports and atomic two-perspective encoding. The active legacy
projector directly rejects non-public audiences without partial encoding. Read-only observations
preserve physical-clock expiry and frozen-result behavior. Archived contracts and replay records
are unchanged.

The pre-correction image `sha256:1b5b5e39a78178c50df3de226621c0307249d8003baffc33ab170db0826eb3e7`
passed Compose 25/25 and Kubernetes functional 15/15 with zero retries. Kubernetes latency was
not run: the evaluation was interrupted after confirming the release-profile takeover defect.
Evidence remains at `/tmp/grenadiers-v0323-acceptance`; that image does not certify the fix.
The fix and strengthened acceptance are integrated, and full debug/release gates pass.

Clean source `fc37bbf996ca3fbed5ff2c085702a26ede47b8ea`, archive SHA-256
`d42dc6a3a9edd29bd9ccf4a6ba8c6c987d79023deaed5159644f0cca6395b178`, builds fresh image
`sha256:9e3d9611f5430b91002ec15fc2ef79941570862476a0a577510d264d40da29c9`.
The image archive hashes to `39ac9b8792a07fb3b4123cf3449e11d5a44998012e28711a35fc5ca99d92f69c`.
Fresh first-attempt acceptance passes **26/26 Compose and 26/26 Kubernetes**, retries zero,
with independent evidence review clear. Local/shaped maxima are **56.641/142.426 ms** in
Compose and **49.553/159.893 ms** in Kubernetes; each profile has 20 samples, zero exceedances
and zero CPU throttling. Pinned archive-only Trivy finds zero fixable HIGH/CRITICAL findings;
strict Kubernetes schemas validate 3/3. Runtime hardening, strict no-store probes, external
mode-0444 fixtures, fresh pod/forward lifecycles and cleanup pass.

Evidence is `/tmp/grenadiers-v0323-guarded-acceptance`; its summary SHA-256 is
`810e04c5015119d58782276afc07b4ac42b1b2e00f8c9b886cbea693fe0d602a`. The selective evidence
manifest verifies; it does not hash every raw report. The post-success shell wrapper variable
failure, wrong preparation arity, missing generated dependency and sandbox-blocked schema fetch
remain preserved as setup-only failures, before any browser scenario. No scenario failed or
was retried. Saved files prove approval-gate ordering and continuous direct-pod forwarding;
the coordinator ACK content and literal PTY attachment remain external to those logs. Runtime
seed modes are recorded, but their hashes were not independently read inside containers.

This certifies local native acceptance, without a hosted-CI or production-deployment claim.
Publication requires exact-head hosted gates after the predecessor is merged. Later changes
to application or browser test inputs require their corresponding evidence to be refreshed.

## Static terrain cache — 2026-10-18 — native 52/52, final CI pending

Clean source `a9619e6a033e8956081bb7d4804078d61e4ae69c`, source archive SHA-256
`6505637293a5e172f11b229ecc854a0571c4a71677d2172c924b0b4d41abfb07`, builds one image:
`sha256:4c936d86c563c05ab3b6192a15da6975e6c6e0a9803cba0f51931dd581011f47`.
Its saved archive hashes to `c0c4348851d208d7b07418fd0c5a79b71a4d171d322129227f60f357fd8fb47f`.
Full fresh acceptance passes **26/26 Compose and 26/26 Kubernetes**, first attempts, retries zero.
All ordinary assertions and budgets remain unchanged; the optional profiler is absent and normal
failure screenshots are enabled. The 42 configured PNGs are retained.

Each of four latency profiles has 20 samples and zero exceedances/throttling. Local/shaped maxima
are **58.617/159.317ms Compose** and **63.426/159.126ms Kubernetes**. Strict health/readiness,
UID/GID 65532, zero capabilities, no-new-privileges, read-only roots, external mode-0444 fixtures,
RuntimeDefault/500m/256Mi, three fresh pod UIDs and direct-pod forwarding lifecycles pass.
The isolated diagnostic-shell HTTP000 is preserved separately; the same-host gate and actual
runner preflights return exact 200 JSON/no-store. Owned containers, network, namespace, forwards
and listeners are gone. The frozen image scan has zero fixable HIGH/CRITICAL findings and pinned
Kubernetes schemas validate 3/3; neither build nor scan was repeated.

The separately bounded original-workload confirmation passes once in 22.031s. Captures take
2.710/1.885s versus the frozen 11.452/17.169s baseline, as one observation. Submitted indices per
observed animation callback fall from 9,234 to 2,718 (70.57%); draws fall from 1 to 0.5. The cache
also introduces 0.5 wrapped buffer uploads/callback, with an 86,976-byte supplied-source upper
bound/callback. These are CPU-side measurements; callbacks are not frames, source spans are not
actual upload sizes, and different-duration CPU totals do not establish per-frame efficiency or
the CI timeout's cause. Independent code, pixel and one-run evidence reviews are clear.

All 260 web tests and unchanged coverage thresholds pass, alongside contracts, lint, Knip,
production/fixture builds and E2E static parity 27. Isolated exact-pixel tests pass at DPR 1 and 2
through paint, repaint, reset and real context restoration. The first fixture's raw restore call
timed out; its preserved correction uses Pixi's deferred forced-loss restoration path. The passing
list report and retained exact assertions establish zero differing bytes in each phase; attachment
bodies were not separately persisted. The stopped six-cell matrix was not rerun.

Independent full-native evidence review is clear. Evidence:
`/tmp/grenadiers-v0322-terrain-native-acceptance`, 922-file manifest SHA-256
`8f9818480cf59c0e3c2cc2c237ef9e418c60933d5a46555686f674938128eda9`.
One-case evidence: `/tmp/grenadiers-v0322-terrain-capture-acceptance`, 645-file manifest
`422bb4077a082ae21c4505e42f8e20557844174ab7ad6045abd6adde4e18937a`.
Final-head hosted CI and publication remain separate requirements; live rooms still use gameplay-v6.

## Original capture workload — 2026-10-18 — attribution stopped

Clean observer `4fcef4ba32fed92ce3324b09d2f815bf9f4aea7d`, archive SHA-256
`0530df46dcbb8d2806161d7e608a3d12e7172ca6e3bf2dc960d842f8447beb73`, passes the original
first reconnect case once, first attempt/retry 0, 47.529s. Pinned Linux pure fixtures pass 1/1
and all E2E static checks/scenario parity 26 pass. The native profile deliberately reuses original
production image `57f16c...3853`: application source/dependencies/build are unchanged, while
development-only fixture/script additions mean the whole Docker input context differs.

The first/second intended capture commands take 11,452.465/17,169.343ms. All eighteen stage
pairs complete; no pending stage, diagnostic failure or temporary page remains. Automatic failure
screenshots are disabled only in the opt-in profiler; explicit captures, assertions, natural
inputs, seed, 120-second scenario budget and zero retries remain unchanged.

Over the first audit/capture observation window, the primary submits no GL draws; the peer
submits 1,426 draws/13,167,684 indices and no wrapped uploads, with untruncated counters.
These are CPU submissions/callbacks, not completed frames. GPU-process CPU increases 24.53s
across the wider trace-start-through-shutdown CPU bracket; this is not utilization over the
11.873s Node window. The 195-cell/47-block public snapshot is not a per-canvas visible inventory.

Trace commands, fixed clock marker, CPU snapshots and detach succeed, but Chrome reports data
loss, bounded samples truncate and five flow IDs are unsupported. Retained PNG encoding 31.175ms
is positive evidence only; missing trace data cannot establish exact causality or absent work.
Attribution stops. The native capture exceeds one second, so no hosted profiling follow-up is
authorized by this packet. Original baseline/matrix reruns and tracing expansion are excluded.

Complete CPU/submission evidence separately justifies one terrain-only texture-cache package,
with exact-pixel/lifecycle tests and one changed-image original-workload confirmation. This is
a rendering-efficiency change, not proof of the capture cause. No block/dynamic caching, ticker
change or gate relaxation belongs in it. Final-head CI/publication remains outstanding.

Independent evidence review is clear. Strict probes, app hardening/0444 seed and cleanup pass; owned browser/app containers and
network are absent. Frozen evidence: `/tmp/grenadiers-v0322-capture-profile-acceptance`, 595-file
manifest SHA-256 `2863e684890909fce79e5e769f72e838c88862dfddbf4eb34e6cc60b6ec39c48`.

## Isolated renderer matrix — 2026-10-18 — non-reproduction

Clean source `232eba28401af0eb9e50ce3f9649e879e4ecca72`, archive SHA-256
`189468834f21f9d2fa8d031b862abac0494ad716b3b84d4aeb106b4ab057ec48`, runs the predeclared
six-cell matrix once. All six cases pass on their first attempts with zero retries. The pinned
browser is `66b346...a91c`, Node24.21.0 and Playwright1.63.0; browser cpuset0,1 has no CPU quota
and Docker networking is disabled. All11 fixture assets match manifest SHA-256
`afbee68eb81b1886a0fe8ecd383fb4306e6de85d9dbe14fcc7d13b28662541ea`.

| Primary/peer | Capture command, ms |
| --- | ---: |
| Empty/empty |104.066|
| Active/empty |645.068|
| Suspended/empty |60.704|
| Empty/active |547.537|
| Active/active |654.718|
| Suspended/active |552.255|

All54 stage pairs complete successfully, with no pending stage or diagnostic failure.
Pre/post state checks preserve canvas identity, visibility, mode and actual update histories.
Both planned active/suspended comparisons have different delivered ordinal histories, so
neither qualifies as a matched contrast. No multi-second capture reproduces. The experiment
is stopped without enlargement or rerun; no GPU cause or production lifecycle remedy follows.

Independent evidence review validates source, assets, reports, histories, isolation arguments,
probes and cleanup. Inputs are read-only bind mounts; the container root and temporary paths
remain writable. No cross-origin requests or WebSockets are observed; same-origin requests
are permitted and are not independently logged. These limits correct broader original summary
wording in a preserved audit addendum. Evidence: `/tmp/grenadiers-pixi-lifecycle-acceptance`;
original622-file manifest SHA-256 `07c6f63142938a5af0aa5a1d0d511db359d9206b358fc457f8af7411cfe0ffc7`,
audit17-file manifest `7f044086a30e360f12e06036eeedf8e8e1b1c1567bdd97cdd4ce2e5c1a0d1dd2`.

The final attribution task subsequently profiled the original natural reconnect case with explicitly
opt-in, bounded GL submission counters, public scene counts and canvas geometry, process CPU
deltas and an arguments-stripped browser trace around its first audit/capture. Production
rendering and release assertions remained unchanged. Its completed result, interpretation limits and
separate terrain-only optimization decision are recorded above. No hosted profile follows.

## Hosted capture attribution — 2026-10-18 — prior evaluation

Exact instrumented head `482c8ba59ce4c2e0de413ecae7c020cf68ade51b` fails both browser jobs
(`39089475906` PR, `39089473104` push); the eight other jobs pass. Both intended primary
captures have successful `Page.captureScreenshot` boundaries lasting **109,047/110,097 ms**,
with every surrounding stage below20 ms. Capture and restoration finish after the120-second
test budget is exhausted; the following peer audit is then aborted. Both results are first
attempts with zero retries. Captured-page heartbeats continue near one second, and Node
event-loop maxima are78.7/102.0 ms; this does not identify a Chromium/Pixi cause.

Default, pickup, match and the second reconnect case pass. Later latency, hardening, controls,
audio and the image scan do not run after the reconnect failure. Raw artifacts, decoded tests
and fixed-metadata stage summaries remain at `/tmp/grenadiers-v22-capture-ci-{pr,push}-*`.
No unchanged workflow is restarted.

Review finds a diagnostic attribution defect: an automatic failure screenshot starts before
the explicit screenshot's outer promise returns, so its later preparation events reuse the
same capture ordinal. The first canonical capture duration above is independently confirmed;
the automatic tail is not another intended sample. The repair stops recording before any
secondary pipeline while forwarding all original operations unchanged and retaining pending
first-pipeline starts without invented finishes. Independent review is clear; full E2E static
gates/parity26 and the pure Chromium diagnostic pass1/1 in3.2 seconds. An initial synthetic
fixture receiver error is corrected before the passing run; native timeout-path validation
of the cutoff remains pending.

The separate [renderer lifecycle experiment](game/renderer-diagnostics.md) was prepared to compare six
predeclared pristine/active/suspended two-context cells using the actual Pixi renderer and
synthetic public presentation data. It does not modify production rendering, game authority,
release screenshot options, deadlines or retries. Its completed non-reproduction is recorded
above. Independent fixture review is clear after correcting
warm-up error propagation, shared initialization/warm-up/treatment barriers, actual history
and reduced-motion evidence, post-capture invariants and cleanup precedence. Final E2E static,
fixture lint/typecheck/build and ordinary-production marker isolation pass. Normal web gates
pass259 tests with94.81/90.08/94.90/97.42% statement/branch/function/line coverage. The initial
diagnostic build finds a nullable-host type error; a checked host accessor fixes it. No matrix
cell had run at this preparation boundary. The fixture and its output remain separate from normal production assets.

## Screenshot capture attribution — 2026-10-18 — CI blocked

Clean diagnostic source `b2d966647310924257cb591625abe4478199d107` passes the changed
reconnect and pure-diagnostic cases **2/2 Compose and 2/2 Kubernetes**, first attempts with
zero retries, against the unchanged production image `57f16c...3853`. Original screenshot
stage timings localize the multi-second wait to completed `Page.captureScreenshot` calls:
Compose12,780.635/17,782.816 ms and Kubernetes5,368.567/14,757.926 ms. Every surrounding
stage finishes below9 ms. Heartbeats continue during captures, with zero temporary pages,
diagnostic failures or pending stages. Strict probes, runtime hardening, external fixtures,
continuous direct-pod forwarding and cleanup pass. Evidence remains in
`/tmp/grenadiers-v0322-screenshot-stage-acceptance`. This localizes a command boundary
without identifying its cause or resolving the CI timeout.

A separately reviewed foreground experiment at `f770970ee080189164750e1ede1917558c718020`
activates each page before its audit, width assertion and unchanged full-page capture. Its
static gates/parity26 and pure Chromium diagnostic pass; fresh native cases pass2/2 per target.
Activation takes0.4–1.6 ms, but captures still take13.0/18.1 seconds in Compose and12.1/16.9
seconds in Kubernetes. This does not support foregrounding as a remedy. The experiment is
withdrawn; browser test inputs return exactly to the stage-diagnostic source above. Negative
evidence remains in `/tmp/grenadiers-v0322-foreground-acceptance`.

Evidence review corrects both native summaries: reconnect has a120-second budget, while the
pure diagnostic has60 seconds. Two retained audio cases also execute the changed diagnostic
helper, so they require fresh acceptance. That evaluation's strict rollup is **50/52**, consisting
of four fresh cases and46 unaffected retained cases; earlier52/52 generated status files are
superseded by audit addenda. Initial HTTP000/not-ready probes establish no transport cause.
An expanded manifest now seals the source archive, runner, raw reports, probes, fixture modes,
runtime/security and cleanup evidence while preserving earlier manifests. Fresh
audio acceptance closes that gap below. No assertion, screenshot option, deadline or retry is relaxed.

Clean documentation head `6e11870f40bdc6da222f30449c7723ebb5ff9ff2`, archive SHA-256
`3ac713930e83ed8edd91d2e87d5cd4a469111a43026b36ccd5246fa04e88277f`, has byte-identical
E2E inputs to `b2d9666` and identical production inputs to `954b0ad`. Fresh headed audio
passes **1/1 Compose and 1/1 Kubernetes**, first attempts with zero retries and the original
60-second budget. Strict probes, runtime hardening, external/projected0444 fixtures, fresh
pod identity, continuous direct-pod forwarding and cleanup pass. This closes the composite
at **26/26 per target,52/52 total**: four stage-diagnostic cases, two fresh audio cases and46
unaffected retained cases. Evidence and scope addenda remain under
`/tmp/grenadiers-v0322-audio-closure-acceptance`. The native composite does not establish that
the hosted capture timeout is resolved; the next exact-head CI evaluation supplies new
screenshot-substage observations. That evidence-only update changes neither application nor
E2E inputs; later diagnostic changes are recorded separately above.

## Screenshot substage diagnostics — 2026-10-18 — native attributed

Exact guarded-audit head `1549e8fe75c3ea2e0bb97abc90714b93f304d42d` fails the browser job in
both hosted evaluations (`39087361314` push, `39087363884` PR); the eight other checks pass.
Both audits and width checks complete promptly with zero temporary-page events. Push width
commands take29.1/7.3 ms; screenshot intervals are34.0/88.8 seconds before timeout. PR width
commands take11.1/20.4 ms; screenshot intervals are66.0/155.0 seconds before the timed-out
result. High-level returned boundaries also run on errors and do not prove successful capture.
Retained heartbeats continue promptly, while Node event-loop maxima stay94.7/113.2 ms.
Artifacts and decoded steps remain under `/tmp/grenadiers-v22-guarded-ci-{push,pr}-*`.

The next E2E-only probe uses pinned Playwright1.63.0 instance-local hooks to time original
viewport, preparation, font-progress, full-page-size, layout-metrics, capture and restoration
stages. It records only fixed labels, ordinals, clocks and success/error flags in a separate
128-event buffer. Arguments, results, metadata, error text, screenshot bytes and paths are not
retained. Original method receiver, arguments, return/promise identity and errors pass through.
Cleanup restores descriptors/listeners and deactivates recording before pending work settles.
Unsupported versions or private shapes fail explicitly; production sources are unchanged.

Independent review is clear after closing a late-promise-recording lifecycle defect. Regressions
prove pending resolve/reject and retained listener callbacks cannot mutate the stopped buffer.
Full E2E static gates and parity26 pass; the existing pure Chromium case passes1/1 in3.1 seconds,
including an actual safe static full-page screenshot with all nine stage labels. The initial
sandbox launch failure occurs before test execution; the permitted run is recorded separately.
Existing screenshot options, ordering, gameplay/accessibility assertions,120-second deadline
and zero retries remain unchanged. The native attribution result is recorded above; earlier
high-level timings alone do not establish a screenshot substage or browser cause.


## Guarded single-frame accessibility experiment — 2026-10-18 — native verified

Only the reconnect accessibility audits switch to Axe's legacy single-document path, avoiding
its partial-run temporary pages. The helper subscribes before its initial frame check, requires
one frame before and after analysis, and rejects any frame attachment during the audit. Cleanup
removes its listener and preserves the original audit failure. The original rules, violation
assertions, operation order, width expression, screenshots, 120-second deadline and zero retries
remain unchanged. Application sources and the tested production image remain identical.

The existing pure Chromium diagnostic now compares normalized violation-rule IDs against the
current Axe mode on a controlled fixture and proves both report an unlabeled button. It also
covers an existing iframe, a transient frame and cleanup/error precedence. E2E typecheck, lint,
Knip and parity26 pass; the focused Chromium case passes1/1 in2.8 seconds. Its initial sandbox
launch fails before test execution; a later implementation test finds an invalid Set matcher,
which is corrected to `.size` before the passing run. Independent implementation review is clear.

Clean source `96c4608454b69d92b08961cdff0cc3d81d882e24`, archive SHA-256
`aea9f1b215d057e7977f1e10a48aa6e8315c582395356f9c97bf95ea7a134f75`, retains identical
production Docker inputs and uses the same tested `57f16c...3853` image. Changed reconnect
and pure-diagnostic cases pass **2/2 in each target** on their first actual invocation;
48 distinct unaffected cases are retained, giving **26/26 Compose and 26/26 Kubernetes**,
52 total with zero retries. Latency profiles retain 20 samples each, zero exceedances and zero
throttling. The new pure-test prefix followed by those profiles has not run as one current-source
native sequence; the prefix closes its isolated context, and no residue defect was found.
Runtime hardening, strict probes, external fixtures, continuous direct-pod forwarding
and cleanup pass. A sandbox-denied read-only preflight is preserved before any browser launch.
Evidence: `/tmp/grenadiers-v0322-guarded-axe-acceptance`; independent evidence audit is clear.

Both reconnect traces contain zero temporary-page events and diagnostic failures. Width
commands take **2.25–26.42 ms**, with callback entry within 0.78–9.73 ms and actual reads 0–0.1 ms.
Heartbeat delivery stays below 14.2 ms, without the prior 90–109-second backlog. Screenshot
phases instead take **12.5–18.4 seconds**, compared with 0.36–0.98 seconds in the preceding
native run; overall improvement is not claimed. This supports the bounded audit-path change
without establishing a Chromium/Axe root cause or resolving every screenshot delay.

Hosted gates must pass on the exact publication head before merge; local acceptance does not
substitute for those checks. Earlier failed evaluations remain preserved. The later evidence-only
commit changes neither application inputs nor the native-tested browser code.


## Post-audit browser phase diagnostics — 2026-10-18 — native verified, CI blocked

The reconnect probe now records bounded Node command boundaries around the existing Axe,
width and screenshot operations, page-clock markers around the unchanged width expression,
one heartbeat per second on the existing collector interval, and ordinal-only temporary-page
lifecycle events. Diagnostics attach before context teardown and retain the original test
failure if cleanup also fails. Gameplay, accessibility, overflow and screenshot assertions,
the 120-second deadline, and zero retries remain unchanged.

Independent review is clear. E2E typecheck, lint, Knip and scenario parity26 pass; the existing
pure Chromium diagnostic passes1/1 in2.8 seconds. Its initial sandboxed launch failed before
test execution; the permitted launch and its result are recorded separately. No production
source changes accompany these diagnostics. Page timestamps describe event creation, while Node timestamps
include asynchronous binding/IPC delivery and scheduling. Missing markers or heartbeats alone
cannot identify a layout stall; no cause is claimed for the post-audit CI failures below.

Exact source `31be38d2708d91fcc36b3d93ba5179b492d3b73a` has byte-identical production Docker
inputs to `954b0ad`, so acceptance reuses image
`sha256:57f16c866bff370640fe99edd367ee3378e5a7f17983a1cdcb5fd87dc6373853`.
Changed Compose cases pass **2/2**; retained cases supply 24 distinct passes. Changed/remaining
Kubernetes cases pass **15/15**; retained cases supply 11 distinct passes. The transparent rollup
is **26/26 per target, 52/52 total**, retries zero, with an independent evidence audit.
Kubernetes local/shaped maxima are **51.901/158.792 ms**, 20 samples per profile, zero
exceedances and zero throttling. Strict probes, hardening, mode-0444 external fixtures,
continuous direct-pod forwarding and cleanup pass. The initial evidence-runner URL-variable
collision fails before reaching the app; its failed setup and the earlier actual position
failure remain preserved. Evidence: `/tmp/grenadiers-v0322-phase-acceptance`.
Literal PTY attachment and the content of the coordinator quiet acknowledgment cannot be
independently established by saved process logs; the continuous PID lifecycle and gate order
are recorded. This rollup combines retained and fresh cases against the same production image.

Both exact-head CI runs fail the browser job (`39085949336` PR, `39085946734` push); the eight
other checks pass. Push primary Axe passes, then width evaluation remains unresolved for
109.347 seconds before `Test ended`. Retained heartbeats were created after command submission
but delivered in a teardown burst 90–109 seconds later. Missing markers do not prove that page
execution stopped, since marker delivery shares that backlog. In the PR, the peer width
command takes 20.115 seconds while its actual read takes 0.1 ms; this observed delay precedes
callback entry. PR activity arrays fill their 256-event bounds, so absent earlier markers may
have been evicted. The primary width/screenshot take 49.054/25.192 seconds; the peer screenshot
spans115.951 seconds before the test reports its 120-second timeout. Node event-loop maxima are
136.6/165.0 ms. Artifact and decoded step prefixes are `/tmp/grenadiers-v22-phase-ci-{push,pr}-*`.
The local successful width reads take 0–0.1 ms, with 11.4–12.8 seconds before callback entry.
This narrows the measured delay without attributing a browser, protocol or Axe defect.

## Takeover position restoration probe correction — 2026-10-18 — acceptance pending

Clean pose candidate `954b0ad551293e95a5cfc33ea876f476ec05225b` builds image
`sha256:57f16c866bff370640fe99edd367ee3378e5a7f17983a1cdcb5fd87dc6373853`. Pinned scan finds
zero fixable HIGH/CRITICAL vulnerabilities; strict Kubernetes schemas validate3/3. Compose
passes26/26. Kubernetes passes11 cases, fails the first reconnect case and leaves14 unrun.
The failed takeover position is x5680, exceeding an artificial center bound of5632 by48.
The movement helper explicitly accepts positions within128 units of the target center. No
production movement or browser stall failure is observed in that assertion: retained primary/
peer long-task maxima are270/263 ms, with no measured slowGL call. Cleanup and frozen harness
checks pass. Evidence remains `/tmp/grenadiers-v0322-pose-acceptance`.

The corrected probe verifies the intended safe cell(5,1) before takeover and exact equality
between that stationary public position and the replacement admission. Actual replacement
movement, action floor, canvas lifetime, budgets, deadlines and retries remain unchanged.
Independent review and E2E static parity26 are clear; fresh changed-scenario/remaining acceptance
is pending. Earlier failed
runs remain preserved; this correction does not explain the previously recorded13-second task.

Exact pose-head PR/push CI (`39084363475`/`39084360049`) both fail the unchanged120-second
reconnect budget after passing takeover and new-room checks. The primary accessibility audit passes in both; the push also completes the peer audit.
In the PR, the subsequent width evaluation never returns; in the push it takes65,888 ms,
then6,085 ms for its screenshot. The peer width read takes17,786 ms and its screenshot19,730 ms
before timeout. Completed page long tasks peak around209–217 ms with no measured slowGL call.
Cleanup errors follow the original blockage. This differs from the earlier13-second peer task;
post-audit command/context scheduling versus native layout remain to be attributed. Artifacts
and decoded steps are preserved under `/tmp/grenadiers-v22-pose-ci-{pr,push}-*`.

## Idempotent actor pose correction — 2026-10-18 — acceptance pending

The renderer's pose setter previously hid every frame then re-showed the selected one on
all render ticks. Actual Pixi visibility setters invalidate the parent render-group structure,
causing instruction rebuild and buffer upload pressure even for unchanged poses. Snapshot
updates also transiently reset moving actors to frame zero before choosing frame one again.
The candidate assigns final visibility directly and selects the frame once during rendering.
Real-Pixi regressions fail before the correction and pass afterward, proving zero visibility
changes for unchanged poses and repeated snapshots within the same walking frame.

All **259 web tests** across 37 files pass; statement/branch/function/line coverage is
**94.81/90.08/94.90/97.42%**. Contract checks, lint, Knip and production build pass. This fixes
confirmed unnecessary work; it does not attribute the separate 12,928 ms peer task below.
Independent review is clear. A rebuilt image, fresh native acceptance and final-head CI remain required.
## Historical v22 arena renderer lifetime correction — 2026-10-18

The single instrumented Linux run of clean harness `1bf4d74` passes its reconnect case in
27.7 seconds, without retries, using the unchanged `efb5c7f...` image. On the primary page,
`WEBGL_lose_context.loseContext` takes **1,067.4 ms** inside a **1,071 ms** long task,
covering **99.664%** of it. The sanitized caller chain reaches renderer destruction. This
attributes that measured primary stall to context teardown. Peer long tasks of 1,921/779 ms
have no corresponding slow wrapped GL call; their cause remains unproven. Diagnostics and
source/probe/runtime evidence remain at `/tmp/grenadiers-v0322-gl-attribution`.

The candidate retains one lazy renderer per mounted arena, suspends/hides/clears it between
worlds, paints the latest baseline before reveal, and releases it on final component unmount.
Async ownership, StrictMode cancellation and later-activation initialization retries are explicit.
Native reconnect acceptance also checks that the original canvas survives restore and remains
connected but hidden after terminal takeover. Deadlines and retry policy remain unchanged.
The v0.3.22 renderer verification passes **257 tests** across 36 files, with statement/branch/function/line
coverage of **94.81/90.05/94.90/97.42%**. Lint, TypeScript, Knip, production build and contract
checks pass. Focused lifecycle coverage includes first-paint and suspension failures, bounded
activation retries and recovery after an initialization completes while no world is active.
Independent review is clear. Exact source `96d6cade2ddc364986db489ee61caa93b7f654af` builds image
`sha256:2054de0dd56ffd95fe8732db77763366ad382c2e4c384036dfef6ae3fbb81477`. Its fresh acceptance
passes **26/26 Compose** and **26/26 Kubernetes**, retries zero, with an independent evidence
audit. Pinned archive-only Trivy finds zero fixable HIGH/CRITICAL vulnerabilities; strict
Kubernetes schemas validate 3/3 resources. Local/shaped maxima are **55.935/140.741 ms** in
Compose and **74.746/158.153 ms** in Kubernetes, with 20 samples per profile, zero exceedances
and zero throttling. Runtime hardening, strict probes, external mode-0444 fixtures, single-PTY
port-forward ownership and cleanup pass. Initial PF probe attempts return HTTP000 before the
second strict probe succeeds; PATH/DNS setup failures occurred before browser execution and
remain preserved. Source comparison has no source-content differences, only generated/Git
directories. Evidence: `/tmp/grenadiers-v0322-renderer-acceptance` and independent audit
`/tmp/grenadiers-v22-renderer-native-audit.md`. This image predates the pose correction above.

Exact-head PR CI `39083297027` passes all five checks. Push CI `39083293354` passes four;
its browser job fails the replacement-movement observation after ten seconds. The peer has a
**12,928 ms** long task covering Node-clock **13,346.3–26,274.3 ms**, beginning before takeover;
primary snapshots continue while peer observations lag. Node event-loop maximum is 38.896 ms.
No measured GL call exceeds 50 ms, but upload/program-compilation paths are not wrapped.
This peer blockage remains unattributed. Failure logs/artifacts are preserved under
`/tmp/grenadiers-v22-renderer-ci-push-*`; no retry or weakened assertion is used to replace it.

Before the renderer change, the corrected `b062127` source passes all ten exact-head CI checks;
both browser logs contain no retries, flaky results or failures. Native acceptance and its
independent evidence audit support **26/26 Compose** and **26/26 Kubernetes** on the same
image. Compose local/shaped maxima are **53.811/158.586 ms**; Kubernetes **50.276/158.226 ms**,
with 20 samples per profile, zero exceedances and zero Kubernetes throttling. The final
Kubernetes process owns its forward in one PTY through strict probes and all checks. See
`/tmp/grenadiers-v0322-k8s-lifetime-acceptance` and `/tmp/grenadiers-v22-final-native-audit.md`.
All prior failed evaluations remain preserved. No registry image or production deployment is claimed.

## Release-profile connected takeover correction — 2026-10-18 — acceptance pending

The release-profile owner regression fails on the earlier candidate: binding replacement was
performed inside `debug_assert!`, so release builds omitted the mutation. Debug tests and a
browser probe that only read the admission baseline did not detect this. The correction performs
the mutation unconditionally and asserts its result separately. Focused release and debug
regressions now pass, including replacement commands, stale binding rejection, stale guard
cleanup, genuine disconnect and private baseline/action-floor restoration.

The browser takeover scenario now requires a real scoped movement command from the replacement
socket to change the peer's authoritative observation before stopping. The original watchdog,
action-floor assertion and rejection checks remain. The server gate now runs the complete Rust
workspace suite in debug and release profiles. Full gates pass **356 core tests, 170 server
tests and two doctests in each profile**, with four dedicated performance profiles ignored.
Rust line coverage is **93.40%** (9,289/9,945 lines). Fresh native image acceptance remains
pending; the pre-fix image below does not certify this correction.

The mutation defect does not explain the earlier missing-admission timeout: the admission
reply precedes the faulty expression. Final-head CI also recorded a separate audio visibility
failure. Both logs remain at `/tmp/grenadiers-v0322-ci-{pr,push}-failure.log`; neither is described
as fixed. Bounded browser diagnostics now retain per-socket control history, current-document
focus/visibility, long tasks, scheduling gaps and Node event-loop delay without changing
assertions, timers, input order, latency budgets or retry policy. Independent review is clear;
the focused Chromium lifecycle, secret-allowlist and diagnostic fault-injection scenario passes.
The E2E static gates and scenario parity pass for **26 scenarios**.

A further exact-head CI evaluation at `8ca66863d51dde294818c5b8fd0a7946500db1b0` fails
its strengthened takeover scenario. Per-page diagnostics show concurrent 3.954/3.736-second
long tasks while Node event-loop delay remains below 25 ms. The admission succeeds, but a live
second bomb ends the round before the replacement's movement is sent; it receives `invalidPhase`.
The probe's still-attached admission listener then closes that socket on the later error, and
stop cleanup masks the original movement failure. The scenario now escapes and clears the
second bomb before the authority check; the first disconnected restore still validates a live
bomb. Settled admission listeners detach, and cleanup preserves primary failures. These are
test lifecycle corrections, not a claimed explanation or fix for the observed browser stalls.
The companion push run records a 16.038-second primary-page long task, while Node delay
remains below 30 ms; server admission is observed before that stall. Its later movement is
stale after natural round advancement. Both failures are retained under
`/tmp/grenadiers-v0322-fixed-ci-{pr,push}-*`.

Fresh image source `8ca6686` passes Compose functional **15/15**. Kubernetes passes default
8/8, pickups 1/1 and match 1/1; reconnect has **one failure and one pass**, matching the
confirmed test hazard/cleanup defect. Remaining Kubernetes functional groups and both latency
groups are unrun. Evidence is preserved at `/tmp/grenadiers-v0322-fixed-acceptance`; this is
partial, failed acceptance. The fixture/listener/cleanup correction has independent review
clear and all E2E static gates pass; fresh changed-scenario acceptance remains pending.

The corrected `b062127` harness passes its changed Compose reconnect case and all **11 latency
cases**, with zero retries. Combining unchanged functional evidence gives **26/26 Compose**
on the same immutable image. Local/shaped maxima are **53.811/158.586 ms**, with zero values
above 250 ms. A Kubernetes setup attempt lost its forward when the setup shell exited; no
application scenario completed in that attempt. Its failure is preserved separately, and a
single-process orchestrator now owns the forward through preflight and browser invocation.
Kubernetes acceptance remains pending.

Optional diagnostics now time existing WebGL draw/delete/context calls, including requested
`WEBGL_lose_context.loseContext`, without adding GL calls. Only calls taking at least 50 ms
are retained (64 per page), with context ordinals and bounded sanitized caller frames. Native
return values, receivers, arguments, exceptions and extension identity remain unchanged.
Independent review, static gates and the focused real-Chromium transparency/bounds test pass.
A separate instrumented Linux reconnect run is required before attributing any stall to teardown
or rendering; the instrumentation itself is not a production fix.

## Dormant v7 contract — 2026-10-18

Clean archive source `456f47b33d808691295d48688d7b8c3ce916a01e` builds image
`sha256:29ee6050fecc543be326bf30b650bf24bc4074bfba2eb111c5e4710a9b813490`.
The initial evaluation passes **24/24 Compose scenarios**, but **23/24 Kubernetes scenarios**.
No retries were performed. Kubernetes passes all fifteen functional cases and eight of nine
latency cases; one shaped-network west action takes **305.948 ms**, exceeding the unchanged
250 ms budget. That initial evaluation failed release acceptance.

The failed action records key start 14363.782990, key finish 14388.211900, outgoing message
14515.360157 and observed state 14669.730949 ms. Kubernetes CPU throttling is zero; that does
not establish the cause of the spike. Original logs and failure attachments remain at
`/tmp/grenadiers-v0322-acceptance`; investigation must preserve this result. Compose local/shaped
maxima are **49.899/157.624 ms**, zero exceedances. Kubernetes local maximum is **54.463 ms**;
its shaped maximum is **305.948 ms**, with one exceedance. These measure public-state delivery,
not FPS, pixel latency or WAN guarantees.

Artifact analysis splits the failed sample into key start to bridge ingress **101.296 ms**,
configured outgoing delay **50.281 ms**, command forwarding to moved-snapshot ingress
**41.816 ms**, incoming queue **57.177 ms**, and dispatch to page-observation binding
**55.378 ms**. Nearby runner timers are up to **145.951 ms** late. This establishes delayed
measurement callbacks, without establishing their root cause or actual server send timing.
Active server and v6 client input sources are unchanged from v0.3.21. A bounded instrumented
comparison of both exact images passes two live latency profiles per image without reproducing
the spike. That comparison is diagnostic evidence, not release acceptance or a demonstrated fix.

A separately planned instrumented evaluation uses the same candidate image and harness source
`215adfa562cffd62fb6d2eb2194ae96d1f9d2a00`. Compose passes **25/25 scenarios**, retries zero,
with local/shaped maxima **51.576/155.392 ms** and zero exceedances. Kubernetes passes the
default eight cases, then the pickup case fails at initial navigation: the seed rollout replaced
the pod, terminating its existing port-forward. A failed readiness request did not halt that
outer orchestration. The remaining sixteen cases were not run and no failed case was retried.
Evidence remains at `/tmp/grenadiers-v0322-instrumented-acceptance`; its outcome is failed and
incomplete. All owned resources were removed. A corrected fail-fast orchestration evaluation
will create a new port-forward after every rollout and require both probes before Playwright.
This transport correction does not explain or fix the original latency observation.

The corrected Kubernetes evaluation passes **25/25 scenarios**, retries zero, using the same
candidate image and harness. It stops and waits for each old forward before rollout, records
each new ready pod/UID, starts a fresh pod-bound forward, and requires strict health/readiness
before browser execution. The pre-fix rollup is retained **Compose 25/25** plus corrected
**Kubernetes 25/25**, with an independent evidence audit clear. No assertion, sample count,
250 ms budget or retry policy changed. Prior failed evaluations remain failed and preserved.

Corrected Kubernetes local/shaped maxima are **190.340/165.235 ms**, with twenty samples per
profile, zero exceedances and zero CPU throttles. Its Node event-loop maxima are
86.180/35.095 ms; page DOM-key/send absolute gaps are at most 1.7/1.3 ms. These diagnostics
do not establish the original spike's root cause. That **305.948 ms observation remains an
unresolved latency risk**, accepted explicitly alongside the new passing evaluation rather
than described as fixed. Corrected evidence is at
`/tmp/grenadiers-v0322-portforward-fix-acceptance`; all owned resources and listeners are gone.

Local gates pass 356 core tests, 170 server tests and two doctests; four dedicated performance
profiles remain intentionally ignored. All **247 web tests**, contracts, ESLint, Knip, builds,
E2E static/25-scenario parity, dependency audits and strict pinned Kubernetes schemas pass.
Rust line coverage is **93.40%**, above the unchanged 80% floor. Independent contract review and
conservative serialized-frame proof review are clear: Snapshot at most 61,586 bytes, personal
admission 36,472 bytes, below the unchanged 65,536-byte ceiling. A separate schema-valid
representative and explicit oversize rejection tests remain.

Correct API health/readiness, UID/GID 65532, read-only roots, dropped capabilities and no
privilege escalation pass on both deployments. Kubernetes retains RuntimeDefault and 500m/256Mi.
Seeded cases use external mode-0444 fixtures, root-owned in Kubernetes, with no seed in the image.
Pinned archive-only Trivy reports zero fixable HIGH/CRITICAL findings under existing policy.
All owned resources, namespaces, ports and browser containers were removed. Application inputs
remain identical through integration. The v7 contract is dormant; v6 remains active and no new
pickup is enabled. That evaluation retains the original unexplained observation and both
failed evaluations as material limitations. Current acceptance also requires the takeover
correction and fresh gates described above.

## Typed domain bomb triggers — 2026-10-18

Clean archive source `82fe88c299b8e74a410d231b24cb3aad92e7e242` builds image
`sha256:68ed7ca87c1434c6b309b3894ee5814fa77453c022bc62ad2b99e5bde0f729f4`.
Application inputs match reviewed source `8653522543c4ff964128597a4261170dc019863c`;
the accepted harness includes the bounded reconnect diagnostics from v0.3.20.
The same image passes **24/24 scenarios in Compose and 24/24 in Kubernetes, retries zero**:
eight default, pickup, match/rematch, two reconnect, controls, headed visibility, audio and
nine latency cases. All sixteen result files pass. Independent evidence audit is clear.
Evidence is at `/tmp/grenadiers-v0321-acceptance`.

346 core tests, 170 server tests and two doctests pass; four dedicated performance profiles
remain intentionally ignored. All **230 web tests**, contract checks, lint, Knip, builds and
E2E static/24-scenario parity pass. Rust line coverage is **94.22%**, above the unchanged 80% floor.
Independent code review is clear. Regression tests cover checked trigger construction, Classic
rejection before mutation, exact timed behavior, unsupported event-only projection and
publication failure preserving revisions and reports.

Dependency audits pass with zero npm vulnerabilities. Strict pinned Kubernetes 1.33 schemas
validate all three resources. Pinned archive-only Trivy passes the existing HIGH/CRITICAL
`--ignore-unfixed` filter. Both deployments use UID/GID 65532, read-only roots, dropped ALL
capabilities and no privilege escalation; Kubernetes retains RuntimeDefault and 500m/256Mi.
Every seeded run uses an external mode-0444 fixture, root-owned in Kubernetes; no seed is baked
into the image. Correct `/api/v1/health` and `/api/v1/ready` probes return HTTP 200,
`application/json`, status ok and `Cache-Control: no-store`.

Observed local/shaped latency maxima are Compose **50.388/173.047 ms**, Kubernetes
**89.947/161.098 ms**, with zero samples above 250 ms and zero Kubernetes CPU throttles.
These are bounded public-state delivery measurements, not pixel latency, FPS or WAN guarantees.
All owned deployments, namespaces, ports and browser containers were removed.
Integration/documentation changes preserve accepted application inputs. Current gameplay-v6,
Classic rules and journal formats remain unchanged; Remote/Contact triggers are not activated.

## Ordered manual actions — 2026-10-18

Clean archive source `305a5844f3d1add8dff95eca26090c3f21918671` builds image
`sha256:961c04ee26ef79a3bac4459dbb9397a0291db18e882d692503ae2ea2870584a5`.
The same image passes **24/24 scenarios in Compose and 24/24 in Kubernetes, retries zero**:
eight default, pickup, match/rematch, two reconnect, controls, headed visibility, audio and
nine latency cases. Seven latency cases cover queues/observers; two measure native input.
All sixteen deployment result files pass. Evidence is at `/tmp/grenadiers-v0320-acceptance`.

An initial PR browser run timed out in the private connected-takeover probe. Its peer CI run
passed all 24 cases on identical source. Sanitized histories ended in Running at room ticks
653 and 677 with both seats connected; they do not retain state ticks or arbitrary error frames.
The cause remains unproven. No product change, retry, deadline or assertion relaxation was made.
Test source `bd83619e9af68146f4609197530e4414733cd035` adds bounded diagnostics containing only
stage, socket state, public message count/type and elapsed time, with a closed message-type list.

Both reconnect cases pass again through each deployment on the accepted image, retries zero;
Compose takes 38.4 seconds and Kubernetes 40.0 seconds. The other 22 cases are unchanged, retaining
a final rollup of 24 per path. Recheck evidence is at `/tmp/grenadiers-v0320-reconnect-recheck`.
Application build inputs remain identical to the clean image source. The independent matrix
audit and coordinator recheck audit are clear; the initial CI failure remains diagnostic evidence.

336 core tests, 169 server tests and two doctests pass; four dedicated performance profiles
remain intentionally ignored. All **230 web tests**, contract checks, lint, Knip, builds and
E2E static/24-scenario parity pass. Rust line coverage is **94.24%** above the unchanged 80% floor.
Independent code review is clear. Tests cover retained commands, raw Clear versus server fencing,
A/B/A shared-cell conflicts, admission limits/high-water IDs, reconnect and complete replay queues.

Cargo-deny and both npm audits pass with zero npm vulnerabilities. Strict pinned Kubernetes
1.33 schemas validate all three resources. Pinned archive-only Trivy passes the existing
HIGH/CRITICAL `--ignore-unfixed` release filter without a Docker socket. Both paths use UID/GID
65532, read-only roots, dropped ALL capabilities and no privilege escalation; Kubernetes keeps
RuntimeDefault and 500m/256Mi limits. All seeded runs use external mode-0444 fixtures, with a
root-owned Kubernetes projection; no seed is baked into the image. Correct `/api/v1/health`
and `/api/v1/ready` probes return HTTP 200, JSON status ok and `Cache-Control: no-store`.

Observed local/shaped latency maxima are Compose **46.857/159.602 ms**, Kubernetes
**54.549/143.498 ms**, with zero samples above 250 ms and zero Kubernetes CPU throttles.
These are bounded public-state delivery measurements, not pixel latency, FPS or WAN guarantees.
All owned matrix/recheck projects, namespaces, ports and browser containers were removed.
No new item, equipment action, rules profile or wire field is activated by this release.

## Expanded item core foundations — 2026-10-17

Clean archive source `88ac9842be3f32c7c29189feb19a19f4883d24ea` builds image
`sha256:2ba39fe1a0c1faa82e87169a3e1e85169b29d56bcd6cf5cde15f01e90f32384b`.
The same image passes a final rollup of **24/24 scenarios in Compose and 24/24 in Kubernetes,
retries zero**: eight default, pickup, match/rematch, two reconnect, controls, headed visibility,
audio and nine latency cases. Seven latency cases cover queues/observers; two measure native input.

The corrected browser harness source is `9a7cb77d1f5ca2161b3c9d5026cb4c821c37a88d`.
All eight default cases pass again through each deployment using round/revision-correlated
result assertions and one worker. The accepted seeded sixteen cases are unchanged. Original
matrix evidence remains at `/tmp/grenadiers-v0319-acceptance`; recheck evidence is at
`/tmp/grenadiers-v0319-harness-recheck`. Application build inputs remain identical to the clean
image source through integration/documentation commits.

The initial seeded Compose harness mounted a read-only file with host mode 0644. That run is
invalidated and excluded: all sixteen seeded Compose scenarios were rerun with explicit mode
0444 external fixture copies. The random default eight had no seed mount and were unaffected.
Kubernetes used a root-owned mode-0444 projection throughout. No application source or image
changed during either harness correction. Both final matrix and recheck evidence audits are clear.

334 core tests, 163 server tests and two doctests pass; four dedicated performance profiles
remain intentionally ignored. Rust line coverage is **94.21%**, above the unchanged 80% floor.
All **230 web tests**, contracts, lint, Knip, builds and E2E static/24-scenario parity pass.
Independent code review is clear after Heart grace was changed to checked elapsed time;
tests cover its exact 60-tick boundary, repeated damage at the tick horizon and atomic step
overflow, plus expiry, modifier interactions, stack caps and transient-state cleanup.

Cargo-deny and both npm audits pass with zero npm vulnerabilities. Strict pinned Kubernetes
1.33 schemas validate all three resources. Both deployment paths retain UID/GID 65532,
read-only roots, dropped capabilities and no privilege escalation. Kubernetes keeps
RuntimeDefault and 500m/256Mi limits. No seed is baked into the image. Archive-only pinned
Trivy passes the existing HIGH/CRITICAL `--ignore-unfixed` filter without a Docker socket.

Observed local/shaped latency maxima are Compose **47.049/159.419 ms** and Kubernetes
**148.334/163.423 ms**, with zero samples above 250 ms and zero Kubernetes CPU throttles.
These are bounded public-state delivery measurements, not pixel latency, FPS or WAN guarantees.
All owned matrix/recheck projects, namespaces, helpers and forwarding were removed;
unrelated workloads were preserved.

The release establishes inventory/effect/damage state only. No new item pickup, equipment
action, effect visibility, expanded profile or wire field is activated. Classic gameplay
remains the live experience; [expanded-item decisions](game/expanded-items.md) define the next steps.

## Retro pixel-art courtyard — 2026-10-17

Clean archive source `1f591c2e86c78e19e4af1fb12a020f3ecd72c1ed` builds image
`sha256:d8a99fef496cd0cb719308e62d77914945d4f695d04ec718354922db6c0b1967`.
That exact image passes **24/24 scenarios in Compose and 24/24 in Kubernetes, retries zero**:
eight default, pickup, match/rematch, two reconnect, controls, headed visibility, audio and
nine latency cases. Seven latency cases exercise queues/observers and two measure native input.
All sixteen deployment result files pass with no failed tests. A separate native visual helper
passes 1/1; it is additional capture evidence, outside that matrix.

Actual running-bomb and exposed-pickup desktop frames, plus a Polish 320-pixel terminal-flame
result frame, pass visual inspection and overflow checks. [Committed captures](screenshots/pixel-art.md)
show the application rather than the earlier art studies. The narrow flame frame is a retained
RoundEnded result, with decorative animation stopped. New item icons and richer transition
effects remain follow-up artwork; this increment delivers the existing arena/entity set.

The integrated gates pass 317 core tests, 163 server tests and two doctests; four dedicated
performance profiles remain intentionally ignored. All **230 web tests** pass with coverage
above unchanged floors, together with contracts, lint, Knip, build and E2E static/parity checks.
Independent rendering/lifecycle review is clear. Rust behavior and public contracts are unchanged.
Dependency audits pass; strict pinned Kubernetes 1.33 schemas validate all three resources.

Both paths run as UID/GID 65532 with a read-only root, dropped capabilities and no privilege
escalation. Kubernetes retains RuntimeDefault, 500m/256Mi, the exact image digest and root-owned
mode-0444 seed projection; Compose mounts its seed read-only at 0444. No seed is baked into
the runtime image. Pinned archive-only Trivy reports zero vulnerabilities under the existing
HIGH/CRITICAL `--ignore-unfixed` release filter, without a Docker socket.

Observed local/shaped latency maxima: Compose **58.633/157.695 ms**, Kubernetes
**122.876/165.744 ms**, zero samples above 250 ms and zero Kubernetes CPU throttles. These are
bounded public-state delivery measurements, not FPS, pixel latency or WAN guarantees.
Independent evidence audit is clear. Evidence remains at `/tmp/grenadiers-v0318-acceptance`;
owned containers, namespace, helpers and forwarding were removed, with unrelated workloads
preserved. Integration/documentation commits retain byte-identical application build inputs.

The final harness source is `9a7cb77d1f5ca2161b3c9d5026cb4c821c37a88d`. A CI run exposed
two pre-existing harness flakes also reproduced on the pre-art settings head: an unbound
latest-snapshot read could cross the three-second result hold, and two workers overcommitted
software WebGL on a two-core runner. Result assertions now capture the earliest and later
same-round snapshots and the peer's exact revision. CI defaults to one worker; local and
explicit overrides remain. Assertions and deadlines are unchanged; independent closure
review is clear.

The corrected eight default cases pass again through **both deployment paths, retries zero**,
on the exact accepted image. Only two E2E files differ; application build inputs are identical.
The previously accepted seeded sixteen cases are unchanged, giving a final rollup of 24/24
per path. Recheck health/readiness, exact image identity, limits and cleanup are audited in
`/tmp/grenadiers-v0318-harness-recheck`; the original acceptance retains scan, security,
latency and visual evidence. All owned recheck resources were removed.

## Shared room settings / gameplay v6 — 2026-10-17

Clean image source `cf82db46477e0fddc8b9b0e137e5ffb64bb3ec56` builds
`sha256:2a95de3f2252c63a59a293a672454509aca3261d78b0cff627e049c503028ccd`.
Accepted test source `50228642ab9bf9d91faaeaaf343df327d2051e5b` changes only the Polish match
assertion; all Docker application inputs are byte-identical. The same image passes **24/24
scenarios in Compose and 24/24 in Kubernetes, retries zero**: eight default cases, pickup,
match/rematch, two reconnect cases, controls, headed visibility, audio and nine latency cases.
The latency suite includes seven queue/observer regressions and two native measurement profiles.
All sixteen result files report passed with no failed tests.

The new native two-browser scenario applies starting speed level four (5 cells/s), turn
tolerance 64/1,024, one win and a 60-tick fuse. Both peers observe the accepted settings;
actual actor speed, bomb lifetime, one-win match completion and retained rematch rules agree.
English/Polish result copy reflects the configured target. Live Axe caught a heading-order
defect, fixed before final acceptance with a regression; stale result-copy assertions were
corrected without weakening the product checks. Earlier candidates remain diagnostic evidence.

317 core and 163 server tests plus two doctests pass; four dedicated performance profiles
remain intentionally ignored. Rust line coverage is **94.10%**, above the unchanged 80% floor.
All **226 web tests**, contracts, lint, Knip, build and E2E static/24-scenario parity pass.
Cargo-deny and both npm audits pass, with zero npm vulnerabilities. Independent behavioral
review is clear after replay action-profile and minimum-tolerance findings were resolved.
Strict pinned Kubernetes 1.33 schemas validate all three resources.

Both paths return healthy/ready and run as 65532 with read-only roots, dropped capabilities
and no privilege escalation. Kubernetes retains RuntimeDefault, 500m/256Mi limits, the exact
image digest and a mode-0444 seed; no seed is baked into `/app`. Archive-only pinned Trivy
passes the existing HIGH/CRITICAL `--ignore-unfixed` policy; this is a filtered release check.

Observed latency maxima are Compose **49.466 ms local / 161.413 ms shaped** and Kubernetes
**46.800 ms local / 180.287 ms shaped**, with zero samples above 250 ms. No Kubernetes CPU
throttling was recorded. These bounded page-delivery samples are not WAN or human-playtest
guarantees. Actual desktop and 320-pixel Polish [settings captures](screenshots/settings.md)
pass visual inspection and overflow checks. Evidence is retained at
`/tmp/grenadiers-v0317-acceptance`; owned projects, namespace, helpers and forwarding were
removed while unrelated workloads remained. Later documentation commits preserve identical
application build inputs.

## Forgiving turns / gameplay v5 — 2026-10-17

Clean git-archive candidate `14ded36ac84ce09ef770ed5ba5bb8e2eab9d7b6f` builds image
`sha256:7ced5869efeb7e107b64ce5c2821d81a0adaf601470c744e9345d2af3cccfbf7`.
The same image passes **23/23 scenarios in Compose and 23/23 in Kubernetes, retries zero**:
seven default cases including the new native wall-stop turn, pickup, complete match, two
reconnect cases, controls, natural headed visibility/audio and nine latency cases. The latency
suite includes seven queue/observer regressions and two native measurement profiles.
All sixteen suite result files report passed with no failed tests.

The new browser case moves to the physical west-wall stop at x=1,344, 192 units off center,
then turns south to x=1,536 through the reserved opening. Both players observe the living,
aligned actor. Classic-v3 accepts that offset; legacy classic-v1/v2 retain 128-unit tolerance.
Domain tests cover all four requested directions and all five speed levels at ±256 units,
out-of-window rejection, blockers and swept fire. A same-seed/input server regression proves
v2/v3 movement differs and both replay exactly with their recorded rules.

303 core and 151 server tests plus two doctests pass; the four dedicated performance profiles
remain intentionally ignored in the ordinary suite. Rust line coverage is **93.61%**, above the
unchanged 80% floor. All 202 web tests, lint, contracts, Knip, build and E2E static/parity checks
pass. Cargo-deny and both npm audits pass (zero npm vulnerabilities). Independent review found
no actionable issue. Strict pinned Kubernetes 1.33 schemas validate all three resources.

Both deployment paths return healthy/ready and run as 65532 with read-only roots, dropped
capabilities and no privilege escalation. Kubernetes retains RuntimeDefault, 500m/256Mi limits,
the exact image digest, and a read-only mode-0444 seed; no seed is baked into `/app`.
Pinned archive-only Trivy passes the existing HIGH/CRITICAL `--ignore-unfixed` release policy;
this filtered result does not assert an advisory-free image.

Observed latency maxima: Compose local **51.152 ms**, shaped **167.763 ms**; Kubernetes local
**57.610 ms**, shaped **232.816 ms**. No sample exceeds the existing 250 ms reference envelope.
These are bounded page-delivery measurements, not WAN or human-playability guarantees.
Evidence is retained at `/tmp/grenadiers-v0316-acceptance`. Two pre-execution harness setup
diagnostics were corrected only in temporary helpers; final suites pass without retries.
Owned projects, namespace, helpers and forwards were removed; unrelated workloads remain.
Later documentation-only commits must preserve byte-identical Docker application inputs.

## M7 optional procedural sound — 2026-10-17

Final clean-export candidate `cd7813c0c1951be486e30da7deaf2e917b1b8069` builds image
`sha256:0e9924ee1b21ef3406b1cf7c59ac7effe57a8ea7fc22839668c8f8ed7c321a62`.
The **same image passes 22/22 scenarios in Compose and 22/22 in Kubernetes, all retries zero**:
the six default cases, pickup, match, two reconnect cases, controls, natural headed visibility,
natural headed audio and nine corrected latency cases. Node 24.21.0 and browser tooling are pinned.
Health/readiness return `{status:ok}` after rollout; the final Pod uses the exact image and is
ready 1/1 with zero restarts. Both paths run as 65532 with a read-only root, dropped capabilities
and no privilege escalation; Kubernetes retains RuntimeDefault, its committed 500m/256Mi limits
and the read-only mode-0444 seed Secret. No seed file is baked into `/app`.

Compose local latency: 20 observations, median **38.919 ms**, maximum **45.995 ms**;
50±10 ms per-direction shaping: median **145.299 ms**, maximum **160.615 ms**.
Kubernetes local: median **40.941 ms**, maximum **53.206 ms**; shaped: median **149.096 ms**,
maximum **159.909 ms**. All forty observations per deployment stay below 250 ms. Browser helpers
use CPUs 0–1 without a CFS quota; Compose application affinity matches. The Kubernetes application
keeps CPUs 0–17 and cpu.max=50000/100000; no throttled periods occur across 197 measured periods.
These are page-delivered public-state observations, not pixel latency or a WAN guarantee.

Strict Kubernetes 1.33 schemas validate all three resources. Pinned archive-only Trivy passes
the committed HIGH/CRITICAL, `--ignore-unfixed` release policy. A separate broader diagnostic
records 58 HIGH and one CRITICAL advisory excluded by that policy as unfixed/deferred/will-not-fix;
the passing policy scan does not claim an advisory-free base image. Actual Kubernetes desktop
and 320-pixel Polish sound captures pass visual inspection. Evidence, including both scan modes,
is retained in `/tmp/grenadiers-v0315-acceptance`. Owned namespaces, Compose projects, helpers,
forwarding, exports and archives are removed; the accepted image and evidence remain.

The active v4 frontend adds four original synthesized public-event cues, default muted, explicit
native-gesture activation and EN/PL sound status. Preferences contain only a validated bounded
boolean/version. One context and at most four voices are owned explicitly; no sound asset,
new dependency, wire change or authoritative-rule change is introduced. Accepted public state
and a fresh non-secret admission marker prevent React-batched reconnects from replaying history.
Mute, hidden, disconnected and room/epoch boundaries advance/discard the cursor without queues.

Independent review found a StrictMode effect-rehearsal bug that disposed the retained engine.
Each effect now owns a fresh engine/ref generation; stale callbacks and pending resumes cannot
activate the replacement. A real userEvent StrictMode test clicks Resume, observes enabled state
and a new accepted-event oscillator. Engine tests prove old disposed activation stays false.
Closure review clears lifecycle, cursor, admission, preference, privacy and bounded-resource behavior.

The pinned full web gate passes **194 tests** with **94.47% statements, 89.70% branches,
94.21% functions and 97.33% lines**; lint, Knip, contract drift and build pass. Before integration,
all 13 E2E scenario static checks pass. Native Chromium passes the final scenario **1/1 at retries
zero in 15.9 seconds**: no context before gesture, real running context and public bombPlaced
before oscillator creation, mute suppression, no replay on re-enable, actual hidden suspension,
visible paused state and explicit Resume to running without old nodes. Sound-button focus releases
held movement without planting. Axe and actual desktop/320-pixel Polish captures pass containment
and coordinator visual inspection.

The diagnostic attempt to observe contextWillBeDestroyed after navigation did not receive that
CDP event; it is retained and no destruction observation is claimed. Explicit close/disposal has
unit evidence; owned browser/profile teardown is bounded separately. Graph processing does not
prove hardware audibility, comfortable volume or listening quality. Evidence remains at
`/tmp/grenadiers-m7-audio-evidence`; integrated deployment evidence is recorded below.
The integrated v0.3.15 server gate passes **296 core tests, 149 server tests and two doctests**,
with four optimized reference profiles intentionally ignored. E2E typecheck/lint/Knip/parity
pass after the additive manifest merge. Dependency policies pass and both npm audits report zero
vulnerabilities. Web source is byte-identical to the independently reviewed 194-test candidate.
M7/v0.4.0 and human playtesting remain open.

## M7 v0.3.14 exact-image delayed-message acceptance — 2026-10-17

Application build source `e7b222fff45efa1fdf9c34c223f938c427c3b5de` produces
`sha256:b62b840b695aeb5177d30f47f64f818503ae6334f1f0ea7f96662c8deae7827d`.
The corrected clean harness is `c50af54d2b5b12322a435fa7ebd67813c86b3e80`; Docker/application
input comparison between those commits is empty. That same image passes **21/21 cases through
each of Compose and Kubernetes**, retries disabled: six default, pickup, match/rematch, two
reconnect, controls, headed visibility and nine latency cases (seven pure plus two native profiles).
The original failed forced-alternation run remains separate; the reviewed trajectory correction
then passes once on both paths without weakening observations, clearance or the fixed budget.

| Deployment | Unshaped 20: median/max | Shaped 20: median/max | Above 250 ms |
| --- | --- | --- | --- |
| Compose | 41.057 / 57.953 ms | 148.672 / 161.787 ms | 0 |
| Kubernetes | 36.191 / 51.095 ms | 147.674 / 160.793 ms | 0 |

Both browser helpers use Node 24.21.0, Playwright 1.63.0, pinned Chromium revision 1243,
CPUs 0–1 affinity and unlimited cpu.max, with no observed CFS throttling. Compose's app also
uses that two-core/no-quota profile. Kubernetes retains the committed **500m/256Mi** resources:
app CPUs 0–17 are scheduler-managed, cpu.max is 50000/100000, and observed throttling remains
0→0 across 107 periods (71,787 µs CPU used). These are explicitly different deployment CPU
profiles; twenty observations per path establish no tail, WAN or human-playability guarantee.

Actual probes return 200/ok after each rollout. The Pod uses the same image digest, is ready
1/1 with zero restarts, and passes nonroot UID/GID 65532, read-only root, dropped capabilities,
no escalation, RuntimeDefault, read-only fixture Secret 0444, correct probes and no `/app` seeds.
Compose security inspection also passes. Strict Kubernetes 1.33 validation reports three valid
resources; pinned archive-only Trivy reports zero fixable HIGH/CRITICAL findings. Actual desktop
and 320-pixel Polish controls screenshots pass visual inspection/containment. Evidence remains
at `/tmp/grenadiers-v0314-acceptance`; owned namespace, Compose project, helpers, forward,
source export and scan archive are removed. Accepted images remain. M7/v0.4.0 stays open.

## M7 exact-image trajectory correction — 2026-10-17

The first v0.3.14 Compose latency attempt passed the shaped profile (20 observations,
maximum 148.829 ms) but failed unshaped release observation after five cycles. The public report
shows accumulated forced-alternation drift: x=1536→1945→1587→1843→1485→1792, then the sixth
westward leg reaches x=1344 at the wall. The unchanged collision predicate correctly rejects
zero clearance. This is a test-trajectory failure, not a passing reference or network failure.
The failing log and public attachment remain in `/tmp/grenadiers-v0314-acceptance` and
`/tmp/grenadiers-v0314-local-summary.json`; no retry conceals the result.

The reviewed correction selects whichever horizontal direction has more public collision-free
runway and requires at least 1,024 units before each native keydown. All twenty observations,
128-unit witness clearance, causal dispatch checks, FIFO limits and the fixed 250 ms budget remain
mandatory. Ten bounded public trajectory records explain each choice. The new regression proves
both corridor-side choices; all seven pure tests and 21 scenario static checks pass. Independent
closure review clears geometry, trajectory and evidence. Application/Docker inputs are unchanged
from the clean accepted-image build candidate `e7b222fff45efa1fdf9c34c223f938c427c3b5de`.
Final same-image acceptance uses the corrected, clean committed harness separately.

## M7 causal ordered-WebSocket reference — 2026-10-17

Independent review fixed three test-validity defects: the original observer watched pre-delay
network traffic; old queued snapshots could mimic release; and a wall collision could mimic
stopping. The final observer receives sanitized facts from actual page-side WebSocket delivery,
requires a prior downlink dispatch, and ties advancing/stable revisions to actual command forwarding.
Both movement and release witnesses retain at least 128 units of forward clearance in public
geometry, with no live bomb. Setup fails closed unless both page wrappers are installed and see
exactly one socket. Context ownership and finally cleanup are explicit and buffers bounded.

The corrected Linux reference passes **8/8 cases, retries zero, in 19.4 seconds**: six pure
queue/observer regressions and two real profiles, each with ten native movement/release cycles.
Unshaped 20 observations: median **39.39 ms**, observed p95/max **54.47 ms**. Shaped 50±10 ms
per direction: median **145.82 ms**, observed p95/max **157.30 ms**, zero 250 ms exceedances.
The maximum shaped queue is two frames/7,204 bytes; closing cancels pending frames without late
delivery. Application prototype image is the accepted v0.3.9 image; final integrated image
acceptance is separate. Node 24.21.0, Playwright 1.63.0 and Playwright Chromium revision 1243 are pinned; app/browser
use CPUs 0–1 affinity with unlimited cpu.max and no CFS throttling observed at startup.

Evidence: `/tmp/grenadiers-m7-latency-corrected-reference.log`, SHA-256
`4a1bc6373490c3d56e4d54258b9814976d4143a2abc16d35d7a7f82cd89f72c0`.
The dirty-source manifest is retained alongside it; source is committed unchanged for integration.
Earlier 284–300 ms quota diagnostics and the 113.05 ms pre-delay observer run are explicitly
invalid measurement evidence. Independent closure review passes; six pure regressions and all
19 scenario static checks passed before controls integration. After controls integration, the v0.3.14 server gate passes 296 core tests, 149 server tests and
two doctests, with four optimized reference profiles intentionally ignored. The first sandboxed
attempt denied ephemeral binds; the unchanged gate passes with permitted loopback access. All
20 documented E2E scenarios pass typecheck/lint/Knip/parity. Dependency policies pass and both
npm audits report zero vulnerabilities. Web production source is identical to the accepted
v0.3.13 180-test/94.85% statement coverage reference; Rust implementation remains the v0.3.11
93.56% line coverage reference. Version changes only the two local package records and dependency.

This is reliable ordered message delay, not TCP/WAN simulation, a twenty-sample tail guarantee
or human playtest approval. M7 remains open.

## M7 v0.3.13 exact-image controls acceptance — 2026-10-17

A clean export of `02fb8a699fe55436704d6f5ef2d29ab43d1e9368` produces
`sha256:51eb91c570aaed8be84cf36af385611f574d26757d0367e3f2777e0a17739177`.
Compose and Kubernetes use that exact image, including the actual Pod digest. Each path passes
**12/12 scenarios with retries disabled**: six default, pickup, full match/rematch, two reconnect,
real headed Linux visibility, and custom controls/persistence/reset. Node is exactly 24.21.0;
every fixture rollout is followed by successful real JSON health/readiness polling.

Both paths pass UID/GID 65532, read-only root, dropped capabilities, no-escalation, probe and
no-`/app`-seed inspection. Kubernetes adds RuntimeDefault seccomp and a read-only fixture Secret
mode 0444. Strict Kubernetes 1.33 schemas validate all three resources. Pinned archive-only Trivy
reports zero fixable HIGH/CRITICAL findings without a Docker socket mount. Actual desktop and
320-pixel Polish controls screenshots pass Axe, containment and visual inspection; all five
bindings and modal actions remain readable. Evidence: `/tmp/grenadiers-v0313-acceptance`.
Owned Compose/namespace/forward/export/archive/tools and listeners are removed; accepted images
remain. Source prerelease distribution and candidate/rebase provenance limits are explicit in
[releases.md](releases.md). These checks do not complete human playtesting or M7.

## M7 persistent configurable controls — 2026-10-17

The active v4 panel now offers five-action EN/PL keyboard remapping with unchanged arrows,
WASD and Space defaults. It validates bounded, globally unique physical-code aliases and stores
only the non-secret preference under `grenadiers.controls.v1`; invalid/unavailable storage falls
back safely. Opening settings or changing mappings releases held direction through the old
callback before new bindings install. Still-held keys remain fenced until key-up. Repeats,
modifiers, unrelated shortcuts, native Tab navigation and editable inputs retain their contracts.

Independent review found ambiguous accessible names, consumed rejected shortcuts, within-action
stored duplicates and missing generated-output ignores. All four are fixed and closure review
passes. Remap/capture names identify the action in both locales; regression tests inspect native
`defaultPrevented` behavior. Dynamic instructions use their own controls template, preserving
archived raw instructions; that integration correction also passes independent inspection.

The combined Node 24.21.0 web gate passes **180 tests** with **94.85% statements, 89.62%
branches, 95.07% functions and 97.67% lines**. Schema, lint, Knip, shuffled coverage and build
pass. E2E static checks confirm **12** documented scenarios. The locked server gate passes
**296 core tests, 149 server tests and 2 doctests**, with four separate optimized reference
profiles intentionally ignored in ordinary CI. Rust implementation is unchanged from the v0.3.11
93.56% coverage reference. Both dependency audits report zero vulnerabilities.

Native Chromium against the candidate frontend passes real UI remapping, old-key rejection,
new-key movement and exactly-once bomb input, reload persistence and default reset (1/1 first run).
Desktop and 320-pixel Polish screenshots pass Axe/containment and coordinator visual inspection;
all five controls and actions are readable. Evidence is retained in
`/tmp/grenadiers-m7-controls-evidence`; its port-8097 server is stopped. Exact final-image
Compose/Kubernetes acceptance passes as recorded above. This does not alter authoritative rules or
complete the remaining M7 latency/human-playtest work.

## M7 v0.3.12 exact-image visibility acceptance — 2026-10-17

A clean Git export of `ded5749dc345556fa75e06d5fcf3a0d2137486a1` produces
`sha256:61ea43aa86f1089fa2331a0149c5c831750e5bddeef2ddfc832f952a252ca3a5`.
Compose and an isolated Kubernetes namespace use that exact image, confirmed from the Pod image ID.
Each path passes **11/11 scenarios with retries disabled**: six default, pickup, full match/rematch,
two reconnect cases, and real headed Linux/Xvfb hidden/visible recovery. The helper pins Node
24.21.0, Playwright 1.63.0 and Chromium; no private capabilities or CDP endpoints enter artifacts.

Two setup diagnostics remain recorded: an Xvfb wrapper without a container init did not launch
npm, and the `.app` hostname caused Chromium HSTS upgrade before game interaction. The isolated
helper uses `--init` and the reserved `grenadiers.test` alias; Kubernetes uses the explicitly allowed
host-forward origin. These corrections change neither browser/game semantics nor assertions.
All fixture rollouts are followed by actual JSON health/readiness polling, returning 200/ok.

Strict Kubernetes 1.33 schemas validate all three resources. Both runtimes pass nonroot/read-only,
dropped-capability, no-escalation, probe, read-only 0444 fixture Secret and no-`/app`-seed checks;
Kubernetes uses RuntimeDefault seccomp. The pinned archive-only Trivy scan reports zero fixable
HIGH/CRITICAL findings. Desktop/320-pixel Polish screenshots pass containment and visual inspection.
Evidence remains at `/tmp/grenadiers-v0312-acceptance`; owned containers, namespace, forward,
source export, scan archive and temporary tools are removed. Accepted images remain.

This verifies the specified short visibility transition, not long suspension, mobile eviction,
network latency or human playability. M7 and v0.4.0 remain open.

## M7 natural hidden-tab recovery — 2026-10-17

The new headed Chromium scenario observes actual `hidden` and `visible` transitions in one
natural browser window. Native held movement emits a null command and stops in fresh peer
snapshots; authoritative time advances by more than 50 ticks during at least one second hidden.
Transport count stays stable. Return replays neither movement nor a bomb, and fresh native
movement, peer HUD and snapshots work. Production code, visibility APIs and game clocks are
unchanged; no fake events or direct control clearing are used.

The pinned Linux/Node 24.21.0/Playwright 1.63.0/Chromium 153 reference initially exposed
Playwright's focus emulation, which holds a capturer and forces pages visible. Manually spawning
fresh Chromium and connecting with supported `noDefaults: true` in its default context restores
real visibility. No window manager or lifecycle injection is required. The gameplay scenario
passes an initial run and three repeats with retries disabled; after the final cleanup-only
correction it passes another first-run acceptance. Failure diagnostics remain public and bounded;
capabilities/private CDP endpoints and network traces are not persisted.

Independent review found cleanup gaps; the owner now observes exit from spawn, bounds CDP close,
TERM and KILL, signals only its owned process group, and removes the profile after exit. The test's
finally directly reaches that bounded owner cleanup, with no preliminary unbounded CDP calls.
A forced SIGSTOP/endpoint wedge completes in 13 seconds and leaves no Chromium or profile; normal
fixture containers/network are removed. Coordinator closure inspection confirms the final path.
The locked server gate passes **296 core tests, 149 server tests and 2 doctests**, with four
separate reference profiles intentionally ignored in ordinary runs. Full E2E static checks and
all **11** documented scenarios pass. Rust production source is unchanged from v0.3.11's
93.56% coverage reference. Same-image Compose/Kubernetes acceptance passes as recorded above.

This is one-second focus/visibility recovery on the stated Chromium profile, not long-term
throttling, suspension, mobile eviction, delayed-network support or human playtest evidence.
M7 and v0.4.0 remain open.

## M7 v0.3.11 exact-image application acceptance — 2026-10-17

Clean source `8e8ca124917b516e85e5a232fa8ff147b73c81e6` produces application image
`sha256:dd271415652b92e00669547268aba5cb5820a5a93cc835c15d7542bc793cda39`.
Compose and Kubernetes, confirmed from actual Pod image ID, each pass **10/10 first-run browser
scenarios with retries disabled**: six default, pickup, full match/rematch and two reconnect cases.
Actual JSON health/readiness is polled before every default/fixture run; Node is exactly 24.21.0.
Strict Kubernetes schemas report **3 valid resources**. Nonroot UID/GID 65532, read-only root,
all capabilities dropped, no escalation, RuntimeDefault seccomp, read-only mode-0444 fixture
Secrets, actual probe paths and absence of `/app/*.seed` pass runtime inspection. Pinned Trivy
0.74.0 scans a saved archive with the official database and no Docker socket: **0 HIGH/CRITICAL**.

Actual desktop and 320-pixel Polish screenshots are visually inspected without clipping/overlap.
Evidence remains at `/tmp/grenadiers-v0311-acceptance`; temporary project/namespace/Secret,
forward, source export and scan archive are removed. The accepted application image and separate
optimized reference image/bundle remain. This completes this engineering increment's deployment
check; later evidence changes affect docs only and M7 playtesting remains open.

## M7 optimized reference harness — 2026-10-17

The clean committed harness at `f1678dfc9d9255504fce8b0c8dcafdb2ac07f96f` passes all four
optimized measurement profiles on native macOS and constrained Linux. The Linux development
reference image is `sha256:b5629e98174e72e4ab2c1a510d2a80c1dc5659b4a94ad4b6b96c78e807a13607`;
its isolated container exits 0 without OOM, with cpuset 2–3 (two allocated of 18 visible CPUs),
512 MiB memory/swap-total limit, 128 PID limit, and no network. Each record identifies source
state, exact binary SHA-256 and image. Both references and all limits/provenance/RSS units are
independently reviewed; the [performance record](game/performance.md) preserves exact results.

Linux runtime p99 is **6 µs**; one/eight-room observed publication-lag p99 is **8.898/5.113 ms**,
within the selected 8.33 ms runtime and 50 ms lag budgets. Every normal owner advances 1,800
ticks over its 30-second window, maximum encoded snapshot is 4,824 bytes, and all cleanup callbacks
complete. Pressure runs separately: 32 slots fill, command 33 is rejected, cleanup completes.
Watch observations can coalesce, sampled occupancy is not a true peak, and process RSS is not
per-room memory. These measurements exclude HTTP/WebSocket/browser and human playability.

Independent review caught stale executable/provenance selection, overlapping pressure RSS and
user-specific/unverified Node selection; all are fixed and closure review passes. The normal
server gate now runs the three artifact/provenance helper tests and shell syntax. It passes
**296 core tests, 149 server tests, and 2 doctests**; four machine-specific measurements are
explicitly ignored in ordinary CI and executed successfully through the reference runner.
Rust coverage is **7,991/8,541 lines, 93.56%**, above the unchanged 80% floor. The complete web
gate passes 163 tests; ten E2E static scenarios and dependency policies pass; both npm audits
report zero vulnerabilities. Exact-image application deployment acceptance is recorded separately.

Raw native/Linux JSON, environments, timing, build/run and container evidence remain at
`/tmp/grenadiers-v0311-reference`; the isolated benchmark container is removed. M7 remains open
for real visibility, delayed-network support and human playtests. This is not a capacity claim
for the configured 32-room limit or a v0.4.0 release.

## M6 v0.3.10 exact-image deployment acceptance — 2026-10-17

A clean Git export of `df9886ff17a662fdfccc19d0769153c091b5d9d2` produces
`sha256:54988e9f3d4bf0d345c4fe7e25e6d3d6a136c75a7390461f006a6342c6448cf7`.
Compose and isolated Kubernetes run that same image, confirmed from the actual Pod image ID.
Each path passes **10/10 browser scenarios on the first run, with retries disabled**: six
default, one pickup, one full match/rematch, and two reconnect cases. Node is exactly 24.21.0;
every fixture rollout is followed by real JSON health/readiness polling before browser tests.

Strict Kubernetes 1.33 schemas report **3 valid resources** with no invalid/error/skipped results.
Inspection confirms UID/GID 65532, read-only root, all capabilities dropped, no privilege
escalation, RuntimeDefault seccomp in Kubernetes, read-only fixture Secret mode 0444, correct
probes, and no seed under `/app`. Pinned archive-only Trivy 0.74.0 reports **zero HIGH/CRITICAL
findings**. Its default database mirror first returned an upstream 404; that diagnostic is
preserved, and the same pinned scanner passes with the explicit official GHCR database. No
Docker socket is mounted. Actual desktop/320-pixel Polish screenshots were visually inspected.

Evidence: `/tmp/grenadiers-v0310-acceptance`. Temporary Compose/namespace/Secret, forward,
source export and scan archive are removed; the accepted image remains. Later documentation
changes preserve the application tree. Together with the independently reviewed 42-rule matrix,
real v4 recovery and private replay assertions, this completes M6 engineering exit evidence.
M7 measured browser/latency support and human playtesting remain separate.

## M6 complete baseline rule assertions — 2026-10-17

Independent source/assertion audit and closure review cover **all 42 baseline rules** in the
[traceability matrix](game/rule-coverage.md). Three new regressions compare complete stepped
chain results after bomb-order permutation, preserve a disconnected actor's original fuse and
vulnerability while fencing input, and prove a previously safe assisted turn enters fire and
kills without canceling its movement. No production behavior changed.

The final v0.3.10 locked server gate passes **296 core tests, 148 server tests and 2 doctests**;
Rust coverage is **7,955/8,491 lines, 93.69%**, above the unchanged 80% floor. Formatting,
all-target Clippy with warnings denied, locked build, complete web gate, all ten documented E2E
static checks and dependency policies pass. Both npm audits report zero vulnerabilities.
The web suite remains **163 tests**, with **94.82% statements, 89.43% branches, 94.42%
functions and 97.60% lines**. Exact-image runtime checks are recorded separately before release.

The rule matrix includes composed and conditional lower-layer evidence honestly; RULE12 does
not invent a normal actor placement into already lethal fire. Measured latency, real background-tab
recovery, broader browser support and human playtesting remain M7 work. Complete rule assertions
alone do not establish v0.4.0 readiness.

## M6 v0.3.9 exact-image deployment acceptance — 2026-10-17

A clean Git export of `ead3f3fc9c67caea7aea9866bfa5c4970d9d80ed` produces
`sha256:b1cee9317f0d30cee36d05c9a32a7d2933a4394f08c83a324cb713f5846b13ee`.
Compose and isolated local Kubernetes use that same image, confirmed from the actual Pod image ID.
Each path passes **10/10 first-run browser scenarios with retries disabled**: six default,
one pickup, one complete match/rematch, and two real reconnect/takeover/late-rejection cases.
Node is exactly 24.21.0. Both paths return real 200 JSON health/readiness before testing.

Runtime inspection confirms UID/GID 65532, a read-only root, all capabilities dropped,
no privilege escalation, and the Kubernetes RuntimeDefault seccomp profile. Fixture Secret files
are read-only with mode 0444; probes retain their actual API paths, and no seed is packaged under
`/app`. Strict Kubernetes 1.33 validation reports **3 valid resources**, with no errors/skips.
Pinned Trivy 0.74.0 scans a saved image archive without a Docker socket and reports
**zero HIGH/CRITICAL findings**. Desktop and 320-pixel Polish screenshots were visually inspected;
board, status, controls, localization and diagnostics have no clipping or overflow.

Evidence remains in `/tmp/grenadiers-v039-acceptance`; temporary Compose/namespace/Secret,
forward, source export and scan archive are removed. The accepted image remains available.
Later release-evidence commits change documentation only. This completes deployment evidence
for coordinated activation; the independent full rule matrix follows in v0.3.10, while M7
measured playtests and broader support remain separate.

## M6 coordinated reconnect activation — 2026-10-17

The locked server gate passes **294 core tests, 147 server tests, and 2 doctests**.
Rust line coverage is **7,954/8,491 lines, 93.68%**, above the unchanged 80% floor.
Independent owner/runtime, replay, browser and acceptance-source reviews are clear. Closure
checks include unavailable-room rejection, deferred admission during catch-up, canceled replacement
receivers, transactional takeover input fencing, maximal action floors, and legacy replay facts.
The Node 24.21.0 web gate passes **163 tests**, with **94.82% statements, 89.43% branches,
94.42% functions, and 97.60% lines**. Contract checks, ESLint, Knip, shuffled coverage, build,
all ten documented E2E scenarios, and locked dependency policies pass; both npm audits report zero
vulnerabilities. Initial-countdown App fixtures now satisfy connected-and-ready semantics.

Native Chromium acceptance passes six default scenarios, the complete match/rematch, and two
reconnect scenarios. Recovery preserves the live bomb, physical position, private seat identity,
capability and action floor; native input escapes the real fuse and subsequent bomb IDs advance.
Connected takeover ends the old UI without a retry contest; wrong room/capability are uniformly
rejected. Dual transport loss followed by eleven real seconds produces terminal rejection and
usable lobbies. These native runs preceded the final owner scheduling/error fixes; exact-image
acceptance of the final committed source is a separate required check. Browser fixtures use no
physics, outcome or clock injection. Native traces are disabled because personal v4 frames carry
capabilities; public screenshots, redacted summaries and reports remain. Desktop and 320-pixel
Polish match-result screenshots were visually inspected; accessibility and containment pass.

A final-source native process probe also passes: two room members receive identical full
ServerStopped terminal snapshots; both members, an unauthenticated game socket and a diagnostic
socket close cleanly with code 1001. SIGTERM drains and exits with code 0 in 2 ms, and the
isolated listener is released. Evidence: `/tmp/grenadiers-v039-native-shutdown.json`.

The independent 42-rule audit finds 39 covered and three partial integration cases (RULE06,
RULE28, RULE38). Their bounded follow-up tests precede complete M6 exit evidence. M7 measured
playtest/hardening remains outstanding; this activation is not the v0.4.0 milestone.

## M6 dormant reconnect browser — 2026-10-17

The complete Node 24.21.0 web gate passes **163 tests**, with **94.74% statements, 88.22%
branches, 95.47% functions, and 97.50% lines**. Schema drift, Spectral, ESLint, Knip,
shuffled coverage and production build pass. The combined locked server gate passes
**294 core tests, 136 server tests, and 2 doctests**; Rust implementation is unchanged from
the preceding private-helper checkpoint.

Independent review caught unbound arena callbacks, premature capability loss after rejected
leave, and unusable stale ownership after terminal recovery. All three are fixed and closure
review passes, including **51 focused v4 cases**. The panel invokes movement, release and bomb
callbacks with their correct receiver. Rejected leave retains updates and resume authority;
matching acknowledgement clears identity. Transport loss during pending explicit leave exits
locally without blind resume. Replacement, rejected capability or exhausted retries stop and
return to a usable lobby. Offline rematch controls are disabled.

Tests cover fresh-session bounded retries, callback/timer fences, full private baselines,
action floors/exhaustion, stale shared frames, no buffered actions, identity privacy, and
localized grace/held-result status. Active App and v3 remain unchanged. Real reconnect/takeover
browser acceptance belongs to coordinated activation; exact-image acceptance is separate.

A clean committed-tree build of `5865ca3c0b36b4aba9b74bcca7afa4cdc4c057dc` produces
`sha256:d457710f0a86af74e102f26b5e38873da5f38cc42aa3dd8419d6fe0deca94c3a`.
Compose and local Kubernetes run that exact image. Each passes real JSON health/readiness,
six default browser scenarios, one pickup scenario and one full-match/rematch scenario;
fixtures disable retries. Strict Kubernetes 1.33 schemas pass 3/3; pinned archive-only Trivy
reports zero fixable HIGH/CRITICAL findings. Runtime checks verify non-root UID/GID 65532,
read-only root, dropped capabilities, seccomp/probes and read-only mode-0444 fixture mounts;
no seed file exists under `/app`. Final immutable client `cdcaa516` and exact hashes,
logs and screenshots are retained in `/tmp/grenadiers-v038-acceptance`. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; the image remains.

## M6 private resume credential/binding helpers — 2026-10-17

The combined locked server gate passes **294 core tests, 136 server tests, and 2 doctests**.
Rust coverage is **7,373/7,913 lines, 93.18%**, above the unchanged 80% floor.
Independent review reports no actionable helper findings. Four focused cases cover independent
32-byte entropy, exact three-attempt collision bounds, failure atomicity, stable capability
matching, fully redacted diagnostics, and exact connection/generation replacement fences.
The module remains test-compiled; staging publication atomically and supplying fresh binding
generations remain coordinated activation duties. Exact-image acceptance is separate.

A clean committed-tree build of `71d9e23c1a269c4aecb59543d5c13ccdf2291522` produces
`sha256:cf1bdab3e473d039e330168a3342ff52f4f06bc9ec5210d54eac6fc28c543288`.
Compose and local Kubernetes run that exact image. Each passes real JSON health/readiness,
six default browser scenarios, one pickup scenario and one full-match/rematch scenario;
fixtures disable retries. Strict Kubernetes 1.33 schemas pass 3/3; pinned archive-only Trivy
reports zero fixable HIGH/CRITICAL findings. Runtime checks verify non-root UID/GID 65532,
read-only root, dropped capabilities, seccomp/probes and read-only mode-0444 fixture mounts;
no seed file exists under `/app`. Final immutable client `cdcaa516` and exact hashes,
logs and screenshots are retained in `/tmp/grenadiers-v037-acceptance`. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; the image remains.

## M6 checked gameplay-v4 projection — 2026-10-17

The locked server gate passes **294 core tests, 132 server tests, and 2 doctests**.
Rust coverage is **7,315/7,854 lines, 93.14%**, above the unchanged 80% floor.
Independent review reports no actionable findings. Twenty-nine focused v4 cases include real
lifecycle traces and physical projection binding, settled grace, held/canceled completed worlds,
terminal winner preservation, admission identity/nullability, and current-epoch action context.
The existing v3 contract and active runtime regressions pass. All handwritten files remain
below 400 lines. Actual capability ownership and numeric action provenance remain owner duties;
network activation and exact-image deployment acceptance are separate checks.

A clean committed-tree build of `81fdf89032e08370953c8bd2ca53913af8b629b4` produces
`sha256:6fd85f60f7176f1376b9f8ea5ef6ceba1684a3159e85979b104fdae560d859a2`.
Both Compose and local Kubernetes run that same artifact and pass real JSON health/ready
probes, six default browser scenarios, one pickup scenario and one full-match scenario.
Fixture suites disable retries. Strict Kubernetes 1.33 schemas pass 3/3; pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL findings. Runtime checks verify UID/GID 65532,
read-only root, dropped capabilities, seccomp/probes, and read-only mode-0444 fixture mounts;
no seed file exists under `/app`. Immutable client `817cd78` and complete image/build hashes,
logs and screenshots are retained in `/tmp/grenadiers-v036-acceptance`. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; the image remains.

## M6 dormant gameplay-v4 structural contract — 2026-10-17

The combined locked server gate passes **280 core tests, 132 server tests, and 2 doctests**.
Rust coverage is **6,807/7,287 lines, 93.41%**, above the unchanged 80% floor. Independent
review reports no actionable structural-contract findings. Fifteen focused Rust cases plus
AJV/TypeScript conformance verify every closed variant, required nullable fields, nested extras,
credential validation/redaction, namespace separation, local references, and exact physical
v3 parity. Complete maximal structural Snapshot and Admitted frames measure **44,874** and
**31,444 UTF-8 bytes**, respectively, below the unchanged 65,536 cap; oversized output fails.
Client parsing retains its inclusive 4,096-byte cap.

The full web gate passes **119 tests** with **96.83% statements, 92.68% branches, 98.87%
functions, and 98.94% lines**. Three-version generation/drift, lint, Knip, and build pass.
Active endpoint/client remain v3, and the diagnostic contract remains v1. Semantic checked
projection, network reconnect activation, and exact-image acceptance are separate checks.

A clean committed-tree build of `943413cec21e9ecbf4ce18ccf85177ca95488819` produces
`sha256:4418f66b9298998bc6da4a5f3f2c90d6cd8b1e2b229001b8befbac73f21f12ed`.
Both Compose and local Kubernetes run that same artifact and pass real JSON health/ready
probes, six default browser scenarios, one pickup scenario and one full-match scenario.
Fixture suites disable retries. Strict Kubernetes 1.33 schemas pass 3/3; pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL findings. Runtime checks verify UID/GID 65532,
read-only root, dropped capabilities, seccomp/probes, and read-only mode-0444 fixture mounts;
no seed file exists under `/app`. Immutable client `817cd78` and complete image/build hashes,
logs and screenshots are retained in `/tmp/grenadiers-v035-acceptance`. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; the image remains.

## M6 grace-aware pure match lifecycle — 2026-10-17

The combined locked server gate passes **265 core tests, 132 server tests, and 2 doctests**.
Rust coverage is **6,748/7,186 lines, 93.90%**, above the unchanged 80% floor. Independent
review reports no actionable lifecycle findings. Fourteen new cases cover exact grace
boundaries, both seat orientations, staggered expiry, complete loss batches before expiry,
first/second versus third-win finality, mixed departures, held/canceled countdowns, actual-round
parity, terminal access, rematch, and atomic deadline/epoch/hold overflow.

Running clock/scoring never settles grace before the complete loss batch. Held results retain
their original completed record and earliest restart deadline; resumed players receive a fresh
countdown on the next idle tick. Legacy v3 departures still interrupt and have no new-API
call sites. Existing projection, owner, private capture, and native socket regressions pass.
Network reconnect activation and exact-image deployment acceptance remain separate checks.

A clean committed-tree build of `2d66e2af2e027dde9294ffec6092b4c60b1c0e44` produces
`sha256:18d648aab0530c58fc4acbecff42f040ce128f988c3638a33e019cc78d2b320c`.
Both Compose and local Kubernetes run that same artifact and pass real JSON health/ready
probes, six default browser scenarios, one pickup scenario and one full-match scenario.
Fixture suites disable retries. Strict Kubernetes 1.33 schemas pass 3/3; pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL findings. Runtime checks verify UID/GID 65532,
read-only root, dropped capabilities, seccomp/probes, and read-only mode-0444 fixture mounts;
no seed file exists under `/app`. Immutable client `817cd78` and complete image/build hashes,
logs and screenshots are retained in `/tmp/grenadiers-v034-acceptance`. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; the image remains.

## M6 accepted-operation capture — 2026-10-17

The locked server gate passes **251 core tests, 132 server tests, and 2 doctests**.
Rust coverage is **6,496/6,926 lines, 93.79%**, above the unchanged 80% floor. Independent
review reports no actionable capture/privacy/bounds findings. Seven replay cases compare
complete semantic private checkpoints from actual captured generated matches, including
cancellation, draws, three wins, rematch, TTL refresh, immediate fences, and atomic shutdown
with pending departure. Wrong phase/epoch, rate-limit, duplicate, and wire-bound rejections
produce no records. No-op accepted readiness and refreshes remain recorded.

Journal allocation plus header measures **3,840,056 bytes** and never grows past its cap.
Truncation leaves later gameplay checkpoints unchanged; construction failure is explicitly
incomplete and cannot certify a replay. Production capture retains no credential/frame/binding
identity. Current unrecorded notice callers have empty reports; future notice changes must
preserve that invariant or record their effect. Transport revision is excluded from semantic
comparison. Replay inspection remains test-only, with no public export or restart persistence.
Exact-image deployment acceptance is recorded below.

A clean committed-tree build of `cc616e3e459514dc156d95b36fe6abff02fc2628` produces
`sha256:a66fdfde941675c7dd8cc26829a6b4e7a1e867e373f95c0386895ebae95781e3`.
Both Compose and local Kubernetes run that same artifact and pass real JSON health/ready
probes, six default browser scenarios, one pickup scenario and one full-match scenario.
Fixture suites disable retries. Strict Kubernetes 1.33 schemas pass 3/3; pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL findings. Runtime checks verify UID/GID 65532,
read-only root, dropped capabilities, seccomp/probes, and read-only mode-0444 fixture mounts;
no seed file exists under `/app`. Immutable client `817cd78` and complete image/build hashes,
logs and screenshots are retained in `/tmp/grenadiers-v033-acceptance`. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; the image remains.

## M6 private replay-runner foundation — 2026-10-17

The locked server gate passes **251 core tests, 129 server tests, and 2 doctests**.
Rust coverage is **6,356/6,787 lines, 93.65%**, above the unchanged 80% floor. Independent
review and malformed-action follow-up report no outstanding findings. Four replay cases
exercise real generated worlds through cancellation, three wins, rematch, shared-bomb draw,
interruption, TTL refresh, immediate input fencing, and nonempty report windows. Complete
private world/lifecycle/input/report/revision checkpoints match.

Mutation checks reject unsupported versions/profiles, time/order errors, missing/extra seeds,
wrong actual-start metadata, zero/above-wire-bound bomb actions, and truncation. The exact
maximum bomb action ID is accepted. Records measure 64 bytes on this target; preallocated
60,000-record capacity is 3,840,000 bytes and stops growing at truncation. Debug output does
not expose private generation seeds. This module remains test-only; automatic live capture
and exact-image deployment acceptance are separate checks, not claimed here.

A clean committed-tree build of `e9eb97822ca4bdd3bd9c1ef24c5ae6e39bf3e186` produces
`sha256:6967b11ec659e0f68161d8a128afbf4f29449de4f50f58bbe2f5c0bf72cec6f4`.
Compose and local Kubernetes use that exact image. Each passes the real JSON health/ready
probes, **six default browser scenarios, one pickup scenario, and one full-match scenario**.
Both fixture suites disable retries. Strict Kubernetes 1.33 schemas pass **3/3**; runtime
inspection confirms UID/GID 65532, read-only root, dropped capabilities and seccomp settings.
Read-only fixture Secrets use mode 0444; no seed file appears under `/app`. Pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL findings. The immutable accepted client is `3456e48`;
its exact hashes and screenshots are recorded with the image evidence. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; images and logs remain.

## M6 pure reconnect-grace foundation — 2026-10-17

The locked server gate passes **251 core tests, 125 server tests, and 2 doctests**.
Rust coverage is **6,142/6,566 lines, 93.54%**, above the unchanged 80% floor. Independent
review reports no actionable findings. Seven focused cases exercise exact 599/600-tick
eligibility, original duplicate-loss deadlines, staggered and simultaneous expiry, rejected
expired resumes, successful no-op clock advancement, stale calls, and atomic errors/overflow.
The two-seat component returns bounded reservation/expiry facts and never fabricates outcomes.

This is a pure API release. Active v3 transport-loss interruption, credentials, publication,
and browser behavior are unchanged. Lifecycle and coordinated reconnect activation follow.
A clean committed-tree build of `53d42fe95819d1252c17004cb3aa04b426ead796` produces
`sha256:b9b753d049ee7e933e2b2afe18843e4ad58b431ca98e00cc5e6c0a8b2b57d648`.
Compose and local Kubernetes use that exact image. Each passes the real JSON health/ready
probes, **six default browser scenarios, one pickup scenario, and one full-match scenario**.
Both fixture suites disable retries. Strict Kubernetes 1.33 schemas pass **3/3**; runtime
inspection confirms UID/GID 65532, read-only root, dropped capabilities and seccomp settings.
Read-only fixture Secrets use mode 0444; no seed file appears under `/app`. Pinned archive-only
Trivy reports zero fixable HIGH/CRITICAL findings. The immutable accepted client is `3456e48`;
its exact hashes and screenshots are recorded with the image evidence. Temporary stacks,
namespaces, Secrets, forwards, exports and scan archives are cleaned; images and logs remain.

## M5 coordinated classic matches — 2026-10-17

The active endpoint/browser use gameplay protocol 3 and the real MatchLifecycle, MatchRuntime,
and bounded MatchOwner. Independent frontend, acceptance-design, and backend reviews report
no outstanding findings. The locked server gate passes **244 core tests, 125 server tests,
and 2 doctests**. Rust coverage is **6,049/6,453 lines, 93.74%**, above the unchanged 80% floor.
The complete web gate passes **112 tests**, with **96.83% statements, 92.68% branches,
98.87% functions, and 98.94% lines**. Schema drift, Spectral, lint, Knip, build, and E2E static
checks pass; eight scenarios match their documentation. Dependency-security checks pass;
both npm audits report zero vulnerabilities.

A real main-process SIGTERM probe, separate from library tests, starts a two-player Running
match and observes identical final checked serverStopped physical snapshots before both member
sockets close with 1001. Unauthenticated gameplay and diagnostic sockets also receive 1001.
The process exits code 0 in the measured 2 ms run, and its listener is released. AppState
regressions separately cover the full five-second hung-permit deadline, retained prior final
outcomes, stop-time admission rejection, room cleanup, and sticky shutdown. This explicit drain
is required because Axum's upgraded callbacks run independently of HTTP connection shutdown.

The first full-match browser run completed a draw, three real wins, and rematch/reset, then
caught a persistent phase-boundary movement error. The client now correlates only movement
errors with bounded command context and newer authoritative phase/epoch progress; bomb and
current-phase failures remain visible. Idle controls no longer refresh null input. Regression
tests cover both error/state orderings. Native Chromium now passes the full two-browser
scenario with retries disabled: real shared-bomb draw, three actual wins, automatic fresh
rounds, alternating spawns/base upgrades, final hold, vote withdrawal/consensus, one-epoch
rematch reset, fresh readiness, and action-ID reset on the same sockets. Axe and horizontal
containment checks pass in English and Polish at desktop and 320 pixels. Actual compositor
screenshots were visually inspected.

A clean committed-tree build of `6b18f8609278af7eb7de4e69ea40c104d392e596` produces
`sha256:aebaffdc8bf122bc49bf879709eb52d0979c0003610ec99a451b90657e2b5ae4`.
Compose and local Kubernetes use that exact image. Each path passes actual API health/readiness
JSON, all six default scenarios, the separate pickup fixture, and the complete match/rematch
fixture, with fixture retries disabled. Both fixture mounts are read-only; Kubernetes uses a
temporary 0444 Secret readable by UID/GID 65532. Pod image ID matches the accepted image.
Strict schemas validate all three resources; archive-only pinned Trivy reports zero fixable
HIGH/CRITICAL findings. `/app` contains no seed files. Non-root/read-only-root/drop-ALL/no-new-
privileges/RuntimeDefault settings remain enforced. Temporary stacks, namespace/Secrets,
forwards, exports, and scan archive are removed; logs and reviewed screenshots remain.
Later acceptance-driver/documentation commits retain identical Docker build inputs.

## Dormant bounded match owner — 2026-10-17

The locked server gate passes **243 core tests, 134 server tests, and 2 doctests**.
Rust line coverage is **6,333/6,799 lines, 93.15%**, above the unchanged 80% floor.
Independent review and follow-up closure report no outstanding findings. Twenty-seven owner
cases cover real match finality, departure batching and generation fences, queue bounds,
seed/start boundaries, expired/canceled admission, input fencing, 20 Hz publication, overload,
lobby/rematch deadlines, nonfinal retention, command fairness, and sticky shutdown.

Review fixes make shutdown/overload atomically apply pending departures before one final
publication, preserving reports, vacancy, final outcome, and prompt leave acknowledgments.
Canceled queued admission causes no occupancy/revision mutation. Source remains below 400
lines per file. The four-update loop/yield bound is statically checked and fairness tested;
a direct deterministic exact-yield test is not present. Production routing and process/socket
draining remain activation checks; this owner is deliberately test-only until that switch.

A clean committed-tree build of `c73986376b01fea0b8947995125ae0fc787920e1` produces
`sha256:0e16c2da99dd620b7bc38662e3c2ceaafd1048882fba6937a0ea1923f96516b1`.
Compose and ready local Kubernetes use that exact image. Each path passes health/readiness,
all six default browser scenarios and the separate read-only pickup-fixture scenario. Strict
schemas validate all three resources. Pinned archive-only Trivy reports zero fixable
HIGH/CRITICAL findings; `/app` contains no seed files. Kubernetes retains UID/GID 65532,
a read-only root, dropped capabilities, no privilege escalation, RuntimeDefault, and a
read-only 0444 fixture Secret. Temporary resources and exports are removed; logs and
screenshots remain. This evidence update leaves production routing unchanged.

## Dormant v3 browser client and match panel — 2026-10-17

The complete web gate passes **120 tests** with **94.88% statements, 89.47% branches,
94.53% functions, and 97.76% lines**, above unchanged coverage floors. Independent review
and terminal-vacancy follow-up report no outstanding findings. Cases cover authenticated v3,
strict schema validation, revision/epoch ordering, direct coalesced new-round snapshots,
control/action reset with retained sequence, bounded counters, initial-only readiness,
authoritative rematch holds/votes, cleanup, localized match/round facts, and vacant seats
alongside frozen live/dead actors. Twelve ported security/transport regressions preserve token,
stale-generation, admission, malformed-frame, counter, leave, cleanup, and native fetch receiver
behavior; partial dependency overrides retain the existing defaults. Both generated contracts,
lint, Knip, and production build
pass. The existing Pixi arena is shared through a narrow presentation boundary.

The panel remains dormant. No visual/WebGL or live match acceptance is claimed until the
coordinated endpoint/client activation.

A clean committed-tree build of `c8de0537b8298cbef9d99b8f9e1e22973a6e2591` produces
`sha256:68e7c577cd618feca1f40f2f5221fb30ec67d8349a827a4faab228d50cc46646`.
Compose and ready local Kubernetes use that exact image. Each path passes health/readiness,
all six default browser scenarios and the separate read-only fixture scenario. Strict schemas
validate all three resources. Pinned archive-only Trivy reports zero fixed HIGH/CRITICAL
findings; `/app` contains no seed files. Kubernetes retains non-root UID/GID 65532, read-only
root, dropped capabilities, no privilege escalation, RuntimeDefault, and a read-only 0444
fixture Secret. Temporary resources and exports are removed; logs/screenshots remain.
This documentation-only evidence update does not activate the dormant implementation.

## Dormant authoritative match runtime — 2026-10-17

The integrated locked server gate passes **243 core tests, 107 server tests, and 2 doctests**.
Rust line coverage is **5,607/6,019 lines, 93.16%**, above the unchanged 80% floor.
Independent runtime review reports no actionable findings. Twenty focused adapter cases
exercise real wins/draws/timeouts, fresh assignment/entropy/upgrades, forfeits during running
and completed holds, rematch reset, stale controls/deduplication, finite input TTL, bounded
report windows, frozen publication, privacy, profile mismatch, frame bounds, and exhaustion.
The active v2 caller retains its explicit first-round assignment. The new adapter remains
test-only: scheduling, shutdown, and actual endpoint/browser activation are later checks.

A clean committed-tree build of `49b142e13b62640dbb08331cedc8ef39ad1bc276` produces
`sha256:c63b2b4a8ae61bae66e8a79ea1b7be2f4934a869ec0da8e40e06c6f220bcb429`.
Compose and ready local Kubernetes use that exact image. Each path passes health/readiness,
all six default browser scenarios and the separate read-only fixture scenario. Strict schemas
validate all three resources. Pinned archive-only Trivy reports zero fixed HIGH/CRITICAL
findings; `/app` contains no seed files. Kubernetes retains non-root UID/GID 65532, read-only
root, dropped capabilities, no privilege escalation, RuntimeDefault, and a read-only 0444
fixture Secret. Temporary resources and exports are removed; logs/screenshots remain.
This documentation-only evidence update does not activate the dormant implementation.

## Checked v3 room projection — 2026-10-17

The locked server gate passes **243 core tests, 85 server tests, and 2 doctests**.
Rust line coverage is **5,091/5,484 lines, 92.83%**, above the unchanged 80% floor.
Independent review and follow-up closure report no actionable projection findings.
The 18 focused cases exercise real draw/three-win flows, cancellation gaps, initial and
inter-round countdowns, independent forfeits/interruption/abandonment, rematch reset, and
post-final departure. Mutation regressions reject impossible consent, draw-awarded wins,
invalid epoch chronology, and occupied forfeit losers. The adapter remains dormant; live
protocol and browser behavior are unchanged.

A clean committed-tree build of `4e6633c416a249d246056acb7eef06e3892a18c9` produces
`sha256:8f25752f6a59ca26ebdec4ef1da7a8f3dd103bc51711f6400400b7f99a218e11`.
Compose and ready local Kubernetes run that exact image. Both paths pass health/readiness,
all six default browser scenarios, then the separate read-only seed fixture scenario. Strict
schemas validate all three resources; pinned archive-only Trivy reports zero fixed
HIGH/CRITICAL findings. Kubernetes retains UID/GID 65532, read-only root, dropped capabilities,
no privilege escalation, and a read-only 0444 temporary fixture Secret. No seed files exist
under `/app`. Temporary resources/exports are removed; logs and screenshots remain.
This evidence update changes documentation only.

## Dormant gameplay-v3 contract — 2026-10-17

The locked server gate passes **225 core tests, 85 server tests, and 2 doctests**.
Rust line coverage is **4,615/5,001 lines, 92.28%**, above the unchanged 80% floor.
Independent review reports no actionable contract findings. Strict Rust/AJV cases cover
all new variants, scalar bounds, unknown-field rejection, physical v2/v3 parity, and the
44,742-byte maximal-body fixture against the unchanged 65,536-byte limit.

The complete web gate passes **90 tests** with **96.80% statements, 92.08% branches,
98.64% functions, and 98.82% lines**. Both generated schemas pass drift checks; Spectral,
ESLint, Knip, shuffled coverage, and the production build pass. The v3 contract is dormant;
these results do not claim match runtime or browser activation.

A clean committed-tree build of `14c93f9e5b97b03ffe6804495640b51203adaa2a` produces
`sha256:568d7437899f4b215fe1bb9c26676629cfa3237ac257d8167e7b97f04c254307`.
Compose and ready local Kubernetes run that exact image. Both paths pass health/readiness,
all six default browser scenarios, then the separate read-only seed fixture scenario. Strict
schemas validate all three resources; pinned archive-only Trivy reports zero fixed
HIGH/CRITICAL findings. Kubernetes retains UID/GID 65532, read-only root, dropped capabilities,
no privilege escalation, and a read-only 0444 temporary fixture Secret. No seed files exist
under `/app`. Temporary resources/exports are removed; logs and screenshots remain.
This evidence update changes documentation only.

## M5 pure automatic match lifecycle — 2026-10-17

The locked server gate passes **214 core tests, 85 server tests, and 2 doctests**.
Rust line coverage passes the unchanged 80% floor at **4,585/4,914 lines, 93.30%**.
Independent review found stale rematch consent after a participant departed. Departures now
clear that vote and vacant opponents cannot complete consensus; both seat orders are covered.
The correction is independently verified, with no remaining lifecycle findings.

Tests cover initial cancellation, automatic inter-round countdowns, exact hold/start
boundaries, actual-round parity across epoch gaps, first-to-three including draws, long
global time, prematch versus match interruption, batched departure precedence, terminal
finality, rematch readiness/reset, and transactional clock/epoch/series overflow. The new pure
lifecycle remains dormant alongside the live single-round v2 owner; activation follows after
truthful contracts and real server/browser adapters are verified.

A clean committed-tree build of `c2e76e6b1b27ae74c322c56e9e795acf353c2230` produces
`sha256:d46a8911d0d1ecf94d4cd073ac62f987c6a13e5e6b632769bc228973e93b19b2`.
Compose and ready local Kubernetes run that exact image. Both paths pass health/readiness,
all six default browser scenarios, then the separate read-only seed fixture scenario. Strict
schemas validate all three resources; pinned archive-only Trivy reports zero fixed
HIGH/CRITICAL findings. Kubernetes retains UID/GID 65532, read-only root, dropped capabilities,
no privilege escalation, and a read-only 0444 temporary fixture Secret. No seed files exist
under `/app`. Temporary resources/exports are removed; logs and screenshots remain.
This evidence update changes documentation only.

## M5 headless first-to-three series — 2026-10-17

The locked server gate passes **194 core tests, 85 server tests, and 2 doctests**.
Rust line coverage passes the unchanged 80% floor at **4,275/4,592 lines, 93.10%**.
Independent review identified a stale-time readiness inconsistency after rematch; it is fixed
and the regression independently verified. No findings remain in the series component.

Tests cover win/draw score sequences, actual-round parity across epoch gaps, exact hold
boundaries, completed-round versus match outcomes, batched forfeits/abandonment/interruption,
final-match precedence, idempotent/withdrawable votes, retained rematch epoch/time high-water,
and atomic tick/deadline/round-number failures. Storage remains bounded to one latest round.
This is a pure headless API; live generated gameplay remains single-round v2. Lifecycle,
contracts, server adapters, and browser progression follow as separate focused releases.

A clean committed-tree build of `10320666a7b6d4aaa56b48983f36047bac743133` produces
`sha256:6e470d7505bab6778601b53a3c39f6489252e1a943f290dbeb15194b4498389e`.
Compose and ready local Kubernetes use that same image. Both paths pass health/readiness,
six default-random browser scenarios, and the separate read-only fixture scenario. Strict
schemas report 3 valid resources; pinned archive-only Trivy reports zero HIGH/CRITICAL findings.
The Linux steering correction changes only the browser harness, not application inputs; both
fixture paths also pass using the revised `dba46766688fe00d1a39de4d291a6d0d8d5aab18` tests against
this image. Temporary runtime resources are removed; the image and evidence logs remain.

## M4 headless compositor diagnosis and correction — 2026-10-17

Hosted trace profiling finally isolated the repeated native-key overshoot: Chromium's hybrid
software compositor synchronously called `GLES2::ReadPixels` during canvas commit, blocking
its renderer thread 100–175 ms per frame. The baseline captured 38 readback stalls and a
176 ms maximum commit; consistent ANGLE/SwiftShader compositing captured zero readback stalls
and a 0.7 ms maximum commit. Native key-down acknowledgements improved from roughly 90 ms
to 1–8 ms. React/AJV/Pixi JavaScript was not the dominant cost; changing game logic or pulse
constants was not justified. Independent key-up scheduling was tested and rejected because
one pulse collapsed to zero movement.

The shared Playwright config now selects `--enable-gpu --use-gl=angle --use-angle=swiftshader`
only for CI on Linux. Local macOS retains its existing hardware path. No browser security
option, game code, tolerance, fuse, assertion, timeout or retry changes. The exact unchanged
pickup source (`ab5a63d1a12175d9bd203bb28bee27fd7f84c069a5af841f4f88734a14f8188b`)
passes the complete two-client scenario with retries disabled in 31.4 seconds. Independent
configuration review and the full seven-scenario E2E static gate pass. Causal profiles and
experiment logs are retained at `/tmp/grenadiers-m4-input-experiment/FINDINGS.md`.
Both final hosted browser checks must pass before merge. The accepted application image is
unchanged because this correction affects only browser execution configuration.

## M4 settled turn alignment — 2026-10-17

Exact Node 24.21.0 Linux reproduction exposed the last corner condition: native key-up can
coast to the outer wall, 192 units from cell center, while the movement model accepts a
perpendicular turn only within 128. The test now waits for two newer stationary public
snapshots after key-up, then uses the unchanged precise helper for that bounded corner offset.
It turns only after the existing <=128 assertion; the second native leg reaches blast safety
before real hazards clear. A stale pre-key-up snapshot can no longer skip the correction.

Independent source/model closure review is clear. Final exact-Node Linux Chromium passes
**6/6**, native Chromium **3/3**, all with retries disabled. The full E2E static gate passes.
Logs: `/tmp/grenadiers-m4-corner-settled-{linux,native,static}.log`. Physics, turn tolerance,
fuse, precise position assertions, pickups, both HUDs, bomb ranges/IDs, and events are unchanged.
Earlier failing Linux corner variants remain recorded as failures, not passing evidence.
Final hosted checks must pass at this revision before release.

## M4 live-fuse corner-turn follow-up — 2026-10-17

Hosted `c86668b` exposed the remaining delay before the safe leg: exact centering at the
corner still consumed the live fuse. Both escape routes now use native key holds throughout:
West until `x <= 1728` before South, or North until `y <= 1728` before East. These bounds
place the actor's 320-unit half footprint entirely outside the adjacent blocked/bomb column
or row before turning. Exact centering runs after real hazards clear. No precise-position,
pickup, HUD, bomb-range/identity, event, or two-client assertion is removed.

Independent geometry and cleanup review is clear. Final native Chromium passes **3/3** with
retries disabled (`/tmp/grenadiers-m4-corner-native.log`), and the full seven-scenario E2E
static gate passes. The application artifact and real fuse remain unchanged. Hosted checks
are required at the final head before merge.

## M4 pickup escape safety follow-up — 2026-10-17

Hosted CI at `071a952` still caught a first-escape death despite passing local pulse-controller
runs. The pickup-only route now holds native South until the live public actor reaches
`y >= 2560`, or East until `x >= 2944` for the second escape. Both exceed the actual 2368-unit
blast-clear boundary. Key release runs in `finally`; real bombs/flames clear before exact
centering resumes. Generic movement, the three-second fuse, physics, <=128 position bounds,
pickups, both HUDs, captured bomb ranges, IDs, and collection events remain unchanged.

Independent geometry/control review is clear. The pinned Playwright 1.63 Linux image with two
affinity-limited CPUs and retries disabled passes **10/10**, followed by **6/6 on final bytes**.
Native Chromium against the isolated Compose fixture passes **3/3** with retries disabled.
Typecheck and zero-warning ESLint pass. Linux evidence is retained in
`/tmp/grenadiers-m4-route-debug/e2e/linux-route-safety{,-final}.log`; the native result was
observed directly, not saved to a separate log. Application build inputs are unchanged from
the previously accepted exact image. Final hosted CI is required before merge.

## M4 live generated rounds — 2026-10-17

The locked server gate passes **178 core tests, 85 server tests, and 2 doctests**.
Rust line coverage passes the unchanged 80% floor at **4,030/4,340 lines, 92.86%**.
Independent review reports no actionable seed/profile/activation findings. Tests cover strict
bounded file parsing, read-once reproduction, startup redaction, independent authentication/
discovery entropy, actual-start-only generation, canceled countdowns, profile/state agreement,
no substitute world after entropy failure, private retained records, and explicit snapshot
seed/private-key absence. A malformed-seed native process exits with a redacted RoundSeed
error before trying an already occupied listener.

Native Chromium passes all **six generated-map smoke scenarios** and the separate **two-browser
pickup acceptance**. The latter reveals and collects capacity/range through keyboard/public
positions, updates both HUDs, and proves simultaneous range-2/range-3 bombs. A core fixture
trace independently survives that route at tick 631. CI runs random smoke before recreating
the same image with the read-only fixture; the fixture command fails without explicit setup.
The complete web gate passes 83 tests with 96.80% statement, 92.08% branch, 98.64%
function, and 98.82% line coverage. Contract/lint/unused-code/build, E2E static parity, and
dependency policies pass; npm audits report zero vulnerabilities.

A clean committed-tree build of `483346a3191d90a66f0701de9b84b2fb79ce47ef` produces
`sha256:01740d0eaf194b0d85daba3cc54937dc8a6912a153c4e3c82cd8a3554995a209`.
Both Compose and ready local Kubernetes use that exact image. Each path passes health/
readiness, all six default-random smoke scenarios, then the separate pickup scenario after
read-only fixture recreation. Kubernetes uses a temporary Secret readable by UID 65532 while
preserving probes and security settings. Strict schemas report 3 valid resources and pinned
archive-only Trivy reports zero HIGH/CRITICAL findings. Runtime inspection finds no seed files
under `/app`. Actual fixture screenshots were visually inspected; both HUDs show the upgrades.
Temporary stacks, namespace/Secret, forwards, exports, and archives are removed. The image and
logs/screenshots remain available.

Linux CI exposed controller timing sensitivity. The final controller records sanitized
movement direction/stop summaries, requires actual public actor movement, then two newer
identical stationary snapshots before another correction. Coarse movement releases early
with key-up protected by `finally`; fine movement uses one native `keyboard.press` operation
with a dynamic 34–60 ms pulse. Both axes retain the existing 128-unit turn-assist tolerance.
Concurrent live assertions precede compositor screenshots; production physics, bomb fuses,
and every bomb/HUD/event assertion remain unchanged.

The final source passes **10/10 official Linux Playwright 1.63 Chromium runs** pinned to two
CPU cores (`nproc=2`, unrestricted bandwidth), plus **3/3 native Compose** and **3/3 native
Kubernetes** fixture runs against the accepted image above. Every run disables retries;
the fixture config now also disables CI retries. A separate hard CPU-bandwidth-quota diagnostic
was 7/10 and remains preserved, not counted as passing evidence. Runtime/render performance
budgets under arbitrary throttling remain M7 work.

Per-action DOM/compositor tracing is disabled only for this real-time fixture. Action/source
traces, failure screenshots, explicit visual checkpoints, and bounded redacted WebSocket
summaries remain. Default smoke tracing is unchanged. Application build inputs are unchanged;
the accepted image is retained. Final stacks, namespace/Secret, forwards, debug projects,
and exports are removed; acceptance logs and screenshots remain available.

## M4 exposed-item rendering and upgrade HUD — 2026-10-17

The complete web gate passes **83 tests** with **96.80% statements, 92.08% branches,
98.64% functions, and 98.82% lines**. Contract drift, Spectral, ESLint, Knip, shuffled coverage,
and the TypeScript/Vite production build pass. E2E lint, Knip, types, and scenario parity pass.
Independent review reports no actionable renderer findings. Tests cover all three item
symbols, retained reuse, replacement/removal/reset cleanup, layer order, capped displayed
levels, authoritative live-bomb availability, status, and localization.

The duel scenario checks both clients' upgrade HUD before/during/after a bomb, result
liveness, and containment at 320 pixels. A clean committed-tree build of
`43ced9cee551336e81dce48f20a0fd18fe332cfa` produces
`sha256:ebb4a60eb53cce84ddd749e4d13bfad6b2de9ae02f07fad5e3647ce80e51d8b9`.
Compose and ready local Kubernetes use this exact image. Health/readiness and all six candidate
browser scenarios pass through both paths. Strict schemas report 3 valid resources; pinned
socket-free Trivy reports zero HIGH/CRITICAL findings. Desktop and 320-pixel screenshots were
visually inspected: the HUD is readable and contained. Temporary resources are removed;
the image and acceptance logs/screenshots are retained. This evidence update changes docs only.
The immutable server profile remains fixed-v1; visible generated items are not yet activated.

## M4 gameplay-v2 contract and room profiles — 2026-10-17

The locked server gate passes **177 core tests, 69 server tests, and 2 doctests**.
Rust line coverage passes the unchanged 80% floor at **3,861/4,171 lines, 92.57%**.
The complete web gate passes **82 tests** with **96.64% statements, 91.66% branches,
98.61% functions, and 98.77% lines**, including contract drift, Spectral, ESLint, Knip,
shuffled coverage, and production build. E2E static checks and dependency policies pass;
npm reports zero vulnerabilities.

Independent schema/browser and Rust/server reviews report no actionable findings. Tests
cover old/future versions before credential claims, full bounded incompatibility responses,
strict pickup/event shapes, visibility and occupancy checks, same-update collection/result
ordering, fixed-profile construction and mismatch rejection. The maximal structural v2
snapshot is **44,466 bytes** against the unchanged **65,536-byte** limit. Diagnostic v1
serialization is unchanged. Live rooms retain fixed-v1 rules/map and empty pickups.

A clean committed-tree build of `5ec3243c65b2226fa41805562cfa5ad5a1bff76d` produces
`sha256:fdc3910e21878b091512bcd07c2bfeaedcbe5cfb88f263c5be5680c71557dc3b`.
Compose and ready local Kubernetes use this exact image. Candidate-source health/readiness
and all **six browser scenarios** pass through both deployments. Strict schemas report 3
valid resources; pinned socket-free Trivy reports zero HIGH/CRITICAL vulnerabilities.
Temporary runtime resources are removed; the image is retained. This evidence update changes
only documentation.

## M4 collection and permanent upgrades — 2026-10-17

The locked server gate passes **169 core tests, 67 server tests, and 2 doctests**.
Rust line coverage passes the unchanged 80% floor at **3,796/4,115 lines, 92.25%**.
Independent review reports no actionable findings. Tests exercise damage-before-collection,
final-center eligibility, distance/tie priority, row-major event order, collection at caps and
under bombs, old/new bomb range coexistence, next-update speed with preserved residuals,
round-number spawn parity, canceled-attempt independence, and classic-v1 compatibility.

This release adds headless classic-v2 behavior. Live rooms still use the fixed classic-v1
preview; gameplay protocol/client support and generated activation remain separate slices.

A clean committed-tree build of `b30f9528a542eb26572c57dd0a1e92d2801b504b` produces
`sha256:a3f985a351968f0e1a706ee3380e97e846c7419910727c8967c72c9d08b09046`.
Compose and ready local Kubernetes use this exact image. Health/readiness and all **six
browser scenarios** pass through both deployments. Strict schemas report 3 valid resources;
pinned socket-free Trivy reports zero HIGH/CRITICAL vulnerabilities. Temporary runtime
resources are removed; the image is retained. This evidence update changes documentation only.

## M4 typed pickup visibility layers — 2026-10-17

The integrated locked server gate passes **155 core tests, 67 server tests, and 2 doctests**.
Rust coverage passes the unchanged 80% floor at **3,675/3,987 lines, 92.17%**. Independent
review reports no actionable findings. No dependency, wire, browser, or deployment behavior
changes; network rooms still use the item-free fixed preview.

Tests cover checked layer construction and duplicate/invalid placement, concealed-to-pending
blast transitions, revealing-blast immunity, exclusive flame expiry, renewed/overlapping
flame delay, reveal followed by same-update burning, downstream rays, nonblocking movement,
exposed-only public ordering, private Debug output, generated-world adaptation, and frozen
terminal layers. Collection, upgrade application, and wire projection remain explicit next
slices rather than claimed behavior of these typed components.

A clean committed-tree build produces
`sha256:2b26a573390b8be4f900f0031a5156aea2e80283d96fccd73be9bc0ba7b0a1d4`.
Compose and ready local Kubernetes use this exact image. Health/readiness and all **six
candidate browser scenarios** pass through both deployments. Strict schemas pass with 3
valid resources; pinned socket-free Trivy reports zero HIGH/CRITICAL vulnerabilities.
Temporary runtime resources are removed; the image is retained. This evidence update changes
documentation only.

## M4 deterministic generator groundwork — 2026-10-17

The integrated locked server gate passes **146 core tests, 67 server tests, and 2 doctests**.
Rust coverage passes the unchanged 80% floor at **3,547/3,861 lines, 91.87%**. Independent
review reports no actionable findings. Dependency audits pass: all cargo-deny policies and
zero npm vulnerabilities. The direct rand_chacha 0.9.0 edge adds no package/version to the
lockfile. Browser/schema/deployment inputs remain those of the playable v0.2.0 baseline.

Generator tests lock exact seeded outputs and conditional random consumption, threshold
boundaries, center handling, half-turn symmetry, reserved openings, bounded rejection,
invalid-map fallback, redacted records, and unchanged fixed-arena fixtures. A dense-layout
simulation executes the first-bomb escape route and survives. The structural validator checks
both mirrored openings; the executable escape trace currently exercises seat one. No active
network seed or concealed content is exposed; live rooms remain on the fixed preview.

A clean committed-tree build produces
`sha256:21f8ba9f0920e214dbcc32096c82f4f712f35eacac9b5448aa26dc8db0bb2f47`.
Compose and ready local Kubernetes use this exact image. Health/readiness and all **six
candidate browser scenarios** pass through both deployments, preserving the playable fixed
preview. Strict schemas pass with 3 valid resources; pinned socket-free Trivy reports zero
HIGH/CRITICAL vulnerabilities. Temporary runtime resources are removed; the image is retained.
The post-acceptance rebase preserves the tested tree; this evidence update changes docs only.

## M3 playable browser duel — 2026-10-17

The version-aligned server gate passes **137 core tests, 67 server tests, and 2 doctests**,
with unchanged Rust coverage **3,355/3,668 lines, 91.47%**. Locked dependency audits pass with
zero npm vulnerabilities and all cargo-deny policies satisfied.

The full web gate passes **80 tests**, generated API/gameplay/diagnostic drift checks, lint,
Knip, coverage, and production build. Coverage is **96.64% statements, 91.66% branches,
98.61% functions, 98.77% lines**, against unchanged 85% floors. E2E static checks pass six
documented scenarios. Independent reviews cover the client, renderer/input, and acceptance
assertions with no actionable findings remaining. PixiJS is exactly 8.22.0 in the real lockfile.

All **six integrated native-browser scenarios** pass against the compiled authoritative
server. The rendered duel creates/joins through the UI, changes EN/PL without reconnecting,
readies both seats, draws actual WebGL pixels, moves through native keyboard input, clears
movement on blur, plants exactly one sent bomb despite repeated Space, destroys a previously
present block, and displays the same authoritative winner and frozen remaining clock.
Common public snapshot revisions are identical; no browser page errors occur.

Axe reports zero violations across lobby, waiting, Running, and Result. Desktop 1440 px and
narrow 390 px screenshots were inspected, with actual vertically stacked controls, usable
arena width, and no horizontal overflow. The narrow context uses reduced motion. Manual
lobby/waiting checks at 320 px also pass Axe and overflow checks. Renderer tests cover async
StrictMode/unmount, resizing, interpolation, reduced motion, input focus, and deterministic
owned GraphicsContext teardown. Native testing found and fixed the Window fetch receiver
and terminal timer display defects before release.

The production build reports a 516.85 kB entry chunk warning; the Pixi renderer is lazy-loaded
separately. No warning threshold or coverage rule is relaxed. Performance budgets remain M7
work; no measured frame-rate or touch-control guarantee is claimed.

A clean committed-tree build produces
`sha256:84fa6ac094f1816ec05c2e86cef5527ed50ccc1ea44667db325163b421223dad`.
Compose and ready local Kubernetes use this exact image. Separate health/readiness checks
and all **six candidate browser scenarios** pass through both paths, including the actual
rendered Pixi duel. Exact-image desktop/narrow Running and Result screenshots were inspected;
the result retains its authoritative remaining clock. Strict schemas pass with 3 valid
resources; pinned socket-free Trivy reports zero HIGH/CRITICAL vulnerabilities. Temporary
runtime resources are removed, with image/screenshots retained. This evidence update changes
documentation only.

## M3 authoritative room loop — 2026-10-17

The locked server gate passes **137 core tests, 67 server tests, and 2 doctests**. Rust
coverage passes the unchanged 80% floor at **3,355/3,668 lines, 91.47%**. The E2E static
gate passes five documented scenarios. Independent review reports no actionable findings.
Review found and resolved missing 20 Hz Countdown views with a public-clock regression.

Controlled-time/runtime tests cover the exact countdown boundary and zero-update first
snapshot, canceled-attempt fencing/current-round Leave, identical encoded frames, movement
expiry/ordering, bomb deduplication/coalescing, per-seat budgets, accepted-result finality,
catch-up/overload, and terminal retention. Real native browser sockets complete a fixed-map
round with movement, duplicate-action rejection, block destruction, elimination, actual events,
identical public result, and frozen later world state. Long waiting expiry and terminal
retirement remain controlled-time actor tests rather than multi-minute browser waits.

A clean committed-tree build produces
`sha256:3ba01e6038a03ca0abedc67efc229eb2677bbcd522dffcec398c96b879b09c0b`.
Compose and ready local Kubernetes use this exact image. Separate health/readiness checks
and all **five candidate browser scenarios** pass through both paths, including the complete
native-socket round. Strict schemas pass with 3 valid resources; pinned socket-free Trivy
reports zero HIGH/CRITICAL vulnerabilities. Temporary deployment/export/archive/forward/
symlink resources are removed; the candidate image is retained. This evidence update changes
documentation only.

## M3 network room admission — 2026-10-17

The locked server gate passes **136 core tests, 51 server tests, and 2 doctests**. Rust
coverage passes the unchanged 80% floor at **2,957/3,218 lines, 91.89%**. The E2E static
gate passes four documented scenarios. Independent review reports no actionable findings.
The browser application and contracts are unchanged from v0.1.12.

Tests exercise personalized two-seat admission, full/unknown rooms, explicit and abrupt
departure, vacancy/rejoin, generation fencing, capacity/rate/identifier failures, fixed expiry,
and exact discovery cleanup. Owner cancellation before the first poll and after admission
releases the registry synchronously. Controlled-time owner tests cover the five-minute
boundary; the live socket expiry path shares this notification handling but is not held open
for five minutes in browser acceptance. No countdown or gameplay execution is claimed.

A clean committed-tree build produces
`sha256:4532c6b48d24baf645de2040ada5d59c45b54ecd8ee08f4afa980eb7edd64794`.
Compose and ready local Kubernetes use this exact image. Separate health/readiness checks
and all **four candidate browser scenarios** pass through both paths, including actual
create/join/full/leave/rejoin behavior and authentication retention. Strict schemas pass with
3 valid resources; pinned socket-free Trivy reports zero HIGH/CRITICAL vulnerabilities.
Temporary deployment/export/archive/forward/symlink resources are removed; the candidate
image is retained. This evidence update changes documentation only.

## M3 authenticated gameplay WebSocket — 2026-10-17

The integrated locked server gate passes **136 core tests, 37 server tests, and 2 doctests**.
Rust coverage passes the unchanged 80% floor at **2,340/2,550 lines, 91.76%**. The full web
gate passes **46 tests**, generated API/schema checks, lint, Knip, coverage, and build with
unchanged web percentages/floors. The E2E static gate passes three documented scenarios.
Independent review reports no actionable findings.

A synchronized 16-thread race proves one live claim winner and 15 rejected duplicates, with
permanent credential revocation after release. Real diagnostic/gameplay sockets exhaust the
same final semaphore slot in both directions, then demonstrate reuse after close. Abrupt
transport drop releases both the credential lease and connection permit. Live tests cover
compatibility-before-claim, replay/semantic ordering, invalid/binary input, upgrade rejection,
and a five-second authentication deadline that control frames cannot extend.

Inactivity, absolute unseated expiry, and input-rate boundaries use deterministic private
policy tests directly coupled to the driver. Explicit checks run after every receive/select
wake-up so a ready input cannot defeat an expired deadline. A tiny authentication-response
send failure cannot be forced reliably over loopback; the abrupt live-driver disconnect test
covers its shared RAII cleanup ownership without adding test-only production interfaces.
No rooms or gameplay snapshots are fabricated by this transport increment.

A clean committed-tree build produces
`sha256:4e5f9dd3ffadf4db2833092b069fa45e4b598924716cec5ec2da526da922460b`.
Compose and ready local Kubernetes use the exact image. Separate health/readiness checks
and all **three candidate browser scenarios** pass through both paths, including actual native
gameplay authentication, compatibility-before-claim, duplicate-owner protection, and replay
rejection. Strict schemas pass with 3 valid resources; pinned socket-free Trivy reports zero
HIGH/CRITICAL vulnerabilities. Temporary deployment/export/archive/forward/symlink resources
are removed; the candidate image is retained.

## M3 pure room lifecycle — 2026-10-17

The locked server gate passes **136 core tests, 24 server tests, and 2 doctests**. Rust
coverage passes the unchanged 80% floor at **2,038/2,190 lines, 93.06%**. Independent review
reports no actionable findings. No dependency, schema, generated client, socket handler,
or browser behavior changes; the documented core API is intentionally headless.

Ten lifecycle tests cover either seat completing readiness, the exact 179/180 countdown
boundary, cancellation/readiness clearing/attempt fencing, vacant and terminal operations,
repetition, deterministic sequences, and atomic tick/deadline/attempt overflows. Completed
results survive both departure reasons; Interrupted cannot become Result. Result clocks
advance with a fixed end tick while Interrupted clocks remain frozen. Scheduler ownership,
connection binding, and network room creation remain future integration.

A clean committed-tree build produces
`sha256:bb571d5c54c8311be1e9793055dfbcdde14280dfeda192046c8d5d416c769489`.
Compose and ready local Kubernetes use this exact image; separate health/readiness checks
and both candidate browser scenarios pass through each deployment path. Strict schemas pass
with 3 valid resources; pinned socket-free Trivy reports zero HIGH/CRITICAL vulnerabilities.
Temporary deployments, exports, archives, port-forwards, and dependency symlinks are removed.

## M3 anonymous session issuance — 2026-10-17

The integrated locked server gate passes **126 core tests, 24 server tests, and 1 doctest**.
Rust coverage passes the unchanged 80% floor at **1,868/2,012 lines, 92.84%**. The complete
web gate passes **46 tests**, all contract generation/lint/build checks, and unchanged coverage
of **95.34% statements, 90% branches, 100% functions, 96.05% lines**. The E2E static gate passes
with two documented scenarios, including the new real browser issuance checks.

Deterministic server tests cover canonical unique credentials, inclusive expiry, atomic concurrent
capacity, burst/refill boundaries, collisions, entropy failure, generic responses, exact/duplicate/
non-UTF-8/null Origin handling, and tracing that excludes credential-bearing query strings.
Independent review caught null-Origin configuration and out-of-order timestamp refill defects;
both are fixed with regressions. Registry time now clamps under its mutex for consistent
expiry and rate accounting. HTTP issuance grants no room/seat and no socket handler yet.

A clean committed-tree build produces
`sha256:6ee60a78bfac63db543f10577577332b65d35b86c1871d4da6137f50f847cca4`.
Compose and ready local Kubernetes use this exact image. Separate health/readiness checks pass;
the exact candidate browser suite passes **both scenarios** through each deployment path:
actual session issuance/Origin/query/cache policy and independent diagnostic roundtrips/Axe.
Strict schemas pass with 3 valid resources; pinned socket-free Trivy reports zero HIGH/CRITICAL
vulnerabilities. Temporary deployments, port-forwards, exports, and archives are removed.

## M3 checked domain projection — 2026-10-17

The locked server gate passes with **126 core unit tests, 9 server tests, and 1 doctest**.
Rust coverage passes the unchanged 80% floor at **1,744/1,887 lines, 92.42%**.
Independent review finds no production defect. The wire schema, generated frontend types,
dependencies, network routes, and diagnostic behavior remain unchanged.

Tests project actual initial, active, chain/result, and late frozen states, validate the
resulting wire shapes, and reject malformed clocks, dimensions, identities, owners, ordering,
counts, positions, deadlines, outcomes, event ticks, and report windows. Frozen results accept
old bomb/flame deadlines as the absolute clock advances. Three-report event conversion
preserves phase order and same-tick placement/detonation; per-tick indexing is exercised
across multiple eventful updates.

A clean committed-tree build produces
`sha256:149b66e34170c8dd8b861e888601578dff757867c533dc0b766ad30734aad857`.
Compose and ready local Kubernetes use that exact image. Separate health/readiness requests
and actual two-context WebSocket/Axe browser checks pass through both deployment paths with
required native-browser access. Strict schemas pass with 3 valid resources; pinned socket-free
Trivy reports zero HIGH/CRITICAL vulnerabilities. Temporary deployments, exports, and archives
are removed; the verified image remains available.

## M3 snapshot/event contract — 2026-10-17

The locked server gate passes with **115 core unit tests, 9 server tests, and 1 doctest**.
Rust coverage passes the unchanged 80% floor at **1,474/1,604 lines, 91.90%**.

The complete web gate passes with **46 web/tooling tests** and unchanged coverage of
**95.34% statements, 90% branches, 100% functions, 96.05% lines**. Generated gameplay and
diagnostic drift checks, nested AJV validation, ESLint, Knip, TypeScript, and production build
pass. The independent contract review reports no actionable findings.

Rust tests cover schema-conforming full snapshots, all four events and round outcomes,
JavaScript-safe tick limits, and actual frame encoding. The maximal structural fixture is
**33,989 UTF-8 bytes** with 16 bombs, 195 blocks, 195 flames, and 64 events; a real oversized
snapshot is rejected. No dependency, diagnostic contract, route, or runtime behavior changes.
Checked projection of domain values and canonical cross-field invariants is the next slice.

A clean committed-tree build produces
`sha256:340a4a16b3624bbb0076248cea8193c9b3e832508647e036dde66a31e27e360d`.
Compose and local Kubernetes use this exact image; health/readiness and actual two-context
WebSocket/Axe browser checks pass through both deployment paths. The first Kubernetes browser
launch hit macOS sandbox denial before assertions; rerunning the correct browser gate with
native-browser access passes. Strict schemas pass with 3 valid resources; pinned socket-free
Trivy reports zero HIGH/CRITICAL vulnerabilities. Temporary runtime resources and archives
are removed; the tested image is retained.

## M3 control/lifecycle contract — 2026-10-17

The integrated locked server gate passes with **111 core unit tests, 9 server tests, and
1 public API doctest**. Rust coverage passes the unchanged 80% line floor at
**1,474/1,604 lines, 91.90%**. Two independent review scopes (Rust/security and schema/web)
report no actionable findings or material gaps.

A fresh locked npm install and final complete web gate pass: **40 web/tooling tests**;
coverage remains **95.34% statements, 90% branches, 100% functions, 96.05% lines**. Diagnostic
generated bytes remain identical; gameplay generation/drift checks are in the shared gate.
Only the new generated schema receives the same precise lint/coverage exclusion as existing
generated contracts; no application rule or coverage threshold is relaxed.

Conformance tests exercise all seven client and five lifecycle response variants, closed
objects, required nullable direction/error sequence, canonical strings, integer-valued numeric
forms, scalar bounds, version compatibility responses, credential Debug redaction, payload-free
errors, and inclusive UTF-8 message byte limits. Existing serde_json becomes a core production
dependency; jsonschema remains dev-only, with no new external package or version. Snapshot
wire types, gameplay routes, HTTP session issuance, and runtime rooms remain future slices.


A clean Git-tree image build passes for
`sha256:c8aea33bb438685f6d66b0a385226218e1d9fc44f95c157f946996cfbfff0d8f`.
Compose and ready local Kubernetes use that image. Health/readiness, actual two-context
WebSocket/Axe browser scenarios through both paths, strict Kubernetes schemas (3 resources),
and pinned Trivy's socket-free image scan pass. The unchanged scan policy reports zero fixable
HIGH/CRITICAL findings. Temporary runtime resources and archives are removed. Runtime acceptance
still exercises diagnostics because this increment defines contracts without gameplay handlers.

## M3 bounded-schema prerequisite — 2026-10-17

The complete web gate passes after a clean locked npm install: generated diagnostic types
remain byte-identical; API lint/drift, ESLint, Knip, shuffled Vitest coverage, and production
build pass. **31 web/tooling tests** pass, including 8 generator tests. Application coverage
remains **95.34% statements, 90% branches, 100% functions, 96.05% lines** against unchanged
85% floors. Independent review and targeted AJV Unicode-pattern parity checks pass.

The version-aligned Rust server gate also passes: **97 core unit tests, 9 server tests,
1 doctest**; coverage remains **1,321/1,406 lines, 93.95%**. No dependencies or current wire
contracts change. Invalid bounds, malformed patterns, and misplaced/unknown keywords remain
rejected rather than silently generating misleading types. Runtime AJV, rather than ordinary
TypeScript string/array types, enforces the newly admitted constraints.


A clean Git-tree image build passes for
`sha256:19d8588a227e4f36f1a6b53e704a9f3333481392359f5aa48fdace614b662919`.
Compose and ready local Kubernetes use that image. Health/readiness and actual two-context
WebSocket/Axe browser checks pass through both paths; strict schemas report 3 valid resources.
Pinned socket-free Trivy passes with zero fixable HIGH/CRITICAL findings. Temporary runtime
resources and archives are removed. Rebase onto completed M2 changes documentation only;
application and package inputs match the tested candidate.

## M2D public projection and phase events — 2026-10-17

The locked server gate passes with **97 core unit tests, 9 server tests, and 1 public API
doctest**. Rust line coverage passes the unchanged 80% floor at **1,321/1,406 lines, 93.95%**.
Independent review reports no actionable findings or material coverage gaps. No dependencies,
wire contracts, browser behavior, or deployment manifests change.

Projection tests verify visible layers, ownership counts, timer/result values, stable ordering,
and equality despite different private movement/input/grace/allocator bookkeeping. Event tests
prove same-tick placement/detonation remains observable, old-fire elimination precedes survivor
placement, canonical seat/ID ordering, and no failed-placement, fixture, or repeated terminal
events. Earlier movement, combat, timeout, and atomic-error regressions remain passing.


A clean Git-tree image build passes for
`sha256:4e081ed9475a1bd40ae8a2a6937ec44795167b9e3105e57d954317ff5052dd38`.
Compose and local Kubernetes use that image; health/readiness, ready rollout, and the actual
two-context WebSocket/Axe browser scenario pass through both deployment paths. Strict schemas
pass with 3 valid resources. Pinned Trivy's archive-input scan passes with zero fixable
HIGH/CRITICAL findings. Temporary runtime resources and archives are removed.

This completes M2's headless world/combat/result/public-boundary exit. It does not implement
networked player sessions, rendered gameplay, generated maps, pickups, or match scoring.

## M2C damage and one-round outcomes — 2026-10-17

The locked server gate passes with **90 core unit tests, 9 server tests, and 1 public API
doctest**. Rust line coverage passes the unchanged 80% floor at **1,206/1,297 lines, 92.98%**.
Independent review finds no actionable production defects. The existing same-tick planted-bomb
chain fixture now explicitly verifies its owner dies later that update without suppressing
the bomb's effect.

New evidence covers stationary/edge/swept and turn-correction fire contact, expiry before
damage, new-fire non-retroactivity, pre-placement death, self/mutual elimination, complete-chain
result ordering, all 10,800 updates from a fresh round, final-update elimination precedence,
immutable outcomes, stationary post-result reports, and atomic errors after finalization.
Dependencies, wire contracts, browser behavior, and deployment manifests remain unchanged.


A clean Git-tree image build passes for
`sha256:741cb9fac0c7924cc98419edcb53a565e995d1cc09d6d6523ac091191e39f25a`.
Compose and the ready local Kubernetes Pod use that image; health/readiness and the actual
two-context WebSocket/Axe browser scenario pass through both paths. Strict Kubernetes
validation passes with 3 valid resources. Pinned Trivy's socket-free archive scan passes
with zero fixable HIGH/CRITICAL findings. Temporary deployment resources and archives
are removed. Documentation-only release evidence does not change the tested image inputs.

## M2B explosions and flames — 2026-10-17

The explosion increment passes the locked server gate. The final suite has **78 core unit
tests, 9 server tests, and 1 public API doctest**. Rust line coverage passes the unchanged
80% floor at **1,125/1,216 lines, 92.52%**. Independent review has no remaining actionable
findings after strengthening the shared-block snapshot and same-tick placement-chain tests.

Evidence covers exact fuse/expiry boundaries, wall/block/bomb obstruction, complete-chain
occupancy, repeated triggers, bomb-ID permutations, overlapping flames, active-fire ignition,
soft-block removal, grace cleanup, and capacity becoming available on the following tick.
This core-only increment changes no dependencies, wire contracts, browser behavior, or
deployment manifests. Actor damage and round results remain deferred.


A clean Git-tree image build passes. The tested image is
`sha256:3efcdc07674e7b8eb1a5271eadce170b87fa90584abd1f75a4e7fcdeeefd35bc`.
Compose and a ready local Kubernetes rollout use that image; health/readiness and the
actual two-context WebSocket/Axe browser scenario pass through both deployment paths.
Strict schemas pass with 3 valid resources. Pinned Trivy's archive-input scan finds zero
fixable HIGH/CRITICAL vulnerabilities under the unchanged policy. Temporary runtime
resources and archives are cleaned up. Rebase onto v0.1.2 changes documentation only;
reviewed Rust/package/application inputs remain byte-identical.

## M2A owned bomb placement — 2026-10-17

The placement increment passes `scripts/gates.sh server` with **63 core unit tests,
9 server tests, and 1 public API doctest**. The unchanged Rust coverage gate passes at
**990/1,083 lines, 91.41%**. No dependencies, wire contracts, frontend behavior, or deployment
manifests change. Independent review found no remaining actionable issues after constructor
regression coverage and the retired obstacle API documentation were corrected.

Tests exercise post-movement cells, ownership/range/deadline snapshots, derived capacity,
same-cell conflicts, invalid-request isolation, ordered cancellation, all-actor exit grace,
ID/deadline exhaustion, and reconstruction from a nonempty owned-bomb world. Actor IDs at
`u32::MAX` do not exhaust lower unused bomb IDs. Fuse execution and damage remain deferred.


Runtime verification uses candidate image
`sha256:62a27015d91eabda211e4ce038fb5782521e9dbc5446cfb6e6d8b55b91c89233`:

- Compose starts without PostgreSQL; health/readiness return 200. The actual two-context
  WebSocket/Axe browser scenario passes against the built app.
- The same image rolls out ready in local Kubernetes. Health/readiness, SPA fallback,
  unknown API 404, and the browser scenario pass through the Service port-forward.
- Strict rendered Kubernetes validation passes: 3 valid resources, no errors or skips.
- Digest-pinned Trivy 0.74.0 passes the unchanged fixable HIGH/CRITICAL filter with zero
  findings. The local scan reads a `docker image save` archive through `--input` and a
  read-only archive mount, without exposing the Docker socket to the scanner container.
- A clean Git-tree export builds successfully; its runtime filesystem layers exactly match
  the tested candidate. Attestation/manifest identities can differ between builds.
- Temporary Compose containers, Kubernetes namespace, port-forward, and scan archive
  are removed after verification; local images remain available.

## M1 world and movement — 2026-10-17

The integrated headless domain passes `scripts/gates.sh server`: exact repository pins,
400-line source ceiling, rustfmt, Clippy with warnings denied, locked tests, and build.
Tests pass: **49 core unit tests, 9 server regression tests, and 1 public API doctest**.
`scripts/gates.sh coverage` passes the unchanged 80% line floor with **825/916 lines, 90.07%**.

Evidence includes all five movement speeds, exact obstacle faces, corner gliding, multi-tick
turn correction and cancellation, held-input priority, actor overlap, bomb exemption exit/re-entry,
stationary/piecewise traces, invalid-coordinate rejection, atomic errors, and repeated logs
producing equal full states. Independent review found no remaining actionable issues after
the reported defects and evidence gaps were fixed.

M1 adds no dependencies and changes no wire contracts, browser behavior, or deployment manifests.
The diagnostic UI remains the browser experience. Combat and rendered gameplay are future
milestones; [ADR 0003](decisions/0003-arena-rendering.md) records the PixiJS direction for M3.

## M0 foundation — 2026-10-16

Verified locally on **2026-10-16**, macOS ARM64 with OrbStack Docker/Kubernetes.
Earlier network/socket/Git denials were resolved through permitted sandbox escalation.
This record covers the diagnostic scaffold; gameplay is not implemented.

### Required gates

| Check | Observed result |
| --- | --- |
| `scripts/gates.sh all` | Passed: repository pins/source sizes, locked Rust fmt/Clippy/tests/build, contracts, web lint/Knip/coverage/build, E2E static checks. |
| Rust tests | 11 passed (2 core, 9 server), including real ephemeral WebSocket sockets. |
| `scripts/gates.sh coverage` | Passed unchanged 80% line floor: **214/265 lines, 80.75%**. Core 100%; server library 93.37%; binary entry point 0%. |
| Web tests | **7 files, 29 tests passed**, including generated-type compilation, Spectral rule enforcement, and rendered Kubernetes invariants. |
| Web coverage | 95.34% statements, 90% branches, 100% functions, 96.05% lines; configured floors remain 85%. |
| `scripts/gates.sh security` | Passed. cargo-deny advisories/bans/licenses/sources passed; both npm audits reported **zero vulnerabilities**. |
| `scripts/gates.sh k8s` | Passed rendered invariants and strict schemas: 3 resources valid, none invalid, errored, or skipped. |
| `scripts/gates.sh browser` | Passed against both Compose and Kubernetes Service port-forward: two independent browser contexts, real WebSocket roundtrips, disconnected/connected Axe checks. |
| CI-configured Trivy scan | Passed: no fixable HIGH/CRITICAL findings under the configured filter. Debian runtime scanning complements Rust/npm dependency checks. |

Tools: Rust **1.99.1**, Node **24.21.0**, cargo-llvm-cov **0.9.1**, cargo-deny
**0.20.2**, kubeconform **0.8.0**, kubectl **1.33.9**, Docker **29.8.2**, and
digest-pinned Trivy **0.74.0**. Kubernetes schemas target **1.33.0** as configured in CI.

### Runtime evidence

- `docker compose build app` built the locked Rust/React image. Rust and runtime base-image
  manifest indexes are now resolved and pinned. Runtime apt security updates repaired the
  inherited `perl-base` findings; no audit threshold or scanner policy was weakened.
- A clean staged Git-tree export built successfully using Docker's layer cache. Its runtime
  filesystem layers match the tested image, without relying on untracked local source files.
- The final tested image is
  `sha256:97aec4259fd0f931e92080d504fd389cc740c32a938b7f8a9b863b93dc89e2fe`.
  Kubernetes reported that same image digest, a successful rollout, and a ready container.
- The database-free app started successfully. Configured local PostgreSQL readiness returned
  200; stopping that database changed readiness to 503 Problem JSON while liveness stayed 200;
  restarting PostgreSQL restored readiness to 200.
- Through the Kubernetes Service port-forward, health/readiness returned 200 with `no-store`,
  an unknown frontend route returned the SPA index, and an unknown API route returned 404
  Problem JSON. Browser execution also loaded the actual built assets.
- Independent review covered the generator, rendered-manifest guardrails, Rust/configuration
  fixes, Spectral wrapper, dependency changes, and contrast correction. Reported defects were
  fixed and regression-tested; no actionable findings remain in those reviewed changes.
- Temporary Compose containers/database volume, Kubernetes namespace, and port-forward were
  removed after verification. The locally built images remain available for development.

### Reproduce

Install the pinned tools described in [testing.md](testing.md) and
[deployment.md](deployment.md), then run:

```sh
npm --prefix web ci
npm --prefix e2e ci
scripts/gates.sh all
scripts/gates.sh coverage
scripts/gates.sh security
scripts/gates.sh k8s
docker compose up --build -d
npm --prefix e2e run install:browsers
scripts/gates.sh browser
docker compose down
```

Use the exact scanner command in `.github/workflows/ci.yml` for the image scan. Deploy the
same tested image using [deployment.md](deployment.md), port-forward the Service to 8085,
and rerun the browser gate. No game, load, multi-node, or recovery guarantee follows from
these scaffold checks. These are local results; hosted CI status is separate evidence.

The original `AGENTS.md` remains unchanged under the explicit preservation instruction;
current engineering conventions live in `CONTRIBUTING.md`.
