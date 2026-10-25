# Adding a gameplay item

Start with [contributor conventions](../../CONTRIBUTING.md), the [item overview](expanded-items.md), and
the [live protocol](protocol-v11.md). This guide maps the current implementation; it does
not require a new copy of every historical adapter for each item.

## Define the behavior and compatibility boundary

Specify collection, replacement or stacking, duration, consumption, death/reset behavior,
interaction order, and what each player may observe. Separate persistent equipment, timed
effects, single-use weapons, and world objects. Record a focused [ADR](../decisions/README.md)
when the change introduces a new invariant or simulation phase.

The server currently accepts only gameplay v11. Decide whether the change needs a new wire,
rules, generator or replay identity before editing contracts. Retained schemas reserve wire
vocabularies while adapters choose their activation catalogs; do not expand a retained contract
merely to satisfy a current catalog assertion. Historical adapter retirement and replay migration
follow the [retention policy](../decisions/0021-gameplay-retention.md);
this does not authorize deleting adapters still used by the live path.

## Extend the domain once

| Responsibility | Current extension point |
| --- | --- |
| Configurable item identity/order | `core/src/protocol/item_foundation/items.rs` (`ItemId`, currently 22 items) |
| Enabled settings bits | `core/src/game/enabled_items.rs` |
| Pickup kinds and optional-item mapping | `core/src/game/pickups.rs` (`PickupKind`, including three basic upgrades) |
| Replaceable equipment and weapon slots | `core/src/game/inventory.rs` |
| Timed effects, expiry and stacking | `core/src/game/effects.rs` (`TimedEffects`) |
| Collection, actions and simulation phases | `core/src/game/simulation_*.rs`, relevant damage/bomb/world modules |
| Deterministic placement and seeds | `core/src/game/generation/` |
| Checked public and recipient-private projection | `core/src/protocol/v11/`, shared item mappings in `protocol/item_foundation/` and version-specific helpers in v7/v8 |

Reuse the matching category's lifecycle. For example, use `TimedEffects::apply` and its tick
expiry rules for a timed buff instead of introducing a separate timer. Keep clocks, randomness
and I/O outside pure simulation. A terrain-like object such as Oil needs world collision/reveal
rules as well as collection behavior; it is not just another timed-effect entry.

`ItemId` and `PickupKind` serve different purposes. Review catalog order, bit bounds, default
settings, generation filtering and projection together. Stable item-bit/catalog order differs
from weighted generation sampling order; preserve both when changing deterministic placement.
Shared item/effect wire types now have neutral ownership; other shared Rust code still has live
callers under older protocol namespaces. Find those callers before changing or deleting it.

## Update contracts and presentation

Change the chosen contract first, then its checked Rust DTO/projection and browser runtime
validation. Regenerate types with `npm --prefix web run gen:gameplay`; never edit generated
`schema.ts`. Run `npm --prefix web run check:gameplay` to check every retained contract.
For v7 and later, the gameplay generator emits `ITEM_IDS` from that version’s own
`$defs.ItemId.enum`; the live settings panel and validators consume this generated catalog.
Archived validators keep their narrower activation subsets. A new protocol version also
requires an intentional update to the shared generator inventory.

Review `web/src/game/v11/validation.ts` and the shared helpers in `web/src/game/validation/`,
the generated catalog consumed by `web/src/game/client/GameplaySettingsPanel.tsx`, and
settings/client tests. Add matching labels
and descriptions to `web/src/locales/{en,pl}/game.json`. Preserve private recipient fields and
resume behavior; a hidden item must not leak through events, HUD, art or diagnostics.

Use the [artist handoff](../../assets/art/README.md) and [atlas guide](art-atlas.md) for graphics.
Update the existing procedural and atlas rendering paths where both remain supported, including
actor equipment art, pickup art, effects and accessible text.

## Prove the behavior

Add domain regressions for boundaries, exact expiry, replacement/stacking, interactions and
reset. Cover generation/settings round trips, replay determinism, strict wire parsing,
projection privacy and browser validation. Retain exhaustive acceptance and existing coverage
floors; a fast focused check is additional feedback.

Use Heart/Shield tests for ordinary collection or Oil tests for materialized objects. Browser
tests must use a documented deterministic fixture, native controls and read-only observations.
Add the spec to `e2e/scripts/scenario-map.mjs` and a document with exact `## Scenario:` titles;
run `scripts/gates.sh e2e`. For example, the existing Heart route uses
`E2E_HEART_FIXTURE=arena-v2-heart npm --prefix e2e run test:heart` against the separately started
Heart stack.

Run affected focused tests while iterating, then the applicable [gates](../testing.md) and
[release checklist](implementation-plan.md#release-checklist). Update the canonical rules,
item documentation and Unreleased changelog in the same contribution. Record which source and
environment passed; source/static checks alone do not establish native browser acceptance.
