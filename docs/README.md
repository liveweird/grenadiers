# Contributor documentation

Use this index for current guidance. Dated acceptance records describe the source tested at
the time; they do not add approval steps to the current workflow.

| Task | Start here |
| --- | --- |
| Install, run and contribute | [Project README](../README.md), [conventions](../CONTRIBUTING.md) |
| Understand the boundaries | [Architecture](architecture.md), [design decisions](decisions/README.md) |
| Change gameplay | [Rules](game/rules.md), [adding an item](game/adding-items.md), [implementation plan](game/implementation-plan.md) |
| Change a transport contract | [API guidelines](../api-guidelines/API-GUIDELINES.md), [gameplay protocol](game/protocol.md) |
| Change sprites or tiles | [Artist handoff](../assets/art/README.md), [atlas renderer](game/art-atlas.md) |
| Choose and run checks | [Testing](testing.md) |
| Configure or expose the server | [Security](security.md), [deployment](deployment.md) |
| Prepare a release | [Release policy](releases.md#release-verification-policy), [release checklist](game/implementation-plan.md#release-checklist) |
| Find planned work | [Product backlog](../BACKLOG.md), [milestones](game/implementation-plan.md), [quality-review response](quality-review.md) |
| Inspect past acceptance or contracts | [Verification history](history/verification.md), [release history](history/releases.md), [protocol guide archive](history/protocols/README.md), [testing narrative archive](history/testing.md), [deployment narrative archive](history/deployment.md), [backlog history](history/backlog.md) |

Behavior, transport contracts, generated types, tests and the relevant topic document change
together. Local checks are authoritative for the environment they exercise; use remote checks
for distinct coverage. See [CONTRIBUTING](../CONTRIBUTING.md#local-first-verification).

## Documentation ownership

Follow the Flow/Lettuce pointer-and-topic pattern: the root backlog contains open work and
status; feature guides own implemented invariants; design proposals are clearly marked; ADRs
own architectural decisions; release/history records own dated delivery evidence. Keep each fact
in its authoritative document and link to it. Read the relevant guide before changing a feature.

| Area | Authoritative guide | Verification reference |
| --- | --- | --- |
| Rules, items and shared configuration | Implemented feature guides under `game/`, including [settings](game/settings.md) and [item extension](game/adding-items.md) | [Rust, web and browser checks](testing.md) |
| Protocol changes | [API guidelines](../api-guidelines/API-GUIDELINES.md) and the live versioned protocol guide | Contract generation/drift, conformance and recipient privacy |
| Graphics | [Artist handoff](../assets/art/README.md), [atlas guide](game/art-atlas.md) | [Artist/renderer checks](testing.md#artist-atlas) |
| Planned CPU mode | [Accepted design](game/single-player-cpu.md) | Milestone-specific acceptance in that design; not implementation evidence |
| Operations and releases | [Deployment](deployment.md), [release policy](releases.md) | [Latest verification](verification.md) |

Keep local-first verification and existing authorization rules in [CONTRIBUTING](../CONTRIBUTING.md).
Historical protocol/ADR links and `history/` records retain their compatibility role.
