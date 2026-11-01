use crate::cpu::{Observation, WorkBudget};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{CellId, Position, position_contacts_spike_pit};

use super::{PulseLease, RouteFailure};

/// Rejects a short pulse that would voluntarily acquire contact with a spike
/// pit. A pulse that starts in contact with a particular pit keeps the legacy
/// time-aware spike-pit proof for that pit.
pub(crate) fn avoids_new_pit_contact(
    observation: &Observation,
    lease: &PulseLease,
    work: &mut WorkBudget,
) -> Result<bool, RouteFailure> {
    let pits = observation.spike_pits().map_or(&[][..], |pits| pits.pits());
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor)
                if actor.seat == observation.parts().recipient =>
            {
                Some(actor)
            }
            _ => None,
        })
        .ok_or(RouteFailure::InvalidRequest)?;

    for pit in pits {
        charge(work, 1)?;
        if position_contacts_spike_pit(actor.position, pit.cell) {
            continue;
        }
        let mut previous = lease.pose_after(0);
        for completed in 0..=lease.hold_ticks() {
            charge(work, 1)?;
            let pose = lease.pose_after(completed);
            let min = Position {
                x: previous.min().x.min(pose.min().x),
                y: previous.min().y.min(pose.min().y),
            };
            let max = Position {
                x: previous.max().x.max(pose.max().x),
                y: previous.max().y.max(pose.max().y),
            };
            if envelope_contacts_cell(min, max, pit.cell) {
                return Ok(false);
            }
            previous = pose;
        }
    }
    Ok(true)
}

fn envelope_contacts_cell(min: Position, max: Position, cell: CellId) -> bool {
    let cell_min_x = i32::from(cell.column) * UNITS_PER_CELL;
    let cell_min_y = i32::from(cell.row) * UNITS_PER_CELL;
    let cell_max_x = cell_min_x + UNITS_PER_CELL - 1;
    let cell_max_y = cell_min_y + UNITS_PER_CELL - 1;
    min.x - HALF_FOOTPRINT <= cell_max_x
        && max.x + HALF_FOOTPRINT > cell_min_x
        && min.y - HALF_FOOTPRINT <= cell_max_y
        && max.y + HALF_FOOTPRINT > cell_min_y
}

fn charge(work: &mut WorkBudget, units: u32) -> Result<(), RouteFailure> {
    work.try_take(units)
        .map_err(|_| RouteFailure::WorkExhausted)
}

#[cfg(test)]
mod tests {
    use crate::cpu::forecast::TickWindow;
    use crate::cpu::routing::{PulseRequest, RouteMode, certify_respawn_pulse};
    use crate::cpu::{MovementIntent, Observation, ObservedActor, WorkBudget};
    use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
    use crate::game::{Direction, Position, SpikePitPhase, Tick};

    use super::avoids_new_pit_contact;

    fn observation_at(position: Position) -> Observation {
        let (basis, _) = crate::cpu::routing::spike_pits::tests::observed(
            SpikePitPhase::Idle,
            0,
            Some(20),
            0,
            false,
        );
        let environment = basis.environment().unwrap().clone();
        let pits = basis.spike_pits().unwrap().clone();
        let mut parts = basis.into_parts();
        let actor = parts
            .actors
            .iter_mut()
            .find_map(|actor| match actor {
                ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
                _ => None,
            })
            .unwrap();
        actor.position = position;
        actor.heading = Direction::East;
        Observation::new_rules_v10(parts, environment, None, Some(pits)).unwrap()
    }

    fn east_lease(observation: &Observation) -> super::PulseLease {
        let forecast = crate::cpu::routing::tests::forecast_for(observation);
        certify_respawn_pulse(
            observation,
            &forecast,
            PulseRequest {
                movement: MovementIntent::Direction(Direction::East),
                hold_ticks: 1,
                start_window: TickWindow {
                    start: Tick(1),
                    end: Tick(2),
                },
                stopped_safe_until: Tick(12),
                mode: RouteMode::CertifiedOnly,
                settled_stop_tick: Tick(0),
            },
            &mut WorkBudget::new(65_536),
        )
        .unwrap()
    }

    #[test]
    fn partial_footprint_entry_is_rejected_and_charged() {
        let (_, pit) = crate::cpu::routing::spike_pits::tests::observed(
            SpikePitPhase::Idle,
            0,
            None,
            0,
            false,
        );
        let observation = observation_at(Position {
            x: i32::from(pit.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
            y: Position::center(pit).y,
        });
        let lease = east_lease(&observation);
        let mut work = WorkBudget::new(65_536);
        assert!(!avoids_new_pit_contact(&observation, &lease, &mut work).unwrap());
        assert!(work.spent() > 0);
    }

    #[test]
    fn pulse_starting_on_pit_keeps_the_checked_legacy_result() {
        let (_, pit) = crate::cpu::routing::spike_pits::tests::observed(
            SpikePitPhase::Idle,
            0,
            None,
            0,
            false,
        );
        let observation = observation_at(Position::center(pit));
        let lease = east_lease(&observation);
        assert!(
            avoids_new_pit_contact(&observation, &lease, &mut WorkBudget::new(65_536)).unwrap()
        );
    }
}
