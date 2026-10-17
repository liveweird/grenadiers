# Staged gameplay-v3 contract

Status: structural contract and checked room projection implemented; production activation follows. `/api/v1/game/ws` still serves [gameplay v2](protocol.md).
V3 DTOs and generated types do not activate match play or a second endpoint.

V3 keeps existing commands and physical snapshot/event shapes, adding
`rematch {sequence, roomId, roundId, ready}`. Authentication success is fixed to 3;
incompatibility responses retain the bounded numeric version input type.

`RoomView` adds `roundNumber` (0..2,147,483,647), `scores {one, two}` (each 0..3), nullable
`lastRound {roundId, roundNumber, endedAt, result}`, and per-seat `rematchReady`.
Command `roundId` remains an independent monotonic room-lifetime epoch. `roundNumber` counts
actual starts, stays on the preceding round during inter-round countdown, and resets on rematch.

| Phase | Required payload |
| --- | --- |
| waiting | No additional fields |
| countdown | `kind: initial/interRound`, `endsAt` |
| running | No additional fields |
| result | `nextCountdownAt` after the nonfinal 180-tick hold |
| matchResult | `endedAt`, independent match `result`, `rematchAt` |
| interrupted | Prematch `endedAt` and nonparticipant interruption `reason` |

Match results are winner with seat and `roundWins/forfeit` reason, abandoned, or interrupted
with connection-loss/overload/shutdown/internal-error reason. Last completed-round facts remain
visible alongside later forfeits or interruptions; rematch clears them. Readiness is meaningful
only before an initial start; rematch readiness only after a final match's hold.

The schema enforces structure and bounds. `project_room_view(RoomMetadata, &MatchLifecycle)` now derives canonical room facts and
`validate_room_view` checks independently constructed DTOs. These enforce canonical seats,
phase/score/history consistency, time relationships, consent transitions, and epoch chronology.
Forfeit losers must be vacant; later winner departure remains valid. Draws award no score.
Server adapters still own physical-state/profile agreement and private-state separation. No hidden items, seeds, generation records, or assignment bookkeeping enter DTOs.

`npm --prefix web run gen:gameplay` and `check:gameplay` cover both versions. Rust/AJV tests
exercise strict fields, variants, scalar boundaries, physical parity, and frame limits.
The maximal-body/worst-metadata fixture is 44,742 bytes against the unchanged 65,536-byte cap.
See [ADR 0016](../decisions/0016-dormant-gameplay-v3-contract.md).
