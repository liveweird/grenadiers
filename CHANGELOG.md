# Changelog

## [Unreleased] — 0.3.50

### Fixed

- Add browser response hardening and revalidate HTML while caching content-hashed assets
  immutably; preserve API/WebSocket no-store responses.
- Keep throttled room operations connected, isolate create/join/resume budgets, and recover
  seats using bounded same-socket retries with an absolute ten-second client deadline.
- Return no-store problem responses for unknown API paths and unsupported methods, preserve
  route-specific `Allow` headers, and handle WebSocket HEAD requests without response bodies.
- Isolate diagnostic WebSocket connection and upgrade-rate limits from gameplay, enforce
  strict Origin/inbound/write guards, and drain both connection pools during shutdown.
- Show a localized recovery screen for application rendering failures.
- Run previously orphaned workflow/coverage-runner tests through the relevant local gates
  and reject stale installed web/E2E dependency versions before checking code.

### Added

- Contributor documentation index and an adjudicated quality-review remediation plan.

## 0.3.49 — 2026-10-25 (published)

- Use the canonical artist PNG/JSON for gameplay terrain, actors, pickups, bombs, flames and Oil.
- Reuse the validated pixel pipeline and shared scene behavior; preserve cached terrain,
  animation, reduced motion, visibility and renderer resource ownership.
- Fade completed status icons once, normalizing the previous per-stroke overlap blending while
  retaining pulse clocks and status visibility.
- Bundle artist inputs in normal web/Docker builds without requiring Chromium at build time.
- Add real-Pixi gameplay parity and lifecycle acceptance alongside the artist tooling suite.


## 0.3.48 — 2026-10-24 (published)

- Add the canonical editable arena PNG and stable-ID manifest covering 91 baseline drawings,
  four reserved entries, animation timing, pivots, aliases and compositions.
- Add reproducible template/export commands, optional individual slices, strict pixel/metadata
  validation, and a single padded/extruded nearest-neighbour Pixi atlas.
- Add an EN/PL artist workbench with composed arena proof, desktop/narrow reduced-motion
  checks, context restoration and export lifecycle regressions.
- Keep gameplay on the existing renderer pending the bounded v0.3.49 adoption. The baseline
  flame-frame gap is documented for the artist; this release preserves the existing drawings.

## 0.3.47 — 2026-10-24 (published)

- Add opt-in Oil Barrel arena objects with an eight-site lifetime cap, private delayed revelation,
  solid unlit collision, and passable fire that persists until the round ends.
- Preserve immutable explosion planning, typed Wall Pass exit grace, shared fire hazards, replay,
  terminal freeze and new-round reset without adding an inventory slot, timer, event or audio cue.
- Activate gameplay v11 with a 22-item catalog and all 4,194,304 settings masks, English/Polish
  presentation and a repeating-byte-31 browser scenario. Full checks and 32 scoped same-image
  Compose/Kubernetes cases pass, including persistent fire, real reconnect and silent restoration.
  Gameplay v11 requires older browser clients to reload. [Publish v0.3.47](https://github.com/liveweird/grenadiers/releases/tag/v0.3.47)
  from source `494f620cf5eb13f0f18d0bf13d91bc74291bb51b` and promote the accepted image locally.
- Optimize assertion-enabled test compilation while retaining overflow checks, exhaustive masks
  and official coverage accounting. The full 4,194,304-mask gate takes 7m53s; Rust coverage is 93.07%.

## 0.3.46 — 2026-10-24 (published)

- Allow applying the entire advertised item catalog by deriving the client limit from the authoritative room catalog.

- Add opt-in Bracelet equipment: moving into the other player shoves them at most one tile along
  the attempted direction, clamped by authoritative collision and hazard checks.
- Require separation and a fixed 30-tick cooldown after a successful shove while keeping the
  cooldown marker and forced-movement report private across admission, resume and replay.
- Extend gameplay v10 to 21 lobby switches and all 2,097,152 settings masks, with English/Polish
  presentation and a deterministic production-seed browser scenario. Integration with the
  published v0.3.45 parent is complete; scoped native acceptance and final checks are complete on
  the exact image. [Publish v0.3.46](https://github.com/liveweird/grenadiers/releases/tag/v0.3.46) from source
  `7b4b47cfdd7dea9efec9d3c2f2fd61f2c232ce54` and promote the accepted image locally.

## 0.3.45 — 2026-10-24 (published)

- Add opt-in Punch equipment: the remappable Equipment action moves the adjacent facing bomb to the
  next open three-cell wrapped landing while preserving its identity, ownership, blast and fuse.
- Keep the fixed 12-tick success cooldown and Punch history private across capture, admission,
  resume and replay; archived protocols reject those facts and no new wire field or event is added.
- Extend gameplay v10 to the twentieth lobby switch and all 1,048,576 settings masks, with
  English/Polish presentation and deterministic production-seed browser source on the published
  v0.3.44 parent. Local native acceptance completed on the exact image
  `sha256:3f13b26f7a4a41ba2e90b8cc9261c9e95a60eceeddf49504341e65258eb7d37d`: 26 Compose cases plus
  one Kubernetes case (27 total), all with zero retries. The same image is promoted locally.
  Published source `5be42456cee80e06e2f138bdfd458ef821bb34c2` has tree
  `94f83a7650e2c57955103d91060af10b1a2f439c` and annotated tag object
  `0e6224643ce85bf8f0a77894dcb929ab2655e397`.

## 0.3.44 — 2026-10-24

- Add opt-in Kick equipment: walking into a stationary Timed or Remote bomb starts authoritative
  one-cell motion on the next update and every ten updates until blocked; Contact mines never move.
- Preserve bomb identity, owner, blast and fuse while keeping motion and stopped Kick history private
  across capture, admission, resume and replay; archived protocols reject that provenance.
- Extend gameplay v10 to the nineteenth lobby switch and all 524,288 settings masks, with
  English/Polish presentation and deterministic production-seed browser source.
- Source `cb70cba891ed972380f30f52133622ce36fd9ee8`, tree
  `5be27219ba749d2811396d6f84af7d31f528a692`, passes fresh source-bound gates: each Rust profile
  passes 763 core and 210 server tests with four documented ignores, all 524,288 masks complete,
  and the web gate passes 1,059 tests across 66 files with 57 static scenarios. Configured security
  passes with three affected moderate packages, six GHSA advisories and no high or critical
  findings. Fresh image `sha256:8ff3005861127f8ac57cc32f7e6c72ce5fa996fe628ad48426dbb8cf133e1af9`
  passes 56 Compose plus 56 Kubernetes cases with zero retries or carried results: 54 reports, 126
  PNGs, 44 runtime records, 22 Kubernetes identities, 21 fixtures and 42 seed records. Independent
  audit, result/binding review and durable post-seal review are clear under seal
  `94c60dac44c639f7e9a214061e9daf9475628126d15526346ed7be2ce4c9a947`.
- Publish source `f32fd86d8f0c7c4bebba8fa60d1c66c65d1a174a`, tree
  `a0f2addb9204c40b4b30be2830fde4df368f019c`, through annotated tag object
  `c5b1ab6d4cad94ba2f16614103903067745c5179`, and promote the accepted image locally.

## 0.3.43 — 2026-10-24

- Add opt-in Wall Pass equipment for movement through breakable blocks while hard walls, bombs,
  contact, fire and placement rules remain authoritative.
- Preserve private per-block exit grace across equipment replacement and reconnect without adding
  a wire marker, timer or public concealed-content cue.
- Extend gameplay v10 to the eighteenth lobby switch and all 262,144 settings masks, with pinned
  archive rejection, English/Polish presentation and deterministic production-seed browser source.
- The published v0.3.42 source parent is integrated. Fresh source-bound gates pass on source
  `11d0dc7e39e8e2cafcf70c1548be339f52c740ab` (tree
  `655c7d8a80b4dd3fdd3b232534c2cbaa45b054cc`): every debug, release and coverage profile passes
  711 core and 209 server tests with four historical ignores, the web gate passes 797 tests across
  64 files, all 56 scenarios pass, and all 262,144 masks complete in each Rust profile. Rust line
  coverage is 92.358954%; web statement/branch/function/line coverage is 95.24/90.88/96/97.66%.
- Correct the Zod advisory by pinning 4.6.6 in the web and E2E lockfiles. Parallelize only the
  independent English match-result inspections, preserving all 43 assertions, three screenshots,
  fixtures, cases and budgets. CI-only fixes complete bounded procfs metadata reads and create the
  coverage export parent when `CARGO_TARGET_DIR` is external; they do not claim a Chrome I/O cause.
- Correct GHSA-834v-8h66-3h79 in candidate `5dbcd58e920f852a8036c4bf741e26517793b072`
  by updating the development-only Happy DOM dependency from 20.14.5 to 21.0.4. The fresh web gate
  passes 797 tests across 64 files with unchanged coverage, and the complete configured security
  gate passes with three moderate development advisories and zero high or critical findings. The
  first PR #104 head (`98dffe5`) failed security in push run `40050321682` and PR run
  `40050338206`; that failure is preserved as history and superseded by the published delivery.
- At the `2026-10-24 01:00 UTC` native checkpoint, exact image
  `sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`
  passes all 55 Compose plus 55 Kubernetes cases with zero retries or carried passes: 52 reports,
  122 PNGs, 42 runtime records, 21 Kubernetes identities, 20 fixtures, 40 seed records and all 80
  latency samples below 250 ms. Independent audit `25e48638872f7b61fd64c28af30abf5e44910cf29e3e8356246236991819380e`
  and result review `c8652f4e67852002db9e49e2766481e6c58b071d3e93bbb7866bfd6c5194848d`
  are clear. Binding review `7c6eda746a703f599ced928798750f05478b78dbf5fc387d03fbe5adac551d46`
  is clear. Sealer root `96118` completes with consumed exit zero; final manifest
  `3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c` authenticates 2,665
  payloads, while seal `31fe5eef2900a0229d002e45b5a310888f3915829db0aaf23bc3535a8eea5e2c`
  is durable. Independent post-seal review
  `5d253559ba95a9d30fc83f16fd0786a6d283356d892f26abed3611771588d33f` is clear. Eighteen requested items are native
  verified; Kick, Punch, Bracelet and Oil Barrel remain.
- Publish source `c086f719ea470b9fe24963adef84b5e3bdf3e17a`, tree
  `d74fad8a1c870f2834fca48f80ea4e38f0c512d9`, as
  [v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43) through annotated tag
  object `c8395fd7dcc5701842644967802e61c76211931c`. Candidate push/PR runs `40051959641` and
  `40051963999`, master run `40053637247`, and tag run `40055400664` pass all ten jobs, 55 browser
  cases and 262,144 masks per Rust profile on their first attempts. Promote accepted image
  `sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`
  in container `d7f68cc89bfc87d06c202fbedae78c1892f54ea3ccdc6571fee0d1c5ea2f4240`.
  The 110 native passes remain attributed to `11d0dc7`; reviewed artifact equality bridges
  corrected `5dbcd58` to the published source through eight documentation-only changes without
  claiming unchanged Docker inputs, a new Rust compilation or new native execution. Eighteen of
  22 requested items are published.

## 0.3.42 — 2026-10-23

- Add opt-in Teleporter pickups that atomically swap both living players only when both complete destination footprints are safe; unsafe pickups are consumed without moving either player.
- Publish a persistent nullable `lastTeleportTick` position epoch in gameplay v10 so clients snap an authoritative swap without fabricating walking or audio.
- Add the seventeenth lobby switch, all 131,072 settings masks across debug, release and coverage profiles, replay and archived-v9 guards, English/Polish cues, and a deterministic production-seed browser scenario. Source checks cover 689 core, 208 release server and 657 web tests.
- At `2026-10-23 04:37 UTC`, source `880fff7c67269f2886746b45239cd888576832f4` and image `sha256:8ae9818d27c4345bceec6dad77f02e21ee7148626efded48c421d86421285477` pass all 54 Compose plus 54 Kubernetes cases with zero retries or carried passes. Independent audit `0dbb15b27d5a23eae680ca9cfa2d9766a7e64e173f5f8652fd753b4611135521` and result review `cf3380353ca06f36cc61f67c8898ed766d6263bdc6108e003a0a9ac17da4923c` are clear; durable manifest `d81c0923f5a3d0a50208fe43a12412f60ca09e2fddfc1c6aabfb793779676b7f` authenticates the sealed evidence. Hosted candidate, merge, tag, publication and promotion checks remain pending at that native checkpoint.
- Publish merge `0ff1512c5a8fdbd1e1b16583ae7791e668bf070b` (tree `b4201abe9546d22b26b3231506d1e2ab6b1d3613`) as [v0.3.42](https://github.com/liveweird/grenadiers/releases/tag/v0.3.42) through annotated tag object `ea3930a3490fb483de46692e1009743e20c61648`. Master run `39982027955` and tag run `39984896509` independently pass all nine jobs, 54 browser cases and 131,072 masks on their first attempt. Promotion root `84311` exits zero and serves the exact accepted image in container `0a3bcbac08bc2c2cf46365917a3719d3c5e522c4555234da8e1f661f6edcf214`.

## 0.3.41 — 2026-10-23

- Add opt-in Invisible Bombs with a 600-physical-tick refreshable curse that hides every live bomb from the affected recipient while preserving collision, capacity, chains and public flames.
- Capture placement and detonation audiences at emission, intersecting Cloak ownership privacy with blinded seats, including events visible to neither recipient.
- Add the sixteenth lobby switch, all 65,536 v9 settings masks, replay and archived-v8 guards, English/Polish cues, and a deterministic production-seed browser scenario.
- Correct Chromium startup evidence handling with a bounded complete-line endpoint parser, redacted diagnostics and cleanup that preserves the original launch failure. Startup, visibility and shutdown deadlines, Chromium flags, retries and gameplay assertions are unchanged.
- Source `eb0b756df43fdd896cf87573a5224e4eb5f659a3` passes fresh repository, E2E, scenario, startup-parser, server-mask, mirror and current security checks. A single headed diagnostic passes with zero accepted cases. The typed gate assembly retains authenticated Rust/web computations from original source `035c6a1` under an independently reviewed input-equivalence proof; it does not relabel them as executions on the new source.
- Source-bound native root 40405 completed with consumed exit zero: all 53 Compose plus 53 Kubernetes cases, 48 reports, 114 PNGs, 38 runtime records, 19 Kubernetes identities, 18 fixtures and 36 seed-mode records pass with zero retries or carried passes. The facts bind image `sha256:d063462c3130e3c8653734da72e8dfd6785b0c036e9579d9046cee884ccd9f82`; independent audit and result review are clear under SHA-256 values `b0c72b1ff651257930f0db8df9db3cfc1274bd6882f142336e508e699c41097e` and `0cff7c9dd3bd4895cfcb426f340eca146b8c9627a55bac04c01111819b9008be`. The durable seal byte-verifies 2,503 payloads, 2,504 files and 272 directories under final manifest `25e8d3753b24a5dca94028880922b220afd26a6a8035ac5f14c4db619269d813`; independent seal review is clear. PRs 89 and 90 contribute zero hosted cases, and no v0.3.41 publication or deployment is claimed at that historical checkpoint.
- Add bounded CI-only process observations for the headed visibility check and preserve Xvfb error output in failure artifacts. Post-merge run 39870771097 failed before CDP startup; its cause remains unknown. The observer does not change browser flags, assertions, retries or deadlines. Native evidence remains bound to original source `eb0b756`; fresh hosted checks are required for the observer revision.
- Prepare the installed full Chromium distribution with one bounded file pre-read immediately before the headed visibility check, and collect bounded fault, I/O, memory and pressure diagnostics. This targets observed kernel page waits without changing browser invocations or test budgets; the underlying file/storage cause remains unproven.
- Publish commit `f5c4adf66f8af9a5ef66b48f9230d17aea14f1d5` as prerelease tag `v0.3.41`. Distinct post-merge run `39879618476` and tag run `39881306987` each pass all nine jobs, 53 browser cases and 65,536 settings masks. The sealed native image `sha256:d063462c3130e3c8653734da72e8dfd6785b0c036e9579d9046cee884ccd9f82` is promoted locally with consumed exit zero.

## 0.3.40 — 2026-10-21

- Add opt-in Stopwatch with a 180-physical-tick refreshable effect that freezes the living opponent's voluntary actions while ordinary simulation and network clocks continue.
- Pause every Timed bomb once per affected update using checked captured-fuse provenance; preserve untimed Remote and Contact bombs, fire chains, Diarrhea and transactional rollback.
- Add the fifteenth lobby switch, all 32,768 v9 settings masks, replay and archived-v8 guards, English/Polish cues, and a deterministic two-window production-seed browser scenario. At the 21 October 2026 20:56:41 UTC native checkpoint, corrected source `fa25467` passed full debug/release/coverage gates and fresh 52 Compose plus 52 Kubernetes cases with zero retries or carried passes. Independent raw audit is clear and 2,434 payloads are durably sealed; hosted CI, PR/merge, publication, tag verification and promotion remained pending at that checkpoint.

## 0.3.39 — 2026-10-21

- Check responsive result geometry atomically in stable running state before movement or bomb placement; preserve both grids, bounds, viewports and all accessibility assertions.

- Add the paired gameplay-v9 server/browser contract. Visible actors now require
  `voluntaryActionsFrozen`, published as `false` until Stopwatch activation; hidden actors retain
  their exact privacy-preserving shape.
- Preserve Expanded-v5/generated-v2/journal-v2 identity, all fourteen available items, all 16,384
  masks and existing simulation behavior. Gameplay-v8 remains archived and rejects v9 connections.
- Integrate published v0.3.38 accepted raw parent `04c070c5` and its exact tree `d1dc83fc` while
  retaining the reviewed gameplay-v9 foundation byte-for-byte outside the eight parent
  documentation/capture paths.
- At the 21 October 2026 18:44:23 UTC native acceptance checkpoint, source `272b950` and image
  `9a4dc108` passed 51 Compose plus 51 Kubernetes cases with zero retries or carried passes.
  Independent raw audit is clear and the evidence is durably sealed. v0.3.39 had not yet been
  normalized, pushed, merged, published or promoted; candidate, post-merge and tag CI remained
  pending.

## 0.3.38 — 2026-10-21

- Add opt-in Invisibility Cloak with a 600-physical-tick refreshable actor-privacy effect.
- Publish exact hidden actors and event-time private attributable cues while retaining public bombs, hazards, elimination and results.
- Add the fourteenth lobby switch, checked asymmetric v8 publication, replay coverage, English and Polish cues, and a deterministic compositor-based browser scenario. Corrected source `1c6724f` passes a fresh 51 Compose plus 51 Kubernetes matrix with zero retries or carried passes; the independent raw audit is clear and 2,298 evidence payloads are sealed and durably byte-verified.
- Preserve the 20-minute browser-job limit by dividing complete coverage between item and classic jobs using one verified image. The complete 320px HUD layout proof now runs in stable Running before gameplay and restores 390px afterward. No timeout, threshold, sample count, retry policy or product behavior was relaxed. At the 21 October 2026 16:47 UTC native acceptance checkpoint, both seven-job candidate workflows, merge/post-merge CI, publication, exact-image promotion and tag CI remained pending; v0.3.38 was not published.

## 0.3.37 — 2026-10-21

- Correct Remote/Landmine rolling-history assertions using causal placement IDs; preserve Piercing/Power CI diagnostics. Fresh corrected-source acceptance passes 100/100 Compose/Kubernetes cases, independently audited and sealed; both new five-job candidate workflows and the five-job post-merge workflow pass all 50 browser cases; PR 83 is merged, v0.3.37 is published, and its accepted image is deployed locally.
- Add opt-in Landmine as a passable, untimed Standard bomb that detonates on swept actor contact after per-actor exit grace.
- Preserve movement ordering and immutable explosion planning: accepted contact wins, while fire and other blasts silently destroy untriggered mines without extending rays.
- Add the thirteenth lobby switch, checked v8 state and events, replay coverage, English and Polish pressure-plate cues, and a deterministic production-seed browser scenario. Fresh same-image acceptance passes 50 Compose and 50 Kubernetes cases with zero retries; the raw audit is clear and evidence sealed. Both candidate workflows and post-merge CI pass five jobs and all 50 browser cases; v0.3.37 is published and its accepted image deployed locally.

## 0.3.36 — 2026-10-21

- Add opt-in Remote Bomb as a one-charge weapon whose placed bomb waits without a fuse for its owner's next manual bomb action.
- Preserve fire and chain detonation, transactional action resolution, one armed bomb per owner, and independent replacement weapons.
- Add the twelfth lobby switch, checked v8 state and events, replay coverage, English and Polish cues, and a deterministic production-seed browser scenario. Fresh same-image native acceptance passes all 49 Compose and 49 Kubernetes cases with zero retries; the evidence audit is clear and sealed. Both five-job candidate workflows and the five-job post-merge workflow pass all 49 browser cases; PR 82 is merged, the source release is published, and the accepted image is deployed locally.

## 0.3.35 — 2026-10-20

- Pin Vite 8.3.4 and its locked build dependencies to fix newly reported launch-editor advisories while retaining the security audit thresholds.

- Prepare opt-in Power Bomb as a one-charge weapon consumed only by a successful manual placement.
- Capture an immutable Manhattan-area blast that ignores obstruction, preserves hard walls as holes, and chains every bomb in range from the pre-explosion world.
- Add the eleventh lobby switch, checked v8 state and events, replay coverage, English and Polish pixel cues, and a deterministic production-seed browser scenario. Fresh same-image native acceptance passes all 48 Compose and 48 Kubernetes cases; independent evidence review, final sealing, both candidate CI runs and the five-job post-merge run pass. PR 81 is merged, [v0.3.35](https://github.com/liveweird/grenadiers/releases/tag/v0.3.35) is published as a source prerelease, and its accepted image is deployed locally.

## 0.3.34 — 2026-10-20

- Prepare opt-in Piercing Bomb as a one-charge weapon consumed only by a successful manual placement.
- Capture an immutable Piercing blast that crosses breakable blocks while retaining ordinary bomb stops, range accounting, chaining, and transactional placement failures.
- Add the tenth lobby switch, checked v8 state and events, replay coverage, English and Polish pixel cues, and a deterministic production-seed browser scenario. Fresh same-image native acceptance passes all 47 Compose and 47 Kubernetes cases; independent evidence review, candidate CI and post-merge CI pass. v0.3.34 is published as a source prerelease.

## 0.3.33 — 2026-10-20

- Add opt-in Slow foot: each pickup adds an independent 600-tick stack and halves movement speed, up to six exact stacks.
- Preserve fixed-point residuals, hazards, collision rules, Reverse Controls input semantics and deterministic pickup arbitration while allowing eligible rivals to bypass capped actors.
- Add the ninth lobby item switch, English and Polish grouped timers, pixel cues, checked v8 publication, deterministic replay and a production-seed browser scenario. Fresh same-image acceptance passes 46 Compose and 46 Kubernetes cases after reviewed browser-route and movement-window corrections; required candidate and post-merge CI pass, and v0.3.33 is published as a source prerelease.

## 0.3.32 — 2026-10-20

- Add opt-in Diarrhea: collection forces at most one ordinary bomb-placement intent per physical tick for 600 ticks, beginning after collection and ending at the exact expiry boundary.
- Resolve manual and automatic requests together with existing capacity and same-cell conflict rules while preserving manual action IDs, stored weapons, captured stats, and transactional overflow behavior.
- Add the eighth lobby item switch, English and Polish pixel cues, checked v8 publication, deterministic replay, and a production-seed browser scenario. Fresh same-image acceptance passes 45 Compose and 45 Kubernetes cases; final CI and publication remain separate gates tracked in `docs/verification.md`.

- Establish one rendered, running, synchronized client baseline before latency measurement, with complete startup diagnostics and eleven new regression cases; retain all twenty observations, the 250 ms shaped limit and zero retries.

## 0.3.31 — 2026-10-18

- Add opt-in Reverse Controls: collecting the curse reverses raw held directions for exactly 600 physical ticks while preserving deterministic movement and input state.
- Clear pending turn assistance when inversion starts or expires, retain refresh behavior, and restore ordinary movement on the exact expiry tick.
- Add the seventh lobby item switch, English and Polish pixel presentation, checked v8 publication, replay coverage, and a deterministic browser scenario. Fresh same-image acceptance passes 33 Compose and 33 Kubernetes cases; final CI/publication are recorded in `docs/verification.md`.

## 0.3.30 — 2026-10-18

- Add the opt-in Long Fuse curse: generated pickups double the fuse captured by newly placed bombs for 10 seconds.
- Keep planted bomb schedules immutable and make simultaneous Short Fuse and Long Fuse effects cancel for new placements while their timers remain independent.
- Add Long Fuse lobby controls, English and Polish HUD presentation, protocol validation, and focused automated scenarios. Same-image native acceptance and exact-head CI passed; v0.3.30 is published as a source prerelease.

## 0.3.29 — 2026-10-18

- Add opt-in Short Flame curses: ten seconds of refreshable range-one captures for newly
  planted bombs, overriding Golden Flame while retaining independent effect timers.
- Preserve base upgrades, existing bomb ranges, Heart-only defaults and every previous
  seeded item-pool path; add settings, pixel cues, English/Polish text and acceptance scenario.

## 0.3.28 — 2026-10-18

- Add opt-in Short Fuse pickups: ten seconds of refreshable half-length fuses for newly
  planted bombs, independent of other effects and preserving every captured bomb schedule.
- Validate only ordinary or enabled short schedules, rejecting overdue frozen bombs before
  zero-remaining observations can alias another duration; retain archived contracts and defaults.

## 0.3.27 — 2026-10-18

- Add opt-in Golden Flame pickups: ten seconds of refreshable maximum blast range for newly
  planted bombs, independent of Shield and Heart; existing bombs retain their captured range.
- Preserve Heart-only defaults and every prior seeded item-pool path; add independent settings,
  localized guidance and distinct pixel artwork while retaining the gameplay-v8 contract.

## 0.3.26 — 2026-10-18

- Add opt-in Shield pickups: five seconds of refreshable immunity, independent of Heart equipment,
  with exact expiry before damage and at most one prevention event per player per physical tick.
- Add independent Heart/Shield lobby switches, a pixel shield glyph and current protection ring,
  matching EN/PL guidance, deterministic generation and quiet reconnect/replay support.
- Preserve Heart-only defaults and prior empty/Heart-only seeded arenas; add a separate ordinary-control
  browser scenario for generated Shield collection, immunity and later unprotected elimination.

## 0.3.25 — 2026-10-18

- Add the first optional pickup: Heart replaces the equipment slot and absorbs one lethal
  blast, followed by exactly 60 physical ticks of damage grace.
- Introduce Expanded v5 rooms and generator v2 with Heart enabled by default; expose its
  toggle and an explicit Classic settings preset while preserving archived gameplay.
- Move the coordinated live contract to gameplay-v8, retaining recipient validation,
  transactional publication, deterministic replay and bounded messages.

## 0.3.24 — 2026-10-18

- Migrate the live server and browser together to gameplay-v7, with checked Classic room
  projection, personal admission and two prepared recipient perspectives.
- Route every publication and retained closing notice by the authenticated seat; preserve
  transactional admission, reconnect action floors and private replay behavior.
- Add explicit visible-actor presentation and inventory/effect status while retaining Classic
  gameplay and empty optional item availability. Heart activation remains the next increment.
- Wrap player HUD labels and values, with two columns on narrow screens and four on desktop;
  verify all eight labels remain visible inside their own cells without overlapping.

## 0.3.23 — 2026-10-18

- Validate complete physical world and event observations before recipient filtering; keep
  future-item projection fixtures separate from strict Classic room certification.
- Capture event audiences at emission and prepare both recipient encodings atomically,
  preserving authoritative capacity counts and exact physical-clock effect observations.
- Retain active gameplay-v6, archived replay behavior and disabled optional pickups; live
  server/browser migration and Heart activation remain separate increments.

## 0.3.22 — 2026-10-18

- Fix connected takeover in production release builds by executing the binding-fence update
  outside debug assertions; verify replacement commands and stale-connection cleanup.
- Require both debug and release workspace test suites in the server gate.
- Guard reconnect accessibility audits to one document while preserving rule coverage,
  screenshots, deadlines and zero retries; retain bounded browser phase diagnostics.
- Avoid redundant actor pose visibility changes on unchanged frames and moving snapshots.
- Cache immutable terrain at the initialized renderer's resolution with nearest sampling;
  refresh on terrain changes, new worlds and WebGL restoration while keeping dynamic layers live.
- Retain one arena renderer across room and reconnect transitions; suspend and clear it
  between authoritative worlds, then paint the new baseline before revealing the canvas.
- Stage a dormant gameplay-v7 contract for inventory, timed effects, typed bombs, hidden
  actors and shared manual-action IDs, while keeping the active v6 endpoint unchanged.
- Add checked item masks, effect multiplicity and contextual availability validation with
  shared Rust/browser fixtures; distinguish structural schema checks from semantic checks.
- Bound event windows and encoded frames for the expanded contract. Canonical projections,
  recipient filtering, runtime migration and playable item activation remain follow-up work.

## 0.3.21 — 2026-10-18

- Represent domain bomb triggers explicitly as timed, remote or contact, without invented
  deadlines for untimed variants. Current rules still accept timed bombs only.
- Keep legacy snapshot/event wire records unchanged and reject unsupported triggers as a
  whole projection failure; preserve complete events and publication atomicity.
- Retain Classic physics, archived replay outcomes and gameplay-v6 compatibility. No remote
  bomb, landmine or other new item is activated by this foundation.

## 0.3.20 — 2026-10-18

- Retain accepted manual bomb actions in bounded ordered queues through server admission
  and core input processing, ready for later remote-bomb commands.
- Preserve Classic simultaneous placement claims, raw-core Clear cancellation, server
  movement-before-actions ordering, reconnect action IDs and private replay completeness.
- Keep new items disabled and preserve gameplay-v6, existing rules and journal formats.
- Add bounded, secret-free reconnect probe diagnostics with unchanged deadlines and assertions.

## 0.3.19 — 2026-10-17

- Establish independent replaceable equipment/weapon slots and bounded timed effects in the
  deterministic core, with exact expiry, refresh and independently expiring Slow-foot stacks.
- Derive temporary range/fuse/speed capabilities separately from base upgrades; unify flame
  damage resolution with Shield priority, consumable Heart protection and exact damage grace.
- Document the optional expanded-item activation sequence and interaction/visibility policies.
  Items remain unavailable in live rooms; Classic rules and public contracts are unchanged.

## 0.3.18 — 2026-10-17

- Replace smooth placeholder geometry with original detailed pixel art: weathered courtyard
  floors, mossy masonry, layered brick blocks, grenades, connected flames and beveled pickups.
- Add distinct gold/blue directional character poses and bounded walking/spark/flame frames.
- Keep crisp responsive pixel scaling, authoritative interpolation, reduced-motion static
  frames and retained-object cleanup; simulation and gameplay contracts are unchanged.
- Fix pre-existing browser acceptance races by correlating retained round results; serialize
  CI browser workers within the two-player reference while preserving assertions and deadlines.

## 0.3.17 — 2026-10-17

- Add shared, creator-managed lobby settings for starting speed, corner tolerance, wins to
  finish and bomb fuse, with validated choices and EN/PL controls.
- Bind readiness to the accepted settings revision; genuine changes clear readiness, match
  rounds retain settings, and rematches allow editing again.
- Activate configurable rules v4 and gameplay v6; carry exact settings through room views,
  personal admission and private replay while preserving archived profiles.

## 0.3.16 — 2026-10-17

- Double lane-centering tolerance from one eighth to one quarter tile in classic-v3 rooms,
  allowing turns from natural wall stops while retaining swept collision, flame damage and
  the normal movement budget. Preserve classic-v1/v2 physics for deterministic replay.
- Activate gameplay v5 so older browser contracts are rejected before credential binding;
  retain archived schemas, shared match/reconnect semantics and private replay provenance.
- Add native keyboard acceptance for a 192-unit off-center turn visible to both players.
- Record the requested settings, pixel-art, equipment, buff, curse and weapon backlog.

## 0.3.15 — 2026-10-17

- Add original procedural bomb placement/detonation, pickup and elimination cues, default muted,
  with explicit gesture activation, validated non-secret preferences and localized sound controls.
- Bound voices and discard old event history across admission, rounds, mute and visibility;
  hidden return requires a fresh gesture. Keep all essential information visual.
- Verify real Chromium audio context/node activity and hidden/visible resume without replay;
  fix StrictMode engine ownership and stale asynchronous activation with behavioral regressions.

## 0.3.14 — 2026-10-17

- Add bounded reliable ordered WebSocket delay with deterministic 50±10ms jitter per direction.
- Measure native input through actual page delivery, enforce causal moving/stable witnesses and
  public collision clearance, and retain only sanitized timing/public-state evidence.
- Record the corrected two-CPU-affinity reference and fixed 250ms observed envelope; invalidate
  earlier pre-delay observer results and keep twenty-sample, WAN and human-playability limits clear.

## 0.3.13 — 2026-10-17

- Add localized, accessible keyboard remapping with validated persistent non-secret preferences,
  dynamic instructions, capture/cancel/reset, and unchanged arrow/WASD/Space defaults.
- Stop held input before mapping changes or settings, keep old physical keys fenced until key-up,
  and preserve unrelated shortcuts and native focus navigation.
- Add real custom-key movement/bomb, reload persistence, reset, and desktop/320px Polish acceptance.

## 0.3.12 — 2026-10-17

- Add real headed Chromium hidden/visible acceptance with native held-input clearing, continued
  authoritative time, stable transport, no replay on return, and fresh movement recovery.
- Use a natural default browser context without forced focus emulation; keep CDP private,
  traces disabled, and owned browser/profile cleanup bounded through TERM/KILL escalation.
- Run the visibility scenario against the same matches-fixture image in browser CI.

## 0.3.11 — 2026-10-17

- Add optimized reference measurements for the real runtime, journal, projection and encoding,
  plus real-clock one/eight-room owner profiles and separately isolated queue pressure.
- Record exact source, binary, image and environment provenance; reject ambiguous artifacts and
  unpinned Node before measurement. Keep observed cadence/queue/memory limits explicit.
- Select runtime and publication-lag budgets; retain M7 playtesting and network scope separately.

## 0.3.10 — 2026-10-17

- Close integrated baseline rule coverage for explosion-order independence, disconnected-player
  vulnerability through the original bomb fuse, and safe assisted turns into active fire.
- Publish a 42-rule traceability matrix with concrete assertions and explicit evidence limits.
- Keep production behavior unchanged; measured M7 playtest and latency work remains separate.

## 0.3.9 — 2026-10-17

- Activate gameplay v4 with private reserved-seat capabilities, strict ten-second grace, full
  state recovery, stable bomb-action floors, and generation-fenced connected takeover.
- Settle physics, batched departures, and expiry before publication/admission; hold completed
  worlds across canceled countdowns and preserve scoring/forfeit precedence.
- Capture profile-2 grace, restore/replacement, expiry, and report flushes for deterministic
  private replay; retain the archived profile-1 adapter and explicit truncation/incompleteness.
- Activate localized reconnect/grace UI with bounded fresh-session retries, truthful disabled
  controls, explicit leave handling, and usable terminal recovery.
- Add real browser live-world/bomb recovery, takeover, late dual rejection, and privacy checks.
  Keep stable capabilities out of persisted Playwright traces and public diagnostics.

## 0.3.8 — 2026-10-17

- Stage the v4 browser with private in-memory seat capability, bounded fresh-session retries,
  generation fences, full admission baselines, and preserved bomb-action high-water.
- Present localized connection/grace/held-result status and disable controls before recovery.
- Retain authority after rejected leave and return to a usable lobby after terminal recovery.

The panel remains dormant; active gameplay switches with the coordinated server release.

## 0.3.7 — 2026-10-17

- Stage private, independently minted resume capabilities with bounded collision retries and
  redacted diagnostics, plus exact connection/generation binding fences.
- Verify entropy/collision failure atomicity and stable seat capability matching.

Helpers remain test-compiled until coordinated owner/socket activation.

## 0.3.6 — 2026-10-17

- Check v4 room projection against actual lifecycle connectivity, grace, scores, and history.
- Validate physical state epochs for held/canceled rounds and final outcomes, plus private
  admission identity, full-state nullability, and bomb-action high-water context.

These APIs are dormant; active gameplay remains v3 until coordinated reconnect activation.

## 0.3.5 — 2026-10-17

- Stage the closed gameplay-v4 reconnect contract with distinct redacted resume capabilities,
  personal admitted baselines/action high-water, shared room updates, and connection status.
- Add strict Rust/AJV conformance, three-version generated-type drift checks, physical v3
  parity, and maximal complete-frame bounds for snapshots and private admissions.

The contract is dormant. Active gameplay remains v3 until checked projection and coordinated
server/browser reconnect activation are verified.

## 0.3.4 — 2026-10-17

- Integrate reserved-seat grace into the pure match lifecycle with atomic two-seat loss and
  expiry settlement, strict resume boundaries, and eligible-participant forfeits.
- Hold completed rounds through disconnects, cancel inter-round countdowns without consuming
  actual rounds, and start fresh countdowns after return.
- Preserve round scoring before expiry, third-win finality, terminal reservations, and rematch
  reset while retaining the legacy v3 transport-loss interruption path.

The new APIs remain inactive in the network adapter until the v4 server/browser switch.

## 0.3.3 — 2026-10-17

- Capture accepted production match operations in private bounded in-memory journals.
- Preserve immediate input fences, TTL refreshes, actual round seeds, and atomic shutdown/
  departure ordering; rejected commands never enter the journal.
- Mark construction failures incomplete and capacity exhaustion truncated without changing
  gameplay. Replay refuses to certify either as a complete record.

Journals are process-local evidence, released with their owner. Replay/inspection are test-only;
there is no public export, durable storage, credential recording, or reconnect activation.

## 0.3.2 — 2026-10-17

- Add a private, bounded semantic replay journal and runner using the real match runtime.
- Validate format/rules/generator versions, command chronology and bounds, exact round seeds
  and assignments, missing/extra starts, and explicit truncation.
- Compare full private world, lifecycle, input, reports, and revision checkpoints through real
  generated draws, wins, rematch, cancellation, interruption, and immediate input fences.

This runner is test-only. Live accepted-operation capture follows separately; no replay
endpoint, durable storage, credential recording, or reconnect activation is introduced.

## 0.3.1 — 2026-10-17

- Add a pure bounded two-seat reconnect-grace model with checked 600-tick deadlines.
- Reject expired/stale resumes atomically, retain original duplicate-loss deadlines, and expose
  batched expiry facts without deciding match outcomes.

This is a reconnect foundation. Active gameplay v3 still interrupts on transport loss;
credential, lifecycle, server, and browser activation follow separately.

## 0.3.0 — 2026-10-17

- Activate authoritative two-player first-to-three matches over gameplay protocol 3.
- Carry scores and completed-round history through automatic result holds, fresh worlds,
  alternating spawns, and inter-round countdowns; draws award no score.
- Add consensual rematches with withdrawable votes, fresh scores/readiness, and retained
  connection sequencing; distinguish round results from final forfeits and interruptions.
- Activate localized accessible match scores, phase clocks, final outcomes, and rematch controls.
- Drain upgraded game and diagnostic sockets explicitly on shutdown, delivering the final
  checked match view before close within a bounded deadline.
- Add an isolated generated-map browser fixture for a real draw, three wins, and rematch.

## 0.2.13 — 2026-10-17

- Add the dormant bounded match owner with generation-fenced admissions, two-seat departure
  batches, fixed-tick scheduling, lobby/rematch deadlines, and final-match retirement.
- Signal shutdown independently of the command queue and retain one checked final publication
  with actual vacancy, pending reports, and preserved match finality.
- Reject canceled admissions before mutation and settle overloaded prematch leaves promptly.

Production registry/socket wiring and coordinated browser activation follow in v0.3.0.

## 0.2.12 — 2026-10-17

- Stage the protocol3 browser client, match scores/timers, completed-round and final-match
  presentation, and authoritative two-player rematch consent.
- Accept newer-epoch snapshots even when intermediate room views are coalesced; reset
  controls/action IDs while preserving transport sequencing.
- Share the existing Pixi arena and preserve authoritative vacancy over frozen actors.

The new client/panel remains dormant until the bounded owner and coordinated activation land.

## 0.2.11 — 2026-10-17

- Add a dormant authoritative adapter for real multi-round simulation and checked v3 publication.
- Build fresh worlds and input state per actual assignment, flush bounded report windows,
  and freeze physics during result/rematch holds while retaining the authoritative room clock.
- Fence stale input before action IDs, preserve private generation records, and publish
  truthful interruptions when round construction fails.

The adapter is test-only until the bounded owner and coordinated browser/server switch land.

## 0.2.10 — 2026-10-17

- Add checked v3 room projection from actual match lifecycle facts.
- Validate canonical seats, readiness phases, score/history budgets, epoch chronology,
  result holds, rematch eligibility, and independent forfeit/interruption outcomes.
- Reject impossible unanimous-consent snapshots and occupied forfeit losers.

This adapter is dormant; production gameplay remains v2 until runtime/browser activation.

## 0.2.9 — 2026-10-17

- Stage a self-contained gameplay-v3 contract for truthful match scores, actual round count,
  completed-round records, automatic countdowns, match results, and rematch votes.
- Add namespaced Rust DTOs and generated TypeScript, strict conformance/physical-parity
  checks, and dual-version generation/drift checks.
- Guard the new metadata/frame budget with a 44,742-byte maximal-body fixture.

This contract is dormant. Live gameplay remains protocol v2 and diagnostics v1; checked
projection, real adapters, and coordinated activation follow separately.

## 0.2.8 — 2026-10-17

- Add a pure match lifecycle with initial and automatic inter-round countdowns, result holds,
  actual-round assignment, first-to-three termination, and consensual rematch reset.
- Separate prematch interruption from post-start match outcomes, batch same-tick departures,
  retain the global clock, and make transition failures transactional.
- Invalidate departed players' rematch consent and reject consensus with a vacant opponent.

The lifecycle is staged alongside the live single-round runtime. Gameplay v2 remains active;
truthful v3 contracts/adapters and coordinated activation follow separately.

## 0.2.7 — 2026-10-17

- Add bounded headless first-to-three match-series facts: scores, actual round assignment,
  immutable latest completed round, and a separate final match outcome.
- Preserve result holds, batched forfeit/abandonment/interruption precedence, and finality.
- Add idempotent, withdrawable two-player rematch votes after the 180-tick hold, with retained
  command-epoch/time high-water checks and atomic overflow failures.

Live generated gameplay remains the single-round v2 preview. Automatic orchestration, v3
contracts, browser match UI, and full first-to-three acceptance follow as focused increments.

## 0.2.6 — 2026-10-17

- Activate private generated-v1 arenas with classic-v2 pickups and upgrades in live rooms.
- Draw fresh round entropy only at an actual start; retain private generation records and
  keep authentication/discovery randomness independent.
- Add a strict read-once operator seed-file mode that fails startup on invalid configuration.
- Prove generated-map smoke behavior and deterministic keyboard collection in real browsers,
  including simultaneous old-range/new-range bombs and both authoritative upgrade HUDs.

The two-player preview still supports a single round. Complete match scoring/rematch is M5.

## 0.2.5 — 2026-10-17

- Draw exposed capacity, range, and speed items as distinct retained Pixi graphics, with
  owned-resource cleanup on replacement, removal, and room reset.
- Show authoritative available bombs, capacity, range, and speed levels for both players;
  distinguish alive/eliminated status from lobby readiness in English and Polish.
- Exercise actual HUD updates and bounded layouts down to 320 pixels in the rendered duel.

Generated live rounds and real pickup collection acceptance follow in the activation release.

## 0.2.4 — 2026-10-17

- Coordinate gameplay protocol v2 across schema, Rust, server, and browser authentication.
- Transmit exposed typed pickups and collection events with checked rules/map identifiers;
  preserve bounded frames and keep concealed/pending contents and seeds private.
- Bind construction and publication labels to an immutable room profile; retain the fixed
  classic-v1 live preview until generated-room activation.

Gameplay v1 clients must reload with the matching v2 server. Diagnostics remain protocol v1.

## 0.2.3 — 2026-10-17

- Collect exposed pickups after damage using final actor positions and deterministic ties.
- Apply permanent capacity/range upgrades up to eight and five speed levels; preserve
  placement-time bomb range and movement residuals. Consume items even at the cap.
- Add explicit round-number spawn and tie-priority assignment, independent of canceled
  countdown attempts. Keep classic-v1 and the fixed live preview unchanged.

Gameplay wire support, browser items, and generated-room activation follow separately.

## 0.2.2 — 2026-10-17

- Replace inert pickup cells with checked typed concealed, pending, and exposed layers.
- Reveal destroyed-block contents after flames clear; burn exposed items without shielding
  downstream blasts, and preserve revealing-blast immunity for pending contents.
- Add generated-world construction with separate private generation records and exposed-only
  public domain views. Freeze every pickup transition after a result.

Collection, upgrades, wire/client support, and generated network rounds follow separately.
The fixed-map playable preview remains unchanged.

## 0.2.1 — 2026-10-17

- Add a versioned deterministic arena API with canonical mirrored soft blocks and typed
  concealed upgrades, clear spawn openings, and validated first-bomb escape routes.
- Pin ChaCha20 initialization and bounded unbiased draw semantics; use a deterministic
  empty fallback on validation or random-draw exhaustion.
- Keep generation records and concealed locations outside public snapshots and redact
  sensitive debug output. Promote the already-locked rand_chacha 0.9.0 dependency.

This M4 groundwork does not activate generated network rooms or pickup behavior. The playable
fixed-map preview remains unchanged while those dependent features are implemented.

## 0.2.0 — 2026-10-17

- Make the fixed-map two-player duel playable through Create/Join, readiness, countdown,
  keyboard movement, bombs, destruction, elimination, and a shared round result.
- Render the authoritative arena with pinned PixiJS 8.22.0 alongside the React/Mantine HUD,
  English/Polish controls, reduced motion, scoped keyboard focus, and retained diagnostics.
- Validate incoming gameplay frames, fence stale async connection work, and keep credentials
  in tab memory. Release graphics contexts, tickers, observers, and listeners on teardown.
- Verify actual WebGL pixels and desktop/narrow layouts in a complete two-browser duel.

This completes M3’s first playable preview. It requires a keyboard and supports one fixed-map
round; generated upgrades, complete match scoring, touch controls, and reconnect follow later.

## 0.1.14 — 2026-10-17

- Run real two-player rounds with readiness, exact countdown, authoritative 60 Hz updates,
  and checked public snapshots/events published at 20 Hz.
- Bind logical movement and bomb actions to the current seat/attempt; expire stale held
  input, reject duplicate bomb IDs, and bound each player’s admitted work per update.
- Preserve completed results, interrupt live rounds on departure or overload, and retire
  terminal rooms after a fixed retention window.

The protocol now supports a full fixed-map round through native sockets. The playable
browser interface and real graphics acceptance follow in the duel preview.

## 0.1.13 — 2026-10-17

- Add bounded room creation and code-based joining with two authenticated seat owners.
- Publish personalized admission and shared membership views, reject full/stale bindings,
  and acknowledge explicit departures before allowing another room on the same connection.
- Bound discovery, actor queues, identifiers, and waiting lifetime; release registry entries
  even when an owner task is canceled before its first poll.

Rooms remain in Waiting in this increment. Readiness, authoritative simulation scheduling,
public snapshots, and the browser game interface follow separately.

## 0.1.12 — 2026-10-17

- Add authenticated gameplay WebSockets with compatibility checks before credential lookup,
  atomic single-owner claims, and credential revocation when the owning connection ends.
- Share the existing connection limit with diagnostics and bound upgrades, input, frames,
  buffers, sends, authentication, inactivity, and absolute unseated lifetime.
- Reject replayed command sequences and expose typed unseated command errors without
  fabricating rooms or gameplay state.

Room ownership/admission and authoritative gameplay scheduling follow separately. This
preview does not support reconnecting with a previously claimed credential.

## 0.1.11 — 2026-10-17

- Add a pure headless two-seat room lifecycle for occupancy, readiness, the exact 180-tick
  countdown, running updates, results, and preview interruptions.
- Fence canceled countdowns with increasing round attempts, clear readiness consistently,
  and preserve terminal outcomes after either explicit departure or transport loss.
- Precheck clock/deadline/attempt arithmetic so errors leave lifecycle state unchanged.

This domain API does not create networked rooms. Authenticated gameplay sockets and room
ownership/routing follow separately.

## 0.1.10 — 2026-10-17

- Add `POST /api/v1/game/session` for anonymous 256-bit credentials, with exact Origin checks,
  closed no-store responses, a bounded in-memory registry, and five-minute unclaimed expiry.
- Limit issuance bursts/rate, handle entropy failures and collisions safely, and omit query
  strings from HTTP tracing.
- Document the real HTTP operation and exercise it through deterministic server tests and
  blackbox browser requests.

Tokens do not yet authenticate a gameplay socket or reserve a room/seat. Gameplay WebSocket
handlers follow in the next increment; existing diagnostic transport remains available.

## 0.1.9 — 2026-10-17

- Add explicit fallible conversion from public simulation state to bounded gameplay wire DTOs.
- Reject invalid clocks, dimensions, identities, ownership counts, ordering, positions,
  upgrades, deadlines, and outcomes without exposing private simulation bookkeeping.
- Convert up to three consecutive update reports into stable indexed events, preserving
  same-tick placement/detonation and accepting legitimate frozen result snapshots.

Authenticated session issuance and gameplay transport follow separately. This adapter
introduces no network route, room, or browser gameplay.

## 0.1.8 — 2026-10-17

- Define bounded public snapshots for the fixed arena, two actors, owned bombs, and flames
  in Rust, JSON Schema, and generated TypeScript.
- Include indexed placement, detonation, elimination, and round-end events without private
  simulation bookkeeping or concealed future contents.
- Validate full nested messages, collection/scalar limits, and actual encoded frame sizes.

This completes the snapshot/event wire shape. Checked domain conversion, authenticated
transport, rooms, and browser play follow separately.

## 0.1.7 — 2026-10-17

- Define strict gameplay authentication, room control, movement/bomb intent, lifecycle,
  result, and typed error messages in one JSON Schema with Rust and generated TS types.
- Add canonical identifiers, bounded integer parsing, required nullable movement, token
  Debug redaction, payload-free errors, and message byte limits.
- Check gameplay type drift in the shared web gate and compare Rust/AJV conformance.

This is the control/lifecycle contract foundation. Snapshot/event wire types and gameplay
endpoints follow separately; the existing diagnostic protocol remains unchanged.

## 0.1.6 — 2026-10-17

- Support bounded strings, Unicode regular-expression patterns, bounded arrays, and
  uniqueness constraints in the explicit realtime type generator.
- Reject malformed, reversed, unsafe, or misplaced constraints while preserving existing
  fail-closed schema handling and byte-identical diagnostic types.
- Add nested contract compilation and validation regressions for gameplay credentials
  and bounded snapshots. Runtime AJV validation continues to enforce these constraints.

This tooling prerequisite starts M3; gameplay endpoints and browser play follow separately.

## 0.1.5 — 2026-10-17

- Add explicit public-state DTOs for the arena, players, bombs, flames, timer, and result,
  excluding private input, movement bookkeeping, collision grace, and allocator state.
- Emit deterministic placement, detonation, elimination, and round-end events in actual
  phase order, preserving placement and explosion within the same tick.
- Complete the headless M2 combat boundary for the future gameplay transport.

Networked rooms and browser gameplay follow in M3. Domain DTOs are independent of
wire schemas and do not serialize the internal world directly.

## 0.1.4 — 2026-10-17

- Add swept contact damage from existing fire and final-footprint damage from new explosions.
- Prevent eliminated players from moving or planting; keep their existing bombs active
  until the round resolves.
- Decide sole-survivor wins, mutual-elimination draws, and the exact 10,800-update timeout
  after all same-tick damage, with elimination taking precedence over timeout.
- Freeze gameplay after a result while the absolute simulation clock continues.

This remains headless: public gameplay projections and events are the last M2 slice;
networked rooms and the playable browser arena follow in M3.

## 0.1.3 — 2026-10-17

- Execute bomb fuses and same-tick chains against one immutable occupancy snapshot.
- Stop cardinal blast rays at walls, soft blocks, and other bombs; remove hit soft blocks
  after planning the complete chain.
- Add 30-tick flames with exclusive expiry, overlapping lifetime extension, and bomb ignition.
- Release bomb capacity once, available for placement on the following tick.

This headless increment adds explosions and flames. Player damage, round outcomes,
public gameplay snapshots, and browser play follow separately.

## 0.1.2 — 2026-10-17

- Add owned bomb placement after movement, one live bomb per player, captured range,
  and 180-tick detonation deadlines. Fuse execution follows in the next increment.
- Resolve eligible simultaneous placements deterministically; failed requests consume
  no capacity or entity ID and never carry into another tick.
- Preserve collision grace for every overlapping player until they leave the bomb.
- Replace the unused ownerless obstacle API with `InputAction::PlaceBomb`.

This remains a headless engineering prerelease. Explosions, damage, and browser play
are not implemented yet.

## 0.1.1 — 2026-10-17

- Verify the Rust/React foundation, locked builds, coverage, dependency/image scans,
  Docker Compose, and local Kubernetes operation.
- Add the deterministic headless arena/movement model, turn assistance, logical input
  priority, swept collision, bomb exit exemptions, and repeatable state comparisons.
- Specify the classic two-player rules and phased implementation plan.
- Select PixiJS for future arena rendering alongside the React/Mantine interface.

This engineering prerelease has no playable browser arena or combat yet. The browser
continues to expose transport diagnostics. See `docs/verification.md` for observed results.
