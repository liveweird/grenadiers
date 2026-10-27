# Gameplay v13: solo difficulty

The paired contract is `api/gameplay-v13.schema.json`, Rust `core::protocol::v13` and generated
`web/src/game/v13/schema.ts`. Deploy server and browser together; older clients must reload.
V13 retains the [v11 physical/privacy model](protocol-v11.md), Expanded-v5 rules, generated-v2
arenas and the [v12 controller authority model](protocol-v12.md).

## Room creation and metadata

A solo creator chooses one closed difficulty value:

```json
{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"easy"}}
```

Use `standard` for the existing cadence or `{"type":"humanDuel"}` for two humans. Clients
cannot supply a CPU seat, policy, seed or timing parameters. The server chooses the current
profile, and room mode/difficulty remain immutable through reconnect and rematch.

Every solo room publishes matching difficulty in its mode and seat-Two CPU controller. Human
seat One alone receives admission/resume capabilities. Unknown levels, extra authority fields,
inconsistent controller metadata and admission to the CPU seat are rejected. V12 parsers and
schemas retain their Standard-only meaning; they do not silently reinterpret Easy.

## Validation and compatibility

The v13 adapter reuses checked physical conversion and validates difficulty/controller metadata
before publication. Recipient filtering, concealed information, message limits, lifecycle rules
and input validation are unchanged. Both profiles use ordinary movement and manual actions.
See [solo behavior](single-player.md) for cadence and [CPU contracts](cpu-contracts.md) for
private profile provenance and accepted-input replay. HTTP sessions and diagnostic protocol
version 1 are independent of this gameplay upgrade.
