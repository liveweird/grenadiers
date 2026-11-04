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
board sizes and bounded frames. CPU62/64 observe accepted settings and public pit facts without learning concealed items.
New rooms use generator7 with lair clearance and pit-layout2. The explicit published v21
constructor remains pinned to generator6/pits1 and CPU60/61; older construction/replay profiles
retain their original settings and algorithms. Snapshot and admission binding validate the
expected construction profile as well as accepted settings; they never infer it from a capture.

Private replay33 binds gameplay21, generator7, pit-layout2, CPU62/64 and complete accepted
settings. Published replay32 retains generator6, pit-layout1 and CPU60/61.
Reconstruction uses the same initial configuration and accepted updates. Historical replay
profiles reject new multiplier provenance rather than silently ignoring it. The
[archived v20 contract](protocol-v20.md) remains a reference for the unchanged contact controls.
