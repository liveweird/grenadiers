# Repository verification

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
