# Contributor documentation

Use this index for current guidance. Dated acceptance records describe the source tested at
the time; they do not add approval steps to the current workflow.

| Task | Start here |
| --- | --- |
| Install, run and contribute | [Project README](../README.md), [conventions](../CONTRIBUTING.md) |
| Understand the boundaries | [Architecture](architecture.md), [design decisions](decisions/README.md) |
| Change gameplay | [Rules](game/rules.md), [headless participants](game/multiplayer-core.md), [multiplayer generation](game/multiplayer-generation.md), [adding an item](game/adding-items.md), [future-track boundaries](game/implementation-plan.md#expansion-tracks-after-the-two-player-baseline) |
| Change a transport contract | [API guidelines](../api-guidelines/API-GUIDELINES.md), [gameplay protocol](game/protocol.md), [protocol consumer inventory](game/protocol-consumers.md) |
| Change sprites or tiles | [Artist handoff](../assets/art/README.md), [atlas renderer](game/art-atlas.md) |
| Choose and run checks | [Testing](testing.md) |
| Configure or expose the server | [Security](security.md), [deployment](deployment.md) |
| Prepare a release | [Release policy](releases.md#release-verification-policy), [release checklist](game/implementation-plan.md#release-checklist) |
| Find planned work and status | [Product backlog](../BACKLOG.md) |
| Inspect review dispositions | [Archived quality-review assessment](history/quality-review-assessment.md) |
| Inspect feature release evidence | [Item and graphics acceptance](history/game-acceptance.md), [configuration activation](history/configuration-activation.md), [review snapshot](history/quality-review.md) |
| Inspect past acceptance or contracts | [Verification history](history/verification.md), [release history](history/releases.md), [implementation-plan archive](history/implementation-plan.md), [protocol guide archive](history/protocols/README.md), [testing narrative archive](history/testing.md), [deployment narrative archive](history/deployment.md), [backlog history](history/backlog.md) |

Behavior, transport contracts, generated types, tests and the relevant topic document change
together. Local checks are authoritative for the environment they exercise; use remote checks
for distinct coverage. See [CONTRIBUTING](../CONTRIBUTING.md#local-first-verification).

## Documentation ownership

Follow the Flow/Lettuce pointer-and-topic pattern: the root backlog contains open work and
status; feature guides own implemented invariants; design proposals are clearly marked; ADRs
own architectural decisions; release/history records own dated delivery evidence. Keep each fact
in its authoritative document and link to it. Read the relevant guide before changing a feature.

| Information | Authoritative owner |
| --- | --- |
| Implemented behavior and invariants | Current topic guides under `docs/` and `docs/game/` |
| Open work, priority and progress | [Product backlog](../BACKLOG.md) |
| Dated delivery and acceptance evidence | Records under [`docs/history/`](history/) and the current release/verification entry points |
| Architectural decisions | [ADR index](decisions/README.md) |
| Accepted but unimplemented design | The named design document; progress remains in the backlog |

| Area | Authoritative guide | Verification reference |
| --- | --- | --- |
| Rules, items and shared configuration | Implemented feature guides under `game/`, including [settings](game/settings.md) and [item extension](game/adding-items.md) | [Rust, web and browser checks](testing.md) |
| Protocol changes | [API guidelines](../api-guidelines/API-GUIDELINES.md) and the live versioned protocol guide | Contract generation/drift, conformance and recipient privacy |
| Graphics | [Artist handoff](../assets/art/README.md), [atlas guide](game/art-atlas.md) | [Artist/renderer checks](testing.md#artist-atlas) |
| Solo mode and CPU contracts | [Solo behavior](game/single-player.md), [contracts](game/cpu-contracts.md), [accepted design](game/single-player-cpu.md), [evaluation](game/cpu-evaluation.md) | Browser solo flow and bounded/recipient/replay fixtures; [backlog](../BACKLOG.md#single-player) owns remaining work |
| Operations and releases | [Deployment](deployment.md), [release policy](releases.md) | [Latest verification](verification.md) |

Keep local-first verification and existing authorization rules in [CONTRIBUTING](../CONTRIBUTING.md).
Historical protocol/ADR links and `history/` records retain their compatibility role.
