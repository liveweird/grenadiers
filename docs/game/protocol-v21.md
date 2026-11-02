# Gameplay v21: weighted Bomb Capacity drops

Gameplay21 is the paired live contract on `/api/v1/game/ws`. Its authoritative sources are
`api/gameplay-v21.schema.json`, Rust `core::protocol::v21`, generated browser declarations and
the standalone frame validator. Authentication requires protocol21; older versions are rejected
before credential claim. Deploy server and browser together and reload open tabs.

Accepted room settings and `updateSettings` require `bombCapacityDropMultiplier`, an integer
from 1 through 10. New rooms select 5; Classic selects 1, Restore selects 5, and All on/off
preserve the value. It multiplies the existing Bomb Capacity selection weight, leaving the
separate concealment trial unchanged. See [settings](settings.md) for normalized probabilities.
A genuine multiplier-only change advances the settings revision and resets human readiness;
an identical update does neither. Creator, phase, epoch and stale-revision checks are unchanged.

V21 retains rules10, navigation3, ContactV2 Punch, Bomb-only manual input, recipient privacy,
board sizes and bounded frames. CPU60/61 observe the accepted settings without learning
concealed items or changing their algorithms. All new room modes use deterministic generator6;
old construction/replay profiles retain their original generators and multiplier1.

Private replay32 binds gameplay21, generator6, pit-layout1 and complete accepted settings.
Reconstruction uses the same initial configuration and accepted updates. Historical replay
profiles reject new multiplier provenance rather than silently ignoring it. The
[archived v20 contract](protocol-v20.md) remains a reference for the unchanged contact controls.
