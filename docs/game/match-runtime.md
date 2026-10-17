# Authoritative match runtime

Status: active behind the v4 endpoint and bounded match owner. No alternate endpoint or
runtime feature flag is exposed.

`MatchRuntime` composes the pure `MatchLifecycle`, immutable room profile/identity, private seed
source, input mailbox, optional simulation, checked revision, and at most three tick reports.
Only an actual round start requests entropy and constructs a fresh world with that round's
spawn/tie assignment, base upgrades, entity allocator, and cleared input/action IDs.

Seat, epoch, and phase checks precede input high-water mutation. Per-seat fencing removes
pending input without resetting accepted bomb IDs. Simulation advances at 60 Hz; publication
is every three running ticks or immediately at a lifecycle boundary. Final partial reports
flush once; worlds/report windows never cross an inter-round or rematch boundary.

During result and match-result holds the lifecycle clock advances, but physics freezes.
Completed worlds bind to their own completed-round epoch/result/end tick. An unfinished
forfeit/interruption retains `GameState.result = null` and freezes at the match outcome tick;
it never fabricates a round result. Inter-round countdowns publish room views without an old
world, while retaining the completed world privately so a reconnect cancellation can republish
the correctly bound result. A new actual round or rematch replaces it. Rematch consensus clears
state and returns occupied players to unready Waiting.

Personal admission projects a full baseline without consuming pending reports. It includes the
current bomb action high-water only when the physical body belongs to the current command epoch.
A high-water at the protocol maximum prevents further bomb actions in that epoch.

Checked v4 room projection, physical projection, profile agreement, and the 65,536-byte
encoder guard publication. Seeds, generation records, concealed contents, pending inputs,
and assignment bookkeeping remain private. Entropy/construction failure publishes an
InternalError interruption without a substitute world. The [bounded owner](match-owner.md) supplies scheduling, admission, departure batching,
retention, and shutdown. The process explicitly drains upgraded sockets before runtime exit.
