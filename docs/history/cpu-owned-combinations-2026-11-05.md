# Hard CPU owned-bomb combinations — 2026-11-05

Delivery status belongs in the [backlog](../../BACKLOG.md#single-player). The reusable behavior
belongs in [CPU contracts](../game/cpu-contracts.md#owned-bomb-combinations).

## Paired physical scene

This is an explicitly curated public scene, not a generated-arena or restored-seed claim.
Both policies use the same contents and human inputs. The real CPU collects range upgrades at
updates 187 and 199, then extra capacity at 211. Both place bomb 0 at `(5,5)` on update 239;
it detonates at 479 and its finite flames clear at 509.

Hard66's next bomb, ID 4, is placed at update 623, after the first has expired. It reaches at
most one simultaneous owned bomb, and the named soft block `(7,3)` remains after the first
physical detonation. The baseline records detonations at 479 and 863.

Hard67 places distinct bomb 4 at `(4,3)` on update 293 while bomb 0 remains live, reaching two
simultaneous owned bombs. On update 533, bomb 4 is the only detonating bomb and the named block
changes from intact to destroyed. Its Standard blast geometry reaches that block. The equal
first placement plus the baseline's actual detonation leaving the block intact establish that
this is an additional contribution. The CPU survives both full flame windows and subsequently
crosses into another settled cell.

Hard66 delivers all 12 queued jobs; Hard67 delivers all four. Neither case has failed, canceled,
dropped or outstanding work after explicit finalization. Maximum charged work is 24,876 and
20,709 respectively, below the unchanged 65,536 allowance. The baseline has two refuge-handoff
admission rejections; Hard67 has none. A real admitted combination route rejects Hard65/66
profile authority and accepts Hard67.

The six generated board/lair reference cases retain identical recorded Hard66/67 metrics except
for policy identity. This establishes the scoped combination opportunity without claiming a
general strength or win-rate improvement.

## Review and diagnostic corrections

Independent review required the certificate to seal the exact handoff and recheck the current
installed horizon at admission. The opaque handoff now carries its validated real stop time;
new-path observation and forecast comparisons are charged. Generic reactive admission cannot
authorize a combination. Focused tests cover exact admission, a longer installed horizon,
wrong stop time, same-tick first-bomb redundancy, insufficient capacity and charged exhaustion.

An initial positive test fixture was legitimately declined. During diagnosis, a proposed
change to the destruction deadline was rejected and reverted: the route already retains the
absent forecast, so it does not include the proposed second bomb. The legacy purpose rule
remains intact. The final physical witness uses natural queued policy selection rather than a
manually injected positive certificate.

Finalization explicitly drains/cancels pending work instead of omitting a last enqueued job.
Ordinary Hard67/Hard66 parity compares the full plan, contact, work, RNG state and future outputs.
The isolated native production-pacing check passes in 34.62 seconds. Full release verification
and exact source/image identities belong in the [v1.3.26 release record](../releases.md#v1326--2026-11-05).
