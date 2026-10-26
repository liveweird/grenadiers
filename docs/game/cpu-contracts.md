# CPU contracts

These Rust contracts support the [solo opponent](single-player.md) and the broader
[accepted design](single-player-cpu.md). The
[backlog](../../BACKLOG.md#single-player) owns delivery status.

## Controllers and ordinary inputs

`core::room::RoomMode` derives controller kinds independently of occupancy and socket state.
Human duels have two human controllers. Solo fixes the human at seat One and the CPU at seat
Two. `CpuProfile::INITIAL` identifies `UtilityV1` with `Standard` difficulty; these are policy
identities, not claims about measured strength. Seeds are outside participant metadata.

`core::cpu` accepts neutral observations and emits movement or Stop, with at most one Bomb or
Equipment intent. Intent carries no credentials, seat authority, readiness, settings mutation
or manual action ID. The server allocates those IDs and uses ordinary input
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
memory; a new physical world resets it. The local rejection fallback is Stop without a manual
action; it grants no submission authority. Discard stale or wrong-world completions instead of
injecting their fallback into a newer request. Only the current controller's missing/exhausted
request may submit its ordinary Stop. History has bounded, deterministic retention, and work is
counted in logical units rather than hardware milliseconds.

Tie-breaking uses its own seeded ChaCha stream and never draws from arena generation. Small
fixture policies remain test-only; production uses UtilityV1. Replays consume recorded accepted
actions rather than invoking a policy again. Production journal allocation remains disabled;
explicit solo recording adds format-3 header provenance while preserving formats 1/2 and
accepted-input record capacity.

## Planning and committed execution

Two shared workers accept at most 32 queued jobs; a controller has one outstanding request and
one bounded result slot. Planning uses neutral observations and a cloned independent RNG. The
owner never waits for it. Results must still match the issued request and be at most six logical
ticks old; the observation tick identifies that request, while runtime age is checked separately.
Only accepted current input commits prospective memory/RNG.

Plans hold ordinary movement for 1–12 updates, then the owner submits ordinary Stop at its
committed deadline. Once a Bomb request is accepted, `follow_committed_escape` executes only
the already planned route of at most four cells from fresh checked snapshots. It performs no
goal/path search, random draws or manual action and does not wait for another worker. This
keeps worker contention outside the route's scheduling bound. Unsupported effects, unknown
bombs, blocked or displaced route geometry stop the executor rather than inventing safe facts.
New hazards may still invalidate its game-world assumptions.

## Focused verification

Run `cargo test -p grenadiers-core cpu:: --lib` for bounded contracts,
`cargo test -p grenadiers-core cpu_observation --lib` for recipient privacy and public facts,
and `cargo test -p grenadiers-server cpu_contract --lib` for accepted-input replay. These
focused commands supplement the unchanged full Rust/coverage release gates.
