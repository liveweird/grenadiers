# Gameplay v6: shared room rules

V6 adds revision-bound gameplay configuration to the v5 match/reconnect contract. The complete
closed contract is `api/gameplay-v6.schema.json`; Rust uses `core::protocol::v6` and the active
browser uses generated `web/src/game/v6/schema.ts`. Earlier schemas remain archived.

Every room view, shared snapshot and personal admission now includes:

- `settings`: `startingSpeedLevel` (0–4), `turnAssistUnits` (64/128/192/256), `winsToFinish`
  (1–9) and `bombFuseTicks` (60–300, steps of 30).
- `settingsRevision`: a positive bounded counter, initially 1; only actual changes increment it.
- `settingsEditor`: the original creator's seat, or null after permanent release.

The room creator sends a complete atomic replacement with the existing sequence/room/epoch
fences and the settings revision they observed:

```json
{"type":"updateSettings","sequence":2,"roomId":"0123456789abcdef0123456789abcdef","roundId":1,"settingsRevision":1,"settings":{"startingSpeedLevel":4,"turnAssistUnits":128,"winsToFinish":1,"bombFuseTicks":90}}
```

Ready commands also require `settingsRevision`; ready-true is rejected with `staleSettings`
when it refers to older rules. Updates require the original connected, unready creator and
Waiting phase; unavailable editing returns `settingsLocked`. Settings changes clear both
readiness flags. Rejections do not mutate settings; identical valid updates return the current
view without incrementing revisions. Equal-revision acknowledgement cannot replace the
client's authoritative room/world. A full revision update carries the accepted values.

Configurable rules version 4 identifies the immutable parameterized round configuration.
Structural scores allow 0–9, while semantic checks use the room's selected target. Actual
round physics and lifecycle settings are checked during publication and personal admission.
Historical v1/v2/v3 physics remain available for replay with their fixed settings.

The first application frame authenticates with `protocolVersion: 6`. Older versions, including
5, receive `incompatibleProtocol` before credential claim. Reload clients during upgrades;
the endpoint, single-process match ownership, private seat capabilities and bounds remain.
See [player settings](settings.md), [ADR 0020](../decisions/0020-shared-gameplay-settings.md)
and [replay](replay.md) for authority, lifecycle and evidence details.
