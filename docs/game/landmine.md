# Landmine

Corrected-source acceptance at `420d8df` passes fresh **50 Compose plus 50 Kubernetes** cases with zero retries or carried results. Remote and Landmine use fresh placement-ID checks across rolling observer history. Product behavior is unchanged; the new evidence is audited/sealed, while exact-head CI/post-merge CI and publication remain pending. See [verification](../verification.md).

Landmine is a configurable Expanded-v5 one-charge weapon enabled in new rooms. A successful manual bomb action consumes
the stored charge and places a visible, passable, capacity-counted Standard bomb with a Contact
trigger. It has no fuse. Automatic Diarrhea placements remain ordinary Timed bombs and do not
consume the charge.

Every actor overlapping a mine during initial State construction receives exit grace, and newly
placed mines grant grace to actors overlapping their cell. Reconnect and resume preserve the
existing simulation grace state; they do not grant fresh grace.
Standing still and boundary tangency do not trigger contact. Grace ends only after the actor is
fully outside; any later swept entry, including the owner's, detonates the mine. Only mines that
existed before movement are eligible, so placement cannot trigger itself in the same tick.
Contact accepted during movement remains authoritative if the actor subsequently dies in existing
fire during that tick.

Fire and other bomb blasts destroy an untriggered mine without detonating it, publishing a
`BombDetonated` event, or extending flames. Blast rays continue through the destroyed mine. An
accepted contact wins over destruction in the same immutable explosion plan. Timed and Remote
bombs retain their original obstruction snapshot and chain behavior.

Version 0.3.37 introduced Landmine with generator weight 15 after Remote Bomb, growing the pool
to 305 entries while preserving every earlier 4096-mask path. Its original thirteen-item catalog
supported all 8192 subsets; those paths remain stable in the [current catalog](settings.md).
Rules version 5, generator version 2 and journal format 2 remain unchanged. Current deployments
use [gameplay-v11](protocol-v11.md); reload older browsers with the paired server/web upgrade.

## Historical v0.3.37 native acceptance



Fresh same-image native acceptance passes 50 Compose and 50 Kubernetes cases, with zero retries
or carried passes. Independent raw review is clear and the final evidence manifest is sealed.
Both clients observe the single consumed charge, Contact/Standard bomb, persistence beyond
183 ticks, exact-ID contact detonation with a live origin flame, no extra placement, owner
elimination and seat Two's round win. The following fresh Timed Standard bomb is covered by
`consumed_landmine_is_followed_by_a_fresh_standard_timed_bomb_in_the_same_state` in the
[core tests](../../core/src/game/simulation/landmine_tests.rs), rather than a separate browser
assertion. Exact-head candidate/post-merge CI and publication remain required release gates.

Reviewed Compose [English desktop](../screenshots/landmine-desktop.png) and
[Polish 320px reduced-motion](../screenshots/landmine-polish-320.png) captures show the planted
pressure plate before detonation and normal HUD layout. Kubernetes captures were retained but
not visually reviewed. See [verification](../verification.md) for source/image identities,
latency, seal, historical attempts and measurement limits.
