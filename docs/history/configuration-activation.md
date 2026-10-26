# Configuration and generator activation history

These passages preserve the release-era wording removed from current configuration guides.
They describe historical activation and verification, not current work status.

## Room settings activation

Current guide: [room settings](../game/settings.md).

Implemented in v0.3.17; see [ADR 0020](../decisions/0020-shared-gameplay-settings.md) and
the [verification record](../verification.md) for accepted deployment evidence.

Heart is introduced by the v0.3.25 implementation candidate. Expanded rooms retain immutable
rules version 5/map `generated_v2` when Heart is disabled, including after preset changes.
Shield is available separately in the v0.3.26 candidate, Golden Flame in v0.3.27, Short Fuse in v0.3.28, Short Flame in v0.3.29, Long Fuse in v0.3.30, Reverse Controls in v0.3.31, Diarrhea in v0.3.32, Slow foot in v0.3.33, Piercing Bomb in v0.3.34, Power Bomb in v0.3.35, Remote Bomb in v0.3.36, Landmine in v0.3.37, Invisibility Cloak in v0.3.38, Stopwatch in v0.3.40, Invisible Bombs in v0.3.41, Teleporter in v0.3.42, Wall Pass in v0.3.43, Kick in v0.3.44, Punch in v0.3.45, Bracelet in v0.3.46, and Oil Barrel in v0.3.47; all 22 switches preserve one another.

## Generated arena activation

Current guide: [generated arena API](../game/generation.md).

Status: this source candidate uses generator-v2 with Expanded-v5 rules, gameplay-v11, and
multi-round matches. Generator-v1 and classic-v2 are retained historical identities.

Expanded generator v2 appends each enabled optional item to a stable weighted pool. Wall Pass adds
the weight-15 interval `[365,380)` after Teleporter, Kick adds `[380,395)`, Punch adds
`[395,410)`, Bracelet adds `[410,425)`, and Oil Barrel adds `[425,440)`. The all-items pool is 440;
each sole-item pool is 115. Oil accepts whole mirrored placements only while its lifetime eight-site
cap permits them; an over-cap draw keeps the soft blocks and makes no replacement draw. Any mask
without Oil Barrel preserves its v0.3.46 pool, draw count and exact seeded output. All 4,194,304
item subsets are accepted, and all 2,097,152 earlier masks retain their exact outputs.
