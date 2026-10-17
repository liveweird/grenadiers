# Repository verification

## M6 private resume credential/binding helpers — 2026-10-17

The combined locked server gate passes **294 core tests, 136 server tests, and 2 doctests**.
Rust coverage is **7,373/7,913 lines, 93.18%**, above the unchanged 80% floor.
Independent review reports no actionable helper findings. Four focused cases cover independent
32-byte entropy, exact three-attempt collision bounds, failure atomicity, stable capability
matching, fully redacted diagnostics, and exact connection/generation replacement fences.
The module remains test-compiled; staging publication atomically and supplying fresh binding
generations remain coordinated activation duties. Exact-image acceptance is separate.

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
