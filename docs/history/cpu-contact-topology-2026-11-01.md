# Contact topology experiment — 2026-11-01

This inactive geometry experiment supports the
[contact refinement design](../game/single-player-cpu.md#contact-topology-refinement-boundary).
It does not implement a production contact certificate or establish a safe plant.
Work sequencing remains in [BACKLOG.md](../../BACKLOG.md#single-player).

## Captured opportunity

The v1.3.6 diagnostic match has a late farming target at block (9,7), approached from (10,7).
At samples 7980, 9000, 10020 and 10680, the visible mob remains in cell (7,6).
Permanent tiles (6,6)/(8,6), blocks (7,5)/(7,7), and four blocked diagonal neighbors isolate it.
Lair (8,6)'s other birth neighbors—(8,5), (9,6), (8,7)—are blocked; its only open neighbor is
that same pocket. Removing target (9,7) leaves the enclosure intact.

Independent inspection confirms the fixture's 34 blocks and CPU/mob cells match sample 10020.
The CPU component contains 70 cells under deliberately conservative eight-neighbor connectivity,
versus 68 with cardinal-only connectivity. The mob component is `{(7,6)}` in both cases.
This graph difference is intentional; it is not a dataset transcription error.

The existing contact envelope ignores walls. A separate nearby controlled test found a complete
bomb route followed by contact refusal below the total work limit. That isolates a conservative
gate in that controlled case; it does not prove every exact captured route fails or that a missed
safe action exists. The captured topology motivates checking separation rather than dropping
contact validation.

## Tested geometric boundary

The [test-only patch](fixtures/contact-connectivity-v136.patch) adds one `cfg(test)` module to
Core. It labels eight-neighbor components, dilates each source component by one tile, admits
birth cells according to explicitly supplied potential openings, and checks a representative
straight segment's complete footprint using the shared geometry kernel.

Actor and mob half-widths are 320 units each, below one 1024-unit tile in combination; the current
contact predicate uses 639. A source's possible body cells are conservatively covered by its
component's one-tile dilation. Every queried actor footprint cell must be outside that dilation
before the geometry reports exclusion. Face, diagonal and corner-straddling cases retain refusal.

The positive test uses captured topology/cells and a **centered representative segment**. The
captured CPU pose is (10752,7872), offset from center; the experiment does not reconstruct that
motion or extract every real route/turn sweep. Opening (9,7) is an explicit test input, not a
removal set derived from a bound plant certificate.

Five focused tests pass. They cover isolated topology, footprint overlap, blocked/open birth
neighbors, retained-barrier removal, and exact/insufficient work. Build work charges both complete
flood/dilation traversals and collection scans. Each later barrier validation and footprint query
is separately charged; a budget exhausted by construction cannot subsequently return clearance.
Independent review identified and corrected undercharging before these results were accepted.
Strict Core Clippy, formatting, source-size and diff checks pass on the final experimental source.

## Limits and reproduction

This is geometry only: no public API, policy/replay change, production `MobContactLease`,
PlantCertificate/opening binding, complete route-sweep extraction, real spawning execution or
survival witness. Unexpected-world/bomb revocation and publication-latency safety remain production
integration obligations. Do not promote a geometric boolean into movement authority.

Apply the linked patch to a clean `git archive v1.3.6`, then run:

```sh
CARGO_BUILD_JOBS=1 cargo test -p grenadiers-core --locked lair_contact_connectivity_tests --lib
cargo clippy -p grenadiers-core --locked --lib -- -D warnings
```

The accepted local experimental commit is `b55eec8d`. The patch is checked against the published
source archive. Current production code, release version and deployed image remain unchanged.
