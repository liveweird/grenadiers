# Repository verification

## Live-v7 HUD wrapping correction — 2026-10-18 — refreshed image acceptance pending

Visual review of the passing native baseline exposed overlapping Polish HUD labels at 320px.
Labels and values now wrap without hiding or truncating the eight named fields. The real
Polish 320px match scenario checks text-range containment and cross-cell overlap for both seats.
Independent review and all 275 web tests, coverage, builds and E2E static parity 27 pass. A new image and complete fresh acceptance
are required before publication. The audited 52/52 record below certifies source `b1d0340`
only; it does not certify this later layout correction.

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
