# CPU contracts

These internal Rust foundations support the [accepted single-player design](single-player-cpu.md).
They do not add a Solo button, a running bot or a new wire/replay format. The
[backlog](../../BACKLOG.md#single-player) owns delivery status.

## Controllers and ordinary inputs

`core::room::RoomMode` derives controller kinds independently of occupancy and socket state.
Human duels have two human controllers. Solo fixes the human at seat One and the CPU at seat
Two. `CpuProfile::INITIAL` identifies `UtilityV1` with `Standard` difficulty; these are policy
identities, not claims about measured strength. Seeds are outside participant metadata.

`core::cpu` accepts neutral observations and emits movement or Stop, with at most one Bomb or
Equipment intent. Intent carries no credentials, seat authority, readiness, settings mutation
or manual action ID. The server integration must allocate those IDs and use ordinary input
validation; tests exercise accepted actions through the existing runtime and replay path.

## Information boundary

The current-protocol adapter converts the already filtered v11 recipient projection. It must
never feed the policy `State::public_view` or reconstruct an authoritative `State`. A cloaked
rival exposes only identity, seat and alive status. Invisible Bombs makes all bomb knowledge
unavailable, including the observer's bombs; unavailable differs from an empty visible list.

Observations retain public terrain, settings, effects, exposed pickups/Oil, flames and visible
bomb triggers. Private movement residuals, collision grace, action cooldown histories and bomb
motion timing remain explicitly unknown. Concealed content, seeds and opponent intent never
enter observations. The adapter receives server-owned world identity separately because a
public command epoch is not a physical-round identity.

## Bounded decisions and replay

Memory belongs to a room instance, physical round and CPU seat. Decision admission separately
checks the current command epoch, observation tick and request serial. Reconnect preserves
memory; a new physical world resets it. Stale, duplicate or exhausted work yields Stop without
a manual action. History has bounded, deterministic retention, and work is counted in logical
units rather than hardware milliseconds.

Tie-breaking uses its own seeded ChaCha stream and never draws from arena generation. Fixture
policies exist only in tests. Replays consume recorded accepted actions rather than invoking a
policy again. Production journal allocation remains opt-in; policy provenance needs an explicit
compatible format extension when real CPU execution is connected.

## Focused verification

Run `cargo test -p grenadiers-core cpu:: --lib` for bounded contracts,
`cargo test -p grenadiers-core cpu_observation --lib` for recipient privacy and public facts,
and `cargo test -p grenadiers-server cpu_contract --lib` for accepted-input replay. These
focused commands supplement the unchanged full Rust/coverage release gates.
