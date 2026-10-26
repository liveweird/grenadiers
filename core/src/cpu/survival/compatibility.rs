use crate::cpu::{BombKnowledge, Observation, ObservedActor, ObservedBombTrigger, VisibleActor};
use crate::game::Tick;

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
    if old.world != new.world
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
    let (Some(before), Some(after)) = (actor(basis), actor(current)) else {
        return false;
    };
    if !after.alive
        || after.voluntary_actions_frozen
        || before.id != after.id
        || before.speed != after.speed
        || before.equipment != after.equipment
        || before.weapon != after.weapon
        || before.bomb_capacity != after.bomb_capacity
        || before.bomb_range != after.bomb_range
        || before.live_bomb_count != after.live_bomb_count
        || before.last_teleport_tick != after.last_teleport_tick
    {
        return false;
    }
    let expected_effects = before
        .effects
        .iter()
        .filter(|effect| u64::from(effect.remaining_ticks) > elapsed)
        .count();
    if expected_effects != after.effects.len()
        || after.effects.iter().any(|effect| {
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
            expected != actual
        })
    {
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
