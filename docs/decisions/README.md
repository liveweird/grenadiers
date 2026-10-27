# Architecture decisions

Read [architecture](../architecture.md) for the implemented system. These ADRs preserve the
decision and rollout context at the time they were written; an old milestone or protocol in a
record does not mean that version is still served. The [live protocol](../game/protocol.md)
identifies the active network contract. Check a record's status and later decisions together.

| ADR | Decision |
| --- | --- |
| [0001](0001-stack.md) | Rust transport and Flow engineering conventions |
| [0002](0002-deterministic-movement.md) | Integer world, fixed ticks and swept movement |
| [0003](0003-arena-rendering.md) | PixiJS arena with React menus and HUD |
| [0004](0004-bomb-placement.md) | Owned bombs and simultaneous placement |
| [0005](0005-explosion-snapshot.md) | Immutable explosion occupancy and timed flames |
| [0006](0006-round-finalization.md) | Damage phases and immutable round outcomes |
| [0007](0007-public-state-events.md) | Public state projection and phase events |
| [0008](0008-gameplay-protocol.md) | Anonymous gameplay protocol and preview boundary |
| [0009](0009-room-lifecycle.md) | Pure two-seat room lifecycle |
| [0010](0010-authoritative-room-loop.md) | Authoritative scheduling, publication and retention |
| [0011](0011-deterministic-arena-generation.md) | Versioned deterministic arena generation |
| [0012](0012-pickup-phases-and-upgrades.md) | Pickup visibility, collection and upgrades |
| [0013](0013-gameplay-v2-and-room-profiles.md) | Gameplay v2 and immutable room profiles |
| [0014](0014-private-round-seeds-and-live-generation.md) | Private seeds and live generated rounds |
| [0015](0015-match-series-and-staged-progression.md) | Match facts and staged progression |
| [0016](0016-dormant-gameplay-v3-contract.md) | Staged gameplay v3 contract |
| [0017](0017-match-runtime-and-frozen-publication.md) | Authoritative runtime and frozen result publication |
| [0018](0018-bounded-match-owner.md) | Bounded ownership, departure and shutdown |
| [0019](0019-reserved-seat-reconnect-and-replay.md) | Reserved-seat reconnect and private replay evidence |
| [0020](0020-shared-gameplay-settings.md) | Revision-bound shared room settings |
| [0021](0021-gameplay-retention.md) | Live protocol support, archived contracts and private replay retention |
| [0022](0022-standalone-browser-validation.md) | Deterministic standalone validation for live browser messages |
| [0023](0023-browser-content-security-policy.md) | Enforced browser content policy |
| [0024](0024-peer-admission.md) | Bounded peer admission and explicit proxy identity |
| [0025](0025-production-replay-recording.md) | Production journal allocation and explicit test recording |
| [0026](0026-solo-controllers.md) | Server-owned solo controllers and bounded planning |
| [0027](0027-solo-difficulty.md) | Selectable solo difficulty and hazard-safe reaction cadence |

For a new decision, use the next numeric prefix and a descriptive filename. State the context,
decision, consequences and acceptance requirements. When replacing an earlier decision, link
both records and explain the compatibility boundary rather than erasing the original rationale.
