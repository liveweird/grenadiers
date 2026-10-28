# Release history

This archive preserves the older release records originally maintained in
[`docs/releases.md`](../releases.md). Records remain in their original order; the current
release procedure and current release record stay in the [release guide](../releases.md).

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

