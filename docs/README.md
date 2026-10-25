# Contributor documentation

Use this index for current guidance. Dated acceptance records describe the source tested at
the time; they do not add approval steps to the current workflow.

| Task | Start here |
| --- | --- |
| Install, run and contribute | [Project README](../README.md), [conventions](../CONTRIBUTING.md) |
| Understand the boundaries | [Architecture](architecture.md), [design decisions](decisions/) |
| Change gameplay | [Rules](game/rules.md), [implementation plan](game/implementation-plan.md) |
| Change a transport contract | [API guidelines](../api-guidelines/API-GUIDELINES.md), [gameplay protocol](game/protocol.md) |
| Change sprites or tiles | [Artist handoff](../assets/art/README.md), [atlas renderer](game/art-atlas.md) |
| Choose and run checks | [Testing](testing.md) |
| Configure or expose the server | [Security](security.md), [deployment](deployment.md) |
| Prepare a release | [Release policy](releases.md#release-verification-policy), [release checklist](game/implementation-plan.md#release-checklist) |
| Find planned work | [Backlog](game/backlog.md), [quality-review response](quality-review.md) |
| Inspect past acceptance | [Verification records](verification.md), [release history](releases.md) |

Behavior, transport contracts, generated types, tests and the relevant topic document change
together. Local checks are authoritative for the environment they exercise; use remote checks
for distinct coverage. See [CONTRIBUTING](../CONTRIBUTING.md#local-first-verification).
