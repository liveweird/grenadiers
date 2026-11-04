use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::StrategicTarget;
use crate::cpu::routing::{
    PlannedBomb, PlantFailure, PlantRequest, RouteFailure, RouteLimits,
    plan_lair_ordinary_plant_to_refuge,
};
use crate::cpu::{MobContactLease, MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{CellId, Tick};

use super::joint_refuge_geometry::refuge_candidates;

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForeignBombRawHint {
    pub movement: MovementIntent,
    pub refuge: CellId,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignBombRawOutcome {
    Plant(PlantFailure),
    UsefulnessWork,
    NotUseful,
    Contact(RouteFailure),
    Complete,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignBombRawDiagnostic {
    pub hint: ForeignBombRawHint,
    pub allowance: u32,
    pub total_spent: u32,
    pub route_spent: u32,
    pub usefulness_spent: u32,
    pub contact_spent: u32,
    pub placement: Option<PlannedBomb>,
    pub survive_until: Option<Tick>,
    pub useful: Option<bool>,
    pub contact_safe_until: Option<Tick>,
    pub outcome: ForeignBombRawOutcome,
}

/// Returns the existing policy's canonical pair without granting execution authority.
#[doc(hidden)]
pub fn foreign_bomb_raw_hints(
    observation: &Observation,
    target: StrategicTarget,
    mode: ForeignBombRawHintMode,
    work: &mut WorkBudget,
) -> Option<[ForeignBombRawHint; 2]> {
    let StrategicTarget::Block { cell: target } = target else {
        return None;
    };
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })?;
    let origin = actor.position.cell()?;
    let blocked_away = mode == ForeignBombRawHintMode::BlockedAway;
    refuge_candidates(
        observation,
        origin,
        target,
        actor.bomb_range,
        blocked_away,
        work,
    )
    .map(|hints| {
        hints.map(|hint| ForeignBombRawHint {
            movement: hint.movement,
            refuge: hint.refuge,
        })
    })
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignBombRawHintMode {
    Legacy,
    BlockedAway,
}

/// Runs route, usefulness and contact stages from one caller-supplied allowance.
/// The result contains no lease or execution certificate.
#[doc(hidden)]
pub fn diagnose_foreign_bomb_raw_candidate(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    target: StrategicTarget,
    hint: ForeignBombRawHint,
    allowance: u32,
) -> ForeignBombRawDiagnostic {
    let mut work = WorkBudget::new(allowance);
    let route_start = work.spent();
    let route = plan_lair_ordinary_plant_to_refuge(
        observation,
        history,
        PlantRequest {
            execution_tick: Tick(observation.parts().tick.0.saturating_add(7)),
            settled_stop_tick,
            first_movement: hint.movement,
            limits: RouteLimits::default(),
        },
        hint.refuge,
        &mut work,
    );
    let route_spent = work.spent().saturating_sub(route_start);
    let certificate = match route {
        Ok(certificate) => certificate,
        Err(error) => {
            return diagnostic(
                hint,
                allowance,
                &work,
                route_spent,
                0,
                0,
                None,
                None,
                None,
                None,
                ForeignBombRawOutcome::Plant(error),
            );
        }
    };
    let placement = *certificate.placement();
    let survive_until = certificate.route().survive_until();
    let useful_start = work.spent();
    let useful = super::plant_value::useful_plant_for(observation, &placement, target, &mut work);
    let usefulness_spent = work.spent().saturating_sub(useful_start);
    if useful != Some(true) {
        let outcome = if useful.is_none() {
            ForeignBombRawOutcome::UsefulnessWork
        } else {
            ForeignBombRawOutcome::NotUseful
        };
        return diagnostic(
            hint,
            allowance,
            &work,
            route_spent,
            usefulness_spent,
            0,
            Some(placement),
            Some(survive_until),
            useful,
            None,
            outcome,
        );
    }
    let contact_start = work.spent();
    let contact = MobContactLease::certify_route(observation, certificate.route(), &mut work);
    let contact_spent = work.spent().saturating_sub(contact_start);
    let (contact_safe_until, outcome) = match contact {
        Ok(contact) => (Some(contact.safe_until()), ForeignBombRawOutcome::Complete),
        Err(error) => (None, ForeignBombRawOutcome::Contact(error)),
    };
    diagnostic(
        hint,
        allowance,
        &work,
        route_spent,
        usefulness_spent,
        contact_spent,
        Some(placement),
        Some(survive_until),
        useful,
        contact_safe_until,
        outcome,
    )
}

#[allow(clippy::too_many_arguments)]
fn diagnostic(
    hint: ForeignBombRawHint,
    allowance: u32,
    work: &WorkBudget,
    route_spent: u32,
    usefulness_spent: u32,
    contact_spent: u32,
    placement: Option<PlannedBomb>,
    survive_until: Option<Tick>,
    useful: Option<bool>,
    contact_safe_until: Option<Tick>,
    outcome: ForeignBombRawOutcome,
) -> ForeignBombRawDiagnostic {
    ForeignBombRawDiagnostic {
        hint,
        allowance,
        total_spent: work.spent(),
        route_spent,
        usefulness_spent,
        contact_spent,
        placement,
        survive_until,
        useful,
        contact_safe_until,
        outcome,
    }
}
