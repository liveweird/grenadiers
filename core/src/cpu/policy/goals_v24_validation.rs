use crate::cpu::policy::{StrategicSwitchReason, StrategicTarget};
use crate::cpu::{Observation, ObservedActor};

pub(super) fn validate_previous(
    observation: &Observation,
    target: StrategicTarget,
) -> Result<(), StrategicSwitchReason> {
    match target {
        StrategicTarget::Pickup { cell, kind } => observation
            .parts()
            .pickups
            .iter()
            .any(|pickup| pickup.cell == cell && pickup.kind == kind)
            .then_some(())
            .ok_or(StrategicSwitchReason::Completed),
        StrategicTarget::Block { cell } => observation
            .parts()
            .arena
            .blocks
            .contains(&cell)
            .then_some(())
            .ok_or(StrategicSwitchReason::Completed),
        StrategicTarget::Opponent { seat } => match observation
            .parts()
            .actors
            .iter()
            .find(|actor| actor.seat() == seat)
        {
            Some(ObservedActor::Visible(actor)) if actor.alive => Ok(()),
            Some(ObservedActor::Visible(_)) => Err(StrategicSwitchReason::TargetInvalid),
            Some(ObservedActor::Hidden { alive: true, .. }) => {
                Err(StrategicSwitchReason::VisibilityLost)
            }
            _ => Err(StrategicSwitchReason::TargetInvalid),
        },
        StrategicTarget::Tile { cell } => observation
            .parts()
            .arena
            .contains(cell)
            .then_some(())
            .ok_or(StrategicSwitchReason::TargetInvalid),
        StrategicTarget::None => Err(StrategicSwitchReason::NoTargets),
    }
}
