# Release history

This archive preserves the older release records originally maintained in
[`docs/releases.md`](../releases.md). Records remain in their original order; the current
release procedure and current release record stay in the [release guide](../releases.md).

## v1.3.0 — 2026-11-01

Solo and custom rooms now offer Easy, Standard and Hard, with Standard as the default and one
shared tier for every CPU seat. Hard uses range-aware firing positions and gives demonstrated
opponent pressure priority over routine farming while retaining valuable pickups and persistent
goals. A bounded geometric prefilter and legacy pursuit fallback keep distant rivals reachable
when the board clears. See [difficulty](../game/single-player.md#difficulty) and
[CPU pressure](../game/multiplayer-cpu.md#hard-pressure).

Hard has the same permitted observations, movement/damage rules, two-worker planner and 65,536-work
limit. Its optional pressure analysis is capped at 4,096 and one eighth of remaining work.
Planting still needs full escape, contact and exact-target usefulness proofs; fresh admission
withdraws usefulness when a rival leaves the blast. Pressure evaluates current visible positions.
Easy keeps its navigation pause; Easy/Standard retain CPU27, and Hard selects CPU28.

The [controls capture](../screenshots/hard-difficulty.md) records the visible lobby changes.

### Paired upgrade

Gameplay19 is the live contract for every newly created room, including human-only matches.
Reload tabs after upgrading the paired server/web image. Restart ends process-local matches.
Archived wire14–18 and replay readers remain closed to Hard. Runtime publication identity is
explicit and independent of rules, difficulty and navigation. Private replay13 binds gameplay19,
rules10/navigation3 and the exact Easy/Standard27 or Hard28 pairing. Production replay recording
and export remain disabled. See [gameplay19](../game/protocol-v19.md).

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

## v1.2.5 — 2026-11-01

CPU players can certify useful bomb placement while an Oil barrel keeps burning. The previous
Teleporter support guard could prohibit planting globally for the rest of the round. CPU27
models static Oil through the existing persistent lethal-cell forecast instead. With Teleporter
enabled it still requires visible empty bomb knowledge, no finite flames and no exposed
Teleporter. Full escape, selected-target usefulness, mob-contact and bounded-work proofs remain
required. See [CPU support boundaries](../game/multiplayer-cpu.md#persistent-burning-oil).

Easy and Standard select CPU27 with their existing cadence. Historical CPU26 behavior and
navigation policies remain immutable. Private replay12 binds rules10/CPU27/navigation3 and
rejects mixed provenance; production recording/export remains disabled. Wire18 is unchanged.

### Verification

Accepted image source: `6e322f2fce8208e7e03e75c0c9a0b8a27cea057c`.
Independent core and cross-component activation reviews are clear. Later release-record edits
do not change image build inputs.

- Actual State-derived lairs-enabled/disabled fixtures show CPU26 refusing and CPU27 certifying
  a useful bomb in the identical scene. Authoritative execution destroys the selected block
  and survives the complete escape interval. Guard-isolation, lethal escape Oil, visible bombs,
  finite flames, unavailable bomb knowledge, exposed Teleporter, scene invalidation and hidden
  item-reveal controls pass. Per-job work remains at most 65,536.
- A production-paced three-CPU episode records each seat moving and placing its own bomb,
  with worker delivery/admission accounting; cancellation and guarded admission refusals remain
  diagnostic scheduling outcomes, rather than prohibited events. It is operational evidence, not a guarantee that
  productive planting is always available. An earlier accelerated-tick farming-stall hypothesis
  was rejected after identical-input plant proofs showed no omitted productive certificate.
- Native Rust debug/release suites pass: 1,678 core and 609 server tests, plus the separately
  partitioned exhaustive catalogue test. After test-only scheduling corrections, both native
  worker tests and the complete fresh instrumented workspace are rerun. Line coverage is
  53,089/57,336 (92.59%), above the unchanged 90% floor. All 4,194,304 masks run in debug,
  release and coverage with four workers, 32 partitions and no retries. Clippy, formatting,
  build and repository checks pass.
- Distinct Linux ARM64 release checks pass: six Oil proofs, one worker-dispatch test, two
  exact-profile tests and nine replay10–12 tests. The final image retains the exact same
  filesystem layers as that Linux-accepted candidate; only test scheduling and provenance
  labels changed.
- Exact-image Compose authentication/duel: two passed; an all-features three-CPU match moves
  all CPUs and plants bombs without browser errors. Exact-image Kubernetes authentication/duel
  and restricted runtime pass. Owned acceptance stacks and namespaces are removed.
- Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL vulnerabilities. All five image web files
  equal the accepted v1.2.3 production build. Reuse that full web/native camera acceptance and
  v1.2.2 artwork/audio acceptance; these inputs and dependencies did not change.

### Operation

Accepted local image:
`sha256:fbca836651065065bcd9222ec6333efb1e36a8b33f9938ab5c0f44db1169735b`.
The exact accepted image is promoted to localhost:8085; v1.2.4 remains available for rollback.
This is a source/local-runtime release without a hosted service or production registry.
The unchanged artwork remains in the
[v1.2.2 artist ZIP](https://github.com/liveweird/grenadiers/releases/download/v1.2.2/grenadiers-artist-v1.2.2.zip).

## v1.2.4 — 2026-11-01

Mobs can escape corridors whose first few cells remain inside a future timed blast. Live
navigation3 finds bounded, time-aware routes to refuge, checking whole footprints against fuse
and chain deadlines. It rechecks changed obstacles and hazards; uncertain bomb models retain
conservative avoidance. See [lair navigation](../game/lairs.md#movement-and-combat).

Historical navigation and CPU identities remain unchanged. Private replay11 binds rules10,
CPU26 and navigation3; production recording/export remains disabled. Wire18 and the paired web
client are unchanged. Conservative Oil forecasts can refuse possible routes; this is not a
claim that every mob can escape every bomb.

### Verification

Accepted image source: `a335555afbeee8d1a85c84cc3fd840c7bc81c665`.
Independent review is clear. Later release-record changes do not alter image build inputs.

- Full local parallel Rust gate passes: debug, release, Clippy, formatting, build and coverage.
  Core: 1,672 passed and one ignored; server: 601 passed and 15 ignored, plus the separately
  partitioned exhaustive 22-item catalogue test. All 4,194,304 masks run in each of debug,
  release and coverage, with four workers, 32 partitions and no retries. The complete gate
  takes 729 seconds; line coverage is 52,661/56,934 (92.49%), above the unchanged 90% floor.
- Navigation fixtures cover chain deadlines, multi-cell transit, changing obstacles,
  whole-footprint safety, bounded work, conservative Oil states and historical-policy isolation.
  An actual rules10 State with all items/environment enabled survives the corridor where
  navigation2 waits and dies. Replay11 rejects mixed policy identities.
- Distinct Linux ARM64 release checks pass: 38 mob, 70 lair and six replay10/11 tests.
- Exact-image Compose authentication/duel: two passed. A three-CPU all-features match moves
  all CPUs and plants bombs without browser errors. A two-human match verifies both lairs
  spawning one mob each and suspending their respawn timers. Random blocked spawn exits
  retain the documented next-tick retry behavior.
- Exact-image Kubernetes authentication/duel and restricted runtime pass. Acceptance stacks
  and namespace are removed. Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL vulnerabilities.
  All five image web files equal the accepted v1.2.3 build.

Reuse v1.2.3 full web/native camera acceptance, bundle budgets and v1.2.2 artwork/audio
acceptance: those inputs and dependencies did not change.

### Operation

Accepted local image:
`sha256:7e11ff44f5a46148c7deba65a4209add092f7c22155c8a8df9d9fd3390086924`.
The exact accepted image is promoted to localhost:8085; v1.2.3 remains available for rollback.
This is a source/local-runtime release without a hosted service or production registry.
The unchanged artwork remains in the
[v1.2.2 artist ZIP](https://github.com/liveweird/grenadiers/releases/download/v1.2.2/grenadiers-artist-v1.2.2.zip).

## v1.2.3 — 2026-11-01

Each browser randomly chooses a short zoom-in, zoom-out, left-rotation or right-rotation cue at
fresh round openings and observed round/match results. The 900ms effects return exactly to the
normal view; reduced motion stays stationary. The full tray remains framed on every board and
viewport. See the [native camera examples](../screenshots/round-camera.md).

A client presentation latch observes genuine snapshot-free countdowns before React coalescing,
then starts the intro when the first running arena arrives. Controls remain active. Late admission,
repeated reconnect countdowns, suspension and context recovery cannot replay consumed cues.
Animation uses the existing frame loop and adds no timers or GPU resources. See
[arena rendering](../game/arena-3d.md#round-camera-transitions).

### Verification

Accepted image source: `7153ffd877556fb5e9344cb7780f56fb24e95216`,
[PR #303](https://github.com/liveweird/grenadiers/pull/303). Independent cross-component review is clear.
Later changes contain only browser-test corrections, documentation and screenshots; image build
inputs remain unchanged.

- Full web gate: 7,407 tests in 167 files; coverage 95.59% lines, 92.54% statements,
  91.69% functions and 88.82% branches. All existing floors remain unchanged. Types, lint,
  Knip, generated-contract/validator drift and 120 documented browser scenarios pass.
- Production bundle: JavaScript 399,557/400,000 gzip bytes; CSS 15,474/20,000.
- Native renderer regression: 25 passed, 13 project-specific skips. All four opening/ending
  variants change native pixels and settle to byte-identical canonical pixels; reduced motion,
  billboard/death/fire layering, board sizes and WebGL recovery pass.
- Isolated warmed callback submission cost, 40 frames each: empty median/p95/max
  0.20/0.30/0.30ms; worst public fire 1.70/1.80/2.10ms. This is not a hardware FPS claim.
- Exact-image Compose duel/authentication: two passed. The real two-client browser scenario
  verifies intros across first and subsequent rounds without fabricating countdown snapshots;
  both clients use isolated contexts and close cleanly. Fresh three-CPU motion/bomb smoke passes.
- Exact-image Kubernetes duel/authentication and restricted runtime pass. Owned acceptance
  stacks and namespace are removed. Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL findings.
  All five image web files equal the verified production build.

Backend logic, contracts, dependencies and artwork are unchanged. Reuse v1.2.2 artwork/audio
and v1.2.0/v1.2.1 Rust debug/release, coverage, dependency and distinct Linux acceptance.
The release image also compiles the server for Linux.

### Operation

Accepted local image:
`sha256:aea9ced2c51dbbeecc8dba9a22783c77d12d1a279932d6f05c04be47c41be8de`.
The exact accepted image is promoted to localhost:8085; v1.2.2 remains available for rollback.
This is a source/local-runtime release without a hosted service or production registry.
The unchanged artwork remains available in the
[v1.2.2 artist ZIP](https://github.com/liveweird/grenadiers/releases/download/v1.2.2/grenadiers-artist-v1.2.2.zip).

## v1.2.2 — 2026-10-31

Optional retro sound now includes quiet local footsteps, ordinary and special-bomb placement and
explosions, pickups, and player/mob deaths. Sound starts muted; the EN/PL volume slider persists
without autoplay. A quiet master, four-voice limit, event coalescing and per-cue rate limits prevent
crowded matches from stacking excessive volume. Footsteps yield to important cues. See
[audio](../game/audio.md) for the accepted-event and cosmetic local-movement boundaries.

The owner-supplied sprite refresh updates fire, spike pits, related icons and the cosmetic blood
pool. All stable rectangles and character/material pixels are preserved. Import processing uses
a shared twelve-color pit palette and matching flame edges; the root upload is removed after
validation. See the [artist handoff](../../assets/art/README.md).

### Verification

Accepted source: `afa8f5a98005cf6e599d14090984b5d2bc989b14`,
[PR #302](https://github.com/liveweird/grenadiers/pull/302).
Independent art and audio reviews have no unresolved findings. Review caught and corrected
phase-to-phase palette flicker, browser-history sound resumption, and interrupted volume ramps.

- Full web gate: 7,377 tests in 164 files; coverage 95.47% lines, 92.38% statements,
  91.55% functions and 88.71% branches. All existing floors remain unchanged. Types,
  lint, Knip, generated-contract/validator checks and 116 documented browser scenarios pass.
- Production bundle: JavaScript 398,331/400,000 gzip bytes; CSS 15,474/20,000.
- Native artist workbench: 12 passed; final shared-palette/seam recheck: two passed.
  Native Three fire, reduced-motion, restoration and terrain coverage: 14 passed with
  ten project-specific skips; final spike-phase checks: two passed.
- Exact-image Compose and Kubernetes duel/authentication: two passed each. Fresh all-on
  three-CPU motion/bomb smoke passes. Native Web Audio, mute/resume, visibility,
  movement, volume persistence, EN/PL layout and Axe scenario passes.
- Pinned Trivy 0.74: zero fixable HIGH/CRITICAL findings. All five image web files equal
  the verified production build. Owned acceptance stacks and Kubernetes namespace are removed.

Backend logic, contracts and dependencies are unchanged. Reuse v1.2.0/v1.2.1 Rust debug/release,
coverage, dependency and distinct Linux acceptance; the candidate image also compiles the release
server for Linux. Native browser checks verify Web Audio processing and lifecycle; hardware
listening and subjective mix preference require playtesting.

### Operation and artist handoff

Accepted local image:
`sha256:9f5c91b028424b1a3bafa50058f1f729ead6424b37b6f4c924809557e5d52242`.
The exact accepted image is promoted to localhost:8085; v1.2.1 remains available for rollback.
This remains a source/local-runtime release without a hosted service or production registry.

The artist source remains `assets/art/arena-source.png` plus `arena-source.json`: 1120×2240,
192 runtime drawings, four reserved slots and four unused cells. The
[shareable artist ZIP](https://github.com/liveweird/grenadiers/releases/download/v1.2.2/grenadiers-artist-v1.2.2.zip)
contains the paired source, guides, named slices, runtime pack and native previews.

## v1.2.1 — 2026-10-31

Public fire now warms nearby terrain and characters. A persistent four-light pool, six-frame
pixel flames, brief local ignition, embers and smoke improve depth while retaining clear hazard
footprints. Reduced motion keeps static fire and illumination. Existing artist pixels are unchanged.

WebGL recovery now restores the clear colour, atlas upload and shadows after Three rebuilds its
caches; frozen before/after images match exactly. See [the renderer guide](../game/arena-3d.md).

### Verification

Accepted source: `993f2342ffe9aaf7c5ee34a33fbfa575c1fdfd14`,
[PR #301](https://github.com/liveweird/grenadiers/pull/301).
Independent reviews cover public-only lighting, bounded resources, reset/reduced-motion behavior,
artist-frame preservation and the current-validator optimization. Expired ignition slots release
before later bursts; admission into existing fire cannot replay old explosions.

- Full web: 7,363 tests in 164 files. Coverage: 95.40% lines, 92.30% statements,
  91.43% functions and 88.62% branches; all floors remain unchanged. Types, lint, Knip,
  generated-validator drift and 115 documented browser scenarios pass.
- Native Three: 23 passed with 11 project-specific skips; two strict context-recovery cases
  and two final lighting/animation cases pass. Artist workbench: 11 passed.
- A warmed 40-frame diagnostic measures render-callback submission cost, not hardware FPS:
  empty median/p95/max 0.20/0.30/0.30ms; worst public fire 1.70/2.10/2.40ms.
- The hard bundle budgets remain unchanged: JavaScript 397,119/400,000 gzip bytes,
  CSS 15,430/20,000. Only gameplay18 adopts shared canonical Ajv code-generation options;
  boolean acceptance and exact ordered compiler parity pass. Internal schema paths change
  as documented in [ADR 0022](../decisions/0022-standalone-browser-validation.md).
  Archived validators remain byte-identical.
- Exact-image Compose duel/authentication and a fresh all-on three-CPU match pass, with
  physical CPU movement and bomb placement. Kubernetes duel/authentication and restricted
  runtime checks pass. Owned acceptance stacks and namespace are removed.
- Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL findings. Every image web file equals
  the verified build. Backend source, contracts and dependency versions are unchanged;
  reuse v1.2.0 Rust debug/release, coverage, dependency and distinct Linux acceptance.
  The metadata-only duplicate Linux run 41235509963 was cancelled under local-first policy.

### Operation and artist handoff

Accepted local image:
`sha256:27406a134593ae5d81316df71e93e3ee8a9df0ec0a47d1bcbfd1b9457c927dd1`,
version 1.2.1/source 993f2342, tagged `grenadiers:fire-v121`. Reload browser tabs after upgrading;
process-local matches cannot survive a server restart. The previous v1.2.0 image remains available
for rollback. Gameplay stays on protocol18/rules10/CPU26.

The [v1.2.1 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.2.1/grenadiers-artist-v1.2.1.zip)
contains 205 files: canonical PNG/JSON, separate guides, overview, 192 runtime slices and native
fire previews. The source grows to 1120×2240, with 196 assigned entries (192 drawings and four
blank reservations), plus four free cells. Slots 189–195 append four flame poses, ember, smoke
and ignition. Decoded comparison preserves 9,524,224 old RGBA bytes outside new rectangles and
446,464 transparent new-row gutter bytes without differences; all 189 prior frame descriptors
are identical. The runtime atlas remains within its existing 16MiB decoded texture budget.

## v1.2.0 — 2026-10-31

Add playable contact-triggered spike pits: warning, raised damage and automatic retraction.
Pits start enabled with a 0.6s warning and 3s raised interval; creators can adjust both timings.
Shield, Heart and normal grace retain their usual behavior. CPU26 and lair mobs account for
pit contact and timing. The shared artist sheet includes three editable phase tiles.
See [spike pits](../game/spike-pits.md) and [gameplay18](../game/protocol-v18.md).

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

## v1.1.8 — 2026-10-31

Add opt-in rules10 spike-pit simulation: contact arms a warning, raised spikes use normal
protection and damage, and retraction resets the trap. Movement traces, forced movement,
Teleporter destinations, mob deaths/respawns and transactional failures are covered.
The [core guide](../game/spike-pit-core.md) documents the library behavior.

### Verification

Accepted candidate: `5207a45cff9052a79c1004f81750900118a8aee6`,
[PR #299](https://github.com/liveweird/grenadiers/pull/299).
Independent review resolved dead-mob trace damage and unnecessary legacy trace cloning.

- Full core debug and release pass: 1,613 tests plus two doctests each. Strict Clippy,
  formatting and isolated core coverage pass; line coverage is 92.99% against the 90% floor.
- Distinct [Linux debug/release checks](https://github.com/liveweird/grenadiers/actions/runs/41228101948)
  pass for this candidate.
- Exact-image Compose duel/authentication: 2/2 pass; Kubernetes duel/authentication: 2/2 pass.
  Restricted runtime/security checks pass and owned test resources are removed.
- Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL vulnerabilities. All five served web
  files equal the verified v1.1.6 frontend; its unchanged web, graphics and dependency
  acceptance is reused.

### Operation

Accepted local image:
`sha256:d5d70081f399697483e53a4a36be5a1183b63a78627c6813d15f9f7a46bf5e4e`,
labelled 1.1.8/source 5207a45c, tagged `grenadiers:spike-physics-v118`.
Live transport remains gameplay17/rules9/CPU25. Localhost:8085 stays on the accepted v1.1.6
blood-pool image until the paired spike contract, AI and presentation are ready.
The [v1.1.6 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.6/grenadiers-artist-v1.1.6.zip)
remains current. This library increment does not enable playable spike pits.

## v1.1.7 — 2026-10-31

Add dormant spike-pit core primitives: checked warning/raised clocks, strict positive-area
contact helpers and deterministic symmetric clear-floor placement. No live profile selects
these APIs yet. Spike simulation, transport, AI and visuals remain separate integration work.
The [core guide](../game/spike-pit-core.md) defines the implemented library invariants.

### Verification

Accepted source: `676f2a3e94bbd1cf3fd40113092ef8a3f8709c87`,
[PR #298](https://github.com/liveweird/grenadiers/pull/298). Independent review found and
resolved same-tick retraction rearming and undefined stale-tick behavior. Updates and forecasts
now enforce a checked horizon; both deadline additions are atomic.

- Twelve focused regressions pass in debug/release. Full core passes 1,594 tests plus two
  doctests in each profile. Formatting and strict all-target core Clippy pass.
- Isolated core-only coverage passes the unchanged 90% floor at 92.87% lines. An initial
  shared-target report mixed stale objects from other worktrees and unexecuted server objects;
  the isolated report measures only the tested core. No repository exclusion or floor changes.
- Distinct Linux debug/release core/server coverage passes in
  [the platform run](https://github.com/liveweird/grenadiers/actions/runs/41224893047).
- The built image passes Compose duel/auth/CPU, disabled diagnostics, all-on visual smoke
  and Kubernetes duel/auth/runtime checks. Owned test stacks and namespace are removed.
  Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL findings.
- All image web files equal the v1.1.6 verified frontend. Reuse its full web/coverage,
  native graphics, E2E-static, artwork and dependency acceptance: those inputs are unchanged.
  No existing live rules, generation or protocol behavior changes.

### Operation

Exact accepted image:
`sha256:f0d8c16222afc059b902e0ac2a94227133858fea469b7ead32c66d76f7126e70`,
labelled 1.1.7/source 676f2a3e. It is available locally as `grenadiers:spike-foundation-v117`.
Localhost:8085 retains the accepted v1.1.6 blood-pool image while the remaining spike-pit pieces
are integrated. The running game's behavior and artist sheet are unchanged by this foundation.
The [v1.1.6 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.6/grenadiers-artist-v1.1.6.zip)
remains current. This source release does not enable spike pits in playable matches.

## v1.1.6 — 2026-10-31

Leave small dark-red blood pools on the floor at recipient-visible player and mob deaths.
Pools remain after the knockout body and shadow disappear, through the result screen until
the next round. One pool per tile bounds repeated lair deaths. Stains have no simulation,
collision, damage or AI effect and do not reconstruct hidden or baseline death positions.

The shared sheet appends `effect.bloodPool` at slot 185 without changing its 1120×2128 canvas,
the preceding 185 frame entries or any pixel outside the new rectangle. Four reserved and
four unassigned cells remain transparent. The [updated artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.6/grenadiers-artist-v1.1.6.zip)
contains the source pair, new guides, slices, labelled overview and 3D preview.

### Verification

Accepted production source: `6fc9d151276029abd1188c5e53cc70ee3cc9e2c4`. Independent review
found and resolved a replacement-room clock rollback case that dropped a valid coalesced
death. Strictly newer generations retain fresh cues; stale same-generation envelopes remain
fenced. Seen-cue history is capped at 128, independently of the per-tile pool bound.

- Full web gate passes: 7,303 tests in 152 files (263.34s); coverage 95.24% lines,
  88.80% branches, 92.06% statements and 91.33% functions. Lint, Knip, contracts and production
  build pass; JS is 395,197/400,000 gzip bytes, CSS 15,430/20,000.
- E2E static/types/scenario checks pass (107 mappings). Native Three passes fifteen cases
  with seven intentional project skips. All player appearances and mobs expire exactly to
  independent pool-only references in full/reduced motion without another snapshot. Ground
  placement, foreground readability, duplicate envelopes and lifecycle resets are covered.
- Native artwork acceptance passes nine unchanged cases plus the focused inventory rerun.
  The inventory assertion now explicitly separates twenty death frames from the appended
  blood frame. Standard export passes: 182 runtime frames, 2040×704 / 5,744,640 RGBA bytes.
  Direct pixel review confirms 438 opaque pixels, four dark-red colors, binary alpha and
  zero changes to previous pixels. No existing floor, character or item art is repainted.
- Exact-image Compose passes four duel/auth/CPU cases (36.2s), disabled diagnostics (11.0s)
  and an all-on visual smoke with zero browser errors. Kubernetes duel/auth passes (23.2s),
  with runtime restrictions/probes/resources and cleanup of owned test resources.
- Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL findings. Every image web file equals
  the verified build. Backend implementation, dependencies and gameplay contracts are unchanged;
  reuse their recorded acceptance. The image includes a fresh Linux release build.

### Operation

Localhost:8085 runs exact accepted image
`sha256:35a4c62d9bc7dccfa8aa768b703f5e0e47f4a22bf03e488da0f6ec4f80c75c33`,
labelled 1.1.6/source 6fc9d151. Local CPU diagnostics remain enabled; deployment defaults remain
disabled. Promotion checks identity, cgroups/security, probes, CSP/cache and served bytes.
A fresh all-on match renders without browser errors; the test room is left explicitly.
Immutable v1.1.5 remains available for rollback. Refresh old tabs after restart.

Spike-pit work remains a separate backlog item; this release adds no spike gameplay.

## v1.1.5 — 2026-10-31

Adopt the owner-supplied retro-texture PNG directly. All ten 3D material maps now use bolder
tan sandstone, gray masonry and red brick patterns, with distinct lair surfaces. The remaining
171 baseline frames are decoded-RGBA identical, including every player/mob pose, death frame,
weapon, pickup, cue and effect. The paired manifest remains unchanged.

The [current artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.5/grenadiers-artist-v1.1.5.zip)
contains the updated PNG, unchanged JSON/guides, labelled overview, slices, runtime preview and
actual 3D capture. The canonical source remains [assets/art/arena-source.png](../../assets/art/arena-source.png).

### Verification

Accepted source: `134df03f783701bf82c06e0dcc18817f587842ea`. Independent decoded-pixel review
finds no actionable defects: 1120×2128 RGBA8, binary alpha, clean gutters/reservations and no
changes outside the ten 48×48 material rectangles. Every opposite texture edge matches, all
top/side pairs remain distinct, and material palettes contain 10–28 colors. The supplied file
is adopted byte-for-byte; the redundant root copy is removed after native validation and equality checks.

- Raw native import/export passes without adaptation: 181 runtime frames, 2040×704 texture and
  5,744,640 RGBA bytes. Focused atlas/Three/status tests pass: 105 tests in 25 files.
- Artist workbench passes ten native cases (15.8s). Native Three passes thirteen cases (48.7s),
  with five intentional project skips, including compact badges, upright lairs, death playback,
  reduced motion, board sizes, colour calibration, removal and context recovery.
- Production build passes at 394,496/400,000 gzip JS bytes and 15,430/20,000 CSS bytes. All
  five container web files equal the verified build. Pinned Trivy 0.74 finds zero fixable
  HIGH/CRITICAL vulnerabilities.
- Exact-image Compose duel/auth/CPU and disabled diagnostics pass, alongside an all-on four-player
  visual smoke with zero browser errors. Kubernetes duel/auth and runtime restrictions pass.
  Owned test stacks and namespace are removed.
- Application code, contracts, dependencies, manifest and deployment configuration are unchanged.
  Reuse the recorded full 7,294-test web/coverage, E2E-static, dependency/security and Rust
  acceptance under the local-first policy. Cargo product metadata advances to 1.1.5; the image
  includes a fresh Linux release build. No coverage floors or assertions change.

### Operation

Localhost:8085 runs exact accepted image
`sha256:2f950984af61fbe11fc22a6888cae37587a7731d4cae9199eca0bb49f0ebd15f`,
labelled 1.1.5/source 134df03f. The operator's CPU diagnostics preference remains enabled;
deployment defaults remain disabled. Promotion verifies image/runtime identity, probes,
CSP/cache and served assets. Immutable v1.1.4 remains available for rollback. Refresh old tabs
and start a new match after restart to see the retro surfaces.

## v1.1.4 — 2026-10-31

Fix two 3D presentation defects without changing artwork or gameplay. Remote-armed and Invisible
Bombs indicators become compact, separate head badges, leaving player uniforms visible. Lairs
have taller stone bodies with their existing barred-doorway sprite upright on the camera-facing
side. Their ordinary pillar geometry is removed so it cannot obscure the entrance.

### Verification

Accepted source: `a8a5cd18e6c2b6918763fe6c682312aaa7ee42cd`. Independent review finds no remaining
actionable defects after correcting overlapping badges, the underlying lair wall and invalid
diagnostic terrain. Stored weapons, armed ownership and recipient visibility remain unchanged.

- `scripts/gates.sh web`: 7,294 tests in 150 files pass (288.72s), with 95.32% line and
  88.86% branch coverage; compressed JS is 394,500/400,000 bytes. Lint, dependency reachability,
  contract checks, coverage and production bundle limits pass without lowering any floor.
- `scripts/gates.sh e2e`: lint, dependency reachability, types and all 105 scenario mappings pass.
- Native Three acceptance passes 13 executed cases, with five intentional project skips. The
  narrow remote-badge regression retains at least 85% of the actor-only pixel mask and proves
  the badge adds visible pixels outside it. Lair matrix tests prove upright orientation, face
  clearance and ordinary-wall exclusion; a hard-wall native fixture supplies the visual capture.
- Exact-image Compose passes four duel/auth/CPU cases and disabled diagnostics; an all-on
  four-player visual smoke has zero browser errors. The ad-hoc smoke helper initially attempted
  to apply already-current defaults; correcting that helper required no application change or
  repetition of the five passing runtime cases. Kubernetes duel/auth and runtime restrictions
  pass. Owned test stacks and namespace are removed.
- Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL findings. Backend implementation,
  dependencies, contracts and artwork are unchanged; reuse their recorded acceptance, including
  Linux Rust debug/release coverage. The image supplies a fresh production Linux release build.
  All five container web files equal the verified local build.

### Operation

Localhost:8085 runs exact accepted image
`sha256:4cb3efaf0816ef6ac3184ff56cea679594cd6e034dcd5bafcb4a4299d1136c2e`,
labelled 1.1.4/source a8a5cd18. Local CPU diagnostics remain enabled; deployment defaults remain
disabled. Promotion checks image identity, runtime restrictions, probes, CSP/cache and served
asset equality. Immutable v1.1.3 remains available for rollback. Refresh old tabs after restart.

The [v1.1.3 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.3/grenadiers-artist-v1.1.3.zip)
still contains the current, unchanged PNG and manifest. No artist revision is needed for either fix.

## v1.1.3 — 2026-10-31

Adopt the owner-supplied polished arena PNG directly, without resizing, recoloring or metadata
changes. It refines all ten 3D material maps and lightly retouches twenty death poses. All
64 directional player poses, four mob directions and other sprite drawings remain unchanged.
The sheet retains 1120×2128 RGBA dimensions, 185 assigned slots, anchors and animation contracts.
The redundant root `arena-napoleonic-3d-polished.png` is removed only after verifying its adopted
copy is byte-identical.

Use the [updated artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.3/grenadiers-artist-v1.1.3.zip)
for the current PNG/JSON, labelled guides, slices and actual 3D preview. The canonical source
remains [assets/art/arena-source.png](../../assets/art/arena-source.png).

### Verification

Accepted source: `e005b4228ac0f6bf76ccb4a6af794efc9e57f5c8`,
[PR #293](https://github.com/liveweird/grenadiers/pull/293). Independent decoded-pixel review
finds no structural defects: binary alpha, clean gutters/reservations, compatible death
palettes and ground alignment, four distinct poses per death family. Exactly 30 frames change;
151 baseline frames and all pixels outside allowed rectangles remain unchanged.

- Raw input exports through the native validator without adaptation. Focused atlas/Three/UI
  tests pass: 101 tests in 24 files. Artist workbench passes ten cases. Native 3D passes eleven
  cases, with three intentional project skips, including all death frames, reduced motion,
  expiry, flame overlap, board sizes, context recovery and removal privacy.
- Fresh production build passes at 394,351/400,000 gzip JS bytes and 15,430/20,000 CSS bytes.
  Runtime pack remains 2040×704 / 181 frames / 5,744,640 RGBA bytes. All five container web
  files equal the verified build. Exact canonical file bytes equal the supplied input.
- Exact-image Compose passes four duel/auth/CPU cases (36.8s), disabled diagnostics (10.3s)
  and a native four-player visual smoke with zero page errors. Kubernetes duel/auth passes
  (23.6s), alongside runtime restrictions, configured probes/resources and zero restarts.
  Owned stacks and namespace are removed. Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL.
- Application/rules/contracts/dependencies and Kubernetes configuration are unchanged. Reuse
  the recorded full web/coverage/security/Rust and E2E-static acceptance; no floors or assertions
  are weakened. Only duplicate unchanged Rust CI is cancelled. Cargo product versions change
  to 1.1.3; the image supplies a fresh Linux release build and runtime acceptance.

### Operation

Localhost:8085 runs exact accepted image
`sha256:0c553e5c95b1a558608e8b9fcab2cfe1cb1d8dda510dba6ce1f3a37e3f73cc95`,
labelled 1.1.3/source e005b422. The operator keeps CPU diagnostics enabled; base deployments
default to disabled. Promotion checks image/runtime identity, health/readiness, CSP/cache and
served bytes. Immutable v1.1.2 remains available for rollback. Refresh old tabs and create a
new match after restart to see the polished material maps.

## v1.1.2 — 2026-10-31

Keep knockout sprites readable during ordinary explosions and persistent Oil fire. The death
body now uses the minimum layer above world hazard overlays; its grounded shadow stays
unchanged and higher actor status cues remain visible. Artwork, animation timing and gameplay
are unchanged. The [v1.1.1 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.1/grenadiers-artist-v1.1.1.zip)
remains current, including the wall maps and all twenty death poses.

### Verification

Accepted source: `dbf17457781ace59fce4ba7ac18cf48f70b5db67`,
[PR #292](https://github.com/liveweird/grenadiers/pull/292). Independent review confirms the
transparent-depth ordering cause and the minimal replacement layer. The pre-fix native
regression retains only 124/889 death-mask pixels (13.95%). The correction passes ≥75% body
retention for both ordinary and burning-Oil flames, while separate comparisons prove the
surrounding hazard is still visible.

- Fresh focused Three/client tests: 57 pass. Full native graphics: 11 executed cases pass;
  three intentional cross-project skips select the appropriate desktop/reduced-motion cases.
  Lint, Knip, types, repository/tooling, static E2E and 103 scenario mappings pass.
- Fresh production build: 394,348/400,000 gzip JS bytes; 15,430/20,000 CSS bytes. All five
  container web files equal the verified build. The sole production behavior change is a
  death-body draw-order value, exercised by the focused tests and native red/green proof.
- Reuse [v1.1.1](releases.md#v111--2026-10-31) full 7,290-test/coverage acceptance and
  unchanged artwork, dependencies and Rust acceptance. Existing floors remain intact. Rust
  package metadata changes only; the exact image supplies a fresh Linux release build and
  runtime checks. Duplicate unchanged Rust CI is deliberately cancelled.
- Exact-image Compose acceptance passes four duel/auth/CPU cases (37.7s) and disabled
  diagnostics (11.3s). Kubernetes passes duel/auth (23.7s), configured probes/resources,
  restricted runtime and zero restarts. Owned stacks and namespace are removed.
- Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL vulnerabilities in the exact accepted image.
  Dependency/security inputs are unchanged from the accepted v1.1.1 checks.

### Operation

Localhost:8085 runs accepted image
`sha256:aa2a199cddf9e8369a4058aa3e7782ad267f4d1f1ef53da98fd3b6e118a2f90d`,
labelled 1.1.2/source dbf17457. The local CPU-diagnostic flag remains enabled; base deployments
still default to disabled. Promotion verifies image/runtime restrictions, health/readiness,
CSP/cache and served bytes. Immutable v1.1.1 remains available for rollback. Refresh the browser
and create a new match after restart.

## v1.1.1 — 2026-10-31

Add short, non-gory knockout animations for all four player uniforms and mobs. Each appearance
has four artist-owned poses, played once over 600ms; reduced motion shows only the collapsed
pose for 180ms. The detached sprite/shadow then disappear. Elimination, collisions, scoring and
round timing remain immediate and authoritative.

Player cues require a fresh event and the recipient's currently visible dead pose; hidden Cloak
positions never appear. Client ingress latches events before React batching can drop them.
Qualified mob removal uses its last public pose only across tightly continuous same-round/lair
snapshots, with long gaps suppressed. Final round and match wins can animate both together.
Admission/reconnect, suspension, timeline/round changes and preference changes purge cues;
duplicate publications cannot replay them.

The shared sheet appends twenty poses and preserves every previous sprite and material map.
Download the [updated artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.1/grenadiers-artist-v1.1.1.zip),
including canonical PNG/JSON, guides, labelled inventory, slices, and a death-pose overview.
See [presentation](../game/arena-3d.md#death-animation) and [artist rules](../../assets/art/README.md).

### Verification

Accepted production source: `65bcea9656eb3a3fce530fa37aef14c4862a193a`,
[PR #291](https://github.com/liveweird/grenadiers/pull/291). Independent review is clear after
terminal-mob and preference-transition corrections. Tests cover callback coalescing,
recipient privacy, same-tick departures, terminal wins, mob continuity, bounded state, reset
fences and expiry without another snapshot.

- Full web gate: 7,290 tests in 149 files pass in 248.87 seconds. Coverage is 95.20% lines,
  88.84% branches, 92.05% statements and 91.44% functions. Existing floors remain unchanged.
  Verified build is 394,344/400,000 gzip JS bytes and 15,430/20,000 CSS bytes; all five container
  web files equal the verified build. Repository, tooling, API/schema, lint and static E2E
  checks pass, including 102 scenario mappings.
- Native Three acceptance: ten cases pass. Full frame progression for all five appearances
  and exact empty-pixel equality after expiry are checked without another snapshot; reduced
  motion holds the final pose byte-identically. Two cross-project skips intentionally select
  desktop full playback and narrow reduced playback. All eight existing graphics cases pass.
- Ten artist-workbench cases and both DPR reference comparisons pass. All twenty death frames
  are nonempty, binary-alpha, distinct within each family and subsets of their original palettes.
  The 165 previous rectangles, 3,339,264 RGBA bytes, and their frame/animation/alias/composition
  contracts compare unchanged. Source is 1120×2128 with 185 assigned frames; runtime is
  2040×704 with 181 frames and 5,744,640 RGBA bytes. The source-only PNG extent cap is explicitly
  2128; compressed/decoded 16MiB and runtime 2048px/16MiB limits remain intact.
- The legacy DPR2 pulse reference changes one 19-pixel edge row (49 RGB bytes, zero alpha) when
  texture height grows from 584 to 704, due to nearest-sampling UV rounding at 1.125 scale.
  Recreating the old extent reproduces the old hashes exactly; only three current DPR2 hashes
  are updated. Historical and DPR1 hashes, parity assertions and source artwork remain intact.
- Exact-image runtime acceptance passes: four Compose duel/auth/CPU cases (37.1s), disabled
  diagnostics (11.2s), and two Kubernetes duel/auth cases (23.0s). Health/readiness, restricted
  runtime, configured probes/resources and zero restarts are verified; owned stacks/namespaces
  are removed. Strict Kubernetes schemas also pass.
- Fresh cargo-deny/npm audits pass at HIGH/CRITICAL thresholds; existing tooling advisories
  remain 20 moderate web and 12 moderate E2E entries. Pinned Trivy 0.74 reports zero fixable
  HIGH/CRITICAL vulnerabilities in the exact accepted image.
- Rust implementation/contracts are unchanged from v1.1.0; only package versions change.
  Reuse the recorded macOS simulation/coverage acceptance. Fresh
  [Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41206812465)
  pass for the new versions. Final reference/doc changes preserve production inputs.

### Operation

Localhost:8085 runs exact accepted image
`sha256:ecef3dd99eae78e63b8e2b922fb52adb92ee49ad8c0553154e81f3c9b5209b10`,
labelled 1.1.1/source 65bcea96. Local CPU diagnostics remain enabled; base Compose/Kubernetes
still default to disabled. Promotion verifies identity, runtime restrictions, health/readiness,
CSP/cache policy and served bytes. Immutable v1.1.0 is retained for rollback. Refresh old tabs
and create a new match after restart. Human feedback remains in [BACKLOG.md](../../BACKLOG.md).

## v1.1.0 — 2026-10-31

The arena now uses Three.js and React Three Fiber: a miniature textured courtyard with shallow
chamfered walls, directional lighting, grounded shadows and a restrained warm colour/dither
finish. Detailed billboard sprites retain all four uniforms and existing animation. The fixed
orthographic camera preserves keyboard directions. Rust simulation, controls, contracts and
recipient visibility rules are unchanged. Future character models remain a separate proposal.

The artist sheet adds ten named 48×48 material maps and preserves every existing sprite pixel.
Use the [artist workflow](../../assets/art/README.md) and the downloadable
[artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.1.0/grenadiers-artist-v1.1.0.zip),
including canonical inputs, guides, labelled inventory, optional slices and a 3D preview.

### Verification

Accepted production source: `4a9259ec7057ff9ca8595f0fa8423f9e5a124634`,
[PR #290](https://github.com/liveweird/grenadiers/pull/290). Independent renderer and presentation
reviews are clear, including GPU normals/lighting, resource ownership, concealed-history resets,
interpolation, painted shadow anchors and a grounded cloud shadow throughout bobbing.

- Web coverage: 7,268 tests pass in 264.29 seconds; statements 91.70%, branches 88.49%, functions
  91.16%, lines 94.83%. Existing floors remain unchanged. Production bundle is 392,505/400,000
  gzip JS bytes and 15,430/20,000 CSS bytes. The actual emitted module graph excludes the Pixi
  reference renderer and authoring factories.
- Native graphics: eight Three cases, ten source-atlas cases and both DPR1/DPR2 reference
  comparisons pass. Checks cover all three boards, four uniforms, item/entity variants,
  narrow/reduced-motion views, final framebuffer colour calibration, removal/concealment resets,
  suspend/resume, context recovery and teardown. No page, shader or CSP errors are observed.
- Atlas extension: all 155 original assigned rectangles, 3,247,104 RGBA bytes, compare unchanged.
  The source is 1120×1904 with 165 frames; the runtime pack is 2040×584 with 161 frames and
  4,765,440 RGBA bytes, within unchanged texture limits. The artist export and guides validate.
- E2E static checks pass, including 100 scenario mappings and camera-projection checks. Four
  Compose duel/auth/CPU cases and disabled diagnostics pass. All 21 general Kubernetes cases
  are accepted: 18 passed initially; the three affected geometry/cloud/turning cases pass after
  correcting the flat-board assertion and the macOS graphics backend. Runtime probes,
  restrictions and zero restarts are checked; owned namespaces are removed.
- Five focused item/privacy suites pass with unchanged deadlines: Oil Barrel, Cloak, Kick,
  Wall Pass and Bracelet. Projection-aware compositor sampling preserves concealment and
  reconnect assertions; graded colour-family checks retain Oil state evidence. macOS and CI
  explicitly align ANGLE WebGL/compositing and default to one shared-backend worker.
- Fresh dependency policy/audits pass at the HIGH/CRITICAL threshold; existing upstream tooling
  advisories remain 20 moderate web and 12 moderate E2E entries. Pinned Trivy 0.74 finds zero
  fixable HIGH/CRITICAL vulnerabilities in the exact accepted image.
- Rust implementation, contracts and deterministic fixtures are unchanged from v1.0.6; only local
  package-version metadata changes. Reuse that release's full macOS debug/release/coverage
  acceptance rather than repeat the simulation matrix. The image compiles the Linux release
  target; [fresh Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41196323616)
  pass for the new package versions. Final docs/tests preserve production inputs.

### Operation

Localhost:8085 runs the exact accepted image
`sha256:832ab2e4c96db7f6498a5a49a72645d48b585c451edf1944245bdf6986133b43`,
labelled 1.1.0/source 4a9259ec. The local operator keeps CPU diagnostics enabled at
`http://localhost:8085/?cpuDebug=1`; base Compose and Kubernetes default to disabled. Promotion
checks image identity, runtime restrictions, health/readiness, CSP/cache policy and served bytes.
Immutable v1.0.6 remains available for rollback. Restart ends in-memory matches; refresh and
create a new match. Human visual/difficulty feedback and proposals live in [BACKLOG.md](../../BACKLOG.md).

## v1.0.6 — 2026-10-31

Fix CPUs repeatedly stopping at an attained farming/combat approach when no safe bomb can be
certified. Default Teleporter uncertainty can suppress planting while another bomb, flame,
burning Oil or exposed Teleporter exists. CPU25 prefers a certified pulse whose entire final
position envelope excludes staying put, defers the failed subject for two seconds, and discards
the stale approach route before replanning. It still certifies clear-scene plants; trapped CPUs
may still have to wait. Bomb/contact safety checks are unchanged.

Archived CPU24 retains its original selection, scoring and entry points. New rules9 matches use
CPU25; private replay8 accepts matching CPU21–25 and replay9 CPU23–25 with navigation2. Public
gameplay17, web/API contracts, arena rules and artwork are unchanged.

### Verification

Accepted source: `c4d7c49515fde1e327097ed3326314dbe4242895`,
[PR #289](https://github.com/liveweird/grenadiers/pull/289). Independent production and queued
regression reviews are clear. Focused tests cover lairs on/off, clear-scene certified planting,
blocked Teleporter planting, guaranteed-progress bounds, physically advanced follow-up
retargeting, stale-route removal and preserved CPU24/replay identities.

- Fresh debug/release suites pass: 1,582 core tests, 587 server tests and doctests. Repository,
  tooling, formatting, Clippy and build pass. Independent Rust coverage is 48,647/52,687 lines
  (92.33%), above the unchanged 90% floor; the full local helper takes 270.462 seconds.
- The deterministic 1,200-update queued-controller/physics fixture uses all items and three CPUs
  in both lair configurations. CPU24 reproduces same-target/same-position farming pauses of
  213/186 updates; CPU25's longest runs are 11/24 updates, below the asserted 60-update limit.
  It also asserts attained farming, movement plans, Stalled intent, zero planner failures and
  planning work within the existing allowance. This is fixed-scene regression evidence, not a
  general difficulty rating.
- Exact-image native diagnostics pass two enabled cases in 11.6 seconds and one disabled case
  in 5.5 seconds, including EN/PL, accessibility and keyboard controls. Four general Compose
  cases pass in 24.2 seconds; pickup/progress recovery passes in 11.5 seconds. Two Kubernetes cases pass
  in 5.4 seconds.
  Runtime restrictions, probes and zero restarts are checked; owned temporary resources are removed.
- Fresh dependency policy/audits pass at HIGH/CRITICAL thresholds. Existing upstream tooling
  advisories remain 20 moderate web and 12 moderate E2E entries. Pinned Trivy 0.74 finds zero
  fixable HIGH/CRITICAL image vulnerabilities.
- Unchanged web/API/art/E2E-static/Kubernetes inputs reuse the
  [v1.0.5 acceptance](releases.md#v105--2026-10-31): 7,208 web tests and all unchanged
  coverage/bundle thresholds. Extracted candidate HTML/JS/CSS/PNG bytes equal that accepted
  build. Only independently reviewed historical ExpandedV5 exhaustive mask executions are reused:
  all 4,194,304 per debug/release/coverage profile from source 336503ed; fresh Rust coverage
  merges no old counters.
- [Unique Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41190408307) pass for the accepted source. Final release documentation preserves all production inputs.

### Operation

Localhost:8085 runs accepted image
`sha256:2d4d4a9d34a969a3b72361dfccd3ade9aa35c67c6948b8abe91224adb8216803`, labelled 1.0.6/source c4d7c495.
The local runtime keeps diagnostics enabled at `http://localhost:8085/?cpuDebug=1`; base Compose
and Kubernetes still default to disabled. Promotion verifies exact image, runtime restrictions,
health/readiness, CSP/cache policy and served assets. Immutable v1.0.5 remains available for
rollback. Restart ends in-memory matches: refresh and create a new match to try CPU25.
See [CPU intent](../game/cpu-intent.md) for behavior. The artist bundle remains v1.0.2;
remaining playtesting work lives in [BACKLOG.md](../../BACKLOG.md).

## v1.0.5 — 2026-10-31

CPU24 retains an exact pickup, block or opponent independently of its route. Two-second
commitment and utility hysteresis reduce unnecessary switching; real-position progress,
bounded retry and explicit phases handle approach, planting, escape, waiting and re-engagement.
Safety can interrupt immediately while withdrawing execution authority. Farming/combat bombs
must threaten the selected subject; collection cannot trigger unrelated opportunistic bombing.
Archived CPU21–23 and matching replay identities remain supported; gameplay17 is unchanged.

An opt-in EN/PL panel shows each visible CPU's objective, target, approach, phase, latest switch
reason and observation tick. The operator-enabled endpoint authenticates connected human room
participants, uses fresh recipient projection and bounded independent query capacity, and never
exposes seeds, routes, certificates or private world identities. Hidden subjects and bomb-blind
views return unavailable rows. Normal URLs make no diagnostic requests.

### Verification

Accepted source: `833493d716bc380166b1eecfc0a34f262363c26b`, [PR #288](https://github.com/liveweird/grenadiers/pull/288).
Independent CPU and API/privacy/frontend reviews are clear, including final wire-format and
native accessibility fixes. Actual serialized diagnostic rows validate against both schemas.
The HTTP lobby-to-running regression explicitly pumps the queued planner and asserts available
intent without relying on background-thread scheduling.

- Fresh debug/release suites pass: 1,578 core tests, 586 server tests and doctests. Formatting,
  Clippy, build and repository/tooling gates pass. Independent Rust coverage is 48,419/52,463
  lines (92.29%), above the unchanged 90% floor; the full local helper takes 239.127 seconds.
- Full web gates pass at source `be5ecfbac3a955df0afad405e4df24129b41505e`: 7,208 tests
  across 129 files; lines 93.81%, branches 88.09%, statements 90.43%, functions 89.91%,
  retaining all thresholds. API/realtime/gameplay drift, ESLint, dead-code, E2E-static and build
  checks pass. Compressed JavaScript is 366,775/400,000 bytes; CSS 15,430/20,000 bytes.
- Native enabled diagnostics pass two cases in 11.8 seconds: zero requests without URL opt-in,
  lobby-to-running availability, both locales, keyboard movement and accessibility. The disabled
  case passes in 5.6 seconds without polling after the operator-disabled response. Four general
  Compose cases pass, plus pickup/progress recovery and two Kubernetes cases. Exact image,
  resource restrictions, probes and zero restarts are checked; temporary resources are removed.
- Dependency policy/audits pass at HIGH/CRITICAL thresholds. Existing upstream tooling advisories
  report 20 moderate web and 12 moderate E2E entries. Pinned Trivy 0.74 finds zero fixable
  HIGH/CRITICAL image vulnerabilities; strict Kubernetes schema/invariant checks pass.
- The real 1,200-update queued CPU24 scene delivers 41 plans with zero failures and maximum
  planning work 57,695, asserting exact Block→Pickup→Block subject histories. Reviewed archived
  ExpandedV5 mask executions alone are reused: all 4,194,304 masks per debug/release/coverage
  profile from source 336503ed; fresh Rust coverage merges no old counters. Artwork is unchanged.
- [Unique Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41187629517) pass for the accepted source.
  The final web test additions and release documentation preserve all
  production Rust, API and emitted web bytes; the accepted image is promoted without rebuilding.

English and Polish screenshots are published with the
[v1.0.5 release](https://github.com/liveweird/grenadiers/releases/tag/v1.0.5).

### Operation

Localhost:8085 runs accepted image `sha256:8144712091446640def3f46df122660a2804fb181cae9fe660c0e49d9c8caf47`, labelled 1.0.5/source `833493d716bc380166b1eecfc0a34f262363c26b`.
The local runtime explicitly enables diagnostics; open `http://localhost:8085/?cpuDebug=1`,
create a CPU match and expand **CPU intent diagnostics**. Base Compose/Kubernetes configuration
keeps the flag disabled by default; see the [intent guide](../game/cpu-intent.md).
Promotion checks exact image, runtime restrictions, probes, CSP/cache policy and served
HTML/JS/CSS/PNG equality. Immutable v1.0.4 remains available for rollback. Restart ends
process-local matches; refresh and start a new match. The unchanged
[v1.0.2 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.0.2/grenadiers-art-v1.0.2.zip)
remains current. Human difficulty/playability feedback remains in [BACKLOG.md](../../BACKLOG.md).

## v1.0.4 — 2026-10-31

Improve arena opponents through stronger decisions. CPU23 retains reachable objectives and
bounded visit history, seeks productive crate-clearing positions, collects upgrades, and tries
distinct certified bomb inputs before returning safe navigation. Unrelated pickups no longer
suppress planting. Attained non-pickup objectives receive a visit-bounded retry cooldown;
changed or higher-value goals are reconsidered. Easy retains its slower cadence.

Mob navigation2 completes exits from dangerous origin tiles and re-centers interrupted retreats
exactly before resuming pursuit. Safe centers are reachable from either side; genuinely trapped
mobs wait for a safe route. Existing constructors and replay8 retain archived navigation1.
Private replay9 explicitly binds navigation2/CPU23 and reuses checked accepted-input execution.
Public gameplay17, rules9 geometry, generators, randomness and artwork remain unchanged.

### Verification

Accepted source: `b9cca82af5032de427649da0552a497db59249a4`,
[PR #287](https://github.com/liveweird/grenadiers/pull/287). Independent CPU, mob and integration
review is clear. Targeted debug/release regressions cover dangerous-origin oscillation, exact
center arrival, blocked exits, reachable/changed/attained goals, cooldown expiry, pickup
handoff, bounded optional tactics and archived/current replay identities.

- Fresh debug/release suites pass: 1,567 core tests, 569 server tests and doctests. Formatting,
  Clippy and build pass. Independent Rust coverage is 47,338/51,179 lines
  (92.49%), above the unchanged 90% floor. The full local gate finishes in
  330.377 seconds.
- The real queued generated scene tracks physical bomb IDs, crate loss, collection plus later
  movement, alive-player updates and settled-cell transitions. The matched 1,200-update sample
  produces 13 physical bombs, 12 destroyed blocks, a collected Golden Flame and 3,600 alive-CPU
  updates without a three-cycle A/B pattern. This is fixed-workload evidence, not a human rating.
- Native Docker pickup/progress acceptance passes in 14.9 seconds. Four Compose cases pass in
  28.2 seconds and two Kubernetes cases in 5.5 seconds, including default CPU first-bomb escape,
  all-features-disabled movement, rendered duel and authentication. An additional Easy Solo
  scenario passes in 13.2 seconds with completion, rematch, reconnect and leaving. Exact image, restricted
  runtime, resource limits, probes and zero restarts are checked; temporary resources are removed.
- Gameplay contract drift and dependency policy/audits pass at the HIGH/CRITICAL threshold.
  Unchanged upstream minimatch tooling advisories report 20 moderate web and 12 moderate E2E
  entries. Pinned Trivy 0.74 finds zero fixable HIGH/CRITICAL image vulnerabilities.
- Identical web/API/art inputs reuse v1.0.3's E2E-static and v1.0.2's 7,174-test frontend/atlas
  acceptance. The rebuilt image retains the same web asset bytes. Only the independently
  reviewed historical ExpandedV5 mask executions are reused: all 4,194,304 masks per
  debug/release/coverage profile from source 336503ed. Fresh Rust coverage merges no old counters.
- [Unique Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41155836024)
  pass for the accepted source. Subsequent release documentation preserves all Rust/build inputs.

### Operation

Localhost:8085 runs accepted image
`sha256:95b6e06b64a8c8923a54a9b821dac9cf33dcb2505b33c775d391e452eb640776`, labelled 1.0.4/source b9cca82a.
Promotion checks exact image, health/readiness, runtime limits, CSP/cache policy and served
HTML/JS/CSS/PNG equality. Immutable v1.0.3 remains available for rollback. Restart ends
process-local matches; refresh and create a new match to try Standard against three CPUs.
The unchanged [v1.0.2 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.0.2/grenadiers-art-v1.0.2.zip)
remains current. Further human tuning lives in [BACKLOG.md](../../BACKLOG.md).

## v1.0.3 — 2026-10-31

Fix CPUs stopping beside exposed pickups. The old planner discarded the movement lease from
its pickup approach, so the server correctly rejected it repeatedly. CPU22 retains a certified
single-update move, stops, and replans using the actual inventory/effects after collection.
The same handoff works with lairs disabled. Teleporter/Diarrhea collection and unsafe hazard
scenes remain outside this handoff; strict admission and contact checks remain in force.

Gameplay17/rules9 and private replay8 remain unchanged. CPU21 planning is retained; replay8
accepts matching CPU21 or CPU22 header/provenance and rejects mixed or unknown versions.

### Verification

Accepted source: `6681c4d4ffda70e361953cf3bc0352ab1cf4c4fe`,
[PR #286](https://github.com/liveweird/grenadiers/pull/286). Independent review is clear.

- Fresh debug/release suites pass: 1,553 core tests, 566 server tests and doctests. Formatting,
  Clippy and build pass. Independent Rust coverage is 46,393/50,229 lines (92.36%), retaining the
  90% floor. All local Rust gates finish in 337.738 seconds. Only the unchanged historical
  ExpandedV5 mask execution is reused after independent closure review: all 4,194,304 masks in
  each debug/release/coverage profile from source336503ed; no prior counters are merged.
- New real queued regression reproduces the CPU21 stall and proves CPU22 pickup collection and
  continued progress for every surviving CPU, with storm/lairs enabled and disabled. Timing
  tests cover delayed/stale admission, changed facts, Movement Speed, Slow Foot and Reverse
  Controls collection followed by Stop; hazard and lease guards remain checked.
- Native Docker browser regression fails on v1.0.2 and passes on this image in 15.6 seconds.
  Four Compose gameplay cases and two Kubernetes smoke cases pass with exact image identity,
  restricted runtime, resource limits, probes and zero restarts. Temporary stacks are removed.
- Repository/tooling/E2E static/strict Kubernetes-schema checks pass. Unchanged frontend/API/art
  inputs reuse v1.0.2's 7,174-test frontend acceptance and atlas checks; Docker reuses the exact
  web build. No historical coverage counters are merged into the new Rust report.
- Cargo dependency policy passes. npm audits pass the HIGH/CRITICAL threshold; upstream
  minimatch tooling advisories currently report 20 moderate web and 12 moderate E2E entries.
  Dependencies remain unchanged. Pinned Trivy 0.74 reports zero fixable HIGH/CRITICAL image findings.
- [Unique Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41151356039)
  pass for the accepted source; subsequent documentation changes preserve all Rust/build inputs.

### Operation

Stable source release. Localhost:8085 uses accepted image
`sha256:12e1fcd6f26f2cbe5e55700a68d8e66ffc940aa99912809f3eaf47046bbcbed9`, labelled 1.0.3 and
source6681c4d4. Health/readiness, served-asset byte equality and runtime restrictions are verified
on promotion. Retain immutable v1.0.2 for rollback. Refresh and start a new match; restart ends
process-local matches. Artwork is unchanged; the [v1.0.2 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v1.0.2/grenadiers-art-v1.0.2.zip)
remains current. Human playtesting and tuning live in [BACKLOG.md](../../BACKLOG.md).

## v1.0.2 — 2026-10-31

Correct arena playability and adopt the owner-supplied stone-wall sprite sheet. Each lair keeps
one living mob and starts its configurable replacement delay after that mob dies. Living players
occupy distinct cells without overlapping; movement reservations also cover turns, Bracelet
shoves and atomic Teleporter swaps. CPUs seek safe firing positions against all visible rivals,
including other CPUs, while retaining checked escapes and bounded planning work.

Landmines are additional single-use weapons: placement consumes the item without occupying an
ordinary bomb slot; detonation or destruction creates no refund or capacity increase. Ordinary,
Remote and automatic bombs retain their normal slots. The paired browser/server uses
gameplay17/rules9/CPU21, with private replay8 and closed historical contracts.

The new PNG retains the 1120×1792 layout, all 155 frame entries, pivots, anchors and animation
metadata. The redundant input file was removed. The current [artist source](../../assets/art/README.md)
and downloadable artist bundle accompany this release.

### Verification and source identity

Accepted image source: `45a44893de0bdcf665738d3a4908013fc4157355`; source PRs
[#284](https://github.com/liveweird/grenadiers/pull/284) and
[#285](https://github.com/liveweird/grenadiers/pull/285). Independent physics, CPU and
protocol/runtime reviews are clear. Final Rust inputs were verified at
`1900c1c6aefacaaeb440c1fba1a53f969930f588`; later changes preserve those inputs.

- Fresh debug/release workspace suites, formatting/Clippy/build and independent Rust coverage
  pass: 1,548 core tests, 564 server tests and 45,955/49,832 lines (92.22%) in 403.499 seconds.
  Only the unchanged historical ExpandedV5 exhaustive execution reuses the immediately preceding
  full gate at 336503ed: all 4,194,304 masks in each of debug, release and coverage, 32 partitions,
  four workers and no retries. No old coverage counters were merged.
- Frontend: 7,174 tests pass with 93.81% line and 88.17% branch coverage. Production build and
  compressed budgets pass: JavaScript 360,047/400,000 and CSS 15,226/20,000 bytes. The native
  Rust/browser corpus contains 30 authoritative frames, including 34 physical bombs with one
  occupied regular slot and recipient privacy. Typed maximum frames fit the bounded 196,608-byte
  outgoing cap; incoming commands remain 4,096 bytes.
- Targeted native lair, Bracelet, Teleporter and extra-mine scenarios pass, alongside seven-case
  Compose gameplay acceptance. The final art image passes four-case rendered/CPU Compose
  acceptance and two-case Kubernetes smoke with exact digest, restricted security, probes,
  resource limits, zero restarts and scoped cleanup. Atlas/workbench checks pass 10/10;
  two-density renderer parity/lifecycle checks pass 2/2. Legacy procedural fingerprints were
  corrected for the already accepted weather-layer ordering, independently of the PNG.
- Repository/tooling/E2E-static checks, strict Kubernetes schemas and dependency audits pass.
  Trivy 0.74 reports zero fixable HIGH/CRITICAL findings on the exact final image.
  [Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/41146992961)
  pass for the final Rust inputs.

### Distribution, operation and rollback

Stable source release. Localhost:8085 runs the exact accepted image
`sha256:84e147f388036905dba79f108bd0ee16069fa05a426ec8860d8ad993d02cb48b`, labelled 1.0.2 and source45a44893.
Promotion verifies health/readiness, runtime limits, CSP/cache policy and served HTML/JS/CSS/PNG
byte equality, including the adopted sprite sheet. Reload browser tabs and start a new match.
The immutable v1.0.1 image/override is retained for rollback. Restart or rollback ends process-local
matches and resume credentials. No hosted deployment, durable accounts/statistics or production
replay export is added. Human playtesting remains in [BACKLOG.md](../../BACKLOG.md).

## v1.0.1 — 2026-10-30

Fix CPUs dying beside their own bombs in multi-CPU openings. Concurrent bomb placements still
invalidate obsolete escape routes, but fresh rules 8 planning now uses retained origin proof and
certifies movement against the current bomb, storm and mob scene. Short conditional movement
keeps named stationary-bomb assumptions and the three-update limit. Escape guidance avoids
visible bombs; origin grace still ends at footprint exit.

Scheduled route stops retain contact checks, and contact invalidation preserves independently
valid recovery provenance while revoking old movement. Live CPU policy advances to 20; archived
CPU 19 accepted-input replay remains supported. Gameplay16/rules 8, artwork and web/API contracts
are unchanged.

### Verification and source identity

Accepted source: `114b9ce484a62f1dea8f2f0e0d76af26e4770315`; source PR
[#282](https://github.com/liveweird/grenadiers/pull/282) merged as
`b754d6c68639e0754d42b1a5b80f00c5385b8566` with an identical source tree.
Independent production review is clear.

- The regression reproduces all three CPUs planting at tick 205 and dying at tick 385 before
  the fix. Four default maps and all three boards with Easy/Standard now survive the first
  owned detonations and full flame interval. Controller, uncertainty and replay regressions pass.
- Fresh full Rust debug/release suites, formatting/Clippy/build and independent coverage pass:
  42,708/46,311 lines (92.22%) in 242.627 seconds.
- Only the unchanged historical ExpandedV5 exhaustive-mask execution reuses v0.3.116 acceptance
  after a reviewed dependency-closure comparison. Default gates remain exhaustive; no old
  coverage counts are combined. Web/art/API/dependency inputs retain v1.0.0 full-web acceptance.
- Fresh repository/tooling/E2E-static checks, Rust dependency policy and both npm audits pass.
  The exact-image Trivy 0.74 scan reports zero fixable HIGH/CRITICAL vulnerabilities.
- Compose native acceptance passes 5/5, including real three-CPU bomb/flame survival and Easy
  Solo completion/rematch/reconnect; Kubernetes native smoke passes 2/2 with exact digest,
  restricted security, resource limits, probes/readiness, zero restarts and scoped cleanup.

### Distribution, operation and rollback

Stable source release. The accepted local image is
`sha256:8c024abfc87055df9b4f62dd4aa54a35e17eda40a4a2d4e233895b3c13870bf1`, labelled 1.0.1
and source 114b9ce4. Localhost:8085 runs that exact image; fresh promotion verifies health/readiness,
served HTML/JS/CSS/PNG equality, CSP/cache policy and runtime limits.
The immutable v1.0.0 image/override is retained for rollback. Restart or rollback ends
process-local matches and resume credentials. No hosted deployment, durable accounts/statistics
or production replay export is added. Human difficulty and interaction tuning remain in
[BACKLOG.md](../../BACKLOG.md).

## v1.0.0 — 2026-10-30

The first stable Grenadiers milestone promotes the verified v0.3.118 game to 1.0.0:
two-to-four-player human/CPU matches, private Solo with Easy/Standard opponents, three board
sizes, all 22 requested items, animated pixel art, storm clouds and pursuing lair mobs.
Shared settings, per-seat reconnect, authoritative results and consensual rematches are included.

Product versions in the Cargo workspace, local dependency and lockfile advance together.
Gameplay16/rules8/CPU19 and all application behavior remain compatible with v0.3.118.
The [artist bundle](https://github.com/liveweird/grenadiers/releases/download/v0.3.117/grenadiers-lairs-art-v0.3.117.zip)
remains current; see the [artist handoff](../../assets/art/README.md).

### Verification and source identity

Accepted source: `c6a8c33fcf528be0245a874efb678d6027e94bbc`; source PR
[#279](https://github.com/liveweird/grenadiers/pull/279) merged as
`8fc7dde2ea38d92fcffd9193f320b276ff3c63f2` with an identical tree.

- Fresh release-helper/repository checks, locked Cargo metadata and workspace check pass.
  Source, contracts, web/art, toolchain, build profiles and third-party dependencies match
  v0.3.118 exactly; only internal product versions and release documentation change.
- Reuse [v0.3.118 acceptance](releases.md#v03118--2026-10-29): full Rust debug/release
  and independent coverage (92.09%), 7,139 web tests, item/CPU checks, native lifecycle suites,
  audits and distinct Linux checks. No coverage floor or assertion is lowered.
- The fresh labelled release image builds. Trivy 0.74.0 reports zero fixable HIGH/CRITICAL
  vulnerabilities. Fresh native Compose **2/2** (5.3 seconds) and Kubernetes **2/2** (5.4 seconds)
  pass authentication, keyboard turning, exact image, restricted security, resources/probes,
  readiness and zero restarts, with scoped cleanup.
- HTML, JavaScript, CSS and source-atlas bytes in the new image equal v0.3.118. Local promotion
  verifies those served bytes, CSP/cache policy, health/readiness and runtime limits again.

### Distribution, operation and rollback

Published as a stable source tag and GitHub release. Localhost:8085 runs the accepted image
`sha256:cd80414005c2daa806853c491dc94c8c452065dcae13b7dbe088581bb03e1751`, labelled version 1.0.0
and source c6a8c33f. The app remains non-root and read-only, with 256 MiB memory and PID 128 limits.
The immutable v0.3.118 override/image is retained for rollback. Reload browser tabs after upgrading.

Matches and resume credentials remain process-local: restart or rollback ends active matches.
Accounts, durable statistics and production replay export are outside this release. No hosted
production service or image registry is configured. Human difficulty, item-interaction and
browser/network validation continue in [BACKLOG.md](../../BACKLOG.md); the release does not close
those playtesting tasks.

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

The [lair guide](../game/lairs.md), [settings](../game/settings.md),
[reconnect policy](../game/reconnect-server.md) and [artist handoff](../../assets/art/README.md)
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
Human difficulty, interaction and network/browser playtesting remain in [BACKLOG.md](../../BACKLOG.md).

## v0.3.117 — 2026-10-29

[v0.3.117](https://github.com/liveweird/grenadiers/releases/tag/v0.3.117) adds the checked lair
foundation around rules8: gameplay16 configuration/publication, CPU19 finite mob-contact proofs,
atomic roster settings and private replay7. Live rooms continue to use gameplay15/rules7/CPU18;
the paired controls and rendering are delivered separately. See the [contract](../game/protocol-v16.md),
[CPU boundaries](../game/multiplayer-cpu.md#lair-profile-boundaries) and [replay guide](../game/multiplayer-replay.md).

CPU movement and bomb escape retain separate certificates. Guards cover current mobs, possible
births, pickup handoffs, hidden Teleporter collectors, pending pickup revelation and retained
movement. Invalid contact revokes the paired route and queues Stop. Historical profiles stay closed.
Replay7 reconstructs mobs from accepted inputs and checks complete private state, including timers,
retreat anchors and randomness; production recording/export remains disabled.

The artist sheet appends `terrain.lair` and four `mob.<direction>` poses. All 150 preceding
entries retain their pixels and metadata. The source is 1120×1792 with 155 entries, 151 drawings
and four reservations. The runtime texture remains 2040×528 / 4,308,480 RGBA bytes. The
[artist bundle](https://github.com/liveweird/grenadiers/releases/download/v0.3.117/grenadiers-lairs-art-v0.3.117.zip)
contains the editable source, manifest, labelled guides, previews and prompts. Bundle SHA-256:
`dc204420c01b05a98906686f19e923b2364b330a838068b3f3009d46b3df95c1`.

### Verification and source identity

Accepted source: `927049c1f6e403f3308ffd98d300fc302fd511b3`, tree
`27adfe0e0bbcab7166d57a7d7de7a2a0a01c2960`; source PR
[#275](https://github.com/liveweird/grenadiers/pull/275) merged as
`0e6f09cb0302df95c1f455d430a474c32e7bad5b`.

- Fresh workspace debug/release, fmt, Clippy with warnings denied, build and independent Rust
  coverage pass in 384.909 seconds: 42,445/46,093 lines, **92.09%** against the unchanged 90% floor.
  Retained LCOV SHA-256: `bec23b1b7da7c376168dad2bfff5155be8e5892e957576e95795a5d198c944c8`.
- Only the unchanged ExpandedV5 exhaustive-mask test reuses v0.3.116 acceptance: all 4,194,304 masks
  in debug/release/coverage, after an exact legacy execution-dependency audit. Current coverage
  uses fresh profiles; old counts are not merged. Default gates remain exhaustive.
- Web: **7,128 tests / 116 files**, 269.47 seconds; 92.72% lines, 88.42% branches, 89.84% statements
  and 89.68% functions. All unchanged floors pass. Final bundle: JS 348,176/400,000 bytes,
  CSS 15,226/20,000. The final palette-only cleanup preserves all prior pixels and alpha;
  its export and production build pass.
- Atlas: ten browser checks pass; final source exports 151 drawings within the same texture budget.
- Real queued CPU checks cover 72 configurations, six post-spawn board/difficulty cases with all
  items, own-bomb escape and stopped-route invalidation. Replay7 checks exact reconstruction,
  atomic interruption/departure, malformed journals and seed redaction.
- Native Compose and Kubernetes each pass **19 tests** in 3.8/3.9 minutes. Restricted deployment,
  probes/resources, exact image, zero restarts, IPv4 forwarding and scoped cleanup pass.
- Dependency audits and pinned Trivy0.74.0 report zero vulnerabilities at the required thresholds.
  Distinct [Linux debug/release checks](https://github.com/liveweird/grenadiers/actions/runs/40960573851)
  pass on the accepted source. Protocol, CPU and server/replay reviews are clear after fixes.

### Local deployment and rollback

The promoted image is
`sha256:ba7c4c1ff5f53e871d175b1a06be597f9d29bd7ddda494026108cf92865a39ad`, labelled version0.3.117
and accepted source927049c1. Localhost8085 passes exact-image/security/cgroup/health/readiness,
CSP/cache and served HTML/JS/CSS/PNG byte checks. The application remains non-root, read-only,
with dropped capabilities, 256MiB memory and PID128 limits. The retained v0.3.116 override/image
provides local rollback; reload browser tabs after paired upgrades.

<a id="v03116--2026-10-29"></a>
## v0.3.116 — 2026-10-29 — Dormant lair core

Rules8 adds a headless lair/mob simulation: two indestructible pillar overlays, configurable
spawn intervals and living caps, deterministic bounded pursuit and shared bomb avoidance.
NPC contact composes with Shield/Heart, Bracelet and teleport destinations. Mines, flames,
Oil and Stopwatch use the established physics; checked exhaustion rolls back the whole state.
Public layout/speed facts support later consumers. See the [core guide](../game/lairs.md).

This release keeps live gameplay15/rules7/CPU18. Lairs do not appear in the running application;
paired contracts, CPU19, server integration and rendering remain in the [backlog](../../BACKLOG.md).
Old rules constructors, public actor/state DTOs, generation streams and replay contracts remain
closed. The canonical artwork and [v0.3.115 artist bundle](https://github.com/liveweird/grenadiers/releases/download/v0.3.115/grenadiers-storm-art-v0.3.115.zip)
are unchanged.

Accepted source `568dbd643916379d48172c1e1a0c2d2219416c17`, tree
`682b7e33ba0276b1c10de8c8bb80c8b855436c50`, image
`sha256:421c64ff0f614f5d37c2a595dda50d8c3baf4c0211390fad877c7d48b41bc7da`.
The complete parallel Rust gate passes in 543.855s: **39,447/42,814 lines (92.1357%)**.
All 4,194,304 item masks ran separately in debug, release and fresh coverage across 32 partitions
with four workers, zero retries and 37 raw profiles. Official LCOV was preserved before releasing
the source/cache freeze. Navigation and damage reviews are clear after regressions for crossed
cell retreat, Oil reveal, forced-contact ordering, teleport geometry and seeded rollback.

Unchanged frontend acceptance reuses v0.3.115's full web/atlas checks; exact final-image
HTML/JS/CSS/PNG bytes match that accepted source. Compose and restricted Kubernetes each pass
19 native regression cases (3.7m/3.8m) on distinct ports, including all board/controller presets,
cloud gameplay, recovery, rematch and seat replacement. Security/probes/resources, zero restarts
and scoped cleanup pass. Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Distinct
Linux [run 40949621106](https://github.com/liveweird/grenadiers/actions/runs/40949621106) passes.

This source prerelease has an accepted local image. Promotion retains v0.3.115 for rollback;
restarting ends process-local matches. Production replay recording/export remains disabled.

<a id="v03115--2026-10-29"></a>
## v0.3.115 — 2026-10-29 — Storm cloud gameplay

The roaming storm cloud is live: it crosses terrain and bombs, and contact refreshes a
three-second, 50% slowdown shown in the player's effect bar. New rooms enable all items and
weather. Forming-lobby presets update both atomically; rules lock at first start and stay
locked through rematches. Pre-match departures and expired reservations free human seats while
retiring old capabilities. Replacements never inherit the departed creator's editing grant.

All supported boards and two-to-four-player human/CPU lineups now use gameplay15, rules7 and
CPU18. Large boards use generator3, smaller boards generator4; the wire map identity remains
`generated_v2`. Upgrade client/server together and reload old tabs. See the
[weather guide](../game/storm-cloud.md), [contract](../game/protocol-v15.md),
[English desktop](../screenshots/storm-cloud-desktop.png) and
[Polish reduced-motion mobile](../screenshots/storm-cloud-mobile-pl-reduced.png).

The [artist bundle](https://github.com/liveweird/grenadiers/releases/download/v0.3.115/grenadiers-storm-art-v0.3.115.zip)
adds the cloud while preserving every earlier frame: 150 entries, 146 drawings and four reserved
slots on 1120×1680. Runtime packing remains 2040×528 / 4,308,480 RGBA bytes.
[Prompts and provenance](../../assets/art/storm-cloud-prompts.md) document the extension.

Accepted source `1f372a4699a734db252529edd5a4a30d54a257fe`, tree
`9511900910e57dfb31be004a0051daf9762e91ee`, image
`sha256:c7ae99065f75d33ef7da004204ef881e9fedaceb97d78f8d221c236067040b76`.
Fresh Rust debug/release tests, fmt, warnings-denied Clippy, build and coverage pass in 309.453s:
38,643/42,019 lines (**91.9655%**). Independent coverage contains no earlier profiles. Only the
unchanged historical exhaustive mask test reuses v0.3.114's 4,194,304-mask acceptance in each
profile, following dependency review. Web acceptance passes 7,037 tests at 93.81% lines/89.14%
branches; exact final-image frontend bytes match that accepted build. Linux
[run 40941263408](https://github.com/liveweird/grenadiers/actions/runs/40941263408) covers the
distinct platform. All 24 seeded item/CPU scenarios and ten atlas cases pass. Compose and
restricted Kubernetes each pass 19 native cases in 3.8m on separate ports, with probes/resources,
zero restarts and scoped cleanup. Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings.
Independent contract, client and lobby-security reviews are clear.

This source prerelease has an accepted local image. Promotion retains v0.3.114 for rollback;
restarting or rolling back ends process-local matches. Production replay recording/export remains
disabled. Lairs are next in the [backlog](../../BACKLOG.md).

## v0.3.114 — 2026-10-29

[v0.3.114](https://github.com/liveweird/grenadiers/releases/tag/v0.3.114) pairs the dormant storm simulation with **gameplay15, CPU18 and private replay6**. Checked publication binds actual rules7, room lifecycle, physical epoch, configuration and recipient privacy. The shared Rust/browser corpus derives from real roster transitions and physical results. Hidden actors expose no storm timer or position; CPU observations use the same checked publication. The [environment contract](../game/protocol-v15.md), [CPU guide](../game/multiplayer-cpu.md) and [replay guide](../game/multiplayer-replay.md) describe these internal boundaries.

CPU18 retains ordinary manual-free clear-board navigation and pickup goals, while storm-aware planting and survival use bounded certificates. Optional planting cannot consume the whole movement budget; failed attempts retain no hypothetical bomb memory. Easy cadence remains intact. Real queued-driver checks cover 18 board/count/difficulty/environment cases, planting and own-detonation survival, cloud contact refresh/expiry and stale-work recovery. CPU18 accepted-input replay compares complete private state/RNG. Running-seat replacement baselines are verified across all six board/environment combinations.

Independent CPU, protocol, privacy, lifecycle, adapter and replay reviews are clear. Full local Rust debug/release/coverage passes with **92.09% lines** (38,403/41,703), including all 4,194,304 item masks in each profile, 32 partitions/four workers/no retries. Linux debug/release unit [run 40926054382](https://github.com/liveweird/grenadiers/actions/runs/40926054382) passes. **7,027 web tests** across 108 files pass with **93.81% lines and 89.04% branches**, plus contract drift/lint/dead-code/build checks. Resolved dependencies and artwork are unchanged; v0.3.113 dependency-security and v0.3.112 graphics acceptance are reused.

Tested source `b7f07bdef0c3e019e7ccf552a9faefff9dafa498`, tree `ec2919db8e824a932a4826ae1509d1bc4deb399f`, [PR #269](https://github.com/liveweird/grenadiers/pull/269). Exact image `sha256:d1b764d06f734344fce76723aea45d07ccbaf1c027a5cc6549f93e285c635a59` passes **18 browser scenarios each in Compose and Kubernetes**, 3.7m/3.8m on distinct IPv4 ports. Security/resources/probes, zero-restart postflight and cleanup pass. Pinned Trivy0.74.0 reports zero fixable HIGH/CRITICAL findings.

Live gameplay remains **v14**: this release does not enable the cloud in the application. Live server/client/rendering activation is tracked in the [backlog](../../BACKLOG.md). Production recording/export remains disabled. This is a source prerelease with an accepted local image; promotion retains v0.3.113 for rollback. The current artist bundle remains [v0.3.112](https://github.com/liveweird/grenadiers/releases/download/v0.3.112/grenadiers-animation-art-v0.3.112.zip).

## v0.3.113 — 2026-10-29

[v0.3.113](https://github.com/liveweird/grenadiers/releases/tag/v0.3.113) adds the **dormant headless storm-cloud simulation**. Rules7 explicitly chooses Disabled or Single. A seeded cloud crosses terrain at eight coordinate units per tick; swept contact applies a refreshable three-second half-speed modifier independent of Slow Foot. Forced movement, teleport endpoints, terminal behavior, overflow rollback and independent RNG streams are covered. Existing live gameplay14 is unchanged: the cloud is not enabled in the application. See the [simulation guide](../game/storm-cloud.md) and [backlog](../../BACKLOG.md).

Independent core review is clear. Linux debug/release unit [run 40917417210](https://github.com/liveweird/grenadiers/actions/runs/40917417210) also passes. The full local server gate passes debug, release, strict Clippy, formatting and coverage: **91.68% lines** (35,665/38,900), including all 4,194,304 item masks per profile with 32 partitions/four workers and no retries. Repository/tooling, rendered Kubernetes schemas and dependency policies pass; npm audits report zero vulnerabilities. Web, API and art inputs are unchanged, so the v0.3.112 web/graphics acceptance is reused.

Tested source `6329f62958e3c17799b424a2bdaa69dccd8dc5e7`, tree `cdfab1d0de392d24c77d02fd596b8d0065e7d710`, [PR #267](https://github.com/liveweird/grenadiers/pull/267). Exact image `sha256:285d94eb64924dccd92c82582106d35c721e91918ab5e43dfd7d5505a20a87e3` passes **18 browser scenarios each in Compose and Kubernetes**, 3.5m each on distinct IPv4 ports. Security/resources/probes and zero-restart checks pass; temporary stacks are removed. Pinned Trivy0.74.0 reports zero fixable HIGH/CRITICAL findings.

This is a source prerelease with an accepted local image. The current artist bundle remains [v0.3.112](https://github.com/liveweird/grenadiers/releases/download/v0.3.112/grenadiers-animation-art-v0.3.112.zip). Promotion retains v0.3.112 for rollback; restarting interrupts process-local matches.

## v0.3.112 — 2026-10-29

[v0.3.112](https://github.com/liveweird/grenadiers/releases/tag/v0.3.112) completes the **walking and bomb sprite animation** milestone. Every uniform/direction has a four-pose `0 → 2 → 1 → 3` gait. Timed bombs have neutral, compressed and expanded drawings driven by their authoritative fuse; pause, reduced motion, remote/contact and terminal behavior remain stable. All 111 original frame rectangles are unchanged. The 38 appended drawings keep the established anchors; actor poses retain their heads, central torsos and colors, with new passing legs and counter-swinging arms.

Download the [artist bundle](https://github.com/liveweird/grenadiers/releases/download/v0.3.112/grenadiers-animation-art-v0.3.112.zip): canonical PNG/JSON, guides, overview, animated preview, prompts and editing instructions. The source now has 149 entries (145 drawings/four reservations) on 1120×1680; the runtime pack remains one 2040×528 texture (4,308,480 bytes). [Generation prompts and processing provenance](../../assets/art/animation-prompts.md) document built-in imagegen and owner-authorized Pillow cleanup. Captures: [uniforms](../screenshots/animation-four-uniforms.png), [workbench](../screenshots/animation-atlas-desktop.png), [Polish reduced-motion mobile](../screenshots/animation-atlas-mobile-pl-reduced.png).

Independent runtime/catalog/art/lifecycle review is clear. **6,972 web tests** pass across 105 files with **93.81% line coverage**. Art-atlas10/10, WebGL DPR1/2 2/2 and the focused canonical-pose uniqueness regression pass. These cover all 64 walk combinations, neutral states, all bomb poses, original-slot preservation, binary alpha, palettes, gutters, anchors, texture budgets and cleanup. Static/contracts/lint/knip/build, dependency audits and repository checks pass. Backend/API/dependency resolutions are unchanged; prior exhaustive Rust/replay/CPU acceptance is reused. Distinct Linux debug/release unit [run 40792639784](https://github.com/liveweird/grenadiers/actions/runs/40792639784) also passes. Optional Linux/SwiftShader graphics checks were not repeated; pixel evidence uses the local Chromium driver.

Tested source `a2b0ba073dd33a0e6badfdf50a9c3ffaefe67970`, tree `ae4d38d4a18b760a3cd3b2db5a18e4db6152c9b6`, merged through [PR #265](https://github.com/liveweird/grenadiers/pull/265). Exact image `sha256:49a677e58f5923ffd210d1638425ff2083307f825e9e0a428e540954990f571a` passes **18 scenarios each in Compose and Kubernetes**, concurrently on distinct IPv4 ports, in 3.6m/3.7m. Security/resources/probes, owned forwarding and zero-restart postflight pass; temporary stacks were removed. Pinned Trivy0.74.0 reports zero fixable HIGH/CRITICAL findings.

Localhost runs that accepted image; health, readiness, security/cache headers and served HTML/JS/CSS/PNG match its extracted reference. Reload browser tabs after upgrade. v0.3.111 is retained for rollback. This is a source prerelease with an accepted local image, not a hosted production deployment. Storm-cloud and lair work continue through the [backlog](../../BACKLOG.md).

## v0.3.111 — 2026-10-28

[v0.3.111](https://github.com/liveweird/grenadiers/releases/tag/v0.3.111) adds **fuse-driven bomb pulsation**. Timed shells briefly enlarge and their sparks pulse faster as the authoritative fuse approaches explosion, including Short Fuse and Long Fuse. Paused, remote/contact, legacy, reduced-motion and non-running states retain a neutral shell. Shadows, pause marks, positions, collision and server timing remain independent. No new artwork or protocol/archive identity is introduced; additional walking and artist-drawn bomb poses remain in the [backlog](../../BACKLOG.md).

Independent runtime and diagnostic review is clear. All **6,963 web tests** pass across 103 files with **93.74% line coverage**; frontend/contract/static/scenario checks, production and renderer builds, tooling, strict Kubernetes schemas and dependency audits pass. Actual Chromium WebGL checks pass at DPR 1/2 over 49 states, including all timed blast appearances, remote/contact triggers, pause/resume, stale renders, movement, reduced motion and terminal rounds. Fixed procedural pixel fingerprints separately guard the atlas and independent historical renderer; canonical artist PNG/JSON are unchanged. These fingerprints are verified on the local browser/driver; the optional Linux/SwiftShader graphics suite was not executed.

Tested source `3544259a70b17192904d6ec7d6c7c53d7d244caf`, tree `e859ac462a0ffceaae8e6e7a528eb1f00420e3e8`, merged through [PR #263](https://github.com/liveweird/grenadiers/pull/263). Core/server/API sources and resolved dependencies are byte-for-byte unchanged from v0.3.110, so its exhaustive Rust/coverage, replay and CPU acceptance is reused; Cargo metadata advances to this release. The distinct Linux debug/release unit [run 40784321407](https://github.com/liveweird/grenadiers/actions/runs/40784321407) also passes.

Exact image `sha256:dc2d019f30576885b1b4b18bd95e67500c218f72e79cb79e73175ac3819545bf` passes **18 browser scenarios each in Compose and Kubernetes**, run concurrently on distinct IPv4 ports in approximately 3.5 minutes per stack. Kubernetes forward ownership, restricted security/resources/probes and zero-restart postflight pass. Temporary stacks were removed. Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. This source prerelease publishes no production registry image.

Localhost now runs the exact accepted image; health, readiness, CSP/cache headers and served HTML/JS/CSS/PNG bytes match its extracted reference. Keep v0.3.110 for rollback and reload existing tabs after upgrading. Classic-duel captures from that image: [desktop](../screenshots/bomb-pulse-desktop.png), [320px reduced motion](../screenshots/bomb-pulse-mobile-reduced.png). The [rendering guide](../game/art-atlas.md#bomb-animation) defines current animation behavior.

## v0.3.110 — 2026-10-28

[v0.3.110](https://github.com/liveweird/grenadiers/releases/tag/v0.3.110) activates **11×9, 13×11 and 15×13 custom matches** for two to four human/CPU players. Select **Board size** when creating a custom match; it stays fixed through rounds, reconnect and rematch. **Create room** and **Play solo against CPU** retain the 15×13 default. All 22 optional items remain enabled by default. The [board guide](../game/board-sizes.md) includes desktop and Polish 320px screenshots.

Smaller rooms use roster rules6, generator4 and policy17; existing two-player 15×13 rooms retain rules5/generator2/policy16, and larger 15×13 rooms retain rules6/generator3/policy17. CPU forecasts, hazards, routing, Power/Punch geometry and work accounting use actual checked dimensions. Complete private state and historical event coordinates validate before recipient filtering. Shared private replay format5 strictly binds small boards/generator4; format4 remains 15×13/generator3. Literal generator/RNG/derivation identities prevent archive drift. Production recording/export remains disabled.

Independent CPU, publication, runtime, replay and frontend/native-test reviews are clear after their findings were fixed. Focused CPU202, v14 publication44, replay13 and runtime/profile/cleanup checks pass. Full Rust acceptance on source `450d8b4035110e6e2998e6b03a5b7f077caa8691`, tree `4055ca9a89e6065ad21a9a346c692a26ed0a2209`, passes in **626.397s**: 35,389/38,622 lines (**91.6291%**), every 4,194,304-mask combination separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved. All **6,950 web tests** pass with **93.72% line coverage**. All **64 isolated CPU reference cases** exactly match v0.3.109 and verify replay; maximum group planning p99 is 4.70ms and maximum decision is 5.53ms. Distinct Linux [run 40775956097](https://github.com/liveweird/grenadiers/actions/runs/40775956097), formatting, strict Clippy, source caps, generated-contract/static/scenario checks, tooling and dependency audits pass.

Exact image `sha256:928b39ddc9e2eafe38020436e797b3be0953f8b09eaca6dba29312aed329b3eb` passes **18 browser scenarios each in Compose and Kubernetes**, covering nine human board/count combinations, 18 CPU board/count/difficulty combinations with every CPU acting, default paths, higher-seat movement, private recovery and small-board rematch. CPU-item collection and pinned Trivy 0.74.0 pass with zero HIGH/CRITICAL findings. Kubernetes acceptance uses a verified IPv4-owned forward, restricted security/resources/probes and zero-restart postflight. Shared-port and incomplete-helper runs were excluded and retained separately. Setup and test synchronization errors were corrected without weakening assertions or retrying away gameplay failures; temporary stacks were removed.

This source prerelease includes a verified local image. Localhost runs that image with v0.3.109 retained for rollback. Animation and arena-environment work remain in the [backlog](../../BACKLOG.md).

## v0.3.109 — 2026-10-28

[v0.3.109](https://github.com/liveweird/grenadiers/releases/tag/v0.3.109) adds [checked board geometry](../game/board-sizes.md) for 11×9, 13×11 and 15×13 arenas. The legacy 195-entry constructor remains 15×13. Movement footprints, public terrain, Power blasts and Punch wrapping follow actual arena dimensions; historical rules reject smaller worlds. This is a headless foundation: live room admission remains 15×13 while the remaining CPU/publication/runtime/browser changes are paired.

Generator 4 preserves raw seeds and produces reflected terrain/items with protected corner exits, rotated assignments, whole-orbit Oil placement within eight sites, and a dimension-correct empty fallback. Generators 1–3 retain their original outputs and geometry rejection. New tests pin all three preset outputs, exact repeatability/raw-seed records, three-player corner omission and four-player rotation. Independent geometry review is clear; focused arena, movement, blast, Punch, generation and historical-trace checks pass.

Full Rust acceptance on source `a0908bc3d1fa13dc5f95c3e516bf9ea621a65ddb`, tree `67f058b196be4786315781fe0573b6c46514a357`, passes in **669.571s**: 35,129/38,359 lines (**91.5796%**), every 4,194,304-mask combination separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved. Distinct Linux [run 40769703386](https://github.com/liveweird/grenadiers/actions/runs/40769703386) passes. All **64 isolated CPU reference cases** exactly match v0.3.106, including planning work, RNG, trajectories and outcomes; every accepted-input replay verifies. Maximum group planning p99 is 4.64ms; maximum observed decision is 5.93ms. Formatting, strict Clippy, source caps and repository checks pass. Unchanged web, contract and dependency inputs reuse v0.3.108's accepted full gates.

Exact image `sha256:be4ef7dfc73ac3a4cb9428b27dfa257055bdc1a7bd1c4c636126341e2934c9d7` passes Compose and Kubernetes **14 browser cases each**, CPU-item collection and a pinned Trivy 0.74.0 scan with zero HIGH/CRITICAL findings. Runtime image identity, UID/capabilities/read-only root, resource limits, probes, origins, health/readiness and served assets pass. Isolated stacks were removed.

This source prerelease includes a verified local image. Localhost runs that image; v0.3.108 is retained for rollback. The [backlog](../../BACKLOG.md) owns remaining board activation and environment work.

## v0.3.108 — 2026-10-28

[v0.3.108](https://github.com/liveweird/grenadiers/releases/tag/v0.3.108) activates **two-to-four-player matches** with human and CPU-controlled seats. Custom lobbies require at least one human, configure Easy/Standard CPUs and show four distinct uniforms, player identities and scores. Human seats join by invitation and retain private reconnect capabilities. All 22 optional items start enabled; boards remain 15×13.

Gameplay v14 is now the sole live wire. Count-two rooms retain rules5/generator2/policy16 through checked v14 encoding; count-three/four rooms use rules6/generator3 and isolated policy17 controllers. Both owner variants share one capacity-limited registry, admission rollback, sequence fencing, replacement priority and retained closing delivery. Dormant v13 registry entrypoints are test-only. Diagnostics move to v2 with maxPlayers4, separate from their unchanged 16-connection limit; the exact closed v1 archive is retained. Reload older browser clients after upgrade. Production replay recording/export stays disabled.

Independent owner, encoder, registry, socket, diagnostic and native-test reviews are clear. Full Rust acceptance passes on source `b5c8dd55941513ec54f85521671ec5ae91900fab`, tree `cab76c4a47b7e5b2d3afe80bad5ae3b16a24b5c7`: 628.846s, 34,859/38,087 lines (**91.5247%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved. Distinct Linux [run 40761748865](https://github.com/liveweird/grenadiers/actions/runs/40761748865) passes. All **6,918 web tests** pass, with **93.51% line coverage**; formatting, strict Clippy, source caps, generated-contract checks, browser static/scenario checks, dependency audits and tooling pass. Unchanged historical CPU reference inputs reuse the 64 exact-trace/replay cases from v0.3.106.

Exact image `sha256:d0b513efdfa24f15555af8379f89d006e82720674fdfb874e211d0832e5a5c77` passes pinned Trivy 0.74.0 with zero HIGH/CRITICAL findings. Compose and Kubernetes each pass **14 browser cases**, including higher-seat movement, private reconnect, mixed CPU configurations, four-seat scoring and unanimous rematch into a second round. A separate native run verifies that all three live Standard CPUs move and plant owned bombs from authoritative snapshots (one/4.5s). Additional image acceptance passes complete two-player match/rematch (one/57.3s), reconnect (two/32.9s), headed visibility hardening (one/10.5s) and CPU-item collection (one/16.5s). Runtime security/resources/probes, health/readiness, CSP/cache and served asset/source-PNG equality pass. Fixture setup now verifies actual origins and session HTTP201 before browser execution; an isolated misconfiguration was proven to return HTTP403. Temporary stacks were removed.

A final browser-only change directly asserts all four protected spawn cells; its focused native case passes in 15.1s. Runtime build inputs remain identical to the accepted source above. The artist bundle contains the current canonical four-uniform PNG/JSON. This source prerelease includes a verified local image; localhost runs it with v0.3.107 retained for rollback. Variable board sizes are next in the [backlog](../../BACKLOG.md).

## v0.3.107 — 2026-10-28

[v0.3.107](https://github.com/liveweird/grenadiers/releases/tag/v0.3.107) adds the test-only [multiplayer replay format](../game/multiplayer-replay.md). Format 4 records checked configured-room identity, policy-17 provenance, accepted CPU/human inputs, lifecycle transitions and independently derived per-round simulation randomness. Playback drives the actual roster runtime without invoking planners and compares exact state, reports, revisions and RNG progress. Production recording/export remains disabled; historical formats 1–3 remain closed and unchanged.

Eight focused regressions cover actual Teleporter randomness, reservation expiry, rematch, physical-round markers, flush/interruption and incomplete construction. Independent review identified and verified corrections to rematch cleanup and lifecycle-version binding. Strict Clippy, formatting, source caps and repository checks pass. Full Rust acceptance on source `bec260484d04d067e7c2214c1ca437e9c827b532`, tree `c755380082a5e06216e845cd76fdcc1531297c6d` passes in 524.723s: 32,963/35,963 lines (**91.6581%**), every 4,194,304-mask combination separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved. Distinct Linux [run 40756679033](https://github.com/liveweird/grenadiers/actions/runs/40756679033) passes. Unchanged production CPU/browser/art/dependency inputs reuse v0.3.106 acceptance, including its 64 exact reference/replay cases.

Exact image `sha256:4fe38c9f403040ca43492ecc449f8e60ce09f253e25165deef427f665d129a89` passes pinned Trivy 0.74.0 with zero HIGH/CRITICAL findings; Compose 10 browser cases/50.4s; Kubernetes 10/50.3s; CPU-item collection one/16.5s. Both runtime security/resource/probe checks pass. A digest-form Kubernetes helper assertion was incompatible with the runtime's equivalent imageID spelling; the tag-form rerun verified the same immutable image. Initial CPU-item setup omitted the isolated port's allowed origin, producing a proven session HTTP 403; explicit origin configuration restored HTTP 201 and the complete passing test. Temporary stacks were removed.

This source prerelease includes a verified local image. Localhost runs that image, with v0.3.106 retained for rollback. Live gameplay remains v13/two-player pending paired multiplayer activation in the [backlog](../../BACKLOG.md).

## v0.3.106 — 2026-10-28

[v0.3.106](https://github.com/liveweird/grenadiers/releases/tag/v0.3.106) adds the dormant [multiplayer CPU foundation](../game/multiplayer-cpu.md). Rules 6 accepts two to four canonical actors and 32 bombs through private v14 recipient observations. Historical rules retain two actors and the 16-bomb observation, forecast and hypothetical-placement limits.

Explicit Utility v17 Easy/Standard profiles reuse bounded utility/planning machinery. Individual controllers isolate world identity, memory, RNG, pending work, routes, stops and cadence. Cross-seat results cannot commit plans or random draws, including player four. The shared planner remains two workers with a 32-job queue and the existing work allowance. Live two-player rooms continue using policy v16.

Multiplayer Punch and Bracelet proofs consider every relevant rival, hidden uncertainty, victim-sweep blockers and counter-shoves. Additional guards are restricted to rules 6. Recipient adapter regressions compare Cloak, Invisible Bombs, Stopwatch, effects and individual Teleporter history with human projections. An initial controller filter incorrectly rejected archived policy 15; it now accepts historical rules-5 policies through v16 while v17 remains restricted to rules 6. Live owner profile selection remains unchanged. Oversized test/debug files were split by responsibility without changing assertions or exposed debug fields.

Independent CPU and correction reviews are clear. Core CPU family 194, server CPU family 44 and additional player-four, capacity, rival, privacy, admission and archived-trace regressions pass. All 64 isolated Standard/Easy opening/full-round reference cases exactly match v0.3.103 and verify accepted-input replay. Full Rust acceptance passes on source `2cdbde68d0ff09c313ceaa2a6feb084a1f753c57`, tree `37f51c8878dbbeed3de37c7b56d7c64923ff97fd`: 607.589s, 32,488/35,462 lines (**91.6136%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved. Distinct Linux [run 40753242368](https://github.com/liveweird/grenadiers/actions/runs/40753242368) passes. Formatting, Clippy, source caps and repository checks pass. Unchanged browser/art/dependencies reuse their existing full acceptance.

Exact image `sha256:327cb6b02d931a535b11ca9278df1143feb26e0fea776138e304aa1163760763` passes pinned Trivy 0.74.0 with zero HIGH/CRITICAL findings; Compose 10 browser cases/51.6s; Kubernetes 10/52.5s; CPU-item collection one/15.5s. Assets, CSP/cache, runtime security/resources/probes, health/readiness and Polish 320px pass. Temporary stacks were removed.

This source prerelease includes a verified local image. Localhost runs the accepted image; v0.3.105 is retained for rollback. Live multi-CPU ownership and paired v14 activation remain in the [backlog](../../BACKLOG.md); no live wire or replay migration is introduced.

## v0.3.105 — 2026-10-28

[v0.3.105](https://github.com/liveweird/grenadiers/releases/tag/v0.3.105) adds the dormant [roster runtime](../game/multiplayer-runtime.md) and lifecycle for checked two-to-four-seat matches. It provides personal admissions, creator-only settings, per-seat human/CPU input fences, rules 6/generation 3 construction and original-roster participation. Live sockets still use the two-player implementation.

Reservations remain truthfully occupied and disconnected before the first round. Creator authority persists through grace while settings mutations require a connected creator. All human frames and CPU facts share one certified publication before revision/report-flush commit. Ordinary snapshots batch three 60 Hz updates at 20 Hz. Physics precedes running departure/expiry settlement without advancing twice; idle departures precede countdown advancement. Closing cleanup preserves completed bodies and outcomes without accidentally starting rematches.

An independent OS-generated private simulation root derives per-epoch round seeds using pinned ChaCha20 derivation version 1. Root and derived seed debug output are redacted. Independent OpenSSL vectors, request-order tests and entropy-failure tests pass. No arena fixture or CPU stream controls simulation randomness.

Independent lifecycle/runtime/privacy reviews are clear after their fixes. Sixteen focused runtime cases and roster, input, creator-reservation and seed regressions pass. Complete Rust acceptance passes on source `11e2bc3ba772cb6d99c92fb743962f911c1f2af2`, tree `98bb8a440b50589b50d2a7ec5f49de6099065f2c`: 633.108s, 32,154/35,041 lines (**91.7611%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved. Distinct Linux [run 40750262726](https://github.com/liveweird/grenadiers/actions/runs/40750262726) passes. Formatting, Clippy, source caps and repository checks pass. Browser/art/dependencies and legacy CPU acceptance reuse v0.3.104/v0.3.103 evidence; their inputs are unchanged.

Exact image `sha256:9aab389265ca9a8cc0b0461e55843a553a0d91ed25b69ccd5cf776a893e7d14d` passes pinned Trivy 0.74.0 with zero HIGH/CRITICAL findings; Compose 10 browser cases/52.2s; Kubernetes 10/54.4s; CPU-item collection one/16.7s. Assets, CSP/cache, security/resource/probe settings, health/readiness and Polish 320px checks pass. Temporary stacks were removed.

This source prerelease includes a verified local image. Localhost runs the accepted image; v0.3.104 remains available for rollback. No live wire/replay migration is introduced. Live multiplayer, variable boards and later arena work remain in the [backlog](../../BACKLOG.md).

## v0.3.104 — 2026-10-28

[v0.3.104](https://github.com/liveweird/grenadiers/releases/tag/v0.3.104) introduces the dormant [gameplay v14 contract](../game/protocol-v14.md): checked two-/three-/four-seat human/CPU configuration, explicit human creator, canonical seat/scores arrays, closed board dimensions and generated browser types/validators. Live gameplay remains v13 until paired runtime/browser activation.

The v14 publication pipeline certifies complete private simulation facts before recipient filtering, covering concealed pickups and Oil, fuse/pause and grace histories, enabled items, forced movements, roster consensus and physical outcomes. Individual Teleporter markers prevent an unrelated hidden pair from leaking timing to another recipient. Mixed rooms can become abandoned after their last human leaves while retaining server-controlled CPU rows. All recipient frames must encode before a publication commits.

Independent contract, privacy and physical-proof reviews are clear after their fixes. All 37 focused v14 cases and three participant-marker regressions pass. Typed and independently computed recursive maximum frames fit the 98,304-byte limit: Snapshot 93,911 and Admitted 43,359. Generated type/validator drift, schema parity, formatting, warnings-denied Clippy, source caps and repository checks pass; archived contracts remain closed.

Complete Rust acceptance passes on source `c5040e70ad195441ba20c6fc47b087714d4b6a29`, tree `bdb80493318722dea95d34deaf6e5a107f74cb78`: 585.055s, 30,597/33,380 lines (**91.6627%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was retained before ending the source/target freeze. An initial sandbox run could not bind local test listeners; the authorized complete run above passes. Distinct Linux [run 40745468803](https://github.com/liveweird/grenadiers/actions/runs/40745468803) passes. Web acceptance passes 6,894 cases/99 files at 93.61% line coverage and builds within bundle limits. Legacy CPU/reference behavior reuses v0.3.103 acceptance because its observation and decision inputs are unchanged.

Exact image `sha256:231ea120182d41c9f68fbeff98021ef8d03c14b98b0c8f304aa5285111924dae` passes pinned Trivy 0.74.0 with zero HIGH/CRITICAL findings; Compose 10 browser cases/50.1s; Kubernetes 10/51.3s; CPU-item collection one/16.5s. Security/resource/probe settings, health/readiness, Polish 320px and served HTML/JS/CSS/PNG checks pass. Temporary stacks were removed.

This source prerelease includes a verified local image. Localhost runs the accepted image; v0.3.103 is retained for rollback. No live wire or replay migration is introduced. Remaining multiplayer activation and variable-board work stays in the [backlog](../../BACKLOG.md).

## v0.3.103 — 2026-10-28

[v0.3.103](https://github.com/liveweird/grenadiers/releases/tag/v0.3.103) completes all 22 optional-item mechanics in three-/four-player headless states. Punch resolves common-world claims before committing independent kicks; Bracelet resolves disjoint pusher/victim pairs with shared blockers and swept-contact arbitration. Teleporter chooses another living participant using an independent simulation stream with bounded, unbiased selection and transactional rollback. Historical rules 1–5 retain their duel semantics. See the [headless guide](../game/multiplayer-core.md).

Round construction retains the original roster and marks departed participants inactive. Atomic same-tick departure elimination bypasses protection, retains owned bombs and preserves completed physics; callers settle scoring first and append departures without advancing the clock twice. Archived two-seat projections explicitly reject unsupported forced-movement facts.

Independent interaction, legacy-compatibility, randomness and physical-boundary reviews are clear. Full Rust acceptance passes on source `4e404fb8d949b2d6819bf5d09df298b18274d56f`, tree `70698978c9b10201a6108b3ea35445672e816ed3`: 515.207s, 28,561/30,970 lines (**92.2215%**), all 4,194,304 item masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 35 raw profiles. Official LCOV was retained before ending the source/target freeze. Distinct Linux [run 40613820552](https://github.com/liveweird/grenadiers/actions/runs/40613820552) passes. All 64 Standard/Easy opening/full-round reference cases match v0.3.101 exactly and verify accepted-input replay; measured maximum planner work stays bounded. Unchanged browser/art/contracts/dependencies reuse their existing complete acceptance.

Exact image `sha256:a4ff956e4ca6c4180508dc7ea93b26825a31d18bd2a7d24d4029eefcf5a5667e` passes pinned Trivy 0.74.0 with zero HIGH/CRITICAL findings; Compose 10 browser cases/49.6s; Kubernetes 10/50.9s; CPU-item collection one/16.5s. Runtime security/resources, health/readiness/probes, Polish 320px controls and served HTML/JS/CSS/PNG identity pass. The first Kubernetes helper invocation rejected the digest-form image field despite a matching immutable image; using the existing tag passed the unchanged identity/security checks. Temporary acceptance stacks were removed.

This is a source prerelease with a verified local image. Live rooms, gameplay v13, CPU observations and replay remain two-player; live multiplayer and variable boards are separate backlog work. No wire or replay migration is introduced. Localhost runs the accepted image; v0.3.102 is retained for rollback. Outstanding work remains in the [backlog](../../BACKLOG.md).

## v0.3.102 — 2026-10-28

[v0.3.102](https://github.com/liveweird/grenadiers/releases/tag/v0.3.102) enables 19 shared optional-item mechanics for three-/four-player headless states. Higher-seat regressions exercise equipment and timed effects, Wall Pass traversal, weapon placement, Oil collision/ignition, damage, remote bombs/mines, contention and recipient privacy. Punch, Bracelet and Teleporter remain explicitly gated in this release. See the [headless guide](../game/multiplayer-core.md).

[Generator v3](../game/multiplayer-generation.md) adds horizontal/vertical terrain/item symmetry, four protected corner exits, actual-round three-player corner omission and bounded fallback retaining the requested roster. Twelve-round coverage gives each three-player seat every corner three times. Historical generator sampling moved into a private helper without changing function bodies, draw order or v1/v2 golden results.

Checked immutable room configuration describes canonical 2–4 human/CPU seats, requires a human creator and pairs difficulty exactly with CPU presence. The domain recognizes 11×9, 13×11 and 15×13, but current generation explicitly rejects smaller boards. Live rooms, gameplay v13, CPU observations and replay remain two-player; no multiplayer or board selector is exposed yet.

Independent item, configuration and generation reviews are clear. Generation tests pass 93 cases, including legacy goldens; full core tests and additional higher-seat behavior regressions pass. Formatting, warnings-denied Clippy, source caps and repository checks pass. Unchanged two-player CPU behavior reuses the v0.3.101 64-case reference/replay acceptance; unchanged web/art/contracts/dependencies reuse their existing complete acceptance. No browser code or artwork changed.

Complete Rust acceptance passes on source `a8484ae0f051bb3c4ff340bc930f3e474bb50bc3`, tree `19a8229c76cd2fb05d3c90ee9e88be0fe3d69f43`: 645.624s, 28,207/30,614 lines (**92.1376%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was preserved before ending the source/target freeze. Unique Linux [run 40605790929](https://github.com/liveweird/grenadiers/actions/runs/40605790929) passes.

Exact image `sha256:08f8f136cb85681db7e882032522a502a1f7ab7d69dcefb6ea7a297b34120ced` passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings; Compose 10 browser cases/53.5s; Kubernetes 10/54.8s; CPU-item collection one/16.6s. Runtime security/resources, health/readiness/probes, Polish 320px controls and exact served HTML/JS/CSS/PNG identity pass. Temporary acceptance stacks were removed.

This is a source prerelease with a verified local image. Localhost runs the accepted image; v0.3.101 is retained for rollback. Deploy paired server/browser normally; no wire or replay migration is introduced. Outstanding work remains in the [backlog](../../BACKLOG.md).

## v0.3.101 — 2026-10-28

[v0.3.101](https://github.com/liveweird/grenadiers/releases/tag/v0.3.101) adds checked two-/three-/four-participant headless simulation on the fixed 15×13 arena. Canonical actor storage, traces, action priority, placement contention, event audiences and survivor outcomes follow actual participants. Invalid input seats fail before mutation. The [headless guide](../game/multiplayer-core.md) defines the supported construction boundary.

Live rooms, CPU observations, archived gameplay v2–13 and replay identities remain two-player. Larger headless states require dormant rules v6 with optional items disabled. A supplied arena describes one round; three-player omitted-corner rotation belongs to the per-round generator. This release does not advertise playable three-/four-player rooms or configurable boards.

Independent simulation/compatibility review is clear after fixes to nonparticipant spawn/reconnect lookups and unsupported forced-movement facts. Legacy conversions and room/series/reconnect boundaries reject extra seats before mutation rather than truncating or panicking. Real Bracelet report regressions cover base/v7/v8 projection. Formatting, warnings-denied workspace Clippy, source caps and repository checks pass. The lifecycle transaction helper moved without behavioral changes to retain the source-size ceiling.

Complete Rust acceptance passes on source `310a52a1ee54ed098c5ec9e70d3aebe09150ad23`, tree `2c4190f4562d8c5f8a9895e85a29698270b00110`: 593.391s, 27,973/30,368 lines (**92.1134%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was copied before releasing the source/target freeze. Unique Linux [run 40603808867](https://github.com/liveweird/grenadiers/actions/runs/40603808867) passes.

All 64 CPU reference cases retain exact prior case metrics and accepted-input replay, including planner work and RNG consumption. Isolated Standard opening/full runs take 2.53s/13.07s; Easy 2.19s/11.26s. Observed costs increased modestly relative to the previous measurements while work limits and outcomes remain unchanged; these scripted comparisons do not establish human balance. Unchanged web/art/contracts/dependencies reuse the prior full web, atlas, static and dependency-audit acceptance; the exact image serves identical browser artifacts to v0.3.100.

Image `sha256:0d306c8159440c644be9e0dc3a79a6f5d6e023b529f41afda31b9e7dc6d37ece` passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings. Compose passes 10 browser cases/50.9s, Kubernetes 10/53.2s and CPU item collection one/16.5s. Runtime security/resources, health/readiness/probes, Polish 320px controls and exact HTML/JS/CSS/PNG identity pass. Temporary acceptance stacks were removed.

This is a source prerelease with a verified local image. Localhost runs the accepted image; v0.3.100 remains available for rollback. Deploy paired server/browser as usual; no wire or replay migration is introduced. Work priorities remain in the [backlog](../../BACKLOG.md).

## v0.3.100 — 2026-10-28

[v0.3.100](https://github.com/liveweird/grenadiers/releases/tag/v0.3.100) adopts the owner-supplied third/fourth-player uniform redraw. Green and plum appearances now match the first two characters' outlines, shading and detail more closely. The supplied PNG is adopted byte-for-byte; its original 95 frames remain decoded-pixel identical, and all 16 revised poses retain the existing 33-pixel height, `(40, 65)` foot anchor and manifest geometry. The duplicate PNG is removed from the repository root. See [artist provenance](../../assets/art/README.md#provenance).

Independent art review found no asset defects. The 37/33-color uniforms follow a documented 48-color ceiling; nonempty-frame, coat-family, binary-alpha, gutter/reserved-slot, geometry and texture-budget checks remain enforced. The real exporter passes with 107 active frames and 3,590,400 RGBA bytes. Atlas browser checks pass 10 cases/16.5s, gameplay-atlas checks two cases/11.3s, and 37 affected web tests pass. Production build, E2E static/scenario, source-cap and repository checks pass; compressed JS is 369,330/400,000 bytes and CSS 15,092/20,000.

Rust implementation, web application/test code, API contracts, gameplay and external dependencies are unchanged from v0.3.99. Its full Rust/web coverage, exhaustive masks, CPU reference/replay and dependency-audit acceptance are reused for those unchanged inputs rather than rerun for an artwork-only change. Product metadata advances to 0.3.100; live rooms remain two-player and the new appearances are prepared for multiplayer activation.

Source `4f71a196f95f5c66e5ab89860d893cd10061cc21`, tree `547470030cc70162f3d37262511dd96b2dc75c78`, produces accepted image `sha256:6516eb10e76ac25cba7be3dcb6e2fec8d80c995fe3eee526ccc49111b27642d9`. Pinned Trivy 0.74.0 finds zero fixable HIGH/CRITICAL vulnerabilities. Compose passes 10 browser cases/49.4s; Kubernetes passes 10/50.6s; native CPU-item collection passes one/16.5s. Served HTML/JS/CSS/PNG identity, Polish 320px controls, runtime security/resources, readiness and probes pass. The preview was visually inspected and temporary acceptance stacks removed.

This source prerelease retains gameplay v13 and replay format 3. Deploy the paired browser/server and reload for the new atlas. Localhost runs the exact accepted image with v0.3.99 retained for rollback; no production registry publication is claimed. The [backlog](../../BACKLOG.md) owns outstanding multiplayer work.

## v0.3.99 — 2026-10-28

[v0.3.99](https://github.com/liveweird/grenadiers/releases/tag/v0.3.99) prepares multiplayer with explicit CPU seat identity and distinct forest-green/cream/silver and plum-purple/cream/gold uniform appearances. Controllers validate their recipient before changing memory or world state; current Solo runtime authority remains seat Two. The artist sheet adds 16 directional frames while preserving all original 95 frame definitions and decoded pixels. See the [artist handoff](../../assets/art/README.md) and [multiplayer design](../game/multiplayer-design.md).

This release retains two-player live rooms, gameplay v13, replay format 3 and existing simulation/generation identities. Additional appearances are ready for artist review; three-/four-player rooms and configurable arenas are subsequent work. Deploy the paired server/browser and reload clients for the updated atlas.

Independent CPU/art review is clear. Full web passes 6,892 tests/98 files: lines 93.64%, statements 91.31%, functions 90.42%, branches 89.95%. Compressed JS is 369,324/400,000 bytes; CSS 15,092/20,000. Formatting, warnings-denied Clippy, source caps, contract/static checks and tooling pass. Atlas browser acceptance passes 10 cases plus two gameplay cases. All 64 CPU reference cases retain exact v0.3.98 metrics and accepted-input replay; isolated Standard opening/full runs take 2.37s/12.20s and Easy 2.10s/10.48s. These comparisons establish compatibility rather than human balance.

The complete server gate passes on source `f01610a39ce7b2a8d7745583d17ddead0f0b6001`, tree `8aabec7789a18ba086687a269e6040140b8264ed`: 495.393s; 27,722/30,098 lines (**92.1058%**); all 4,194,304 masks in each debug/release/official coverage profile; 32 partitions, four workers, zero hidden retries and 37 raw profiles. Official LCOV was copied before releasing the source/target freeze. Unique Linux [run 40599801460](https://github.com/liveweird/grenadiers/actions/runs/40599801460) passes.

Exact image `sha256:a5654f65a6aaaae14b32e684486665429c7441c9d203bf4dfffa8b90e92d8350` passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings; Compose 10 cases/49.9s; Kubernetes 10 cases/51.3s; CPU item collection 1 case/16.4s; Polish 320px, keyboard, asset identity, runtime security/resources and probes. The adopted uniforms were visually inspected. Temporary acceptance stacks were removed. Unchanged external dependencies retain prior audit acceptance.

The initial full-web run exposed a stale status-bar expectation for the taller atlas; the fixture was corrected and the complete suite rerun without lowering assertions or floors. Generated uniform color noise was normalized to the intended bounded palettes before final review and acceptance, with binary alpha and unchanged original pixels verified.

This is a source prerelease with a verified local image, not a registry publication. Localhost runs the accepted image; v0.3.98 is retained for rollback. Work priorities remain in the [backlog](../../BACKLOG.md).

## v0.3.98 — 2026-10-27

[v0.3.98](https://github.com/liveweird/grenadiers/releases/tag/v0.3.98) adds Easy/Standard solo selection, defaulting to Standard, with English/Polish labels and server-authoritative difficulty retained through reconnect/rematch. Easy adds a provisional 12-tick pause after confirmed Stop ending ordinary clear-board navigation. Unknown bombs, new hazards or unsupported control effects bypass that pause. Committed escapes, Stops, manual confirmation, work limits and Standard cadence remain unchanged.

Gameplay v13 adds the closed difficulty metadata while preserving v12’s Standard-only archive and the checked v11 physical/privacy projection. Clients choose only difficulty; policy, seed and seat authority remain server-owned. UtilityV16 admits two exact current profiles; V1–V15 remain Standard-only replay identities. Deploy server/browser together and reload older clients; replay format 3 and simulation rules are unchanged.

Independent contract/runtime/frontend review is clear. Focused tests exercise actual queued pause/progress/danger bypass, both-profile Plant/Punch timing/rejection/stale handling, profile authority, lifecycle and replay. Full web passes 6,890 tests/98 files with 93.63% lines, 91.28% statements, 90.53% functions and 89.87% branches. Compressed JS is 368,673/400,000 bytes; CSS 15,092/20,000. Formatting, warnings-denied Clippy, E2E static/scenario checks, source caps and tooling pass.

A separate 32-case Standard/profile comparison proves exact accepted-action/frame/RNG/work trace equality. Four isolated 16-case matrices survive and replay exactly. Standard preserves every v0.3.97 case metric: opening 588 calls/17,828,381 work; full 4,062/90,375,728 with 15 timeout draws and one CPU win. Easy uses 525/3,307 calls and 14,600,204/60,072,846 work, with 16 full-round timeout draws and more stationary time. All attempts remain within 65,536 work. Observed opening/full times were 2.50s/13.45s Standard and 2.22s/11.30s Easy. These scripted results do not establish balanced human difficulty. Matrix source `ba0619c8f6846aeb407fe4b5ccaa1295d9f831cf` differs from final acceptance only in corrected socket/browser test fixtures; gameplay, evaluation and full-web inputs are unchanged.

One complete exhaustive server gate passes on final source `f020733b9b81a2d135c577df90a2b8817d6d4131` (tree `005bd71e9d34ed241d6c108ccdef472e5eb0d2f7`): 468.64s, 27,543/29,915 lines (**92.0709%**), all 4,194,304 masks in each debug/release/official coverage profile, 32 partitions/four workers/zero hidden retries and 37 raw profiles. Official LCOV was preserved before releasing the source/target freeze. Unique final Linux [run 40591108533](https://github.com/liveweird/grenadiers/actions/runs/40591108533) passes.

Exact image `sha256:11966d7f74f519c491a901ed72df107fbd9496487f4a2ce445692817bbf7beda` passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings, Compose 10 cases/49.5s, Kubernetes 10 cases/51.5s, CPU-item collection 1 case/16.5s, Polish 320px, keyboard focus/activation, asset identity and runtime security/resources/probes. The new UI was visually inspected. Temporary stacks were removed; unchanged external dependencies retain prior audit acceptance.

Development failures remain recorded: sandbox listeners required native permissions; a third-validator drift build needed a bounded 15-second timeout; initial socket/browser fixtures retained stale protocol arguments and an unsuitable native-select key assumption. Option values use Playwright’s native select operation; real Tab/Shift+Tab focus and Enter activation are checked separately because this Mac Chromium setup does not apply keyboard option-selection defaults. Those fixtures were corrected without lowering assertions or coverage floors before final acceptance.

This is a source prerelease with a verified local image, not a registry publication. Localhost runs the accepted image; v0.3.97 is retained for rollback. Broader CPU tactics and human balance validation remain in the [backlog](../../BACKLOG.md#single-player).

## v0.3.97 — 2026-10-27

[v0.3.97](https://github.com/liveweird/grenadiers/releases/tag/v0.3.97) lets an empty-slot or Kick-holding CPU deliberately collect Punch and Bracelet. Their equal provisional goal value remains below useful buffs/upgrades and above block openings/rival pressure. Held tactical equipment permits duplicate transit while avoiding Kick or cross-tactical replacement; existing Heart/Wall Pass protections remain unchanged. Equipment classification is shared by goal selection and remembered-step validation. UtilityV15 retains V1–V14 replay identities and wire v12.

Independent review is clear after correcting a stale replay-version test. Focused acceptance includes 57 policy/182 CPU core tests, five replay contracts and three queued Driver regressions covering physical empty/Kick acquisition, duplicate transit and detours preserving tactical equipment, Heart and Wall Pass. A shared zero-worker Driver setup preserves existing helper behavior. Crafted open-world fixtures do not claim generated-world replay; formatting, Clippy, repository/tooling checks and the 400-line ceiling pass.

Both 16-case matrices survive and replay exactly. Four arena-a2 cases change trajectories; both idle seeds collect their first item at tick 850 rather than 1306. Their opening stationary time increases by 40 ticks, while full-round stationary time decreases by 151/85 ticks. Full rounds retain 15 timeout draws and the same CPU win. Opening calls/work change from 594/17,958,825 to 588/17,828,381; full calls/work from 4,010/89,659,046 to 4,062/90,375,728 (about 0.8% more work), with the 65,536 maximum retained. Isolated host observations were 3.38s/14.12s; planner p95/max were 5.468/6.885ms opening and 4.568/7.122ms full. These fixtures do not establish human-level strength or a timing guarantee.

The exhaustive server gate passes on source `f59dcd342411d2b8c999e9e923b09337674f5ba0` (tree `3cd46bb3d82a17564ab69b2c9ac02475299a35bb`): 544.006s, 27,108/29,393 lines (**92.2260%**), all 4,194,304 masks in each of debug/release/official coverage, 32 partitions/four workers/zero hidden retries and 37 raw profiles. Official LCOV was preserved before releasing the source/target freeze. Unique Linux [run 40585859344](https://github.com/liveweird/grenadiers/actions/runs/40585859344) passes.

Exact image `sha256:12a09e67661ef3679169d20ebf283a9271e451e0d379bff4b680e7f57dbcc548` passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings, Compose 10 cases/51.6s, Kubernetes 10 cases/52.9s, CPU-item collection 1 case/16.5s, Polish 320px, served-asset identity and runtime security/resources/probes. Temporary stacks were removed. Unchanged frontend/API/art/E2E/tooling/external dependency inputs retain accepted full-web 6,874 tests/97 files and static/audit checks.

This source prerelease has a verified local image; no production registry publication is claimed. Localhost runs the accepted image with v0.3.96 retained for rollback. Broader CPU4 tactics, measured difficulty and human validation remain in the [backlog](../../BACKLOG.md#single-player).

## v0.3.96 — 2026-10-27

[v0.3.96](https://github.com/liveweird/grenadiers/releases/tag/v0.3.96) adds a guarded held-Punch tactic. One stopped Equipment action has a common certified escape for relocation and a no-op, preserving the original bomb identity and absolute fuse. The narrow scene requires a centered CPU, one adjacent Standard timed bomb, no exposed pickups/Oil/flames and a visible rival without bomb-moving equipment. New incompatible observations withdraw the route. Manual player controls remain unchanged.

A recipient-footprint barrier follows the authoritative bomb collision rule: only the named bomb’s motion uncertainty at the continuously occupied origin is waived; lethal checks remain and leaving the cell consumes the exception. Exact basis+6 admission/basis+7 execution and typed manual rejection preserve the checked schedule without retry or legacy Bomb escape. Optional proof work uses at most half of the remaining global allowance. UtilityV14 retains V1–V13 replay identities and wire v12.

Focused tests cover geometry, timing, branch identity and complete physical routes through fuse/flame expiry. A core-private fixture compares ready/cooling histories with identical observations; a queued Driver fixture covers accepted/rejected Equipment in a Kick-enabled ruleset. These compose cooldown and queue coverage; the Driver fixture does not create bomb motion or combine accepted input with private cooldown. Crafted replacement-state fixtures do not claim generated-world replay. Independent review, formatting, Clippy, repository/tooling checks and the 400-line ceiling pass.

Both 16-case matrices replay exactly with unchanged per-case trajectories, outcomes, calls and work versus v0.3.95: 594/4,010 calls, 17,958,825/89,659,046 work units and a 65,536 maximum. Opening cases remain alive/censored; full rounds retain 15 timeout draws and the same CPU win. Isolated host observations were 2.38s opening and 12.13s full; planner p95/max were 3.995/5.248ms and 3.756/4.973ms. These are reference observations, not a strength or performance guarantee.

One exhaustive server gate passes on source `27d074ca5c83544a12b5a31cd97ae410b036b6f5` (tree `a6d3e77e7c95118583b4a75dc2ecb1356222e065`): 521.69s, 27,100/29,389 lines (**92.2114%**), all 4,194,304 masks separately in debug/release/official coverage, 32 partitions/four workers/zero hidden retries and 37 raw profiles. Official LCOV was preserved before releasing the source/target freeze. Unique Linux [run 40583437112](https://github.com/liveweird/grenadiers/actions/runs/40583437112) passes.

Exact image `sha256:5d967f51f91aaea9361cea8dc0878fb7332d37e980c2aeb3ff0ccf38b0f32b15` passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings, Compose 10 cases/51.4s, Kubernetes 10 cases/53.0s, CPU-item collection 1 case/15.6s, Polish 320px, served-asset identity and runtime security/resources/probes. The initial scanner configuration-digest lookup failed; the corrected immutable-image scan is retained separately. Temporary stacks were removed. Unchanged frontend/API/art/E2E/tooling/external dependency inputs retain accepted full-web 6,874 tests/97 files and static/audit checks.

This source prerelease has a verified local image; no production registry publication is claimed. Localhost runs the accepted image with v0.3.95 retained for rollback. CPU4 difficulty, broader tactics and human validation remain in the [backlog](../../BACKLOG.md#single-player).

## v0.3.95 — 2026-10-27

[v0.3.95](https://github.com/liveweird/grenadiers/releases/tag/v0.3.95) adds an opportunistic
held-Bracelet tactic. A certified straight movement pulse can push a visible rival into current
fire, using the existing automatic contact rule without a manual action. The tactic requires
visible empty bombs, zero exposed pickups, a non-Bracelet rival and an unobstructed victim
sweep. Contact scoring reuses the authoritative kernel; fire must remain active at the latest
possible contact. The stationary rival is guidance for effectiveness, not a prediction of input.

The normal seven-start pulse certificate and stopped margin protect the CPU path. An optional
lease guard rechecks the public prerequisites at admission and during execution; ordinary leases
are unchanged. Private cooldown/contact history may prevent a shove. Existing escape/origin
precedence, fallback, movement rules, acquisition priorities and planning limits remain in force.
UtilityV13 retains V1–V12 accepted-input replay identities and wire v12.

Focused verification includes six tactic tests, five guard tests, 164 CPU tests, 35 controller
tests and five replay contracts. The crafted queued Driver fixture collects Bracelet and creates
finite fire through ordinary actions. Equal public observations with ready/cooling private
histories produce identical queued movement: the ready shove kills the rival, while cooldown
failure keeps both actors safe. This crafted replacement-state fixture does not claim
replay from a generated-world header. Independent review is clear; source files remain at most
400 lines.

Both unchanged 16-case reference matrices replay exactly. Opening cases remain alive/censored;
full rounds retain 15 timeout draws and the same CPU win with no loss/censoring. Per-case outcome,
progress, call and logical-work metrics are unchanged from v0.3.94: 594/4,010 calls and
17,958,825/89,659,046 work units; the maximum remains 65,536. The targeted fixtures provide the
new tactic evidence rather than claiming stronger play from unchanged matrix results.

Initial host observations were 2.59s opening and 18.17s full, with a full planner max of 48.145ms.
Because work/results were unchanged, one explicitly declared isolated old/new timing comparison
followed: preserved v0.3.94 took 13.817s and v0.3.95 13.518s, with p95/max 4.242/5.850ms versus
4.178/6.082ms. This comparison suggests host variability in the initial measurement; it does not
establish performance thresholds. Original and comparison artifacts are retained; no correctness
failure was retried.

One exhaustive local server gate passes on tested source
`64f25c964fab26a13fd453a1e949ed57b7578afc` (tree
`826e1a643afe2d7b29de53b6409f93ca6156d241`): 528.544s,
26,423/28,665 lines (**92.1786%**), all 4,194,304 item masks separately in debug,
release and official coverage, 32 partitions/four workers/zero hidden retries and 37 raw
profiles. Official LCOV was copied before releasing the source/target freeze. Unique Linux
[run 40553481595](https://github.com/liveweird/grenadiers/actions/runs/40553481595) passes.

Exact image
`sha256:fd6a9d634fc1f69abc46e5fe272de74f924a023bf10c40556e14b4d33db3ed81`
passes pinned Trivy 0.74.0 with zero fixable HIGH/CRITICAL findings, Compose 10 cases/50.5s,
Kubernetes 10 cases/52.8s, CPU-item collection 1 case/16.6s, Polish 320px, served-asset identity
and runtime security/resources/probes. Temporary projects/namespaces were removed. Unchanged
frontend/API/art/E2E/tooling/dependency inputs preserve accepted full-web 6,874 tests/97 files
and static/audit evidence; no duplicate full web run was needed.

This is a source prerelease with a verified local image, not a container-registry publication.
Localhost runs this image; v0.3.94 is retained for rollback. Further CPU tactics, difficulty
and human validation remain in the [backlog](../../BACKLOG.md#single-player).

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
and human validation remain in the [backlog](../../BACKLOG.md#single-player).

## v0.3.93 — 2026-10-27

[v0.3.93](https://github.com/liveweird/grenadiers/releases/tag/v0.3.93) resumes CPU goals after a
[stationary hazard route](../game/cpu-contracts.md#finite-hazard-responsiveness) observes a real clear
gap. Full-horizon Wait routes previously remained installed after all bombs and flames disappeared,
preventing ordinary pickup navigation during repeated rival bombing. Completion now requires an
originally visible nonempty bomb set, a currently visible empty set, no flames or burning Oil,
only Wait steps, consumed transitions, an executed Stop in bounds, and existing scene/actor/exact-pose
validation. Planting, moving routes, legacy escape and origin/pulse recovery retain their paths.
Completion grants no input: the existing controller requests a fresh ordinary decision.

The unchanged real queued-driver case records 64 directional inputs after tick 1,600, visits seven
cells instead of five and remains alive through 3,600 ticks. Repeated traces and accepted-input replay
match exactly. Core/public-API and controller tests cover Stop bounds, missing confirmation,
finite-flame-only waits, active/hidden hazards, invalid pose, movement preservation and completion
once. Independent source review is clear. UtilityV11 preserves V1–V10 replay identities and wire v12.
Forecast horizons, pickup-changing pulse guards and the 65,536-unit work allowance are unchanged.

Clean-source opening/full-round matrices verify all 32 replays. Opening remains 16 censored/alive
opponents; full rounds remain 15 natural timeout draws and one CPU win, with no losses or censoring.
Every retreat case explores beyond its prior four/five-cell opening: six to 43 cells. Some still
stall later; broader tactics and human feedback remain in the [backlog](../../BACKLOG.md#single-player).
Full-round planning rises from 2,294 to 4,118 calls, while elapsed time rises from 9.38s to 11.15s.
Planner p95/max are 3.679/5.127ms on macOS/aarch64. These deterministic host observations do not
establish balanced difficulty or hardware thresholds. A sandboxed measurement wrapper initially
could not read macOS resource statistics after its successful test; the explicit permitted rerun
produced the accepted clean-source artifacts.

Tested source is `6936e18df5b2624b6fc796a755ef1f6ef5272e92`.
Full local Rust acceptance passes in 470.886s: all 4,194,304 item masks in debug,
release and official coverage, 32 partitions/four workers, zero retries and 37 raw profiles.
Coverage is 25,999/28,234 lines (92.08%). The distinct
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40456096514) passes. Formatting,
workspace Clippy and repository/source-size/tooling checks pass. Proven unchanged frontend,
API, artwork, E2E and external dependencies retain accepted static/audit/full-web evidence
(6,874 tests/97 files).
Accepted image is
`sha256:2001793700caa8a97032ce61a2ba1b4500cbe920b0e36737ca26db4fd643c6e2`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser cases
in 50.1s, Kubernetes ten in 51.1s and dedicated all-items CPU pickup acceptance one in 16.4s.
Polish 320px, resources/security/probes and served assets pass; temporary projects/namespaces are
removed. The exact image is promoted to localhost:8085 with v0.3.92 retained for rollback;
restarting ends in-memory matches. Evidence is under `target/quality-review/cpu4-v0393-*`.
Distribution remains a source prerelease and local image.

## v0.3.92 — 2026-10-27

[v0.3.92](https://github.com/liveweird/grenadiers/releases/tag/v0.3.92) adds a separate
[full-round CPU reference workload](../game/performance.md) while preserving the existing
1,800-tick opening matrix. `scripts/measure-game.sh --cpu-rounds` runs the same 16 independent
arena/CPU seed and human-script combinations to the existing 10,800-tick round limit. Reports
distinguish natural outcomes from censored episodes. Both workloads replay every accepted-input
journal through the existing engine to exact runtime inspection, outside planner timing; incomplete
or truncated journals fail within the unchanged 60,000-event/4MiB recording bounds.

The clean-source opening matrix reproduces all v0.3.91 gameplay/work results exactly, apart from
additive outcome/replay fields: 16 censored/alive opponents, 512 planning calls, 2.23s elapsed.
The full-round matrix finishes in 9.38s: 15 natural timeout draws, one CPU win, no losses or censored
rounds. All 32 episodes replay exactly. Full-round planner p95/max are 3.375/4.908ms over 2,294 calls
on macOS/aarch64. These are deterministic reference observations, not difficulty or hardware limits.
The retreat script exposes a persistent stationary-progress limitation, tracked only in the
[backlog](../../BACKLOG.md#single-player). Gameplay, UtilityV10, production recording and wire v12
remain unchanged. Independent review is clear.

Tested source is `5276a85c8baf15514ad483df6380ef6f10ff89d6`. Full local Rust acceptance passes in
453.969s: all 4,194,304 item masks in debug, release and official coverage, 32 partitions/four workers,
zero retries and 37 raw profiles. Coverage is 25,941/28,176 lines (92.07%). The distinct
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40452719744) passes. Formatting,
workspace Clippy, measurement CLI/source-identity and repository/source-size checks pass. A subsequent
merge of preceding release documentation changes only five Markdown files; recorded input comparison
proves all tested/build inputs unchanged. Its automatic Linux run also passed. Unchanged frontend,
API, artwork, E2E and external dependencies retain accepted static/audit/full-web evidence
(6,874 tests/97 files).

Accepted image is
`sha256:4f71fb1c54f422fe333d61a87b1cd710465914eae7f6be6df53a85e7de484899`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser cases
in 49.5s, Kubernetes ten in 51.4s and dedicated all-items CPU pickup acceptance one in 16.4s.
Polish 320px, resources/security/probes and served assets pass; temporary projects/namespaces are
removed. The exact image is promoted to localhost:8085 with v0.3.91 retained for rollback;
restarting ends in-memory matches. Evidence is under `target/quality-review/cpu4-v0392-*`.
Distribution remains a source prerelease and local image.

## v0.3.91 — 2026-10-27

[v0.3.91](https://github.com/liveweird/grenadiers/releases/tag/v0.3.91) closes the observed
[late-placement escape failures](../game/cpu-contracts.md#interrupted-confirmed-planting).
A unique exact newly observed birth can establish origin provenance even when an unrelated pickup
appears in that publication. Full-scene confirmation still fails and the old route still stops;
only fresh checked pulses can recover. Signature validation is shared with normal confirmation.
Absent, ambiguous, mismatched, paused or expired placement, wrong pose and actor changes still decline.

Recovery guidance reuses the static refuge search from a fixed origin and requires strict
worst-case progress toward a legal exit. This permits necessary turn alignment without the
reproduced East/North oscillation. Hazard proofs, uncertainty, timing, work, one-way grace and
ordinary navigation remain unchanged. The real unchanged a1/a2 cases now recover after second
placements at 424/412 and survive full own-flame expiries at 634/622, without recovery manual actions
or restored old origins. Independent review is clear. UtilityV10 retains V1–V9 accepted-input replay
and the unchanged v12 wire shape.

The same clean-source 16-case matrix completes in 2.07s with all 16 CPU opponents alive at the
1,800-tick bound: 16 censored outcomes, zero losses and no wins. This compares with four losses in v0.3.90
and six in v0.3.89. Across 512 planning calls, p95/max are 3.895/4.857ms on macOS/aarch64. These are reference
workloads and host observations, not balanced-difficulty or hardware-threshold claims. Broader
round evaluation, tactics and human feedback remain in the [backlog](../../BACKLOG.md#single-player).

Tested source is `dc551c7d3116a1d0c8be98e577f435e96fe30982`. Full local Rust acceptance passes
in 470.822s: all 4,194,304 masks in debug, release and official coverage, 32 partitions/four
workers, zero retries and 37 raw profiles. Coverage is 25,913/28,034 lines (92.43%).
The distinct [Linux run](https://github.com/liveweird/grenadiers/actions/runs/40450868513) passes.
Formatting, workspace Clippy and repository/source-size checks pass. Proven unchanged frontend,
API, artwork, E2E, tooling and external dependencies retain accepted static/audit/full-web evidence
(6,874 tests/97 files).

Accepted image is
`sha256:287b976ad207dd50a63ba7e574201c1e9c2760adb3c4db9c0ca08c7ba8521c6e`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser cases
in 49.5s, Kubernetes ten in 50.0s and dedicated all-items CPU pickup acceptance one in 16.4s.
Polish 320px, strict lobby/create, resources/security/probes and served assets pass. Temporary
projects/namespaces are removed. The exact image is promoted to localhost:8085 with v0.3.90 retained
for rollback; restarting ends in-memory matches. Evidence is under `target/quality-review/cpu4-v0391-*`.
Distribution remains a source prerelease and local image.

## v0.3.90 — 2026-10-27

[v0.3.90](https://github.com/liveweird/grenadiers/releases/tag/v0.3.90) fixes
[interrupted confirmed planting](../game/cpu-contracts.md#interrupted-confirmed-planting).
A newly visible enemy bomb still invalidates the old escape route. Validated confirmed-placement
provenance permits fresh short escape pulses after ordinary Stop, covering all seven admission
lanes and permanently withdrawing origin grace on exit. Sole-bomb/floor checks, actor and placement
signatures, lethal hazards, uncertainty, work limits and the conditional three-update cap remain
strict. Legacy or unconfirmed placement cannot grant origin authority. Worker results cannot
restore revoked provenance; ordinary queue lateness preserves independently valid stopped proof.

The real generated a2-c0 case plants at 190, is interrupted at 204, moves again and survives its
own full flame expiry at 400 with one Bomb action and no injected plans. Core and controller cases
cover provenance refusal, clock/pose changes, hostile origin occupancy, exit/reentry, delayed
admission, failed Stop and stale/malformed results. Independent review is clear. UtilityV9 retains
V1–V8 accepted-input replay and the unchanged v12 shape.

The unchanged clean-source 16-case matrix runs in 1.09s: CPU losses decrease 6→4, with no previously
surviving case regressing. Both arena 3 retreat cases now reach 1,800 ticks alive; arena 2 survives
its first blast but loses later. Twelve cases are censored/alive; four end in CPU loss. Across 576
actual planning calls, p95/max are 3.591/4.163ms on macOS/aarch64. These reference workloads establish
a focused repair, not general strength or hardware thresholds; later losses remain in the
[backlog](../../BACKLOG.md#single-player).

Tested source is `b041f1de235cc8614185bc2f9b9b53194c45b8e3` (core and activation commits).
Full local Rust acceptance passes in 471.381s: every 4,194,304 mask in debug, release and official
coverage, 32 partitions/four workers, zero retries and 37 raw profiles. Coverage is 25,806/27,929
lines (92.40%). The distinct
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40447665174) passes. Formatting,
workspace Clippy and repository/source-size checks pass. Proven unchanged frontend/API/art/E2E,
tooling and external dependencies retain accepted static/audit/full-web evidence (6,874 tests/97 files).

Accepted image is
`sha256:529c18269a65fb7049f90fca39a6d5fe037d00e2a343b6c76492c230506a3583`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser cases
in 49.6s, Kubernetes ten in 51.3s and dedicated CPU pickup acceptance one in 16.5s. Polish 320px,
strict lobby/create, resources/security/probes and served assets pass. Temporary projects/namespaces
are removed. The exact image is promoted to localhost:8085; v0.3.89 remains available for rollback.
Restarting ends in-memory matches. Evidence is under `target/quality-review/cpu4-v0390-*`;
distribution remains a source prerelease and local image.

## v0.3.89 — 2026-10-27

[v0.3.89](https://github.com/liveweird/grenadiers/releases/tag/v0.3.89) adds
[deterministic Solo evaluation](../game/cpu-evaluation.md). A bounded test-only queue pump executes
actual planning jobs, delivers their advanced RNG and uses normal controller admission and Driver
physics. Production worker execution is shared; live cancellation, work and delivery limits remain
unchanged. No public mode or difficulty profile changes; UtilityV8 and v12 remain current.

Routine cases cover cancellation, queue limits, failed/disconnected/full delivery, a forced RNG
draw, ordered accepted-input repeatability, separate arena/CPU seed axes and observed event metrics.
The fixed human Bomb/retreat script refreshes held input every ten ticks, respects the normal
15-tick expiry and proves centered refuge plus first-blast survival. First placement/pickup fields
mean observed CPU-recipient events, independently of accepted action counts. Work and physical
outcomes stay separate from wall-clock samples; a capped run remains censored.

The isolated, clean-source release matrix completes all 16 declared cases in 0.93s without retries:
four arena seeds × two private CPU seeds × idle/Bomb-retreat scripts, up to 1,800 ticks each.
Across 612 actual planning calls, p50/p95/p99/max are 0.068/3.427/3.922/4.580ms on macOS/aarch64;
these are host observations, not thresholds. Ten cases are censored with CPU alive and six end in
CPU loss; two observe a CPU pickup. This is reference-workload evidence, not a strength rating.
The new losses motivate the [backlog](../../BACKLOG.md#single-player) escape investigation.

Independently reviewed source is `63283cbef9333f9e06d9bf6039e9dd70342b1652`.
Full local Rust acceptance passes in 462.572s with all 4,194,304 masks in debug, release and
official coverage, 32 partitions/four workers, zero retries and 37 raw profiles. Coverage is
25,319/27,470 lines (92.17%). The distinct
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40444429486) passes.
Formatting, workspace Clippy, repository/source-size, ShellCheck/actionlint, exact-artifact
selection and CLI checks pass. Proven unchanged frontend/API/art/E2E/external dependencies retain
accepted static/audit/full-web evidence (6,874 tests/97 files).

Accepted image is
`sha256:bfc528c594ab295da5d871d65a89863dabbb79a59579565a3eef86b2e8b7f5aa`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser cases
in 49.4s, Kubernetes ten in 51.0s, and dedicated all-items CPU pickup acceptance one in 16.5s.
Polish 320px, strict lobby/create, resources/security/probes and served assets pass. Temporary
projects/namespaces are removed. The exact image is promoted to localhost:8085 with v0.3.88 retained
for rollback; restarting ends in-memory matches. Source/image/binary/environment and matrix
artifacts remain under `target/quality-review/cpu4-v0389-*`. Distribution remains a source
prerelease and local image.

## v0.3.88 — 2026-10-27

[v0.3.88](https://github.com/liveweird/grenadiers/releases/tag/v0.3.88) fixes unnecessarily
long CPU waits after finite flames and around harmless unlit Oil. Non-plant route deadlines
cover the latest current flame expiry and the existing admission/movement/stopped margin;
the complete 840-tick forecast remains intact. Active bombs and burning Oil retain the full
horizon. Unlit Oil remains an obstacle. Metered scans and checked arithmetic fail closed;
planting proofs, pulse safety, route execution and pickup restrictions are unchanged.

The actual Driver regression executes a 30-tick Wait, completes at flame expiry, admits fresh
movement at that same tick and physically collects Bomb Capacity 21 ticks later. Initial/fresh
planning costs are 1,215/20 work units. The previous 840-tick certificate span is source-derived;
the new 30-tick span and Driver progression are measured. Boundary, overflow, budget, all-seven-
lane, multiple-flame and bomb/Oil cases pass. UtilityV8 retains V1–V7 accepted-input replay and
the unchanged v12 wire shape. Independent review is clear.

Tested source is `26ceae8b86b4d1661cd3125c10ae225edcc40077`. Full local Rust acceptance passes
in 470.932s: all 4,194,304 masks in debug, release and official coverage, 32 partitions/four
workers, zero retries and 37 raw profiles. Coverage is 24,883/27,028 lines (92.06%). The distinct
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40441117382) passes.
Formatting, Clippy and repository/source-size checks pass. Proven unchanged frontend, API,
artwork, E2E, tooling and external dependencies retain accepted static/audit/full-web evidence
(6,874 tests/97 files); changed gameplay receives fresh native runtime acceptance.

Accepted image is
`sha256:2fc8cfc5695ea92e694fb845c8b2a31d3e42cc7755c1c55408db4d35a2831e4d`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser
cases in 49.3s, Kubernetes ten in 51.3s, and dedicated all-items CPU pickup acceptance one in
16.5s. Polish 320px, strict lobby/create, resources/security/probes and served assets pass.
Temporary projects/namespaces are removed. The exact image is promoted to localhost:8085;
v0.3.87 remains available for rollback. Restarting ends in-memory matches.

Evidence is under `target/quality-review/cpu4-v0388-*`. After official coverage was copied,
153 application incremental cache directories were removed, freeing 28.53 GiB while retaining
dependencies, binaries and evidence. Distribution remains a source prerelease and local image;
remaining tactics, evaluation and human feedback stay in the [backlog](../../BACKLOG.md#single-player).

## v0.3.87 — 2026-10-27

[v0.3.87](https://github.com/liveweird/grenadiers/releases/tag/v0.3.87) completes the
CPU public-preview item slice with held, enabled Wall Pass and native item acceptance.
A shared capability permits known intact soft-block traversal and certified starts inside
those blocks. Predicted destruction retains an unknown-content fence. Combined blocking,
Oil, bombs, blast geometry, legal planting origins and equipment-change guards remain strict.
Wall Pass cannot become an opening/plant goal; used Wall Pass and held Heart are preserved.

Actual-State/Driver cases exercise traversal and placement constraints. Three authoritative
worlds with empty, concealed pickup and concealed Oil contents produce identical checked
observations and current-policy results while the CPU stands inside the intact block.
UtilityV7 retains V1–V6 accepted-input replay without changing v12, rules 5 or generator 2.
A host-only, read-only arena fixture verifies the actual all-22-items CPU pickup event,
public inventory and rendered gear; private CPU randomness remains independent.

Independently reviewed source is `fd549c227e306b9be51800a99629fdcb31bee2fb`.
Full local Rust acceptance passes in 459.517s with all 4,194,304 masks in each of debug,
release and official coverage, 32 partitions/four workers, zero retries and 37 raw profiles.
Coverage is 24,817/26,977 lines (91.99%). The distinct
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40438620884) passes.
Formatting, workspace Clippy, repository/source-size checks, E2E type/lint, CI contracts and
tooling pass. Unchanged external dependencies and frontend/API/art retain accepted audits
and full web evidence (6,874 tests/97 files); changed E2E/CI inputs were checked anew.

Accepted image is
`sha256:55b54b5a88c3b9fff161747a89fc0bb3048fc84129ccdf5d2a15c682eb4004f2`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser
cases in 50.1s, Kubernetes ten in 51.3s, and dedicated CPU item acceptance one in 16.5s on the
same image. Polish 320px, strict lobby/create, security/resources/probes and served assets
pass. Temporary projects/namespaces are removed. The exact image is promoted to
localhost:8085 with v0.3.86 retained for rollback; restarting ends in-memory matches.

Development fixture encoding and semantic locators were corrected before acceptance;
original failed evidence remains retained. Evidence is under `target/quality-review/cpu3-v0387-*`.
Distribution remains a source prerelease and local image. Remaining tactics, responsiveness
and difficulty measurements stay in the [backlog](../../BACKLOG.md#single-player).

## v0.3.86 — 2026-10-27

[v0.3.86](https://github.com/liveweird/grenadiers/releases/tag/v0.3.86) adds
[certified timed weapons](../game/cpu-contracts.md#certified-timed-weapons) to the CPU. Enabled
Piercing and Power charges reuse the ordinary planting proof, with one escape checked against
both placed and absent outcomes. Confirmation binds the actual blast and charge consumption;
rejection or physical placement veto preserves the weapon and continues the absent route.
Confirmed placement cannot be reinterpreted as absence.

Useful targets use the shared blast geometry, including Power diagonals and Piercing
continuation, with charged field construction and full traversal. Supported weapon pickups
have explicit acquisition, duplicate and replacement behavior. Neutral swaps remain ordinary
transit options; Heart uses an independent equipment slot. Certified paths still reject all
pickup-changing contact. Remote, Landmine and live owned Remote remain outside timed placement.

Independent review is clear. Actual-State and Driver cases cover both weapons through full
flame expiry without another worker, real same-cell placement veto, rejected input, one-time
consumption and changed loadouts. A bounded identical corridor certifies a diagonal refuge for
Standard but rejects it for Power. UtilityV6 retains V1–V5 accepted-input replay; the v12 wire
shape, rules 5, generator 2 and all 22 default items remain unchanged. Unknown facts and new
enemy actions can still invalidate execution. Injected-State driver and generation-journal
replay remain separate test oracles.

Tested source is `acd2e4e621cedc521ffad3ed2b8873512d40a936`. Full local Rust acceptance passes
in 463.484s with all 4,194,304 masks in each of debug, release and official coverage,
32 partitions/four workers, zero retries and 37 raw profiles. Coverage is 24,715/26,876 lines
(91.96%). The distinct [Linux run](https://github.com/liveweird/grenadiers/actions/runs/40432433005)
passes, along with formatting, workspace Clippy and repository/source-size checks. Exact unchanged
external Cargo/npm dependencies and frontend/API/artwork/E2E/tooling inputs retain accepted
audits, static checks and full web evidence (6,874 tests/97 files). The image is rebuilt and its
native runtime acceptance runs independently, including the corrected authoritative-winner test.

Accepted image is
`sha256:f08979eed37e1d78091cc8847272a7de7f7887e2a160ee21bf9bae19cb8ead31`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten browser cases
in 50.5s and Kubernetes ten in 51.1s. Solo lifecycle, human duel, Polish 320px accessibility,
strict lobby/create, security/resources/probes and unchanged served assets pass. Temporary
projects and namespaces are removed. The exact image is promoted to localhost:8085 with v0.3.85
retained for rollback; restarting ends in-memory matches.

Evidence remains under `target/quality-review/cpu3-v0386-*`. Distribution remains a source
prerelease and local image, without a production registry or hosted-service claim. Wall Pass
routing and public-preview item acceptance remain in the [backlog](../../BACKLOG.md#single-player).

## v0.3.85 — 2026-10-27

[v0.3.85](https://github.com/liveweird/grenadiers/releases/tag/v0.3.85) improves
[CPU goal guidance](../game/cpu-contracts.md#goal-guidance). One bounded static flood-fill selects
reachable useful pickups, block openings and visible-rival goals. It avoids known curses,
unsupported weapons and harmful equipment replacement, while harmless duplicates and capped
upgrades remain transitable. The CPU can detour instead of chasing an unreachable nearest item.
Timed survival and planting certificates retain authority; this guidance does not certify safety.

Independent review is clear after fixing unreachable fallback, preprocessing work accounting,
capped Golden Flame value and stationary guidance for planting after temporary effects expire.
Two actual-State tests use the checked recipient projection and ordinary inputs to collect a
useful item while leaving a curse or unsupported weapon untouched. UtilityV5 retains V1–V4
accepted-input replay. The v12 wire shape, rules 5, generator 2 and all 22 default items remain
unchanged. Stored weapons are still held; their tactics are separate work in the
[backlog](../../BACKLOG.md#single-player).

Rust/image source is `e7820d0aced959c66a7823aef7e70e353a1ecaac`. Full local Rust acceptance
passes in 466.722s: all 4,194,304 masks in each of debug, release and official coverage,
32 partitions/four workers, zero retries and 37 raw profiles. Coverage is 24,480/26,680 lines
(91.75%). The distinct [Linux run](https://github.com/liveweird/grenadiers/actions/runs/40428726013)
passes. Formatting, workspace Clippy and repository/source-size checks pass. Unchanged external
dependencies, frontend/API/artwork inputs reuse their accepted audit/static and full web evidence
(6,874 tests/97 files). E2E static checks run again for the correction below.

Compose initially passes all ten browser cases in 49.9s. Kubernetes passes nine and exposes one
invalid lifecycle-test assumption: an independently seeded CPU need not win. Test correction
`9b713cddbf2084b7446581e443a44cfa896ce796` verifies the authoritative winner, both survivor states,
exact score and rendered status while preserving observed Bomb, rematch, reconnect and leave
assertions. The affected case passes on the same image in Compose (13.3s) and Kubernetes (13.5s);
the other nine unchanged cases retain their results. Original failure artifacts are preserved.
No cause of the CPU's elimination is inferred from the screenshot. All Rust/image inputs are
verified unchanged by this E2E/scenario/changelog-only correction, so full Rust/Linux results
and the accepted image are reused rather than repeated.

Accepted image is
`sha256:bb35c41627b83f94465105a05a264b1bda094a1936372bbd5f17788eb95d92a3`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Polish 320px, strict lobby/create,
security/resources/probes and byte-identical served assets pass. Temporary Compose projects and
Kubernetes namespaces are removed. The exact image is promoted to localhost:8085 with v0.3.84
retained for rollback; restarting ends in-memory matches. Evidence remains under
`target/quality-review/cpu3-v0385-*`. Distribution remains a source prerelease and local image,
without a production registry or hosted-service claim.

## v0.3.84 — 2026-10-27

[v0.3.84](https://github.com/liveweird/grenadiers/releases/tag/v0.3.84) adds
[certified ordinary planting](../game/cpu-contracts.md#certified-ordinary-planting). One complete
escape is proved against both a placed bomb and no physical placement: absence can change
blast shielding and chains. The same bounded route continues without another worker or Bomb
retry. First movement precedes placement; future bomb occupancy, consumed origin grace and
absolute range/fuse modifiers follow the shared game rules.

Early results remain in the existing bounded slot until exact admission. Scene, pose, actor,
effect, epoch and deadline checks fence execution. Public observations bind the actual bomb ID
or the absent outcome. Actual Driver tests cover accepted inputs whose simultaneous placement
veto creates neither bomb, rejected movement/manual input and workerless continuation.
Independent review is clear after correcting temporal flame insertion, opponent-Stopwatch
freeze handling and the shared actor-invariant guard. Regressions cover absent-shielding danger,
partial work exhaustion, modifier expiry and changed actor facts at both execution boundaries.

The policy is UtilityV4; V1–V3 recorded inputs remain readable. The v12 wire shape, rules 5,
generator 2 and all 22 default items are unchanged. Conditional routes never authorize planting.
Stored special weapons and a live own Remote remain outside ordinary placement. Existing narrow
CPU1 planting remains a guarded fallback. New enemy actions and unknown facts can invalidate a
route. Injected-State driver execution and generation-journal replay are separate test oracles.

Final tested source is `af29f6a59e36c525379fb31534780b367fcfa48e`. Full local Rust acceptance
passes in 481.898s: all 4,194,304 masks in each of debug, release and official coverage,
32 partitions/four workers, zero retries and 37 raw profiles. Coverage is 24,331/26,509 lines
(91.78%). The distinct [Linux run](https://github.com/liveweird/grenadiers/actions/runs/40426511318)
passes, along with formatting, Clippy and repository/source-size checks. Exact unchanged external
dependencies and frontend inputs reuse accepted audit, API/static and full web evidence
(6,874 tests/97 files); the image is rebuilt and tested independently.

Accepted image is
`sha256:5ef2bca49de2ba8ae664077a9a14e6d58036d4b8530576e478206283e0ad35cf`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten native browser
cases in 50.2s and Kubernetes ten in 51.8s. Solo lifecycle, human duel, Polish 320px accessibility,
strict lobby/create, security/resources/probes and served-asset checks pass. Temporary projects
and namespaces are removed. The exact image is promoted to localhost:8085 with v0.3.83 retained
for rollback; restarting ends in-memory matches.

Canonical source-bound summary, official LCOV and image/runtime evidence remain under
`target/quality-review/cpu2-v0384-*`; the console capture is partial because verbose tool output
was truncated. Distribution remains a source prerelease and local image, without a production
registry or hosted-service claim. Item tactics are next in the [backlog](../../BACKLOG.md#single-player).

## v0.3.83 — 2026-10-27

[v0.3.83](https://github.com/liveweird/grenadiers/releases/tag/v0.3.83) adds complete bounded
[timed CPU routes](../game/cpu-contracts.md#committed-timed-routes): several moves and waits share
one checked program across exact admission-time lanes. The owner executes it without another
worker result. Each move includes three stopped updates; missing post-Stop observations,
rejected commands or missed deadlines invalidate the schedule. Expected bomb removals, blasts,
flame expiry and known Oil ignition are distinguished from new hazards.

The updated opponent is UtilityV3. V1/V2 recorded inputs remain readable and the v12 wire shape,
rules 5, generator 2 and all 22 default items stay unchanged. Conditional routes retain explicit
motion assumptions and never authorize planting. Existing clear-board movement and restricted
CPU1 planting remain separate; broader ordinary planting is next in the
[backlog](../../BACKLOG.md#single-player). New enemy actions and unknown facts can invalidate a plan.
The injected-State driver escape and generation-journal replay are tested separately; this
release does not claim a combined oracle.

Final tested source is `58798266517582bf96b263fbd2655f9c405d54f9`. Independent reviews are clear
after fixing concrete Oil, search dominance, clock, Stop-confirmation and admission issues.
A pre-publication correction also prevents certificates from relying on temporary collision:
neutral motion clips only permanent terrain/boundaries, while sweeps through soft blocks or
unlit Oil are rejected. Actual-State regressions cover WallPass and mid-pulse destruction/
ignition. The earlier passing source is retained as superseded evidence; acceptance below
belongs to the corrected source.

Full local Rust acceptance passes in 500.817s with all 4,194,304 masks in each of debug,
release and official coverage, 32 partitions/four workers, zero retries and 37 raw profiles.
Coverage is 23,552/25,629 lines (91.90%). The distinct
[Linux debug/release run](https://github.com/liveweird/grenadiers/actions/runs/40422219725) passes.
Formatting, Clippy, repository/source-size checks and dependency audits pass; both npm audits
have zero vulnerabilities. Unchanged web/API/artwork/E2E/deployment/tooling inputs reuse their
accepted full web (6,874 tests/97 files), coverage and static evidence; rebuilt served assets
remain byte-identical to v0.3.82.

Exact accepted image is
`sha256:b7f7475fd87d0a47739dc7a7e6264a4342ac0953ff2b6c1bcd198bbb8b3ea580`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Compose passes ten native browser
cases in 55.1s; Kubernetes passes ten in 55.6s. Actual Solo result/rematch/reconnect/leave,
two-human play, Polish 320px accessibility, strict lobby/create, security/resources/probes and
served-asset checks pass. Temporary projects/namespaces are removed. The exact image is promoted
to localhost:8085 with v0.3.82 retained for rollback; restarting ends in-memory matches.

Evidence remains under `target/quality-review/cpu2-v0383-*`; superseded evidence is explicitly
prefixed `cpu2-v0383-pre-motion-*`. Distribution is a source prerelease and local accepted image,
without a production registry or hosted-service claim.

## v0.3.82 — 2026-10-26

[v0.3.82](https://github.com/liveweird/grenadiers/releases/tag/v0.3.82) improves the
[private Solo opponent](../game/single-player.md) with shared blast forecasting and one checked
movement/waiting pulse at a time. Forecasts retain separate burn windows, timed chains,
persistent Oil and explicit motion/trigger/pause uncertainty. Checked actions include swept
footprints, modifier expiry and a stopped safety lease. Fresh observations invalidate unsafe
leased execution; stopped-input provenance and invalidated bomb escapes permit replanning.

The opponent is UtilityV2; live controllers reject older profile labels. Recorded UtilityV1
inputs remain replayable and the v12 wire shape, rules 5, generator 2 and all 22 default items stay
unchanged. Clear-board goals and restricted CPU1 planting remain separate. Ordinary clear-board
movement retains its hold without a hazard lease. Conditional motion assumptions never justify
planting; complete multi-leg survival and broader planting remain in the
[backlog](../../BACKLOG.md#single-player). This release does not claim those guarantees.

Tested source is `ded10b58fba4bae0c7d1e1c31b49b5534be6c672`. Independent forecast, movement and
integration reviews are clear after fixing reported chain-provenance and Stop-scheduling issues.
Full local Rust acceptance passes in 468.227s with all 4,194,304 masks in each of debug,
release and official coverage, 32 partitions/four workers and zero retries. Official coverage is
22,143/24,125 lines (91.78%). The distinct
[Linux debug/release run](https://github.com/liveweird/grenadiers/actions/runs/40413774406) passes.
Workspace formatting/Clippy, repository/tooling, API/generated-contract/E2E static checks and
security audits pass; npm audits have zero vulnerabilities.

Web, API, artwork, E2E and deployment inputs are unchanged from the accepted v0.3.81 baseline.
Reuse its 6,874-test/97-file web result and coverage (91.31/90.03/90.44/93.71% statements/branches/
functions/lines). The candidate rebuild and native acceptance confirm identical served HTML,
JS, CSS and PNG. Exact accepted image is
`sha256:48e830f6d2533dc4a1c6fff271571c28fb52ea8492cced60f82936b110dc8a28`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Ten Compose browser cases pass in
49.9s and ten Kubernetes cases in 51.2s, including real Solo result/rematch/reconnect/leave,
two-human play and Polish 320px accessibility. Strict lobby/create, resource/security/probe and
served-asset checks pass; temporary runtime resources are removed. The exact image is promoted
to localhost:8085, with v0.3.81 retained for rollback. Restart ends in-memory matches. Policy-version 2 journals require the updated reader;
production recording remains disabled.

Evidence is retained locally under `target/quality-review/cpu2-v0382-*`; dependency/build caches
remain reusable. Distribution is a source prerelease and local accepted image, without a
production registry or hosted-service claim.

## v0.3.81 — 2026-10-26

[v0.3.81](https://github.com/liveweird/grenadiers/releases/tag/v0.3.81) adds the first playable
[private Solo match](../game/single-player.md). Human seat One controls settings; the server CPU
occupies seat Two without credentials. Settings, result, rematch, running reconnect and permanent
human departure use ordinary lifecycle/input rules. The initial opponent pursues simple goals
and plants only with a restricted escape proof. Committed escape execution does not wait for
another worker. Conservative item/tactical limitations are documented in the feature guide.

The coordinated [v12 contract](../game/protocol-v12.md) adds mode/controller metadata; server and
browser must upgrade together. Rules 5, generator 2 and all 22 default-enabled items remain.
Solo replay adds format-3 header provenance; formats 1/2 and disabled production recording stay
intact. Independent review is clear. Actual driver escape and replay equality are tested
separately, with ordinary-input capture ordering traced statically.

Full Rust acceptance at `d7b88a39fc30fddaebcd7c27686593a10f4da8da` passes in 558.091s:
all 4,194,304 masks across debug/release/official coverage, 32 partitions/four workers, zero
retries, 37 raw profiles, and 20,051/21,897 lines (91.57%). The final
[Linux run](https://github.com/liveweird/grenadiers/actions/runs/40404217347) passes. Full web
acceptance at `87f975e64764333e541e219059505b1283af4cc9` passes 6,874 tests / 97 files in
244.81s; coverage is 91.31/90.03/90.44/93.71% (statements/branches/functions/lines). Initial
archived/live fixture expectations were corrected without dropping assertions or masks.
Repository, formatting, Clippy, generated-contract/API, E2E static and dependency/security checks
pass; both npm audits report zero vulnerabilities.

The responsive follow-up changes CSS and its native browser regression only; Rust and web
TS/JS unit inputs are unchanged, so their evidence is reused. Its clean source is
`dd457a922a225fe82a633b938432be658b95b6e9`. Accepted image is
`sha256:5799213a545744c2ff5d777abcf8c214529d736cedbcb4e7515708511e5308ab`.
Pinned Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Ten Compose cases pass in 49.2s
and ten Kubernetes cases in 51.6s, including Solo authority/movement/result/rematch/reconnect/
leave, both game modes and Polish 320px accessibility. The strict Polish lobby/create smoke
also passes after fixing button wrapping. Exact image/resource/security/probe checks pass;
temporary projects/namespaces are removed. Gzip budgets remain 400,000-byte JS / 20,000-byte CSS.

Localhost:8085 runs the exact accepted image with verified health/readiness, limits, CSP/cache
and served assets. The atlas is unchanged. Control is
`.local-checkpoints/runtime/compose-v0.3.81.yaml`; `rollback-v0.3.80.yaml` retains the prior image.

## v0.3.80 — 2026-10-26

[v0.3.80](https://github.com/liveweird/grenadiers/releases/tag/v0.3.80) establishes the pure
[CPU contracts](../game/cpu-contracts.md): restricted observations, ordinary intents, bounded
memory/work, independent seeded randomness and distinct world/decision identities. The current
v11 recipient projection supplies observations, including cloak and Invisible Bombs privacy.
Internal controller metadata and the [lifecycle policy](../game/single-player-cpu.md#lifecycle-policy)
prepare the next playable slice. Solo admission, a production policy and a Solo UI are not
introduced here. Wire v11, rules 5, generator 2 and replay formats 1/2 remain unchanged.

Independent review is clear. At clean source `b43bacf6113471296ca29c170c76496356660ba1`, the
source-bound Rust gate passes in 456.334s: all 4,194,304 masks across debug, release and official
coverage, 32 partitions/four workers, zero retries, 37 raw profiles, and 18,566/20,325 lines
(91.35%, above the unchanged 90% floor). Focused contracts, recipient privacy and accepted-input
replay pass 10/5/2 tests; ordinary Equipment and Stop are included. The distinct
[Linux debug/release run](https://github.com/liveweird/grenadiers/actions/runs/40394057002) passes.
Repository, formatting, Clippy and dependency/security policy pass; both npm audits report zero
vulnerabilities. Unchanged frontend/tooling inputs reuse the accepted v0.3.79 web evidence.

Accepted image is `sha256:a56de297988f094b73a6757729e707e321e56f36e746a51bafc2c487ee3f6374`,
built from that clean source. Pinned Trivy 0.74.0's fixable HIGH/CRITICAL gate exits zero. Eight
Compose cases pass in 36.4s and eight Kubernetes cases in 37.2s, with exact image, security,
resource and probe checks; temporary resources are removed. Localhost:8085 runs that image with
verified health/readiness, limits and exact served assets. HTML, JavaScript, CSS and atlas bytes
match v0.3.79. Control is `.local-checkpoints/runtime/compose-v0.3.80.yaml`; rollback retains
v0.3.79 in `rollback-v0.3.79.yaml`.

## v0.3.79 — 2026-10-26

[v0.3.79](https://github.com/liveweird/grenadiers/releases/tag/v0.3.79) imports the used Mantine
stylesheet closure in canonical cascade order. CSS gzip falls from 36,695 to 15,050 bytes (59%);
the CSS budget is ratcheted to 20,000 bytes. Actual A/B screenshots and computed geometry/styles
match across desktop, Polish 320px, focus, disabled controls and modal/settings states. See the
[CSS measurements](mantine-css.md). Existing lint exceptions now have local rationales.

The [real 20Hz React measurement](react-publication.md) supports retaining the current
publication model: inclusive subtree render p95 is at most 2.1ms and max 2.6ms in these profiles,
with one renderer update per accepted snapshot, no remounts and zero duplicate-revision work.
Both the original pacing diagnostic and corrected absolute-deadline run are recorded.

Independent review is clear. Full web acceptance at `218b4ba2341d7d23ebc7a8c85b4a3d5517e9b0e7`
passes 6,861 tests / 94 files in 237.31s with 91.24/89.88/90.12/93.60% coverage. Local gzip
JS/CSS is 337,178/15,050 bytes; Docker is 337,967/15,050, within 400,000/20,000-byte budgets.
Unchanged Rust/exhaustive/Linux/dependency inputs reuse the accepted baseline.

Accepted image source is the tested commit; image is
`sha256:0638ae3a9fbe7a289566378337c20ef05356bc96eaa59a05a05ffa490e75c739`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.3s and
eight Kubernetes cases in 36.7s with exact image/resource/security/probe checks. Golden Flame
active-HUD and Polish 320px room flows pass. Temporary resources are removed. Localhost:8085 has
verified health/readiness, CSP/cache and the exact new HTML/JS/CSS with the unchanged atlas.
Control is `.local-checkpoints/runtime/compose-v0.3.79.yaml`; `rollback-v0.3.78.yaml` retains the prior image.

## v0.3.78 — 2026-10-26

[v0.3.78](https://github.com/liveweird/grenadiers/releases/tag/v0.3.78) shares equivalent Rust
frame-bound builders and open-border arenas plus timed-effect browser helpers. Independent review
confirms identical helper contents, per-version largest states, test names/assertions, seeds,
routes and screenshots. V8–v11 encoded frame sizes remain exact; the browser helper binds each
assertion to the bomb planted by that native key action.

Full source-bound Rust acceptance at `936f4e627fa3ec101f759b02f717b745d0d550f2` passes in
449.469s, including every 4,194,304 mask in debug, release and official coverage with
zero retries. Coverage is 18,049/19,738 lines (91.4429%).
Focused debug/release, Clippy, repository and E2E static checks pass. Application web acceptance
reuses the identical v0.3.77 input; existing Linux acceptance covers unchanged production code.

Accepted image source is `f4fd65220cf4fbdc7322fe54bca36af2c5abd288`, whose Docker inputs match
the tested source; image is `sha256:5281acc1e3a9b2f7959e2952edf57ee8bbee2eaf645fff81648ae3cedf175b5d`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.1s and
eight Kubernetes cases in 36.5s, with effective memory/PID/ephemeral-storage limits, security and
probes verified. Golden Flame, Short Fuse and Short Flame cases also pass through the new helpers
and exact read-only fixture mounts. Temporary resources are removed. Localhost:8085 has verified
health/readiness, CSP/cache and the exact accepted web bytes. Control is
`.local-checkpoints/runtime/compose-v0.3.78.yaml`; `rollback-v0.3.77.yaml` retains the prior image.

## v0.3.77 — 2026-10-26

[v0.3.77](https://github.com/liveweird/grenadiers/releases/tag/v0.3.77) enables exact optional
properties in application/tooling TypeScript, omits absent request/UI properties and uses correctly
typed diagnostic state and test events. Wire and visible behavior remain unchanged.

Independent review and all three TypeScript projects pass. Full web acceptance at
`e34bc505906eec657bf03d899ecc1b1eea2a7fef` passes 6,861 tests / 94 files in 249.42s with
91.24/89.88/90.12/93.60% coverage. Local gzip JavaScript/CSS is 337,188/36,695 bytes; the Docker
build is 337,956/36,695 bytes. Repository, lint, drift and dead-code checks pass. Unchanged Rust,
exhaustive, Linux and dependency acceptance are reused.

Accepted image source is that tested commit; image is
`sha256:46079733b7f6682f8460ba438a766c632a90a00f0d6af26f0a57e5a0fb43e166`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.8s and
eight Kubernetes cases in 36.7s, including exact-image/security/probe/effective resource checks.
Polish 320px locale switching and room creation pass; temporary resources are removed.
Localhost:8085 has verified health/readiness, CSP/cache and the exact new HTML/JS plus unchanged
CSS/atlas bytes. Control is `.local-checkpoints/runtime/compose-v0.3.77.yaml`;
`rollback-v0.3.76.yaml` retains the prior image. No gameplay, protocol, artwork or migration changes.

## v0.3.76 — 2026-10-26

[v0.3.76](https://github.com/liveweird/grenadiers/releases/tag/v0.3.76) adds optional pinned
ShellCheck/actionlint checks, verified tool/dependency cache restoration, selected terrain-pixel
acceptance, exact failure-artifact allowlists and controlled runtime OS refresh guidance.
Cache hits never skip verification; no hosted-cache timing benefit is claimed.

Independent review, repository/workflow contracts and pinned tooling checks pass. The terrain
comparison passes at DPR 1 and 2: baseline, repaint, reset and context restoration each have zero
differing bytes. The separate six-cell lifecycle experiment remains opt-in. Unchanged Rust,
exhaustive, Linux, application web and dependency acceptance reuse the published baseline.

Accepted image source is `8af26e21b420b2f609fc6b5cccf2303a339608c1`; image is
`sha256:f98095085aa41ba03e97a25523a9682393864dc7a798151f066c22d8151e8424`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 35.5s and
eight Kubernetes cases in 36.1s, with exact image, effective v0.3.75 resource limits, restricted
security and probes verified. Temporary projects/namespaces and diagnostic preview are stopped.
Localhost:8085 has verified health/readiness, CSP/cache and exact unchanged web bytes. Control is
`.local-checkpoints/runtime/compose-v0.3.76.yaml`; `rollback-v0.3.75.yaml` retains the prior image.
No gameplay, wire, artwork or migration changes; distribution remains source prereleases.

## v0.3.75 — 2026-10-26

[v0.3.75](https://github.com/liveweird/grenadiers/releases/tag/v0.3.75) bounds the Compose app to
256 MiB and 128 PIDs and requests/limits Kubernetes ephemeral storage at 16/64 MiB. Deployment
checks reject weakened effective security contexts and invalid resource quantities.

Independent review, 42 focused resource/security tests, repository checks, rendered-manifest
validation and strict Kubernetes schema validation pass. Rust, exhaustive, Linux and application
web inputs reuse accepted v0.3.74 results; the resource-checker cases are exercised separately.

Accepted image source is `bfb0141e0373a75073d7ab922990f87a570e776a`; image is
`sha256:2ce17b3d8712817628504d6d472fe2a2a938290388e462e9eec08b479c01ea4b`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.0s and
eight Kubernetes cases in 36.5s. Docker HostConfig and live cgroups enforce 268,435,456 bytes and
128 PIDs; the live Pod has exact CPU/memory/ephemeral-storage requests/limits, restricted namespace,
security contexts and probes. No destructive resource stress was introduced. Temporary resources
are removed. Localhost:8085 has verified identity, security, health/readiness, CSP/cache and exact
accepted web bytes. Control is `.local-checkpoints/runtime/compose-v0.3.75.yaml`;
`rollback-v0.3.74.yaml` retains the prior image. No gameplay, protocol, artwork or migration changes.

## v0.3.74 — 2026-10-26

[v0.3.74](https://github.com/liveweird/grenadiers/releases/tag/v0.3.74) shares 16 Rust-produced
admission/snapshot frames across retained v8–v11 adapters, schemas and browser validators.
Cases retain freeze, teleport and Oil differences, recipient privacy, identity and metadata
negative controls. Production projection, schemas and gameplay remain unchanged. Independent review is clear.

Full source-bound Rust acceptance at `28ab639e6dd1a7def1147b514480d8341d6c2163` passes in
488.608s: 955 core / 316 ordinary server tests plus two doctests, all 4,194,304 masks in debug,
release and official coverage, zero retries. Coverage is 18,049/19,738 lines (91.4429%).
Full web acceptance passes 6,828 tests / 94 files in 286.25s with 91.24/89.87/90.12/93.60%
coverage and local gzip JavaScript/CSS 337,162/36,695 bytes.
[Integrated Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40373348609)
pass at `5db1d0e46c14b7278fe854af9578567999b3ebdf`, with identical core/server/API inputs.
Repository, lint, types and contract drift pass; dependency acceptance is unchanged.

Accepted image source is `28ab639e6dd1a7def1147b514480d8341d6c2163`, image
`sha256:4d4189e6dd868fb831f5aea82371320ba0d44b78da70fc77f406f6faeb4bf469`.
Docker gzip JavaScript/CSS is 337,939/36,695 bytes. Trivy 0.74.0 reports zero fixable HIGH/CRITICAL
findings. Eight Compose and eight Kubernetes cases each pass in 36.9s with exact-image/security/
probe/resource checks; temporary projects/namespaces are removed. Localhost:8085 has verified
health/readiness, CSP/cache and unchanged accepted HTML/JS/CSS/atlas bytes. Control is
`.local-checkpoints/runtime/compose-v0.3.74.yaml`; `rollback-v0.3.73.yaml` retains the prior image.
No gameplay, wire, artwork or migration change; distribution remains source prereleases.

## v0.3.73 — 2026-10-26

[v0.3.73](https://github.com/liveweird/grenadiers/releases/tag/v0.3.73) adds competing-resume
and cancellation regressions, including real socket takeover fencing. Two valid queued resumes
may both commit in order; the last binding is authoritative. Canceled admissions do not commit.
Production ownership and wire behavior remain unchanged.

The parallel Rust runner now fails early when Cargo rejects a restored coverage cache and
isolates official reporting to every instrumented workspace executable selected from one Cargo
JSON stream. Binary/profile identities, the 90% floor, exclusions and every mask assertion remain.
Independent review covers stale-object reproduction, complete lib/main selection, artifact mutation,
preexisting-target preservation, owned cleanup and symlink rejection. A scratch execution record
is written before reporting. This closes the [earlier report-contamination failure](coverage-cache-recovery.md).

Full source-bound acceptance at `822991f6e22817d525d4a300ebb24e21f48bac49` passes in 452.372s:
951 core / 316 ordinary server tests and two doctests per profile, all 4,194,304 masks in debug,
release and official coverage, zero retries. Coverage is 17,864/19,553 lines (91.3619%).
[Integrated Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40373348609)
pass with identical competing-resume tests. Unchanged web/dependency acceptance reuses v0.3.72.

Accepted image source is `806c13d0f25ac7327df49237f08fddd50ea94259`, whose Docker inputs equal
those of the tested source; runner-only changes are outside the image. Accepted image is
`sha256:06d7c75d4148ee2ff9897d984407450a465f4b34ac4ca404ef9473a9ec616121`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose and eight Kubernetes
cases each pass in 36.6s, including exact-image/security/probe/resource checks; temporary resources
are removed. Localhost:8085 has verified health/readiness, CSP/cache and unchanged served web bytes.
Control is `.local-checkpoints/runtime/compose-v0.3.73.yaml`; `rollback-v0.3.72.yaml` retains the
prior image. No gameplay, protocol, artwork or migration change; distribution remains source prereleases.

## v0.3.72 — 2026-10-26

[v0.3.72](https://github.com/liveweird/grenadiers/releases/tag/v0.3.72) shares expanded client
conformance cases across v8–v11 Rust parsers/schemas and the browser, with deterministic malformed
frame, retired-action, required-field and byte-boundary regressions. Production parsers and
schemas remain unchanged. Independent review is clear.

At `f8457ef351af5822996b4f143efe0831abe45909`, ordinary profiles pass 951 core / 312 server
tests plus two doctests; all 4,194,304 masks complete in debug, release and official coverage
with zero product retries. The final runner report initially failed because stale cache objects
contaminated discovery. An official isolated report using the same three executed binaries and
38 profiles passes the unchanged 90% floor: 17,863/19,553 lines (91.356825%). The
[recovery record](coverage-cache-recovery.md) preserves the original failure and provenance;
no successful canonical runner summary or Rust duration is invented.

Full web acceptance passes 6,822 tests / 93 files in 264.03s with 91.07/89.40/90.12/93.48%
coverage. Local gzip JavaScript/CSS is 337,162/36,695 bytes; the accepted Docker build is
337,939/36,695 bytes, within the same budgets. Repository, lint, types and drift pass.
[Integrated Linux units](https://github.com/liveweird/grenadiers/actions/runs/40373348609)
pass at `5db1d0e46c14b7278fe854af9578567999b3ebdf`, whose client corpus/test inputs are identical.

Accepted image source is the tested `f8457ef351af5822996b4f143efe0831abe45909`, image
`sha256:e4989f32b30bb78400a12229dd3eb13c86a0a7f4ffd4bfea2bf3c943a286b5c1`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings; eight Compose cases pass in 35.4s and
eight Kubernetes cases in 35.9s with exact-image/security/probe/resource checks. Temporary resources
are removed. Localhost:8085 has verified health/readiness, CSP/cache and exact accepted web bytes.
Control is `.local-checkpoints/runtime/compose-v0.3.72.yaml`; `rollback-v0.3.71.yaml` retains the
prior image. No gameplay, wire, artwork or migration change; distribution remains source prereleases.

## v0.3.71 — 2026-10-26

[v0.3.71](https://github.com/liveweird/grenadiers/releases/tag/v0.3.71) adds isolated owner,
registry and entropy measurements plus real non-reading TCP regressions. The regressions verify
write recovery, bounded shutdown, healthy-peer delivery and cleanup. Production registry locking,
entropy boundaries and timeout values remain. The [measured reference](registry-admission.md)
does not justify a locking refactor or establish running-match capacity.

Independent review is clear. Full source-bound Rust acceptance at
`0a3c586cd58ce3938763502017e13fb9e8fdd2c9` passes in 490.564s: every 4,194,304 mask in debug,
release and official coverage, zero retries, 940 core / 312 ordinary server tests and two doctests
per profile. Coverage is 17,847/19,553 lines (91.2750%) at the 90% floor.
[Distinct Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40370445863)
pass, including the real socket-pressure cases. Integrated image source
`56639524da946f6d3cab3185fc58df045c766a1c` preserves the tested core/server/API inputs.
Unchanged v0.3.70 web and dependency acceptance are reused.

Accepted image is `sha256:fe1dbe95746a3a608e4070c4045d0d725ff606209f76c336301d7503fedff8ca`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight fresh Compose cases pass in
35.4s and eight Kubernetes cases in 37.7s, with exact-image/security/probe/resource checks.
Temporary acceptance projects and namespaces are removed.

Localhost:8085 serves the accepted image. Health/readiness, runtime security, CSP/cache and exact
HTML/JavaScript/CSS/atlas equality with v0.3.70 pass. Control is
`.local-checkpoints/runtime/compose-v0.3.71.yaml`; `rollback-v0.3.70.yaml` retains the prior image.
There is no gameplay, protocol, artwork or migration change. Distribution remains source
prereleases; measurement profiles are test-only and do not add production endpoints.

## v0.3.70 — 2026-10-26

[v0.3.70](https://github.com/liveweird/grenadiers/releases/tag/v0.3.70) derives game-copy types
from the canonical locale registry, returns stable raw copy objects and uses public i18next
interpolation. Existing EN/PL text, fallback, timers and stack notation remain unchanged.
Literal templates carry typed interpolation names; widened dynamic strings retain their dynamic
parameter map rather than claiming compile-time placeholder validation.

Independent review is clear. Full web acceptance at
`cdd6226ea6fb990f210ac14ac119045336a05443` passes 6,817 tests / 92 files in 226.02s with
91.07/89.40/90.12/93.48% coverage; gzip JavaScript/CSS is 337,939/36,695 bytes. TypeScript, lint,
contract drift, dead-code and repository checks pass. Integrated source
`1890db16d9f4eec5190390fe664a8897733d167b` preserves the tested web/assets/API inputs. Backend,
exhaustive Rust, Linux and dependency acceptance reuse v0.3.68/v0.3.69 unchanged inputs.

Accepted image is `sha256:8f913babc354d219c9d7b2a69669fc79f6e65f2981314a5fcbbeacee8c6a33bc`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight native Compose cases pass in
35.7s and eight Kubernetes cases in 36.3s, with exact image/security/probe/resource checks.
A Polish 320px responsive locale switch and native create-room smoke also pass. Temporary
acceptance projects/namespaces are removed.

Localhost:8085 serves the accepted image with verified health/readiness, CSP/cache and exact
served HTML/JavaScript/CSS/atlas bytes. Control is `.local-checkpoints/runtime/compose-v0.3.70.yaml`;
`rollback-v0.3.69.yaml` retains the previous image. There is no gameplay, wire, artwork or migration
change. Distribution remains source tags and GitHub prerelease notes.

## v0.3.69 — 2026-10-26

[v0.3.69](https://github.com/liveweird/grenadiers/releases/tag/v0.3.69) replaces 24 repeated
Compose seed overlays with `e2e/compose/fixture.yaml`. Set `GRENADIERS_FIXTURE_SEED_FILE` to the
exact seed path; the read-only bind rejects missing files instead of creating host directories.
Feature seeds, browser acknowledgements, scenario identities and base-only teardown remain.
See [fixture usage](../deployment.md).

Independent review and repository/workflow contracts pass. The 96 rendered-model comparisons
cover 24 features, default/custom environment and persistence off/on; the only intentional
model difference is `create_host_path: false`. Missing/unset/empty selectors fail safely.
Integration preserves the v0.3.68 backend and v0.3.65 web inputs; their full Rust, exhaustive,
Linux, web, coverage and dependency results are reused without duplicate suites.

Accepted image source is `137bdd68a4cae1d751bc98183c5fb5b9e1829ae0`, image
`sha256:9eade53a5b910f55627d6ef8f8d06a726698e7879316e41242142122e0266c95`.
Its OS layer was refreshed for the newly reported OpenSSL fix. Trivy 0.74.0 reports zero fixable
HIGH/CRITICAL findings. Fresh exact-image Compose acceptance passes eight default cases in 35.4s
and Heart, Kick, Cloak and Oil cases through the new shared overlay with exact read-only mounts.
Eight Kubernetes cases pass in 36.5s with exact-image/security/probe/resource checks. Temporary
Compose projects and Kubernetes namespaces are removed. Earlier runs using superseded overlays
are retained separately and are excluded from these shared-fixture results.

Localhost:8085 serves the accepted image; runtime/security, health/readiness, CSP/cache and exact
served HTML/JavaScript/CSS/atlas checks pass. Control is
`.local-checkpoints/runtime/compose-v0.3.69.yaml`; `rollback-v0.3.68.yaml` retains the prior image.
No gameplay, protocol, artwork or migration changes. Distribution remains source prereleases.

## v0.3.68 — 2026-10-26

[v0.3.68](https://github.com/liveweird/grenadiers/releases/tag/v0.3.68) makes replay recording
explicitly opt-in for test fixtures. Production rooms reserve no journal vector and skip replay-only
capture work; live snapshots, replay compatibility, reconnect and exhaustive assertions remain.
The allocation observation is 3,840,000 fewer reserved vector bytes per default room, not an RSS
or throughput claim; see [the measurements](replay-allocation.md).

Independent review is clear. Full Rust acceptance at
`6c01cc4c97324ddef87e588a1bdbe8fb1deea135` passes in 431.697s: every 4,194,304 mask in debug,
release and official coverage, zero retries, 940 core / 309 ordinary server tests and two doctests
per profile. Coverage is 17,833/18,933 lines (94.1900%), above the 90% floor. Integrated image
source `6f443a2056b91b713b5fd45d3e2e8de9a7e40056` preserves those core/server/API inputs.
[Distinct Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40369574894)
pass. Unchanged web, dependencies and audits reuse v0.3.65/v0.3.67 acceptance.

The runtime OS layer was refreshed after a current scan found fixable OpenSSL findings in the
older candidate. The replacement image is
`sha256:0bb13e287162a7ba79937dba4f95dd161bb343248ea4e0cbf026f9c74c2d5fc3`;
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight fresh native Compose cases pass
in 35.3s on this exact replacement. Eight fresh Kubernetes cases pass in 36.9s, including exact-image, security, readiness, probe
and resource checks; the temporary namespace is removed.

Localhost:8085 serves the replacement image. Image identity/security, health/readiness, CSP/cache
and served HTML/JavaScript/CSS/atlas bytes match the accepted web artifacts. Control is
`.local-checkpoints/runtime/compose-v0.3.68.yaml`; `rollback-v0.3.67.yaml` retains the previous
release. There is no migration or public replay export. Source tags and GitHub prerelease notes
are the distribution; no production registry is configured.

## v0.3.67 — 2026-10-26

[v0.3.67](https://github.com/liveweird/grenadiers/releases/tag/v0.3.67) adds named admission
projection and personal-metadata inputs for protocol v8–v11. Legacy Rust wrappers remain
available, and the live caller uses the named input without changing wire validation, ownership,
ordering, privacy, schemas or gameplay behavior. The existing v0.3.65 web build is reused.

Fresh source-bound Rust acceptance at `46a1e3320e2ecf5e8c6291884d1304d0dfaa6d22` completes every
4,194,304 mask in debug, release and official coverage in 432.208s with zero retries. The ordinary
profiles each pass 940 core / 307 server tests plus two doctests. Coverage is 17,809/18,909 lines
(94.1827%) at the 90% floor. Integrated source `43dcd78050d5a306b8822b62a17728cb56c8c926`
has identical core, server and API inputs. Full v0.3.65 web acceptance is reused: 6,812 tests / 91
files with 91.10/89.47/90.12/93.47% coverage. Dependency inputs and accepted audits are unchanged.
[Distinct Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40256918030)
pass on the final candidate.

Accepted image source is `43dcd78050d5a306b8822b62a17728cb56c8c926`; its image is
`sha256:0845bc797cc6082e9c331f9d0b0c871695c13e206cdcc8ddb1b9bf525f9c7f84`.
The pinned image scan reports zero fixable HIGH/CRITICAL findings. Eight fresh native Compose
cases plus Invisibility Cloak and Oil scenarios pass. Eight Kubernetes cases pass in 36.6s,
including exact-image, runtime-security, readiness, probe and resource checks; the temporary
namespace is absent afterward. An initial attempt passed a bad helper argument and stopped before
any test ran; the corrected acceptance above passed without product retries.

Localhost:8085 serves the accepted image. Exact image, runtime security, health/readiness,
CSP/cache and served HTML/JavaScript/PNG equality with the accepted unchanged web artifacts pass,
plus one fresh native authentication case in 364ms. Release-documentation changes preserve Docker
and application inputs. Control is `.local-checkpoints/runtime/compose-v0.3.67.yaml`;
`rollback-v0.3.66.yaml` retains the prior image. Temporary acceptance resources are removed.
There is no migration.

## v0.3.66 — 2026-10-26

[v0.3.66](https://github.com/liveweird/grenadiers/releases/tag/v0.3.66) derives reserved item
identities, bits, pickup declarations and total mappings from one private 22-item specification.
Generation order, activation subsets, replay values, public schemas and gameplay behavior remain
unchanged. The existing v0.3.65 web build is reused.

Independent review is clear. Fresh source-bound Rust acceptance at
`ba7d6eea4728f4feae6c60ddb67c45451c39b8c0` completes every 4,194,304 mask in debug, release and
official coverage in 429.862s with zero retries. The ordinary profiles each pass 934 core / 307
server tests plus two doctests. Coverage is 17,615/18,793 lines (93.7317%) at the 90% floor.
Rustfmt, Clippy, repository checks and
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40240422294) pass.
Full v0.3.65 web acceptance is reused: 6,812 tests / 91 files with
91.10/89.47/90.12/93.47% coverage.

Integrated image source `c244193ae048f3dea2864bd120511be60981bb09` preserves those tested
core, server and API inputs. Its accepted image is
`sha256:d4906dd81f0c86803e8d7a96bd890723ec180718d9cc015c286d7d94ebfc941d`.
Eight fresh native Compose cases and eight Kubernetes cases pass in 36.1s and 36.9s,
respectively, including exact-image, runtime-security, origin, readiness/probe/resource and
gameplay checks. Heart, Kick, Invisibility Cloak and Oil browser scenarios pass. The pinned image
scan reports zero fixable HIGH/CRITICAL findings.

Localhost:8085 serves the accepted image. Runtime security, health/readiness, CSP/cache and exact
served-artifact checks pass, plus one fresh native authentication case. An initial attempt found
missing E2E dependencies before any case ran; after installing the declared dependencies, the
fresh acceptance above passed without product retries. Release-documentation changes preserve
Docker and application inputs and skip duplicate CI. Control is
`.local-checkpoints/runtime/compose-v0.3.66.yaml`; `rollback-v0.3.65.yaml` retains the prior image.
Temporary acceptance resources are removed. There is no migration.

## v0.3.65 — 2026-10-26

[v0.3.65](https://github.com/liveweird/grenadiers/releases/tag/v0.3.65) names recurring UI color
roles in `App.css`. All 252 existing resolved declarations, selectors, dimensions and visual
values remain unchanged; distinct one-off shades remain explicit. No artwork or gameplay changes.

Independent review checks resolved values, scope, undefined variables and cycles. Full web
acceptance at `9e4b020cada15c5e36f2e2dd297e4cb7eecc437e` passes 6,812 tests / 91 files in
226.72s with 91.10/89.47/90.12/93.47% coverage. JavaScript/CSS gzip sizes are 337,921/36,695 bytes,
within existing budgets. Repository, TypeScript, lint, drift and dead-code checks pass.

Integrated image source `0a46613dde32585dcf2a08faea98a271d2e10db9` preserves that tested web tree
and the v0.3.64 backend. Accepted image is
`sha256:6aed11a165885225691ae10b70ab349999853a7c14e398a38d12b65b4b90b596`.
Eight native Compose cases and eight Kubernetes cases pass, with exact-image/runtime security,
origin, readiness/probes/resources and gameplay checks. The pinned image scan reports zero
fixable HIGH/CRITICAL findings. Reuse v0.3.64 backend/exhaustive/Linux and dependency acceptance;
there is no distinct Rust behavior to recheck remotely.

Localhost:8085 serves the accepted image. Exact served HTML, JavaScript, CSS and atlas bytes match
the tested build; CSP/cache, health/readiness and runtime security pass, plus one fresh native
authentication case. Release-documentation changes preserve Docker inputs and skip duplicate CI.
Control is `.local-checkpoints/runtime/compose-v0.3.65.yaml`; `rollback-v0.3.64.yaml` retains the
prior image. Temporary acceptance resources are removed. There is no migration.

## v0.3.64 — 2026-10-26

[v0.3.64](https://github.com/liveweird/grenadiers/releases/tag/v0.3.64) adds a validated,
dry-run-first release preparation helper and Cargo caches for container builds. The helper updates
aligned Cargo versions and dated changelog entries without running Git, publication or checks.
BuildKit reuses locked registry/Git caches and a platform-specific target cache; touching the
production crate roots forces changed workspace code to rebuild even when source mtimes are old.
The binary is copied outside the cache mount. Static OCI metadata identifies the project.
The existing bundled build frontend and runtime security policy are retained.

Independent helper/cache review is clear. All fourteen helper cases pass on macOS and Linux;
repository checks pass. Actual cold, older-mtime mutant and restored-source builds execute the
compile step: the mutant changes health to its probe value, and restoring source reproduces the
original binary. These are cache-correctness observations, not capacity or performance guarantees.

Accepted image source `332e3608a64cf9832e62afd6c80a7e9dcb2c7f49` builds
`sha256:1c6bd4094db67619c4f2c77c234bbaee34e5b4cccc17e5cb49bd8f38b0d2e27a`.
Eight native Compose cases and eight Kubernetes cases pass, including actual image identity,
startup/readiness, security, origins, probes/resources and browser gameplay. The pinned image
scan reports zero fixable HIGH/CRITICAL findings; one additional post-promotion native
authentication case passes. [Linux units](https://github.com/liveweird/grenadiers/actions/runs/40245664447) pass.

Application sources, contracts, web assets and external dependency versions are unchanged from
v0.3.63. Reuse that source's full Rust/exhaustive acceptance and dependency audits, plus the
unchanged v0.3.62 web acceptance. Release-documentation edits do not change Docker inputs.
Localhost:8085 serves the accepted image, with exact HTML/JavaScript/atlas equality and runtime,
CSP/cache, health/readiness checks passing. Control is
`.local-checkpoints/runtime/compose-v0.3.64.yaml`; `rollback-v0.3.63.yaml` retains the prior image.
Temporary acceptance resources are removed. There is no migration.

## v0.3.63 — 2026-10-25

[v0.3.63](https://github.com/liveweird/grenadiers/releases/tag/v0.3.63) separates authentication
from the nine sequenced commands before dispatch. Authenticated dispatch now accepts a private
sequenced type, preserving high-water advancement before semantic checks, error correlation,
quotas, admission ownership and every close sequence. Gameplay and diagnostics share the same
five-second per-frame send policy; whole final-delivery sequences retain their independent outer
budget. No wire, schema, replay, dependency or client behavior changes.

Independent review and focused real-socket regressions pass, including all nine pre-auth command
variants, semantic-failure sequence consumption, reauthentication closure and distinct counters.
Fresh source-bound Rust acceptance at `c8cc672812ea04d74a074a9570c9f473bd91dd62` completes every
4,194,304 mask in debug, release and official coverage in 428.571s with zero retries. Each ordinary
profile passes 930 core / 307 server tests plus two doctests; four existing performance cases
remain ignored. Coverage is 17,514/18,719 lines (93.56%) at the 90% floor. Rustfmt, Clippy,
runner/repository checks and [Linux units](https://github.com/liveweird/grenadiers/actions/runs/40237196707) pass.

Integrated image source `c50773d764ea0fd897de809de746719f9a7cde8e` has identical Rust inputs and
the unchanged v0.3.62 web tree. Accepted image is
`sha256:efa9be5a847ed7334388b1129a6be980e5df5fe25c72fc3d7d7620bc9dfc87f9`.
The image passes eight native Compose cases and eight Kubernetes cases, with exact image,
security/probes/resources/origin/CSP/cache and readiness checks. A fresh pinned image scan reports
zero fixable HIGH/CRITICAL findings. Reuse v0.3.62 web acceptance and unchanged dependency audits.

Localhost:8085 serves the accepted image. Runtime security, health/readiness, CSP/cache and exact
HTML/JavaScript/atlas equality with accepted unchanged assets pass, plus one fresh native
authentication smoke. Temporary acceptance resources are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.63.yaml`; `rollback-v0.3.62.yaml` retains the prior image.
There is no migration. Core admission argument grouping and other remaining review work stay
separate; CPU implementation remains deferred.

## v0.3.62 — 2026-10-25

[v0.3.62](https://github.com/liveweird/grenadiers/releases/tag/v0.3.62) removes 120 copied mock
lines from Diarrhea, Slow Foot and Reverse Controls scene tests. All three reuse the existing
Vitest-safe Pixi mock; test bodies, titles and assertions remain unchanged. Specialized renderer
and atlas mocks remain intentional. Production behavior, protocols and dependencies are unchanged.

Fresh full web acceptance at `50e57080aaf90fc03f2675f940be3d4adac9ca95` passes 6,812 tests /
91 files in 307.97s, with 91.10/89.47/90.12/93.47% coverage and 337,937/36,505-byte gzip JS/CSS.
Focused tests, lint, TypeScript, Knip and repository checks pass. Integration of v0.3.61 changes
no tested web files. Application image inputs match `ea90eaf91390db05dfdc0398be0328e1ccc7bede`;
publication adds documentation. Accepted image is
`sha256:a5025d0406d61cdaf1bfd6cd3800a7440f72b39c04c003c5dd3b4a6460ff11af`.

Rust/runtime sources match v0.3.61 apart from product-version metadata, so reuse its complete
source-bound 4,194,304-mask acceptance (93.49% lines), distinct Linux units, 16 native cases
and five startup probes. Dependency inputs also reuse the existing Cargo/npm audit results.
A fresh pinned exact-image scan reports zero fixable HIGH/CRITICAL findings; the project-owned
Trivy cache avoids repeated database downloads without bypassing updates or failures.

Localhost:8085 serves this image. Health/readiness, runtime security, CSP/cache and exact
HTML/JavaScript/atlas equality with the accepted v0.3.60 web artifacts pass, along with one
fresh native admission/room smoke. Control is `.local-checkpoints/runtime/compose-v0.3.62.yaml`;
`rollback-v0.3.61.yaml` retains the previous image. No migration is required. Remaining agreed
Claude review work still precedes CPU implementation.

## v0.3.61 — 2026-10-25

[v0.3.61](https://github.com/liveweird/grenadiers/releases/tag/v0.3.61) centralizes production
startup in a typed configuration. Invalid address/origin settings fail with value-free errors;
OS-native paths, proxy → seed → lazy database validation, existing AppState constructors and
invalid-log fallback remain supported. An explicitly empty origin list still denies browser
access and now emits one warning. Docker logging matches Compose/Kubernetes; the
[defaults table](../security.md#startup-configuration) distinguishes intentional development and
container values. No gameplay, protocol, replay or dependency changes are introduced.

Tested application source is `29093f3a50297f97cba0033a0f34b2244872c98b`; publication adds only
documentation. Localhost:8085 serves accepted image
`sha256:a22899d0d0e0c45063bf658c089ce55428ac02f37d6417a4270705cc02c42696`.
Independent review, 12 focused startup cases and nine origin cases pass. Fresh source-bound Rust
acceptance completes every 4,194,304 mask in debug, release and official coverage in 499.886s,
with zero retries. Ordinary profiles pass 930 core / 304 server tests and two doctests; four
existing performance cases remain ignored. Coverage is 17,388/18,598 lines (93.49%) at the 90%
floor. Rustfmt, Clippy, runner fixtures, repository checks and
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40234533129) pass.

The exact image passes eight native browser cases through Compose and eight through Kubernetes.
Five actual-image startup probes verify typed/redacted address, database, seed and proxy errors,
plus healthy startup/log fallback, one empty-origin warning and browser-session denial.
A fresh pinned Trivy scan reports zero fixable HIGH/CRITICAL findings. Web/contracts/dependency
inputs are unchanged; reuse v0.3.60 web acceptance and v0.3.57 Cargo/npm audits. Served HTML,
JavaScript and atlas bytes match the accepted v0.3.60 artifacts exactly.

Initial temporary checks needed corrections: Compose override setup, browser sandbox permission,
the absent host Trivy binary (use the pinned container), and a helper expecting API cache policy
for HTML. These were setup failures; final checks use documented policies and unchanged product
assertions. Failed outputs or tool excerpts are retained; no hidden retries were added.

Promotion passes image/runtime security, health/readiness, CSP/cache/artifact equality and one
additional native room-admission smoke. Temporary acceptance resources are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.61.yaml`; `rollback-v0.3.60.yaml` retains the prior image.
Restarts still end process-local matches. The review status now separates shipped fixes from
remaining agreed work; CPU implementation waits for that work to finish.

## v0.3.60 — 2026-10-25

[v0.3.60](https://github.com/liveweird/grenadiers/releases/tag/v0.3.60) enables all 22 configurable
items in newly created live rooms, including curses and Oil. All on/All off edit only the item
draft; numeric settings, editing locks and explicit Apply remain intact. Restore match defaults
enables all available items; Classic disables optional items. Existing rooms, rematches and
historical replay defaults retain their settings. Live replay headers record explicit defaults;
no wire, rules, generator or replay format version changes.

A responsive own-player bar shows upgrade totals, equipment/weapon slots, every effect timer,
all Slow Foot stack timers, freeze and damage grace. Counts come from authoritative snapshots;
an opponent's Stopwatch freeze has no invented countdown. Recent own pickups, including basic
upgrades and Teleporter, are announced once, with reconnect/round/death boundaries respected.
Oil remains a world object. English/Polish text and icons reuse the current sheet. See
[settings](../game/settings.md) and [browser presentation](../game/browser.md).

Application inputs match `c5b668f9fddf8cf95a582c499192a92f658a0e95`; later source changes only E2E
assertions, artifact ignores and documentation. Accepted image is
`sha256:ab7cf594b833c7d5c9b3c7dec15a34292dc22854affa00a949b3a309b7df1e8b`.
Independent server, web and scenario reviews are clear. Corrections prove the active collector's
Polish 320px bar and expose remaining grace time in its accessible name. Own/rival numeric and
label-geometry assertions are preserved after moving the own bar.

Fresh Rust acceptance at clean source `5e4743d8d868ee86cf7a078aa88af150688d8d1b` has identical
Cargo/core/server inputs. All 4,194,304 replay masks pass in debug, release and official coverage
in 544.996s, with 32 complete partitions per profile and zero retries. Ordinary profiles pass
930 core and 296 server tests plus two doctests; four existing performance cases stay ignored.
Coverage is 17,290/18,492 lines (93.50%) at the 90% floor, with helper exclusions intact. The
initial sandbox denied local listener binds; the unchanged suite passed with socket permission.
Rustfmt, Clippy and the runner fixture checks pass.

The fresh web gate passes 6,812 tests / 91 files in 292.82s. Coverage is
91.10/89.47/90.12/93.47% above the existing floors; gzip JavaScript/CSS are 337,937/36,505 bytes.
API/schema/standalone drift, lint, unused-code, build, repository and full E2E static checks pass.
The pre-review coverage attempt was interrupted for the accessible-name fix; this final run
uses the corrected source. Dependency inputs are unchanged; reuse v0.3.57's fresh Cargo-deny
and both zero-finding npm audits. A fresh pinned Trivy scan of this exact image reports zero
fixable HIGH/CRITICAL findings under the existing policy.

Exact-image native Chromium passes 32 Compose cases: all 22 item scenarios, generated upgrades,
a complete classic match/rematch, and eight default cases. Eight more default cases pass
through Kubernetes (40 total), with zero configured retries. Collector-side Polish 320px Heart
capture, pickup feedback, replacement/consumption, expiry, privacy and own/rival geometry are
covered. The [distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40232475529)
pass. Two temporary Kubernetes helper preflights failed (resource path and omitted empty env
values) before browser execution; corrected helpers pass, and failed logs remain retained.

Localhost:8085 serves the accepted image. Promotion verifies runtime security, health/readiness,
CSP/cache and exact HTML/JavaScript/atlas equality with the image/local build; one additional
native configured-rules smoke verifies fresh defaults and EN/PL bulk controls. Temporary
Compose projects, namespaces and port-forwards are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.60.yaml`; `rollback-v0.3.59.yaml` retains the prior image.
No migration is required; restarts end process-local matches. The accepted CPU design is next;
this release does not implement single-player mode or finish every quality-review follow-up.

## v0.3.59 — 2026-10-25

[v0.3.59](https://github.com/liveweird/grenadiers/releases/tag/v0.3.59) generates browser item
catalogs from each v7–v11 schema's own `ItemId` enum. Live settings and validation reuse those
tuples; retained validators preserve their narrower activation subsets. V2–v6 generated output,
schema JSON, standalone validators, rules, generation, replay identities and dependencies are
unchanged. An E2E privacy assertion now checks the actual unique secret and redaction markers
rather than rejecting a harmless `/private/tmp` checkout path; sanitizer behavior is unchanged.

Application inputs match `473d1e6e889cfecd5c08178f3b881ca74d775fae`; later source adds only the
E2E test correction and release documentation. Accepted image is
`sha256:946a5f6fbab0c8b573c66badff34157b445cc57cc65798d519bc7fdc6a303fc0`.
Independent catalog review is clear. Fresh full web acceptance passes 6,799 tests / 88 files
in 314.30s; coverage is 90.94/89.49/89.85/93.30% above the unchanged floors. Production gzip
JavaScript/CSS sizes are 336,404/36,292 bytes. All retained generation/standalone drift checks,
API conventions, lint, unused-code, repository and corrected full E2E static checks pass.

Rust behavior is unchanged: reuse v0.3.58's complete 4,194,304-mask debug/release/official-coverage
acceptance (93.50% application lines), plus its shutdown and separate graphics diagnostics.
Dependency inputs are unchanged, so v0.3.57's fresh Cargo-deny and both zero-finding npm audits
remain applicable. The exact new image receives its own scan and runtime acceptance below.

Fresh pinned Trivy 0.74.0 acceptance reports zero fixable HIGH/CRITICAL vulnerabilities.
Native Chromium passes eight default cases through Compose (36.4s) and eight through Kubernetes
(40.3s), with unchanged assertions and no configured retries. The first Kubernetes setup used an
8085 origin allowlist with a browser on 18086; correcting only that isolated environment fixed
session admission. Its original failure log was overwritten; the verifier's command-output
excerpt is retained as a limitation. The initial scan lacked Docker socket access; the corrected
exact-image scan is authoritative. Health/readiness, runtime security and CSP/cache headers pass.

Localhost:8085 now serves that accepted image. Promotion verifies exact HTML/JavaScript/atlas
bytes against the local build and image, runtime security, headers and one additional native
realtime smoke. Temporary acceptance services, namespaces and forwards are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.59.yaml`; `rollback-v0.3.58.yaml` retains the previous
image. No migration or gameplay configuration change is introduced by this release.

## v0.3.58 — 2026-10-25

[v0.3.58](https://github.com/liveweird/grenadiers/releases/tag/v0.3.58) bounds HTTP serving and
application drain with one five-second deadline measured from the shutdown signal. Incomplete
HTTP requests cannot postpone the game/session drain budget. Graceful final state, socket
closure and startup/I/O errors are preserved. On expiry, main returns and runtime teardown
releases remaining asynchronous I/O; this is an asynchronous budget, not a hard real-time bound.
The release also adopts the merged Napoleonic sheet and refreshed gameplay captures without
changing manifest slots, anchors, gameplay or texture budgets.

Application inputs match tested source `e758ad249a066854c35d200cf8e7ce866d8bda74`; the release
source adds documentation. Localhost:8085 serves accepted image
`sha256:cc8f5d0308b8e1d6b774f20dfcee20fdb49691c080e95ac95624d6c2bd3ce0af`.
Independent code, timing and artwork reviews are clear. Nineteen focused shutdown tests pass;
a deliberate nine-second mutant fails all three deadline cases. A real Linux container with
incomplete headers held until process exit terminates successfully in 5.127s with one deadline
warning. A graceful room/diagnostic probe exits in 0.115s, publishes `serverStopped` and closes
member, unseated and diagnostic sockets with code 1001. No frames or credentials are recorded.

Fresh source-bound Rust acceptance completes all 4,194,304 replay masks in debug, release and
official coverage in 433.085s, with zero retries. Each ordinary profile passes 930 core and
293 server tests plus two doctests; four existing performance tests remain ignored and the
heavy case runs separately across 32 complete partitions. Coverage is 17,278/18,480 lines
(93.50%) at the unchanged 90% floor. Rustfmt, Clippy, repository checks and the
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40228900957) pass.
The first runner-fixture setup failed from an inherited target-directory override; the unchanged
fixture suite passed with its isolated environment before the complete runner passed.

The fresh web gate passes 6,791 tests / 88 files with 90.95/89.49/89.80/93.32% coverage.
JavaScript/CSS gzip sizes are 336,430/36,292 bytes within their budgets; contract, standalone
validator, lint and unused-code checks pass. Dependency inputs are unchanged from v0.3.57,
whose fresh Cargo-deny and both zero-finding npm audits are reused. A fresh pinned Trivy scan
of this exact image reports zero fixable HIGH/CRITICAL findings under the existing policy.

Exact-image Chromium acceptance passes nine Compose cases (eight default plus Oil Barrel)
and eight Kubernetes cases. Separate diagnostics under the enforced browser policy pass
14 artist, terrain, atlas and isolated renderer-lifecycle cases. These are representative
checks, not a rerun of every configured browser suite. Post-promotion runtime/security,
health/readiness, CSP/cache, exact HTML/JavaScript/atlas equality and one additional browser
smoke pass. Temporary acceptance resources are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.58.yaml`; `rollback-v0.3.57.yaml` retains the previous
image. No migration is needed; restarts still end process-local matches. Keep external stop grace.

## v0.3.57 — 2026-10-25

[v0.3.57](https://github.com/liveweird/grenadiers/releases/tag/v0.3.57) gives shared Rust item/
effect DTOs, common item projections and bounded-number input neutral protocol ownership.
V7–V11 retain their public names and shared type identity. Reserved bits, activation subsets,
validation stages/errors, generation order, schema JSON and replay identities are unchanged.
Four independent regressions pin identity, retained schemas, serialization and numeric boundaries.
Application inputs match `a94aaddea8211e262c1d0acf3cf3103db797db05`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:170e5d071feeabf6d62c55dcb1dce6b6191d7bcdbeccee511e1c9a1c8a72b9bf`.

Independent review is clear; moved bodies and mappings match baseline after path relocation.
Fresh source-bound Rust acceptance passes all 4,194,304 replay masks in debug, release and
reported coverage in 8m06s, without retries. Each ordinary profile passes 930 core and 287
server tests plus two doctests; four existing performance tests remain ignored, and the heavy
case runs separately across 32 complete partitions. Official line coverage is 17,248/18,445
(93.51%) at the unchanged 90% floor, with helper exclusions intact. Clippy/rustfmt, repository
checks and the [distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40222285097) pass.

The fresh web gate passes 6,791 tests / 88 files with 90.95/89.49/89.80/93.32% coverage.
JavaScript/CSS gzip sizes remain 336,437/36,292 bytes within their budgets. All retained
contract/standalone drift checks pass. Fresh Cargo-deny, both zero-finding npm audits and the
pinned exact-image scan pass; the latter reports zero fixable HIGH/CRITICAL findings.

The exact image passes twenty native Chromium cases without retries: eight default cases
through Compose, eight through Kubernetes, and seeded Landmine, Heart, Cloak/reconnect/expiry
and Oil Barrel scenarios through Compose. Post-promotion checks pass runtime/image/security,
health/readiness, enforced CSP/cache headers, exact served HTML/JavaScript/atlas equality and
one additional native diagnostic/gameplay smoke. Claude's redraw is unchanged. Temporary
acceptance services and port-forwards are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.57.yaml`; `rollback-v0.3.56.yaml` retains the prior image.
No migration is needed; restarts still end process-local matches.

A separate quiet probe of the unchanged shutdown path in v0.3.56 confirms a follow-up issue:
an incomplete HTTP request keeps the process running beyond seven seconds after SIGTERM,
before the application drain timer is reached. This release does not fix that ordering;
bounding total shutdown is the next isolated fix. Keep the configured external stop grace.

## v0.3.56 — 2026-10-25

[v0.3.56](https://github.com/liveweird/grenadiers/releases/tag/v0.3.56) shares the common
Playwright configuration across 24 simple gameplay wrappers. Public config names and commands,
artifact paths, worker/retry limits and base browser/environment settings are unchanged.
Audio keeps its 60-second timeout; the other 23 keep 90 seconds. Trace capture remains disabled.
Specialized configurations and the pickup/matches/reconnect chain remain separate.
Application inputs match `d7849292fed30d728bffcf0c106aff7d4a1cf6f8`; the source tag adds release
documentation. Localhost:8085 serves accepted image
`sha256:5f1e0627b21d18162c6b12e53516a934ea4d370a0e1a285b75bff2d551a6e603`.

Independent review, fresh types/lint/unused-code/scenario/repository checks and the wired
configuration regression pass. Complete before/after exports match in all 96 comparisons
(24 wrappers across four simulated environment/platform profiles), using pinned Playwright
`defineConfig` and device defaults, including undefined/symbol keys. This source evaluation
preserves Linux ANGLE and capture-profile settings; it is not native Linux browser execution.

The exact image passes nineteen Chromium cases without retries: eight default cases through
Compose, eight through Kubernetes, plus controls, natural headed audio and a seeded Landmine
scenario through Compose. The pinned image scan reports zero fixable HIGH/CRITICAL findings.
Post-promotion checks pass image/runtime security, health/readiness, CSP/cache headers, exact
served HTML/JavaScript/atlas bytes and one additional native diagnostic/gameplay smoke.
Claude's redraw is unchanged. Temporary acceptance services are removed.

Production Rust, API, frontend, asset and deployment sources are unchanged; Cargo changes only
local package versions. Reuse ordinary Rust/Linux results from v0.3.54, frontend/dependency/
renderer evidence from v0.3.53 and exhaustive replay/coverage measured at `055d1d2`
(all 4,194,304 masks, 93.15% line coverage at the 90% floor). These results are not claimed as
freshly rerun for configuration reuse. The new production image is built and accepted separately.

Durable control is `.local-checkpoints/runtime/compose-v0.3.56.yaml`; `rollback-v0.3.55.yaml`
retains the previous accepted image. No migration is needed; restarts end process-local matches.
The testing index/archive cleanup is included; its historical labels do not impose current gates.

## v0.3.55 — 2026-10-25

[v0.3.55](https://github.com/liveweird/grenadiers/releases/tag/v0.3.55) shares native responsive
browser-context setup across 26 gameplay scenarios. Two independent contexts still start
concurrently and return in desktop/mobile order. Viewports, mobile reduced motion, pickup's
390px override, privacy observations, assertions and caller-owned cleanup remain unchanged.
Application inputs match `8b7bacb82de085988e1592a280c0119ba3a41bfa`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:a053abd96387ebe50f85b68aeecffcaa48160ccacb70a2f26f7808c1d7cee6d2`.

Independent review and fresh E2E types, lint, unused-code, repository and scenario checks pass.
The scenario gate includes the new exact-options, concurrency, reverse-resolution ordering and
native-identity regression. All 46 spec title inventories are unchanged; every migrated spec
is byte-identical outside its new import and equivalent setup block. Seventy-four mapped
scenarios remain registered; this static count is not a claim that all ran natively.

The exact image passes twenty Chromium cases without retries: eight default cases through
Compose, eight through Kubernetes, and four seeded Compose cases covering Landmine, 390px
pickups, recipient-private Cloak/reconnect/expiry and Invisible Bombs' live pixel/privacy window.
The pinned image scan reports zero fixable HIGH/CRITICAL findings. Post-promotion checks pass
health/readiness, runtime security, CSP/cache headers, exact served HTML/JavaScript/atlas bytes
and one fresh real-browser diagnostic/gameplay smoke. Claude's graphics are preserved.

Production Rust, API, frontend and asset sources are unchanged from v0.3.54; Cargo changes only
local package versions. Reuse v0.3.54 ordinary Rust/Linux results and v0.3.53 frontend/security/
renderer evidence. Exhaustive replay/coverage remains attributed to measured `055d1d2`
(all 4,194,304 masks, 93.15% line coverage at the 90% floor); none is claimed as freshly rerun
for test setup reuse. The production image is rebuilt and accepted independently.

Temporary acceptance services are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.55.yaml`, with `rollback-v0.3.54.yaml` retaining the
previous accepted image. No migration is needed; restarting still ends process-local matches.
Configuration factories and broader fixture sharing remain separate work.

## v0.3.54 — 2026-10-25

[v0.3.54](https://github.com/liveweird/grenadiers/releases/tag/v0.3.54) extends the source-size
guard to every current and future item test tree and aggregates simulation test declarations.
All 926 core test names and all fifteen production module declarations retain their identity.
Application inputs match `c72da79b41b8b8ca83b0396b9cd9f18ab882f77c`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:fd3d8bc815b6169208640b33df4690cb75b7ba98dbcd0d861d087f268760a829`.

Independent review is clear. The repository gate passes eight new filesystem fixtures plus
its existing workflow/dependency checks. Fresh ordinary debug and release runs each pass 926
core and 287 server tests plus two doctests; four existing performance tests remain ignored
and the exhaustive replay is excluded from these ordinary runs. Clippy, rustfmt and the
[distinct Linux unit job](https://github.com/liveweird/grenadiers/actions/runs/40218140208) pass.
The declaration-only change reuses exhaustive debug/release/coverage acceptance measured at
`055d1d2` (all 4,194,304 masks, 93.15% line coverage at the 90% floor). Unchanged frontend,
API, assets and server logic reuse v0.3.53 web, security and renderer evidence; these are not
claimed as freshly rerun v0.3.54 suites.

The exact new image passes eight Chromium cases through Compose and eight through Kubernetes,
with no retries. The pinned image scan reports zero fixable HIGH/CRITICAL findings. Promotion
checks pass health/readiness, image/security identity, CSP/cache headers, exact served
HTML/JavaScript/atlas equality and one fresh real-browser diagnostic/gameplay smoke. The
frontend bytes, including Claude's redraw, match v0.3.53. Temporary acceptance services are
removed; `.local-checkpoints/runtime/compose-v0.3.54.yaml` controls the live image and
`rollback-v0.3.53.yaml` retains the previous image. Matches remain process-local; no migration
is needed.

## v0.3.53 — 2026-10-25

[v0.3.53](https://github.com/liveweird/grenadiers/releases/tag/v0.3.53) adds bounded peer
admission, strict opt-in immediate-proxy identity, outstanding-session quotas and readable,
credential-free denial summaries, alongside the merged arena redraw. Application inputs match
`e1e10b0d889aa110c268bc17167b694423f6c8f2`; the source tag includes release documentation.
Localhost:8085 serves accepted image
`sha256:6035b215bd8ab428a749b9effa63261cc99b2bd4eee3d8bc69cb6c0dfe83ea7f`.

Full source-bound Rust acceptance at `055d1d2` passes every 4,194,304-mask case in debug,
release and official coverage at the 90% floor (7m45s, no retries). Measured application line
coverage is 17,197/18,461 (93.15%). A subsequent logging-target-only correction makes denial
summaries visible under the documented filter: final-source ordinary debug/release each pass
926 core and 287 server tests plus two doctests; clippy and rustfmt pass. The unchanged heavy
replay and coverage results remain attributed to their measured source, rather than claimed
as rerun after that literal change. Distinct Linux debug/release units pass at final source.

After artist PR #123 merges, Rust/Cargo/API inputs remain unchanged and the packaged server
binary is byte-identical to accepted `79a6527` (SHA-256 `13837b25c859c22a7d358c500e86cb304c94732c048823acd11385ae6438b97d`).
The affected full web gate passes 6,791 tests / 88 files at `e1e10b0`, with
90.95/89.49/89.80/93.32% coverage and unchanged floors. Compressed JavaScript/CSS are
336,437/36,292 bytes. Fresh Cargo-deny, both zero-finding npm audits, strict changed Kubernetes
checks and the final image scan pass; the image has zero fixable HIGH/CRITICAL findings under
the existing policy. Independent admission, log-filter and graphics integration reviews are clear.

The final image passes 26 Chromium cases without retries: twelve gameplay cases through
Compose (default eight, controls, audio, two reconnect), six HTTP/WS + HTTPS/WSS CSP cases and
eight through Kubernetes. Three relay/config regressions and fourteen separate enforced-policy renderer/artist cases pass,
including actual exports and an isolated active lifecycle cell. Separate real container TCP
checks prove untrusted forwarding is ignored, trusted malformed forwarding returns 400, sixteen
issued sessions exhaust only their minting peer with 429, and another peer remains eligible.
Actual default-filter summaries are visible and exclude the sentinel request value; unchanged server bytes preserve
that proof across the frontend-only merge.

Promotion health/readiness, exact runtime/image/security identity, real CSP/cache headers,
served HTML/JavaScript/atlas and a fresh browser diagnostic/gameplay flow pass. The served redraw
matches canonical SHA-256 `b057bafe7b32b76dd187fa44bf4bd6ebbb686b1aa0e8bbf8c5f633aea7ac7a6e`. Proxy trust is
empty for the direct local deployment. Temporary services, keys, worktree and branch are removed.
Durable control is `.local-checkpoints/runtime/compose-v0.3.53.yaml`; v0.3.52 remains for rollback.
Reload browser tabs; matches and quotas remain process-local. No database migration is needed.

These quotas are engineering defaults, not durable identity or a public deployment design.
Actual proxy addresses, overwrite behavior, restricted backend access, TLS and ingress limits
remain operator requirements. Claude's redraw is included; its manifest/frame geometry is unchanged,
and parity explicitly uses the packed procedural baseline while lifecycle tests load shipped art.

## v0.3.52 — 2026-10-25

[v0.3.52](https://github.com/liveweird/grenadiers/releases/tag/v0.3.52) enforces a compatible
browser content policy using static Pixi implementations and precompiled validators.
Application inputs match `578f581be1593b56ab589c6a8da2dce8a8e4fc8f`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:9b5918458c30fd8dc8e877c8430607f09dd773abdec52f407bc139e5b58ddb85`.

Full source-bound Rust acceptance passes all 4,194,304 masks in debug, release and official
coverage at the 90% floor, with no retries. Application line coverage is 16,394/17,634 (92.97%).
The parallel gate phase completes in 7m23s while the web gate and image build also run. Web
passes 6,790 tests / 88 files with 90.95/89.49/89.78/93.32% coverage and unchanged floors;
compressed JavaScript/CSS are 336,413/36,292 bytes, within budgets. E2E static checks, three wired
relay/config regressions, the pinned image scan and the distinct Linux Rust unit job pass.
The image has zero fixable HIGH/CRITICAL findings under the existing scanner policy.

Before server delivery, enforced policy trials pass fourteen renderer/artist cases and controls,
audio and both reconnect cases with zero observed violations. Final acceptance uses the actual
server image without relay policy injection: eight native cases through Docker Compose, eight
through Kubernetes and six HTTP/WS + HTTPS/WSS CSP cases pass, with no retries. Real page scripts
prove execution/connection/worker blocking; the frame test isolates CSP from XFO and includes
an attachment control. An earlier relay reset crash and no-test selector are retained as failed
attempts, not passes. Independent review is clear.

Promotion health/readiness/index and a fresh native browser diagnostic/gameplay flow pass.
Served HTML/JavaScript/atlas bytes and runtime security identity match the accepted artifact;
CSP is enforced, including on API responses. Temporary acceptance services are removed.
Durable control is `.local-checkpoints/runtime/compose-v0.3.52.yaml`; v0.3.51 is retained for
rollback. Reload existing tabs; matches remain process-local. No migration is needed.

The style policy explicitly permits Mantine inline styles; scripts do not allow inline
execution or string compilation. Browser acceptance covers Chromium, not an untested wider
matrix. Peer-keyed hosting admission and the separate artist redraw remain future releases.

## v0.3.51 — 2026-10-25

[v0.3.51](https://github.com/liveweird/grenadiers/releases/tag/v0.3.51) precompiles the live
browser validators, preserving schemas, complete errors and semantic/privacy checks.
Application inputs match `a7ac65b36592ad8871a1a78f64f4e9adcc24eb36`; the source tag includes
release documentation. Localhost:8085 serves the accepted image
`sha256:cb7be95e9b5c6c291d8c828579b3de5facff9efb466d5a61e4dcdcadd2c7c76f`.

The full web gate passes 6,789 tests / 88 files at `31219c0`, with coverage
90.95/89.49/89.78/93.32% (statements/branches/functions/lines). Later portability corrections
change generated comments only; 13 focused tests, lint, Knip, drift, TypeScript and fresh
production builds pass on the final generator. Exact artifacts match across checkout depths,
invocation directories and pinned Linux. Compressed JavaScript falls from 357,324 to 333,442
bytes; CSS remains 36,292 bytes, within unchanged budgets. Independent review is clear.

Actual Chromium development and production flows validate live diagnostic welcome/pong and
gameplay admission. Sixteen native cases pass without retries: the default eight each in
Docker and Kubernetes on the same image. Kubernetes retains base security, probes and resource
limits. Strict schema checks and the pinned image scan pass, with zero fixable HIGH/CRITICAL
findings under the existing scanner policy. Isolated acceptance services are removed.

Rust source, wire contracts and external dependencies are unchanged from v0.3.50. Reuse its
full exhaustive/coverage acceptance; Linux debug/release units additionally pass at `b366958`,
whose Rust inputs equal the final candidate. Do not treat these reused results as a new full
Rust run. Docker's clean dependency install reports zero npm vulnerabilities.

Promotion health/readiness/index are 200. Served HTML/JavaScript/atlas match the image; the
atlas still matches canonical source. Runtime hardening remains intact. Durable deployment
control is `.local-checkpoints/runtime/compose-v0.3.51.yaml`; v0.3.50 is retained for rollback.
Reload browser tabs after upgrading; matches remain process-local. No database migration is
needed. CSP enforcement and the separate artist redraw remain pending.

Runtime attribution correction recorded with v0.3.52: the eight Docker cases used direct
`docker run`; the separately verified localhost promotion used Docker Compose. The original
source tag remains unchanged.

## v0.3.50 — 2026-10-25

[v0.3.50](https://github.com/liveweird/grenadiers/releases/tag/v0.3.50) hardens transport,
room recovery, browser responses and contributor checks. Application inputs match tested source
`9122e330379f4ac1bb3e0edd236dcdc5538655f0`; the source tag identifies the complete release.
Localhost:8085 serves the exact accepted image
`sha256:76980168a517474e680d8e346b6b8c38240a1b534eb403ad919b347168d0f5c6`.

Full Rust acceptance covers all 4,194,304 masks in debug, release and official coverage with no
retries (6m40s). Application line coverage is 92.97%; that fresh 80%-gated measurement supports
raising the Rust floor to 90%, verified by reporter contract/real-LLVM tests. The patched web gate
passes 6,780 tests in 86 files at the raised 90% line / 88% other floors. API/E2E static checks,
Cargo-deny, zero-finding full/production npm audits and the pinned image scan pass; the image has
zero fixable HIGH/CRITICAL findings under the existing scanner policy.

Eighteen native cases pass on this image: sixteen Compose cases covering the default suite,
controls, audio, match progression, reconnect/deadline, Cloak, Stopwatch and Oil; two Kubernetes
reconnect cases retain the base pod security/probes/resources. Failed sandbox launches, the
socket timing regression and the stale Equipment-error assertion were corrected and are not
carried as passes. Fresh accepted suites use no retries.

Promotion health/readiness/index are 200; served HTML/JavaScript/atlas bytes match the image,
and atlas pixels match canonical source. UID65532, read-only filesystem, dropped capabilities
and no new privileges remain. v0.3.49 is retained for rollback. Runtime control files now live
in `.local-checkpoints/runtime/`, rather than `/tmp`. Reload existing tabs; active matches and
capabilities remain process-local. No database migration is required.

The artist redraw remains a separate pending change. Further CSP, hosting and structural work
is tracked in the [quality-review response](../quality-review.md).

## v0.3.49 published prerelease — 2026-10-25

[v0.3.49](https://github.com/liveweird/grenadiers/releases/tag/v0.3.49) is published from source
`4157d534553330b457871554d84e973823068798`, tree `dd6a844d841e76e254ee09b339a4e4c6577f982f`,
with remotely verified annotated tag object `1a8f141adbf6c18e372a1b6dd344475727664233`.
Localhost:8085 serves exact accepted image
`sha256:10aa8554f8f22f968a6bd1852a4c0d6fe43e63b365eafc8220e7fbd826dc4d21`.

Promotion exits zero; health/readiness/index return 200. Served HTML, JavaScript and the hashed
atlas PNG match container bytes; the PNG also matches canonical source SHA256
`f2392294ba67207cdaf264ec3343cf096de0281dc281aa4c8c3339ff312d407f`. UID65532, read-only
filesystem, dropped capabilities and no new privileges are retained. v0.3.48 remains the rollback
image. An initial local helper path typo stopped before any service change; the corrected helper
completed promotion and all post-deployment checks. Application inputs match accepted source
`ee664e5`; publication documentation does not require rebuilding the accepted image.

The artist handoff/export/preview and gameplay atlas integration are now complete. New artist
redraws remain separate work. The documented completed-icon pulse compositing is intentional.

See the [renderer guide](../game/art-atlas.md) and [acceptance record](verification.md#v0349-gameplay-atlas-acceptance--2026-10-25).

## v0.3.48 published prerelease — 2026-10-24

[v0.3.48](https://github.com/liveweird/grenadiers/releases/tag/v0.3.48) is published from source
`f6336810f4a539e5e001aa6b1733f445015ae76f`, tree `87a488cc69b45e61b40b8be340f7b09ea4347930`,
with remotely verified annotated tag object `372e84642c4e582c4a632508acb2deeaa9a89aba`.
The uploaded 229,324-byte artist handoff ZIP has verified SHA256
`1f374dc6a76a05b536decc9620bf43edd448d9cc242bf0b15d4b1e28eed4b135`.

Localhost:8085 serves exact accepted image
`sha256:751383fe17e23a6ae46e58006017a15862c6b0cf2fd993645bb2c78187b00cb8`.
Promotion exits zero; health/readiness/index return 200 and served HTML/JavaScript match container
bytes. UID65532, read-only filesystem, dropped capabilities and no new privileges are retained.
Oil v0.3.47 is preserved as the rollback image. Application inputs match tested source `cd369f4`;
subsequent gate/documentation changes do not require rebuilding that image. Gameplay adoption
remains v0.3.49; current gameplay continues using the existing code-generated drawings.

See the [artist guide](../../assets/art/README.md) and [acceptance results](verification.md#v0348-artist-handoff-acceptance--2026-10-24).

## v0.3.47 published prerelease — 2026-10-24

[v0.3.47](https://github.com/liveweird/grenadiers/releases/tag/v0.3.47) completes all 22 requested item types. Release source
`494f620cf5eb13f0f18d0bf13d91bc74291bb51b`, tree `613e1ccd86fa14e3a94af50707a23be4044d70d2`, is tagged by verified annotated
object `565165ae9b5ce2c968fbb83b3ce413b5f5a562e8`. Localhost:8085 serves the exact accepted image
`sha256:969ff363f77ad6dfc6e62fa5f98e4a704d017ad1186d43fe774fedc50fd7561e`. Promotion exits zero; health, readiness and index
return 200, and served HTML/JavaScript match container bytes. The application retains UID65532,
a read-only filesystem, dropped capabilities and no new privileges. Bracelet v0.3.46 is retained
as the rollback image. Reload older browser clients for gameplay v11.

## v0.3.46 published prerelease — 2026-10-24

[v0.3.46](https://github.com/liveweird/grenadiers/releases/tag/v0.3.46) publishes Bracelet as the twenty-first requested item from source
`7b4b47cfdd7dea9efec9d3c2f2fd61f2c232ce54`, tree `7816fe3d0a373523ffe5b4a8b54873dfce3e3b7a`, through verified annotated tag
`57b9b226a2d1cc07ebe1b8487348e89858678132`. Localhost:8085 serves the exact accepted image
`sha256:1460278198b6d604595bfbe4fb8278a85ab39725c43516bb8b55ab2e6a47f5bf`. Promotion exits zero; health, readiness and index
return 200, and served HTML/JavaScript match container bytes. The application retains UID65532,
a read-only filesystem, dropped capabilities and no new privileges. Punch v0.3.45 is retained
as the rollback image. No rebuild or repeated full CI was needed.

## v0.3.45 published prerelease — 2026-10-24

Source `5be42456cee80e06e2f138bdfd458ef821bb34c2` is published as
[v0.3.45](https://github.com/liveweird/grenadiers/releases/tag/v0.3.45) through annotated tag object
`0e6224643ce85bf8f0a77894dcb929ab2655e397`. It publishes Punch as the twentieth requested item.
Localhost serves the accepted image
`sha256:3f13b26f7a4a41ba2e90b8cc9261c9e95a60eceeddf49504341e65258eb7d37d`.
Bracelet v0.3.46 is now published and promoted, as recorded above.

## v0.3.46 Bracelet acceptance checkpoint — 2026-10-24

Scoped native acceptance passes all 29 cases (28 Compose plus one Kubernetes), with zero retries.
The 22 isolated latency cases pass with maxima 62.93 ms local and 160.14 ms with shaped 50 ms
delay, below the unchanged 250 ms budget. Accepted image:
`sha256:1460278198b6d604595bfbe4fb8278a85ab39725c43516bb8b55ab2e6a47f5bf`. Publication and
localhost promotion subsequently completed; see [verification](verification.md).

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

The reviewed Wall Pass captures are [English desktop](../screenshots/wall-pass-desktop.png) and
[Polish 320px reduced motion](../screenshots/wall-pass-mobile-pl.png), with SHA-256 values
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

