use crate::cpu::MovementIntent;
use crate::cpu::policy::PolicyError;
use crate::cpu::routing::{RouteCertificate, RouteStep};
use crate::game::Tick;

use super::route_execution::{RouteCommand, Transition};

pub(super) fn transitions(
    certificate: &RouteCertificate,
    first_update: Tick,
) -> Result<Vec<Transition>, PolicyError> {
    let lane = certificate
        .lane(first_update)
        .ok_or(PolicyError::InvalidObservation)?;
    let mut transitions = Vec::with_capacity(certificate.steps().len().saturating_mul(2));
    let mut previous_stop = None;
    for (index, step) in certificate.steps().iter().copied().enumerate() {
        let step_index = u8::try_from(index).map_err(|_| PolicyError::InvalidObservation)?;
        let start = lane
            .step_start(index)
            .ok_or(PolicyError::InvalidObservation)?;
        match step {
            RouteStep::Move { direction, .. } => {
                push(
                    &mut transitions,
                    start,
                    MovementIntent::Direction(direction),
                    step_index,
                    previous_stop,
                )?;
                let stop = lane
                    .stop_update(index)
                    .ok_or(PolicyError::InvalidObservation)?;
                push(
                    &mut transitions,
                    stop,
                    MovementIntent::Stop,
                    step_index,
                    None,
                )?;
                previous_stop = Some(stop);
            }
            RouteStep::Wait { .. } if index == 0 => {
                push(
                    &mut transitions,
                    start,
                    MovementIntent::Stop,
                    step_index,
                    None,
                )?;
                previous_stop = Some(
                    lane.stop_update(index)
                        .ok_or(PolicyError::InvalidObservation)?,
                );
            }
            RouteStep::Wait { .. } => {}
        }
    }
    Ok(transitions)
}

fn push(
    transitions: &mut Vec<Transition>,
    update: Tick,
    movement: MovementIntent,
    step_index: u8,
    required_stop_snapshot: Option<Tick>,
) -> Result<(), PolicyError> {
    let admission_tick = update
        .0
        .checked_sub(1)
        .map(Tick)
        .ok_or(PolicyError::InvalidObservation)?;
    if transitions
        .last()
        .is_some_and(|previous| previous.command.admission_tick >= admission_tick)
    {
        return Err(PolicyError::InvalidObservation);
    }
    transitions.push(Transition {
        command: RouteCommand {
            movement,
            admission_tick,
            step_index,
        },
        required_stop_snapshot,
    });
    Ok(())
}
