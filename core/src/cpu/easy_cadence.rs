use crate::cpu::{BombKnowledge, Observation, ObservedActor, WorkBudget};
use crate::game::{Equipment, OilBarrelState, Tick};

const EASY_PAUSE_TICKS: u64 = 12;

/// Proves that an optional Easy cadence pause may keep the recipient stationary
/// through `through`. This is guidance only; callers must revalidate every
/// publication and retain their existing input authority.
#[must_use]
pub fn can_pause_easy_v68(observation: &Observation, through: Tick, work: &mut WorkBudget) -> bool {
    if work.try_take(1).is_err() || observation.parts().rules.version != 10 {
        return false;
    }
    let parts = observation.parts();
    if through < parts.tick
        || parts
            .tick
            .0
            .checked_add(EASY_PAUSE_TICKS)
            .is_none_or(|latest| through.0 > latest)
    {
        return false;
    }
    let Some(environment) = observation.environment() else {
        return false;
    };
    let storm_timers = environment.actor_storm_slow_remaining_ticks();
    if !charge_len(work, storm_timers.len()) {
        return false;
    }
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        return false;
    };
    if !charge_len(work, bombs.len()) || !bombs.is_empty() {
        return false;
    }
    if !charge_len(work, parts.flames.len())
        || !parts.flames.is_empty()
        || !charge_len(work, parts.oil.len())
        || parts
            .oil
            .iter()
            .any(|oil| oil.state == OilBarrelState::Burning)
        || !charge_len(work, parts.actors.len())
    {
        return false;
    }

    let mut recipient = None;
    for observed in &parts.actors {
        let effects = match observed {
            ObservedActor::Visible(actor) => actor.effects.len(),
            ObservedActor::Hidden { .. } => 0,
        };
        if !charge_len(work, effects) {
            return false;
        }
        match observed {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => {
                recipient = Some(actor);
            }
            ObservedActor::Visible(actor)
                if actor.alive && actor.equipment == Some(Equipment::Bracelet) =>
            {
                return false;
            }
            ObservedActor::Hidden { .. } => return false,
            ObservedActor::Visible(_) => {}
        }
    }
    let Some(actor) = recipient else {
        return false;
    };
    if !actor.alive || actor.voluntary_actions_frozen || !actor.effects.is_empty() {
        return false;
    }

    if let Some(pits) = observation.spike_pits()
        && (!charge_len(work, pits.pits().len())
            || pits
                .pits()
                .iter()
                .any(|pit| crate::game::position_contacts_spike_pit(actor.position, pit.cell)))
    {
        return false;
    }
    if let Some(lairs) = observation.lairs() {
        if !charge_len(work, lairs.lairs().len())
            || !charge_len(work, lairs.mobs().len())
            || super::has_urgent_lair_threat(observation, through, work)
        {
            return false;
        }
    } else if let Some(lairs) = observation.respawn_lairs()
        && (!charge_len(work, lairs.lairs().len())
            || !charge_len(work, lairs.mobs().len())
            || super::has_urgent_lair_threat(observation, through, work))
    {
        return false;
    }
    true
}

fn charge_len(work: &mut WorkBudget, len: usize) -> bool {
    u32::try_from(len).is_ok_and(|units| work.try_take(units).is_ok())
}
