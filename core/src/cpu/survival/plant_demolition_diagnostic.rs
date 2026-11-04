use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::StrategicTarget;
use crate::cpu::routing::{
    PlannedBomb, PlantCertificate, PlantRequest, RouteLimits, plan_lair_ordinary_plant,
    plan_lair_timed_weapon_plant,
};
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, Tick};

use super::diagnostic::PlantCandidateOutcome;
use super::{PlantSupport, actor, teleporter_plant_supported};

pub(in crate::cpu::survival) struct DemolitionCandidateProbe {
    pub certificate: Option<PlantCertificate>,
    pub spent: u32,
    pub useful: Option<bool>,
    pub outcome: PlantCandidateOutcome,
}

pub(in crate::cpu::survival) fn diagnose_demolition_candidate(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    movement: MovementIntent,
    block: CellId,
    support: PlantSupport,
    allowance: u32,
) -> DemolitionCandidateProbe {
    let mut work = WorkBudget::new(allowance);
    if !teleporter_plant_supported(observation, support, &mut work) {
        return failed(
            &work,
            PlantCandidateOutcome::TeleporterSupport(
                super::diagnostic::TeleporterSupportFailure::Other,
            ),
        );
    }
    let Some(actor) = actor(observation) else {
        return failed(&work, PlantCandidateOutcome::UsefulTargetWork);
    };
    let Some(execution_tick) = observation.parts().tick.0.checked_add(7).map(Tick) else {
        return failed(&work, PlantCandidateOutcome::UsefulTargetWork);
    };
    let request = PlantRequest {
        execution_tick,
        settled_stop_tick,
        first_movement: movement,
        limits: RouteLimits::default(),
    };
    let certificate = if actor.weapon.is_some() {
        plan_lair_timed_weapon_plant(observation, history, request, &mut work)
    } else {
        plan_lair_ordinary_plant(observation, history, request, &mut work)
    };
    let certificate = match certificate {
        Ok(certificate) => certificate,
        Err(error) => return failed(&work, PlantCandidateOutcome::LairPlant(error)),
    };
    let useful = super::super::plant_value::useful_plant_for(
        observation,
        certificate.placement(),
        StrategicTarget::Block { cell: block },
        &mut work,
    );
    DemolitionCandidateProbe {
        certificate: (useful == Some(true)).then_some(certificate),
        spent: work.spent(),
        useful,
        outcome: match useful {
            Some(true) => PlantCandidateOutcome::Certified,
            Some(false) => PlantCandidateOutcome::UsefulTargetRejected,
            None => PlantCandidateOutcome::UsefulTargetWork,
        },
    }
}

fn failed(work: &WorkBudget, outcome: PlantCandidateOutcome) -> DemolitionCandidateProbe {
    DemolitionCandidateProbe {
        certificate: None,
        spent: work.spent(),
        useful: None,
        outcome,
    }
}

pub(super) fn visible_soft_block_yield(
    observation: &Observation,
    placement: PlannedBomb,
) -> (Option<CellId>, Option<u8>, u32) {
    let mut work = WorkBudget::new(crate::cpu::survival::WORK_ALLOWANCE);
    let Some(geometry) =
        super::super::plant_value::ObservedBlastGeometry::new(observation, &mut work)
    else {
        return (None, None, work.spent());
    };
    for &block in &observation.parts().arena.blocks {
        if let Some(yield_count) = geometry.visible_block_yield(
            placement.cell,
            placement.range,
            placement.blast,
            block,
            &mut work,
        ) {
            return (Some(block), Some(yield_count), work.spent());
        }
    }
    (None, None, work.spent())
}
