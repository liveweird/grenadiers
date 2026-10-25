# Teleporter

## Availability

Teleporter is a configurable immediate pickup enabled with every available item in new
Expanded-v5 rooms. Creators can still disable it explicitly before readiness.

## Rules

When a living player collects one and the other seat has a living actor, the game considers one
atomic exchange of their current positions. Both complete actor footprints must be within bounds
and clear of hard terrain, soft blocks, active fire and bombs. An actor may retain only its own
already-valid overlap grace for the same bomb when that grace covers both its current footprint and
destination; this applies to solid Timed or Remote bombs as well as Contact mines. A swap creates
no new grace and never transfers grace between actors. Shield does not make fire safe.

If either destination is unsafe, or the rival is dead or absent, the pickup is consumed and neither
actor moves. A disconnected but still living reserved-seat actor remains the other participant.
Successful swaps clear movement residual and pending turn assistance for both actors, preserve
headings, apply no damage, trigger no swept Contact mine, collect no destination pickup and perform
no extra action. Collection does not draw RNG, occupy an inventory slot or start a timer. Existing
equipment, weapons and effects remain unchanged.

## State and privacy

Both visible actors receive the same persistent `lastTeleportTick` after a successful swap. It
does not change for an unsafe collection, persists through pause, results and reconnect, and clears
only with a newly constructed round state. Gameplay v11 clients treat a newer marker as an
authoritative position epoch: they snap to the published positions without walking interpolation or
invented movement audio. Hidden actors retain their exact four-key privacy shape and expose neither
position nor marker.

## Compatibility

At v0.3.42, Teleporter was added as the seventeenth item, bit 21 of
`EnabledItems` (`EnabledItems::ALL = 3145712`; now 4194303 with all 22 items). Generator v2
appended weight 15 after Invisible Bombs: the all-items pool was 365 (now 440) and a
Teleporter-only pool is 115. All 131,072 subsets were accepted at v0.3.42 without changing earlier
masks or generated maps. See [gameplay v11](protocol-v11.md),
[settings](settings.md), and [production-seed testing](../testing.md#teleporter-production-seed-scenario).

## Verification

Source `880fff7c67269f2886746b45239cd888576832f4` is native verified by 54 Compose plus
54 Kubernetes cases with zero retries or carried passes. The accepted scope covers safe swap,
reconnect, unsafe consumption and exact complete-footprint safety; see the
[verification record](../verification.md). Reviewed pre-collection examples show
[English desktop](../screenshots/teleporter-desktop.png) and
[Polish 320px reduced-motion](../screenshots/teleporter-mobile-pl.png) presentation.
