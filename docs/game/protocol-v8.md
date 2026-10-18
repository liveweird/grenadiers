# Gameplay v8: Expanded rooms and optional pickups

The coordinated server/browser contract is `api/gameplay-v8.schema.json`, Rust
`core::protocol::v8`, and generated `web/src/game/v8/schema.ts`. Reload older clients after
upgrading. Version 7 is rejected before credential lookup, so an old-version attempt cannot
claim or revoke the credential needed for a version-8 connection.

## Immutable rules identity

V8 adds rules version **5** and map ID **`generated_v2`**. Its own rules/map types leave the
shared enums and schemas for earlier protocols unchanged. New production rooms are always
Expanded v5/generator v2. They advertise `availableItems: ["heart", "shield", "goldenFlame"]` and initially enable
`settings.enabledItems: ["heart"]`. All eight subsets of those three available items are supported configurations.
Other catalog IDs remain unavailable.

The Classic settings preset disables optional items and restores the four ordinary settings.
It does not change a room's immutable rules/map identity. With both optional items disabled, v2 generation
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
Untimed bombs, special blasts, concealment effects
and other optional mechanics remain unavailable.

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
