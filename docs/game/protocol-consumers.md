# Gameplay protocol consumer inventory

This inventory applies [ADR 0021](../decisions/0021-gameplay-retention.md): gameplay v19 is the
only live network version, while archived schemas, compatibility exports, shared implementation
and regression corpora remain repository consumers. Retention does not mean an old client can
connect, and no complete historical Rust version directory is selected for deletion.

Wire versions, rules profiles, generator versions and private journal/replay identities are
separate compatibility axes. In particular, rules profile 5 is not a gameplay-v5 wire consumer.

| Wire contract | Current consumers | Unique retained coverage |
| --- | --- | --- |
| v2 | Generic root Rust protocol modules under `core/src/protocol/`, `api/gameplay.schema.json`, and generated `web/src/game/schema.ts`. | Root server/contract tests retain v2 parsing, projection, rules and map closure; Rust and browser assertions pin physical parity with v3. |
| v3 | The archived schema/generated declarations, browser conformance suite and dedicated Rust `v3` adapter. | Its physical model is the parity baseline. No server runtime serves v3. |
| v4 | `api/gameplay-v4.schema.json`, generated `web/src/game/v4/schema.ts`, browser conformance and v6 historical schema-identity assertions; there is no Rust `v4` module. | Authentication, reconnect and capability behavior, plus complete v3 physical parity and v4 rules `[1, 2]`. |
| v5 | `api/gameplay-v5.schema.json`, generated browser declarations and browser conformance only; there is no Rust `v5` module. | Authentication/reconnect and v3 physical parity, excluding `RulesVersion` because v5 adds rules `[1, 2, 3]`. |
| v6 | The dedicated Rust/browser adapter and private replay's legacy-default settings path. | Reconnect/settings validation and direct use by v7 bindings/scalars. Its physical-parity test compares v6 with v3; the separate v4 assertion covers only protocol 4 and rules `[1, 2]`. |
| v7 | Dedicated classic projection/item adapter, archived schema/browser semantics and conformance, and shared callbacks consumed by later adapters. | Classic projection, admission, recipient and semantic fixtures; its capture path also checks the generic root projections. No server runtime serves v7. |
| v8 | Shared expanded-mechanics implementation consumed by v9-v14, plus its archived wire tests and retained corpora. | Expanded item mechanics, checked capture/binding, recipient privacy and version-specific validation. |
| v9 | The v8 shared implementation with a v9 item/catalog boundary. | Freeze and preceding-version rejection, including frozen recipient pairs in the server corpus. |
| v10 | An expanded adapter delegating common mechanics to v8, with teleport-aware capture and binding. | Teleporter and related version edges, including archived-v10 privacy controls in the server corpus. |
| v11 | Retained Rust/browser adapter and standalone validator, plus physical binding, recipient projection and neutral observations consumed by later adapters. | Closed Oil Barrel contract, all item masks, archived metadata rejection and physical/privacy regressions. |
| v12 | Archived Rust/browser adapter, schema and conformance fixtures. | Standard-only solo controller metadata/authority, human-duel compatibility and inherited physical/privacy semantics. |
| v13 | Retained Rust/browser adapter, schema, conformance fixtures and compatibility guide. | Easy/Standard selection, consistent two-seat controller metadata and inherited physical/privacy semantics. No server runtime accepts v13. |
| v14 | Retained Rust/browser adapter, standalone validator and archived E2E fixtures. | Checked two-to-four-seat configuration, canonical seat/score arrays, rules-6 projection and inherited recipient privacy; rejected before credential claim. |
| v15 | Rust `core::protocol::v15`; archived server/browser validators and E2E fixtures. | Actual rules7 binding and cloud/timer privacy retained for compatibility; rejected before credential claim. |
| v16 | Rust `core::protocol::v16`, archived server/browser validators and retained conformance/E2E corpus. | Exact historical rules8 binding, canonical lairs, bounded mob counts/footprints, private Stopwatch freeze facts and checked CPU observations. |
| v17 | Native Rust `core::protocol::v17`, `api/gameplay-v17.schema.json`, generated `web/src/game/v17/schema.ts`, retained standalone validator and conformance corpus. | Exact historical rules9/CPU25 binding, one-living-mob-per-lair respawn/death-delay facts, solid-player collisions and archived server fixtures. No server runtime accepts v17. |
| v18 | Retained Rust `core::protocol::v18`, archived schema, generated types, validator, conformance corpus and replay10–12 publication paths. | Rules10 spike settings and authoritative pit projection, solid collision, respawn lairs and recipient privacy; closed Easy/Standard metadata rejects Hard. No live socket accepts v18. |
| v19 | Rust `core::protocol::v19`, schema/generated types, live server socket/command/dispatcher, browser client, standalone validator and shared conformance/E2E corpus. | Same rules10 physics/privacy and bounds, with checked room-wide Easy/Standard/Hard metadata; CPU27 on Easy/Standard and CPU28 on Hard. This is the sole live network contract. |

The shared client corpus exercises the closed client-message union across retained v8-v11 Rust
parsers, schemas and browser validators. The server corpus checks actual Rust-produced
admissions/snapshot pairs with frozen actors, Teleporter epochs, empty/materialized Oil,
recipient identity and concealment controls. Existing control/error-frame conformance remains.
V12 adds its closed client corpus and checks inherited server fixtures with controller metadata.
A typed fixture bridge preserves archived v11 metadata without cloning physical data; standalone
validator parity retains v11–v15 and covers live v16.
See [Testing](../testing.md) for commands and the release/history records for executed results.

## Diagnostic coverage disposition

Renderer diagnostics remain separate from gameplay wire retention. The CI item shard runs
`art-atlas`, `atlas-gameplay` and the DPR 1/2 `terrain-pixels` checks. Terrain coverage compares
baseline, repaint, reset and context-restored pixels. The six-cell renderer lifecycle experiment
is explicitly opt-in: it is not a new CI or performance gate and is not claimed as automated
six-cell acceptance. See [Testing](../testing.md#artist-atlas) for current commands and scope.
