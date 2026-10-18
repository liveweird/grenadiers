# Gameplay v8: Expanded rooms and optional pickups

This is the archived v8 reference. The live contract is [gameplay v9](protocol-v9.md).

The archived coordinated contract is `api/gameplay-v8.schema.json`, Rust
`core::protocol::v8`, and generated `web/src/game/v8/schema.ts`. Reload older clients after
upgrading. Version 7 is rejected before credential lookup, so an old-version attempt cannot
claim or revoke the credential needed for a version-8 connection.

## Immutable rules identity

V8 adds rules version **5** and map ID **`generated_v2`**. Its own rules/map types leave the
shared enums and schemas for earlier protocols unchanged. New production rooms are always
Expanded v5/generator v2. They advertise `availableItems: ["heart", "remoteBomb", "piercingBomb", "powerBomb", "landmine", "shield", "invisibilityCloak", "goldenFlame", "shortFuse", "diarrhea", "slowFoot", "shortFlame", "longFuse", "reverseControls"]` and initially enable
`settings.enabledItems: ["heart"]`. All 16384 subsets of those fourteen available items are supported configurations.
Other catalog IDs remain unavailable.

The Classic settings preset disables optional items and restores the four ordinary settings.
It does not change a room's immutable rules/map identity. With all optional items disabled, v2 generation
reproduces v1 terrain, blocks and concealed upgrades for the same seed. Legacy rules and
generator v1 continue to reject nonempty masks; archived traces retain their behavior.

## State and events

The recipient-aware shapes introduced by [v7](protocol-v7.md) remain: visible/hidden actors,
inventory slots, bounded effects, physical simulation time, typed bomb triggers, authoritative
owned-bomb counts, 96-event windows and shared manual-action IDs. Representation of a catalog
item does not make it available in a playable room.

Heart is revealed and collected through ordinary simulation. A lethal blast consumes its
equipment charge and emits `damagePrevented` with source `heart`. Grace suppresses repeated
damage through T+59 and expires before damage at T+60. Recollection during grace equips another
charge without extending grace. Inventory and grace clear on death and round reset; reconnect
restores current state without historical event playback.

`manualAction` with `action: "bomb"` retains ordinary placement. Heart is passive, so
`action: "equipment"` remains unsupported and receives correlated `invalidMessage` without
consuming its manual-action ID. Shield is also passive: collection adds or refreshes a 300-tick
effect, and emits at most one `damagePrevented` with source `shield` per actor per physical tick.
Its effect expires before damage at collection tick + 300; reconnect retains its remaining time.
When Piercing Bomb or Power Bomb is enabled, a successful manual placement consumes its stored
charge and publishes an unpaused Timed bomb with the corresponding captured blast. Power burns
every floor cell in its Manhattan range, treating hard walls as unburned holes instead of line-of-
sight barriers. Automatic Diarrhea placements remain Standard and do not consume either charge.
Remote Bomb consumes its charge only on successful placement and publishes a Standard bomb with a
Remote trigger. Its owner's next retained manual bomb action detonates that exact live bomb. Fire
and chains remain valid triggers; death and disconnect add no deadline. Landmine consumes its
charge on successful placement and publishes a passable, untimed Standard bomb with a Contact
trigger. Swept entry after exit grace detonates it; fire and other blasts silently destroy an
untriggered mine. Invisibility Cloak adds or refreshes a 600-physical-tick effect. Its owner remains
fully visible while the other recipient receives only actor identity, seat and alive status. The
pickup, cloaked bomb cues and cloaked damage prevention are private at event time; elimination and
results remain public. Paused Timed bombs and other optional mechanics remain unavailable.

## Checked publication and replay

`capture_expanded` observes actual State rules, enabled settings, physical state and reports.
`bind_expanded` validates room/lifecycle identity and the trusted round's generation record;
production retains State and its record together. Expanded state is never relabeled as Classic
to pass an older validator. Full world and event validation precedes recipient filtering.

`prepare_snapshot_pair` encodes both perspectives before revision/report-flush changes.
`prepare_expanded_admission` prepares the private event-free body before ownership or replacement
changes. Every publication and retained closing notice routes using authenticated membership.
A failure returns no partial publication and preserves pending reports.

Private journal format 2 identifies rules 5/generator 2 with the existing M6 lifecycle policy.
Complete settings carry the enabled mask; replay regenerates from seed, version and settings.
Format-1 rules 2–4/generator-1 archives remain supported. Mismatched tuples and unsupported
masks fail validation. Record size remains at most 64 bytes and allocation below 4 MiB.

Message size remains 65,536 bytes, reports remain at most three consecutive updates, and
oversize events/frames fail without truncation. Heart's conservative bound is 25 events per
tick, or 75 in a three-report publication window, below the existing 96-event limit.

Golden Flame adds or refreshes a 600-physical-tick effect. Actor `bombRange` stays the base
upgrade value; each planted bomb exposes its captured range. Older v8 clients must reload to
accept the extended availability list; deploy the paired server/browser build together.

Short Fuse adds or refreshes a 600-physical-tick effect. Timed bombs capture the configured
fuse or its ceiling-half when this item is enabled, even after its owner's effect expires.
World, placement and detonation projections enforce the captured duration independently of
current effects. Actual overdue deadlines fail capture before remaining-time conversion;
an exact-deadline frozen bomb may expose zero remaining ticks. Paused Timed triggers remain
unavailable. Remote and Contact bombs have no fuse and do not use Short Fuse schedules.
No schema or archived protocol is changed.

Short Flame adds or refreshes a 600-physical-tick effect. Actor `bombRange` remains the base
upgrade value; bombs and events publish their actual captured range one. Historical range
is independent of current owner effects, including after expiry or death. Golden Flame remains
an independent timer underneath the curse. No schema or historical protocol is expanded.

Long Fuse likewise adds or refreshes a 600-tick effect and doubles the configured
fuse captured by new ordinary timed bombs. Short Fuse and Long Fuse cancel for new placements
while both remain active; their independent timers and historical bomb deadlines remain visible.

Reverse Controls adds or refreshes a 600-tick effect. Movement applies the opposite of the
authoritative raw held direction beginning on the tick after collection. Expiry precedes movement,
so the raw direction applies normally at the exact deadline. Browser controls and presented
headings remain literal; only simulation movement is inverted.

Diarrhea adds or refreshes a 600-tick effect. The simulation selects one retained manual bomb
request or one automatic ordinary intent per seat, then resolves both seats together. Automatic
intents never enter the mailbox or action-ID sequence. The first automatic placement is one tick
after collection, and expiry suppresses placement at the exact deadline.

Slow foot adds an independent 600-tick stack, up to six. Movement uses the current speed divided
by `2^stack_count`; every supported speed is exactly divisible by 64. Collection affects the next
tick, while each deadline expires before movement. A capped actor is excluded before collection
distance and tie arbitration, allowing an eligible rival to collect the pickup.

The fourteen-item availability array is exact v8 data. Deploy server and web together and reload
older v8 clients: a client built for thirteen items rejects the new room even when Invisibility Cloak is disabled.
