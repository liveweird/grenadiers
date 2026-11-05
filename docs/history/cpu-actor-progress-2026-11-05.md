# Hard actor progress — 2026-11-05

## Confirmed accounting defect

The current Hard67 selector can renew a goal deadline while the CPU is stationary. In the
controlled scene, CPU `(5,7)` targets a rival at `(11,7)` through approach `(10,7)`, distance 5.
Moving only the rival to `(9,7)` changes the approach to `(8,7)`, distance 3. The archived selector
counts that reduction as progress and renews the deadline despite an identical CPU pose.

Hard69 adds an opt-in actor-movement premise to the existing progress predicate. At the old
deadline, the no-motion case now defers the failed target and chooses an alternative. Genuine
whole-cell movement still renews; an explicit one-unit same-cell movement test preserves
subcell progress without a changed cell distance. Repeated unchanged observations do not renew.
Older policies retain their exact behavior. Ordinary and reactive goal selection use the new
mode; fresh output, contact, bounded work and RNG retain paired archive parity.

## Distinct opening behavior

A separate ordered queued-owner comparison uses generated arena seed 53, simulation root 113,
CPU seeds 71/72/73 and a fixed-idle human. Standard62 and Hard67 start from identical state and
first per-seat observations. At common basis tick 183, seat Four selects:

| Policy | Goal | Approach distance | First plant certificate | Bomb placed (absolute tick) |
| --- | --- | ---: | ---: | ---: |
| Standard62 | Nearby Farm | 2 | Running update 18 | 205 |
| Hard67 | Opponent Attack | 16 | Running update 192 | 379 |

The difference is a 174-tick opening delay. Through 720 updates, Standard places seven bombs and
Hard three. Opening RNG stays at zero, work is below the existing allowance, and admission does
not reject the Hard inputs. No stationary distance-only renewal occurs in the captured Hard
seat-Four jobs. Thus this accounting fix does **not** claim to resolve that opening tradeoff.
The large ordered trace remains an explicit ignored manual diagnostic, outside routine suites.

## Identity and safety inheritance

Private Replay38 binds Easy68/Standard62/Hard69; Replay37 remains closed and byte-identical at
68/62/67. Runtime family, settings/rematch retention, controller admission and worker dispatch
use the exact selected profiles. The evaluator labels its real recorder format as 38.

Hard69 reuses Hard67's physical owned-bomb combination proof. Its opaque handoff seals scene,
route, bomb, refuge, checkpoint, horizon and Stop facts, not a policy number. Explicit canonical
Hard67/69 gates grant the shared capability; other profiles cannot obtain it. The actual Hard69
queued witness collects range at ticks 187/199 and capacity at 211, places bomb 0 at 239 and bomb
4 at 293 while the first is live, attributes the additional named-block destruction to bomb 4
at 533, survives both flame windows and makes subsequent settled-cell progress. All four jobs
are delivered, no admission counters fail, and maximum work is 20,709. Forged profiles and
stale, missing, wrong or shortened-horizon handoffs retain their negative coverage.

## Live reference comparison

Clean source `81090d38e54de5f86601ae1f188f6a7400bbc997` passes the actual
`scripts/measure-game.sh --cpu-live-rounds` runner in 10.83 seconds. All 48 cases terminate
naturally and match both shadow and exact accepted-input replay. Their complete deterministic
results equal v1.3.28 after omitting the Hard profile-version label. The restricted workload
therefore establishes compatibility; the targeted selector regression establishes the fix.
The seed pairs make no RNG draws and represent eight distinct arena/script trajectories per
level, each repeated twice. No general strength ranking follows from this matrix.

Raw ignored artifacts remain in `target/quality-review/hard-approach-*.log` and
`target/quality-review/hard69-live-measurements/`. The canonical
[release record](../releases.md#v1329--2026-11-05) owns full local, Linux and image acceptance.
