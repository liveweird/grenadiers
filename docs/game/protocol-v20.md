# Gameplay v20: contact Punch

Gameplay20 is the sole live paired contract on `/api/v1/game/ws`. Its authoritative sources are
`api/gameplay-v20.schema.json`, Rust `core::protocol::v20`, the generated browser declarations,
and the standalone server-frame validator. Authentication requires protocol20; every other wire
version is rejected before credential claim.

V20 retains gameplay19's room, snapshot and event shapes, rules10, navigation3, recipient privacy,
board/generator selection and finite frame bounds. It advances the closed live CPU identities to
CPU60 on Easy/Standard and CPU61 on Hard. Private replay31 binds gameplay20 and explicit Punch
activation version2; replay30 remains gameplay19 with ManualV1 and its archived CPU identities.

The live rules profile implies `ContactV2`; Punch activation is not a client-configurable wire
field. A living actor holding Punch requests relocation by moving into a punchable bomb. Movement
contact latches the bomb identity and supplies the contact direction; resolution applies cooldown,
trigger, landing and contention checks. The latch clears when that bomb disappears, the actor no
longer touches its current cell, or the actor dies. A failed atomic update retains no latch.
This private state is replayed through exact State equality and is never
added to recipient snapshots.

V20 accepts only `bomb` as a `manualAction`. The archived `equipment` action is not parsed as a v20
command, including before credential claim; contact activation uses ordinary movement instead.
All other command sequencing, action-ID fencing, admission and reconnect behavior is inherited
unchanged from v19.
