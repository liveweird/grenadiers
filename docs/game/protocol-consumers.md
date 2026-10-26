# Gameplay protocol consumer inventory

This inventory applies [ADR 0021](../decisions/0021-gameplay-retention.md): gameplay v11 is the
only live network version, while archived schemas, compatibility exports, shared implementation
and regression corpora remain repository consumers. Retention does not mean an old client can
connect, and no complete historical Rust version directory is selected for deletion.

Wire versions, rules profiles, generator versions and private journal/replay identities are
separate compatibility axes. In particular, rules profile 5 is not a gameplay-v5 wire consumer.

| Wire contract | Current consumers | Unique retained coverage |
| --- | --- | --- |
| v2 | Generic root Rust protocol modules under `core/src/protocol/`, `api/gameplay.schema.json`, and generated `web/src/game/schema.ts`. | Root server/contract tests retain v2 parsing, projection, rules and map closure; Rust and browser assertions pin physical parity with v3. |
| v3 | The archived schema/browser conformance suite and the dedicated Rust `v3` adapter; later Rust adapters import its DTOs and scalars. | Its physical model is the parity baseline. No server runtime serves v3. |
| v4 | `api/gameplay-v4.schema.json`, generated `web/src/game/v4/schema.ts`, and browser conformance only; there is no Rust `v4` module. | Authentication, reconnect and capability behavior, plus complete v3 physical parity and v4 rules `[1, 2]`. |
| v5 | `api/gameplay-v5.schema.json`, generated browser declarations and browser conformance only; there is no Rust `v5` module. | Authentication/reconnect and v3 physical parity, excluding `RulesVersion` because v5 adds rules `[1, 2, 3]`. |
| v6 | The dedicated Rust/browser adapter and private replay's legacy-default settings path. | Reconnect/settings validation and direct use by v7 bindings/scalars. Its physical-parity test compares v6 with v3; the separate v4 assertion covers only protocol 4 and rules `[1, 2]`. |
| v7 | Dedicated classic projection/item adapter, archived schema/browser semantics and conformance, and shared callbacks consumed by later adapters. | Classic projection, admission, recipient and semantic fixtures; its capture path also checks the generic root projections. No server runtime serves v7. |
| v8 | Shared expanded-mechanics implementation re-exported by v9-v11, plus its archived wire tests and retained corpora. | Expanded item mechanics, checked capture/binding, recipient privacy and version-specific validation. |
| v9 | The v8 shared implementation with a v9 item/catalog boundary. | Freeze and other v9-only version edges, with archived privacy controls in the server corpus. |
| v10 | The v8 shared implementation with v10 Kick, Punch, Bracelet, Wall Pass and Teleporter behavior. | Teleporter and related version edges, including archived-v10 privacy controls in the server corpus. |
| v11 | Rust `core::protocol::v11`; server connection, socket, command and dispatcher paths; browser `clientProtocol`; and current E2E suites. | Oil Barrel validation and live identity/metadata/privacy controls. This is the sole live server/browser contract. |

The shared browser client corpus exercises all ten client-message variants with 14 valid and 27
invalid cases, including Rust-schema/browser mutations across v8-v11. The shared server corpus
contains 16 Rust-produced admission/snapshot frames: three each for v8 and v9, and five each for
v10 and v11. It covers Freeze, Teleporter, Oil Barrel, archived-v8/v10 privacy controls and live
v11 identity/metadata controls while retaining the original control and error-frame checks.
Compatibility exports and corpus fixtures are regression evidence, not live-client support.

## Diagnostic coverage disposition

Renderer diagnostics remain separate from gameplay wire retention. The CI item shard runs
`art-atlas`, `atlas-gameplay` and the DPR 1/2 `terrain-pixels` checks. Terrain coverage compares
baseline, repaint, reset and context-restored pixels. The six-cell renderer lifecycle experiment
is explicitly opt-in: it is not a new CI or performance gate and is not claimed as automated
six-cell acceptance. See [Testing](../testing.md#artist-atlas) for current commands and scope.
