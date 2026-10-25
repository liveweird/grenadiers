<a id="grenadiers-implementation-and-release-plan"></a>

# Release checklist and future tracks

This guide owns the reusable release checklist and the design boundaries for future product
tracks. Open work and delivery status live only in the [product backlog](../../BACKLOG.md).
The original M0–M8 execution plan, including its dated evidence and acceptance traceability,
is preserved in the [implementation-plan archive](../history/implementation-plan.md).

## Historical milestone pointers

These aliases preserve links to the former live plan while directing readers to the archived
record.

<a id="current-starting-point"></a>
<a id="planned-releases"></a>
<a id="dependency-and-ownership-model"></a>
<a id="m0-establish-a-verified-foundation"></a>
<a id="m1-build-the-world-and-movement-model"></a>
<a id="m2-implement-bombs-and-one-round-combat"></a>
<a id="m3-make-the-duel-playable-in-two-browsers"></a>
<a id="m4-add-generated-arenas-and-the-three-initial-upgrades"></a>
<a id="m5-complete-classic-match-progression"></a>
<a id="m6-implement-reconnect-resilience-and-complete-replay-evidence"></a>
<a id="m7-playtest-and-harden-the-two-player-game"></a>
<a id="m8-release-the-stable-two-player-baseline"></a>
<a id="baseline-acceptance-traceability"></a>
<a id="decisions-before-dependent-implementation"></a>

For milestone scope and evidence, use the [archived M0–M8 plan](../history/implementation-plan.md).

## Release checklist

Every release candidate must pass applicable existing guardrails without lowering coverage
floors or adding blanket suppressions. Preview features have explicit tested limits; an
unexecuted required check is an open release blocker, not a pass.

- `scripts/gates.sh all`: repository, locked Rust formatting/lint/tests/build, contract drift,
  frontend lint/dead code/coverage/build, and E2E static checks.
- `scripts/gates.sh coverage` and `scripts/gates.sh security`: Rust coverage and dependency
  policies/audits.
- `scripts/gates.sh browser`: actual multi-context scenarios against the built application.
- `scripts/gates.sh k8s`: invariant checks and strict rendered Kubernetes schemas.
- Clean image build and configured image vulnerability scan; Compose and Kubernetes runtime
  smoke checks for startup, probes, UI/API, and WebSockets. A clean render is insufficient.
- New unit/integration/property or permutation tests where behavior needs them; scenario docs
  and browser titles remain in parity. Record RULE coverage and actual outcomes.
- Independent review of substantive rule, protocol, security, and lifecycle changes. Verify
  edge cases at the affected layer rather than multiplying identical tests at every layer.
- Updated contracts, generated files, player/deployment docs, changelog, and verification record.
  Review staged source, commit focused changes, and tag only the tested release candidate.

The `all` target alone does not include security, Rust coverage, browser execution, or Kubernetes
validation. Apply the checklist according to the release policy and record the checks actually run.

## Expansion tracks after the two-player baseline

Choose priority through the [product backlog](../../BACKLOG.md). Each accepted track gets a
separate ruleset increment and acceptance matrix; inclusion here is a design boundary, not
approval or delivery status.

| Track | Deliverable | Prerequisites and limits |
| --- | --- | --- |
| More players | Start with four configurable seats, fair spawn layouts, multi-survivor results, and browser/room load tests. | Generalize seat counts; update the diagnostic `maxPlayers` promise and every affected contract intentionally. |
| Bot competitors | At least one fair AI controller with survival, escape planning and simple offense; then difficulty levels. | Follow the [accepted CPU design](single-player-cpu.md): ordinary validated inputs, restricted observations and bounded work. |
| Advanced abilities and arenas | Select one ability or hazard at a time. | Specify feature interactions and deterministic edge cases before enabling combinations. |
| Campaign and mobs | A small authored stage with exit progression and fully defined enemy state machines. | Separate PvE lives, target, terrain and damage rules; expand only after a complete stage works. |
| Persistent identity and statistics | Durable users/results and optional replay retention. | Add migrations, integration tests, access/privacy/retention rules and idempotent result writes only for a concrete product need. |
| Larger operational scale | Many rooms, routing, deployment draining and optional recovery. | Measurements first; design single-owner routing before replicas or shared match state. |

More-player and bot tracks can be prioritized independently. Campaign mobs are not a shortcut
to bot competitors: they have different abilities, information and objectives. Persistent
accounts and leaderboards can wait until durable data has a concrete product purpose.
