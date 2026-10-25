# Product backlog

Outstanding work only. Implemented behavior belongs in the [feature guides](docs/README.md),
[release records](docs/releases.md) and [changelog](CHANGELOG.md); completed items leave this file.
Historical requests and checkpoints remain in [backlog history](docs/history/backlog.md).

Each top-level item starts with one status: `[next]` current delivery, `[todo]` confirmed work,
`[new]` a proposal awaiting a product decision, `[parked]` deliberately deferred, or `[blocked]`
waiting on a stated dependency. Explicit user requests and accepted plans are confirmed work;
new assistant suggestions start as `[new]`. Existing authorization carries forward without
repeated approval for routine implementation choices. Entries are not delivery-date promises.

## Current delivery — Claude review

- [next] **ENG-REVIEW — finish the remaining agreed review work before CPU mode.** The user confirmed
  this priority on 2026-10-25. The published source baseline is v0.3.64; detailed delivered
  evidence and finding dispositions belong in the [quality-review record](docs/quality-review.md),
  not this backlog. Preserve privacy, exhaustive masks, coverage floors and local-first
  verification. Current release lanes and remaining accepted scope are:
  - **REV-CSS (v0.3.65):** integrate, image-test, review and publish the semantic-color change.
    The 252-declaration replacement and full web gate are recorded; release acceptance remains.
  - **REV-ITEM-SPEC (v0.3.66):** complete exact-image integration and publication of the
    canonical 22-item specification. Independent review and the full 4,194,304-mask runs in all
    three required profiles passed; PR #174 remains open at this documentation cut.
  - **REV-ADMISSION (v0.3.67):** complete image integration, runtime acceptance and publication for
    named admission inputs while preserving old wrappers, validation, privacy and ordering.
    Independent review and the source-bound Rust gate passed on the v0.3.66-based candidate;
    publication remains outstanding.
  - **REV-REPLAY:** decide and test production journal allocation when no consumer exists. Use
    explicit recording fixtures and before/after measurements; durable replay export is not a
    prerequisite.
  - **REV-RUNTIME:** measure registry creation/contention and non-reading upgraded WebSockets,
    then record a disposition. The fixed incomplete-header shutdown defect is historical evidence.
  - **REV-FIXTURES:** consolidate genuinely equivalent Rust worlds/frames, E2E
    selectors/collection and Compose seed/CLI scaffolding without changing identities, assertions,
    version-specific rejection, seeds or independent sessions.
  - **REV-PROTOCOL:** audit retained consumers and unique tests under ADR 0021, reduce selected
    adapters, and complete client/archived/cross-language contract coverage. Preserve archived
    schemas/conformance, replay identity, item bits, activation subsets, generation order and
    privacy.
  - **REV-I18N:** consolidate typed locale access and the raw interpolation path without changing
    visible text or plural behavior.
  - **REV-TOOLCHAIN:** define the OS security-refresh policy; add selected pinned shell/workflow
    rules and verified tool/dependency caches. The v0.3.64 Docker cache/OCI work alone does not
    close this scope.
  - **REV-DEPLOY:** add Compose PID/memory limits and Kubernetes ephemeral-storage/security-context
    regression guards, then run actual runtime acceptance. Public ingress and NetworkPolicy wait
    for a real hosting/network model.
  - **REV-FAST:** measure a structured fast-feedback suite and fault detection while retaining
    exhaustive acceptance; sampling is not approved as a replacement.
  - **REV-PARSER:** assess parser mutation/property tests and simultaneous competing-resume
    invariants at concrete boundaries, crediting existing synchronized concurrency coverage.
  - **REV-PERF:** measure CSS/import opportunities and React commits at the 20 Hz publication
    rate; change subscriptions only if the evidence justifies it.
  - **REV-DIAGNOSTICS:** record the renderer/terrain manual-suite and diagnostic wiring
    disposition as part of the protocol coverage inventory.

  Consumer-dependent stable Problem codes, capability rotation, durable replay export, public
  hosting topology, blanket duplicate CI, arbitrary sibling tests, selector-ratio rewrites and
  historical-schema deletion are not silently CPU prerequisites.

## Single-player

- [todo] **SOLO-CPU — deliver the approved staged hybrid.** A private human-versus-CPU duel uses
  the normal rules/input path, restricted observations, interval hazard forecasts, bounded goal
  planning and recorded bot inputs. Follow the [accepted design](docs/game/single-player-cpu.md)
  after ENG-REVIEW; expose a playable slice early and support the all-features-on default before the
  public preview.
  - **CPU0 — contracts and seams:** participant/mode metadata, neutral observation and intent,
    work budget, replay/lifecycle policy and deterministic fixture harness.
  - **CPU1 — internal playable slice:** solo room, legal bot input, real result and
    rematch/reconnect; a stand-still controller is only a harness step.
  - **CPU2 — survival:** interval forecasts, timed routing, safe planting and bounded memory,
    checked against fully known rule fixtures.
  - **CPU3 — public preview:** scored goals, all-features-on compatibility, localized accessible
    presentation and real-browser acceptance of the complete solo flow.
  - **CPU4 — stronger play:** difficulty tuning, tactical look-ahead and measured performance
    from deterministic tournaments and human feedback.
- [parked] **More seats and campaign mobs.** Revisit after the two-seat CPU mode is stable;
  neither is part of the initial solo duel.

## Engineering follow-ups

- [new] **Capability rotation and production replay export.** Establish recovery/consumer needs
  before changing authentication or adding durable storage. Current limitations are documented
  in [security](docs/security.md) and [architecture](docs/architecture.md).

## Playtesting and extensions

- [todo] **BASELINE — close the two-player release scope.** Complete human playtesting, decide
  supported browser/network conditions and publish the stable baseline with explicit limits and
  upgrade/rollback guidance. Archived M7/M8 criteria define acceptance, not completion; CPU and
  campaign content are separate tracks.
- [todo] **TURN — tune with human evidence.** Tolerance is already configurable. Evaluate early/
  late turns, speed levels, collision/fire contact and network delay; preserve fixed-point safety.
- [todo] **Expanded interactions.** Exercise item combinations, replacement/stacking, reconnect,
  generation and privacy. Keep the [implemented catalog](docs/game/expanded-items.md) authoritative.
- [new] **More room controls.** Consider fixed-round versus wins-to-finish semantics, round time,
  initial capacity/range and caps, flame duration, countdown/result pacing, soft-block density,
  item frequency/pools/weights and configurable effect durations. Existing numeric controls,
  item switches and creator/readiness/rematch policies are already implemented; see
  [settings](docs/game/settings.md). New cross-field combinations need explicit validation.
- [new] **Further art and animation.** Build on the adopted Napoleonic sheet and artist pipeline:
  directional/idle/death animation, environment detail, layered authoring sources, accessibility
  and measured effects budgets. Follow the [artist handoff](assets/art/README.md).
- [new] **AUTO-PUNCH — consider automatic contact Punch.** The current idea is at most one
  activation per contact, reducing gameplay controls to movement and Bomb. The user has not approved this
  behavior; keep the existing manual Punch contract unless a product decision accepts it.
