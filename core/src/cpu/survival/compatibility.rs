use crate::cpu::forecast::ForecastEvolution;
use crate::cpu::routing::{PlantCertificate, PlantPresence};
use crate::cpu::{
    BombKnowledge, Observation, ObservedActor, ObservedBombTrigger, VisibleActor, WorkBudget,
};
use crate::game::Tick;

pub(super) fn visible_rivals_allow_segment(
    observation: &Observation,
    start: crate::game::Position,
    end: crate::game::Position,
) -> bool {
    if !matches!(observation.parts().rules.version, 9 | 10) || start == end {
        return true;
    }
    let (direction, requested) = if end.x > start.x && end.y == start.y {
        (crate::game::Direction::East, end.x - start.x)
    } else if end.x < start.x && end.y == start.y {
        (crate::game::Direction::West, start.x - end.x)
    } else if end.y > start.y && end.x == start.x {
        (crate::game::Direction::South, end.y - start.y)
    } else if end.y < start.y && end.x == start.x {
        (crate::game::Direction::North, start.y - end.y)
    } else if end.x != start.x && end.y != start.y {
        // A cornering tick can contain an alignment segment followed by forward
        // motion. The certificate exposes only the endpoints, so require both
        // possible axis orderings to be clear against the same public rivals.
        let horizontal_first = crate::game::Position {
            x: end.x,
            y: start.y,
        };
        let vertical_first = crate::game::Position {
            x: start.x,
            y: end.y,
        };
        return visible_rivals_allow_segment(observation, start, horizontal_first)
            && visible_rivals_allow_segment(observation, horizontal_first, end)
            && visible_rivals_allow_segment(observation, start, vertical_first)
            && visible_rivals_allow_segment(observation, vertical_first, end);
    } else {
        return true;
    };
    let obstacles = observation
        .parts()
        .actors
        .iter()
        .filter_map(|candidate| match candidate {
            ObservedActor::Visible(rival)
                if rival.alive && rival.seat != observation.parts().recipient =>
            {
                Some(crate::game::player_collision::PlayerObstacle::stationary(
                    rival.seat,
                    rival.position,
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    crate::game::player_collision::clip_player_segment(
        observation.parts().recipient,
        start,
        direction,
        requested,
        &obstacles,
    )
    .distance
        == requested
}

pub(super) fn visible_rivals_allow_bounds(
    observation: &Observation,
    start: crate::game::Position,
    bounds: crate::cpu::routing::PoseBounds,
) -> bool {
    visible_rivals_allow_segment(observation, start, bounds.min())
        && visible_rivals_allow_segment(observation, start, bounds.max())
}

pub(super) fn actor(observation: &Observation) -> Option<&VisibleActor> {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
}

/// Checks a worker's stopped basis before admitting ordinary input. This also
/// fences a narrow planting proof against newly observed hazards.
#[must_use]
pub fn can_admit_ordinary(basis: &Observation, current: &Observation) -> bool {
    compatible(basis, current)
        && actor(basis)
            .zip(actor(current))
            .is_some_and(|(before, after)| {
                before.position == after.position && before.heading == after.heading
            })
}

/// Checks bounded recipient-visible invariants; normal countdowns retain their
/// absolute expiry. New facts require a new forecast, even if harmless.
pub(super) fn compatible(basis: &Observation, current: &Observation) -> bool {
    let old = basis.parts();
    let new = current.parts();
    let Some(elapsed) = new.tick.0.checked_sub(old.tick.0) else {
        return false;
    };
    if !crate::cpu::routing::storm::observation_fits(basis, current, None)
        || old.world != new.world
        || old.recipient != new.recipient
        || old.rules.version != new.rules.version
        || old.rules.tick_rate != new.rules.tick_rate
        || old.rules.settings != new.rules.settings
        || old.arena != new.arena
        || old.oil != new.oil
        || old.pickups != new.pickups
    {
        return false;
    }
    let expected_live_bombs = actor(basis).map(|before| before.live_bomb_count);
    let expected_weapon = actor(basis).and_then(|before| before.weapon);
    if !actor_compatible(basis, current, expected_live_bombs, expected_weapon) {
        return false;
    }
    let (BombKnowledge::Visible(old_bombs), BombKnowledge::Visible(new_bombs)) =
        (&old.bombs, &new.bombs)
    else {
        return false;
    };
    if old_bombs.len() != new_bombs.len()
        || old_bombs.iter().any(|bomb| {
            !new_bombs.iter().any(|candidate| {
                bomb.id == candidate.id
                    && bomb.cell == candidate.cell
                    && bomb.owner == candidate.owner
                    && bomb.range == candidate.range
                    && bomb.blast == candidate.blast
                    && trigger_matches(bomb.trigger, candidate.trigger, elapsed)
            })
        })
    {
        return false;
    }
    let expected_flames = old
        .flames
        .iter()
        .filter(|flame| u64::from(flame.remaining_ticks) > elapsed)
        .count();
    expected_flames == new.flames.len()
        && new.flames.iter().all(|flame| {
            old.flames.iter().any(|previous| {
                previous.cell == flame.cell
                    && expiry(old.tick, previous.remaining_ticks)
                        == expiry(new.tick, flame.remaining_ticks)
            })
        })
}

/// Checks the actor facts which are outside the forecast's expected public
/// scene evolution. The route lane separately checks the exact CPU pose.
pub(super) fn route_compatible(
    basis: &Observation,
    current: &Observation,
    evolution: &ForecastEvolution,
) -> bool {
    if !evolution.matches_scene(basis, current) {
        return false;
    }
    let expected = actor(basis)
        .and_then(|before| evolution.expected_live_bombs(before.id, current.parts().tick));
    let expected_weapon = actor(basis).and_then(|before| before.weapon);
    actor_compatible(basis, current, expected, expected_weapon)
}

pub(super) fn actor_compatible(
    basis: &Observation,
    current: &Observation,
    expected_live_bombs: Option<u16>,
    expected_weapon: Option<crate::game::SpecialWeapon>,
) -> bool {
    let Some(elapsed) = current.parts().tick.0.checked_sub(basis.parts().tick.0) else {
        return false;
    };
    let (Some(before), Some(after)) = (actor(basis), actor(current)) else {
        return false;
    };
    let expected_damage_grace =
        u64::from(before.damage_grace_remaining_ticks).saturating_sub(elapsed);
    before.alive
        && after.alive
        && !after.voluntary_actions_frozen
        && before.id == after.id
        && before.speed == after.speed
        && before.equipment == after.equipment
        && after.weapon == expected_weapon
        && before.bomb_capacity == after.bomb_capacity
        && before.bomb_range == after.bomb_range
        && Some(after.live_bomb_count) == expected_live_bombs
        && before.last_teleport_tick == after.last_teleport_tick
        && expected_damage_grace == u64::from(after.damage_grace_remaining_ticks)
        && effects_match(before, after, elapsed)
}

pub(super) fn plant_actor_compatible(
    basis: &Observation,
    observation: &Observation,
    certificate: &PlantCertificate,
    presence: PlantPresence,
) -> bool {
    let owner = certificate.placement().owner;
    let expected = certificate.expected_live_bombs(owner, observation.parts().tick, presence);
    actor(basis)
        .is_some_and(|actor| actor.id == owner && actor.weapon == certificate.weapon_before())
        && actor_compatible(
            basis,
            observation,
            expected,
            certificate.expected_weapon(presence),
        )
}

pub(super) fn plant_actor_compatible_charged(
    basis: &Observation,
    observation: &Observation,
    certificate: &PlantCertificate,
    presence: PlantPresence,
    work: &mut WorkBudget,
) -> bool {
    let old = basis.parts();
    let new = observation.parts();
    let (Some(before), Some(after)) = (actor(basis), actor(observation)) else {
        return false;
    };
    let Some(effect_width) = before.effects.len().checked_add(after.effects.len()) else {
        return false;
    };
    let Some(effects) = after
        .effects
        .len()
        .checked_mul(effect_width)
        .and_then(|nested| nested.checked_add(before.effects.len()))
        .and_then(|nested| nested.checked_add(after.effects.len()))
    else {
        return false;
    };
    let charge = old
        .actors
        .len()
        .checked_mul(2)
        .and_then(|count| count.checked_add(new.actors.len()))
        .and_then(|count| count.checked_add(effects))
        .and_then(|count| {
            count.checked_add(usize::try_from(certificate.contact_actor_work()?).ok()?)
        })
        .and_then(|count| u32::try_from(count.checked_add(16)?).ok());
    charge.is_some_and(|charge| work.try_take(charge).is_ok())
        && plant_actor_compatible(basis, observation, certificate, presence)
}

fn effects_match(before: &VisibleActor, after: &VisibleActor, elapsed: u64) -> bool {
    let expected_effects = before
        .effects
        .iter()
        .filter(|effect| u64::from(effect.remaining_ticks) > elapsed)
        .count();
    expected_effects == after.effects.len()
        && after.effects.iter().all(|effect| {
            let expected = before
                .effects
                .iter()
                .filter(|candidate| {
                    candidate.kind == effect.kind
                        && u64::from(candidate.remaining_ticks).checked_sub(elapsed)
                            == Some(u64::from(effect.remaining_ticks))
                })
                .count();
            let actual = after
                .effects
                .iter()
                .filter(|candidate| **candidate == *effect)
                .count();
            expected == actual
        })
}

fn expiry(tick: Tick, remaining: u16) -> Option<u64> {
    tick.0.checked_add(u64::from(remaining))
}

fn trigger_matches(before: ObservedBombTrigger, after: ObservedBombTrigger, elapsed: u64) -> bool {
    match (before, after) {
        (
            ObservedBombTrigger::Timed {
                remaining_ticks: old,
                paused: old_pause,
            },
            ObservedBombTrigger::Timed {
                remaining_ticks: new,
                paused: new_pause,
            },
        ) => {
            old_pause == new_pause
                && if old_pause {
                    old == new
                } else {
                    u64::from(old).checked_sub(elapsed) == Some(u64::from(new))
                }
        }
        (ObservedBombTrigger::Remote, ObservedBombTrigger::Remote)
        | (ObservedBombTrigger::Contact, ObservedBombTrigger::Contact) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::policy::tests::observation;
    use crate::cpu::{Observation, ObservedActor};
    use crate::game::{CellId, SpecialWeapon};

    use super::actor_compatible;

    #[test]
    fn actor_compatibility_allows_only_the_proven_weapon_transition() {
        let mut basis_parts = observation(CellId::new(7, 7)).into_parts();
        let ObservedActor::Visible(before) = &mut basis_parts.actors[1] else {
            unreachable!()
        };
        before.weapon = Some(SpecialWeapon::Power);
        let basis = Observation::new(basis_parts).unwrap();
        let mut current_parts = basis.clone().into_parts();
        let ObservedActor::Visible(after) = &mut current_parts.actors[1] else {
            unreachable!()
        };
        after.weapon = None;
        let current = Observation::new(current_parts).unwrap();

        assert!(actor_compatible(&basis, &current, Some(0), None));
        assert!(!actor_compatible(
            &basis,
            &current,
            Some(0),
            Some(SpecialWeapon::Power)
        ));
    }
}
