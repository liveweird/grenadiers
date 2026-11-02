use crate::cpu::routing::{RouteFailure, TeleporterStep, teleporter_max_travel};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBombTrigger, WorkBudget,
};
use crate::game::{Position, Tick};

pub(super) fn admission_fits(
    step: &TeleporterStep,
    basis: &Observation,
    current: &Observation,
    admission_tick: Tick,
    roster_v48: bool,
    work: &mut WorkBudget,
) -> Result<bool, RouteFailure> {
    charge_observations(basis, current, work)?;
    if basis != step.basis_observation()
        || current.parts().tick != admission_tick
        || !scene_fits(step, basis, current, None, roster_v48, work)?
        || !step.admission_contact_fits(current, work)?
        || !step.pit_layout_fits(current, work)?
        || !crate::cpu::routing::storm::observation_fits(basis, current, None)
    {
        return Ok(false);
    }
    Ok(true)
}

pub(super) fn stopped_fits(
    step: &TeleporterStep,
    current: &Observation,
    movement_tick: Tick,
    roster_v48: bool,
    work: &mut WorkBudget,
) -> Result<bool, RouteFailure> {
    let basis = step.basis_observation();
    charge_observations(basis, current, work)?;
    let (Some(before), Some(after)) = (recipient(basis), recipient(current)) else {
        return Ok(false);
    };
    let marker_ok = after.last_teleport_tick == before.last_teleport_tick
        || after.last_teleport_tick == Some(movement_tick);
    if !marker_ok
        || !scene_fits(step, basis, current, Some(movement_tick), roster_v48, work)?
        || !step.contact_fits_with_expected_marker(current, after.last_teleport_tick, work)?
        || !step.pit_layout_fits(current, work)?
        || !crate::cpu::routing::storm::observation_fits(basis, current, None)
    {
        return Ok(false);
    }
    let swapped_partners = current
        .parts()
        .actors
        .iter()
        .filter(|actor| {
            matches!(actor, ObservedActor::Visible(actor)
                if actor.seat != current.parts().recipient
                    && actor.last_teleport_tick == Some(movement_tick))
        })
        .count();
    let root_swapped = after.last_teleport_tick == Some(movement_tick);
    let consumed = !current
        .parts()
        .pickups
        .iter()
        .any(|pickup| pickup.cell == step.target());
    Ok(swapped_partners == usize::from(root_swapped)
        && (after.last_teleport_tick != Some(movement_tick) || consumed)
        && step.allows_stopped_position_charged(after.position, work)?)
}

fn scene_fits(
    step: &TeleporterStep,
    basis: &Observation,
    current: &Observation,
    movement_tick: Option<Tick>,
    roster_v48: bool,
    work: &mut WorkBudget,
) -> Result<bool, RouteFailure> {
    let old = basis.parts();
    let new = current.parts();
    let Some(elapsed) = new.tick.0.checked_sub(old.tick.0) else {
        return Ok(false);
    };
    if old.world != new.world
        || old.recipient != new.recipient
        || old.rules.version != new.rules.version
        || old.rules.punch_activation_policy != new.rules.punch_activation_policy
        || old.rules.tick_rate != new.rules.tick_rate
        || old.rules.settings != new.rules.settings
        || old.arena != new.arena
        || old.oil != new.oil
        || !new.flames.is_empty()
        || !clock_fits(basis, current, elapsed)
        || !bombs_fit(step.safe_until(), current)
        || !pickup_fits(step, basis, current, movement_tick.is_some())
        || old.actors.len() != new.actors.len()
    {
        return Ok(false);
    }
    let travel = teleporter_max_travel(elapsed)?;
    for (index, (before, after)) in old.actors.iter().zip(&new.actors).enumerate() {
        let (ObservedActor::Visible(before), ObservedActor::Visible(after)) = (before, after)
        else {
            return Ok(false);
        };
        let storm_slow_cleared = current
            .environment()
            .and_then(|environment| environment.actor_storm_slow_remaining_ticks().get(index))
            .is_some_and(|remaining| *remaining == Some(0));
        if !actor_facts_fit(
            before,
            after,
            elapsed,
            movement_tick.is_some() && before.seat != old.recipient,
            storm_slow_cleared,
            roster_v48,
        ) || (before.seat == old.recipient && before.live_bomb_count != after.live_bomb_count)
            || (before.seat == old.recipient
                && movement_tick.is_none()
                && (before.position != after.position
                    || before.heading != after.heading
                    || before.last_teleport_tick != after.last_teleport_tick))
            || (before.seat == old.recipient
                && movement_tick.is_some()
                && !matches!(step.movement(), MovementIntent::Direction(direction)
                    if after.heading == direction || after.heading == before.heading))
            || (before.seat != old.recipient
                && if before.last_teleport_tick == after.last_teleport_tick {
                    manhattan(before.position, after.position) > travel
                } else if after.last_teleport_tick == movement_tick {
                    let Some(movement_tick) = movement_tick else {
                        return Ok(false);
                    };
                    let partner_elapsed = new
                        .tick
                        .0
                        .checked_sub(movement_tick.0)
                        .ok_or(RouteFailure::InvalidRequest)?;
                    let partner_travel = teleporter_max_travel(partner_elapsed)?;
                    !step.allows_partner_position_charged(after.position, partner_travel, work)?
                } else {
                    true
                })
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn actor_facts_fit(
    before: &crate::cpu::VisibleActor,
    after: &crate::cpu::VisibleActor,
    elapsed: u64,
    rival_may_die: bool,
    storm_slow_cleared: bool,
    roster_v48: bool,
) -> bool {
    // Dead roster members are inert, but remain in every public snapshot.
    // Certification excludes them from swap partners; require their complete
    // facts to remain unchanged rather than rejecting every subsequent step.
    if roster_v48 && !before.alive {
        return before == after;
    }
    let stable = before.id == after.id
        && before.seat == after.seat
        && before.alive
        && before.speed == after.speed
        && before.bomb_capacity == after.bomb_capacity
        && before.bomb_range == after.bomb_range;
    let alive = after.alive
        && before.equipment == after.equipment
        && before.weapon == after.weapon
        && u64::from(after.damage_grace_remaining_ticks)
            == u64::from(before.damage_grace_remaining_ticks).saturating_sub(elapsed)
        && before.effects.is_empty()
        && after.effects.is_empty()
        && !after.voluntary_actions_frozen;
    let dead = !after.alive
        && rival_may_die
        && after.equipment.is_none()
        && after.weapon.is_none()
        && after.effects.is_empty()
        && after.damage_grace_remaining_ticks == 0
        && !after.voluntary_actions_frozen
        && storm_slow_cleared;
    stable && (alive || dead)
}

fn pickup_fits(
    step: &TeleporterStep,
    basis: &Observation,
    current: &Observation,
    after_movement: bool,
) -> bool {
    if !after_movement {
        return basis.parts().pickups == current.parts().pickups;
    }
    basis.parts().pickups == current.parts().pickups
        || basis
            .parts()
            .pickups
            .iter()
            .filter(|pickup| pickup.cell != step.target())
            .eq(&current.parts().pickups)
}

fn bombs_fit(safe_until: Tick, current: &Observation) -> bool {
    let BombKnowledge::Visible(bombs) = &current.parts().bombs else {
        return false;
    };
    bombs.iter().all(|bomb| match bomb.trigger {
        ObservedBombTrigger::Timed {
            remaining_ticks,
            paused: false,
        } => current
            .parts()
            .tick
            .0
            .checked_add(u64::from(remaining_ticks))
            .is_some_and(|detonates| detonates >= safe_until.0),
        ObservedBombTrigger::Timed { paused: true, .. }
        | ObservedBombTrigger::Remote
        | ObservedBombTrigger::Contact => false,
    })
}

fn clock_fits(basis: &Observation, current: &Observation, elapsed: u64) -> bool {
    current.parts().rules.running_updates
        == basis
            .parts()
            .rules
            .running_updates
            .saturating_add(elapsed as u16)
        && current.parts().rules.remaining_ticks
            == basis
                .parts()
                .rules
                .remaining_ticks
                .saturating_sub(elapsed as u16)
}

fn recipient(observation: &Observation) -> Option<&crate::cpu::VisibleActor> {
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

fn manhattan(left: Position, right: Position) -> i64 {
    (i64::from(left.x) - i64::from(right.x)).abs() + (i64::from(left.y) - i64::from(right.y)).abs()
}

fn charge_observations(
    basis: &Observation,
    current: &Observation,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let units = observation_scan_units(basis)
        .and_then(|value| value.checked_add(observation_scan_units(current)?))
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(units)
        .map_err(|_| RouteFailure::WorkExhausted)
}

fn observation_scan_units(observation: &Observation) -> Option<usize> {
    let parts = observation.parts();
    let bombs = match &parts.bombs {
        BombKnowledge::Visible(bombs) => bombs.len(),
        BombKnowledge::Unavailable => 0,
    };
    let lairs = observation.respawn_lairs();
    let pits = observation.spike_pits();
    parts
        .arena
        .cell_count()
        .checked_add(parts.actors.len().checked_mul(4)?)?
        .checked_add(parts.pickups.len().checked_mul(3)?)?
        .checked_add(bombs)?
        .checked_add(parts.flames.len())?
        .checked_add(parts.oil.len())?
        .checked_add(lairs.map_or(0, |value| value.lairs().len()))?
        .checked_add(lairs.map_or(0, |value| value.mobs().len()))?
        .checked_add(pits.map_or(0, |value| value.pits().len()))?
        .checked_add(16)
}
