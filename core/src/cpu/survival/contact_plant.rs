use crate::cpu::lair_contact::{PLANT_CONTACT_RENEWAL_ALLOWANCE, PlantContactProof};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{PlantCertificate, PlantPresence, RouteCertificate};
use crate::cpu::{Intent, MovementIntent, Observation, WorkBudget};
use crate::game::Direction;

use super::compatibility::plant_actor_compatible_charged;
use super::{ExecutionCertificate, LairSurvivalPlan, SurvivalPlan};

#[allow(clippy::too_many_arguments)]
pub(super) fn certified_candidates(
    observation: &Observation,
    history: &crate::cpu::forecast::ObservedHazardMemory,
    settled_stop_tick: crate::game::Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    strategic: bool,
    support: super::plant_tactic::PlantSupport,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let selected = ordinary.next_memory.strategic_intent();
    if strategic
        && selected.is_none_or(|intent| {
            !matches!(
                intent.phase,
                crate::cpu::policy::StrategicPhase::Approach
                    | crate::cpu::policy::StrategicPhase::Reengage
            )
        })
    {
        return None;
    }
    if selected.is_some_and(|intent| {
        !matches!(
            intent.target,
            crate::cpu::policy::StrategicTarget::Block { .. }
                | crate::cpu::policy::StrategicTarget::Opponent { .. }
        )
    }) {
        return None;
    }
    let mut prefilter = WorkBudget::new(work.remaining() / 8);
    let useful = super::plant_value::could_be_useful(observation, &mut prefilter);
    work.try_take(prefilter.spent()).ok()?;
    if !useful {
        return None;
    }
    let mut movements = Vec::with_capacity(5);
    movements.push(ordinary.intent.movement);
    for movement in [
        MovementIntent::Stop,
        MovementIntent::Direction(Direction::North),
        MovementIntent::Direction(Direction::East),
        MovementIntent::Direction(Direction::South),
        MovementIntent::Direction(Direction::West),
    ] {
        if movements.len() == 5 {
            break;
        }
        if !movements.contains(&movement) {
            movements.push(movement);
        }
    }
    let total = work.remaining();
    let count = u32::try_from(movements.len()).ok()?;
    let reserve = total / count.saturating_mul(2);
    for (index, movement) in movements.into_iter().enumerate() {
        let slice = if index == 0 {
            total.saturating_sub(reserve.saturating_mul(count.saturating_sub(1)))
        } else {
            reserve
        };
        let candidate = ControlPlan {
            intent: Intent {
                movement,
                manual: None,
            },
            hold_ticks: ordinary.hold_ticks,
            next_memory: ordinary.next_memory,
        };
        let mut child = WorkBudget::new(slice);
        let planned = super::plant_tactic::certified_plant_inner_prefiltered(
            observation,
            history,
            settled_stop_tick,
            memory,
            &candidate,
            support,
            &mut child,
        )
        .and_then(|mut planned| {
            planned.control.next_memory = planned
                .control
                .next_memory
                .carry_progress(ordinary.next_memory);
            authorize_lair_plant(observation, planned, true, &mut child)
        });
        work.try_take(child.spent()).ok()?;
        if planned.is_some() {
            return planned;
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
pub(super) fn certified_lair_candidate(
    observation: &Observation,
    history: &crate::cpu::forecast::ObservedHazardMemory,
    settled_stop_tick: crate::game::Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    opportunistic: bool,
    strategic: bool,
    topology: bool,
    support: super::plant_tactic::PlantSupport,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    if opportunistic && topology {
        return certified_candidates(
            observation,
            history,
            settled_stop_tick,
            memory,
            ordinary,
            strategic,
            support,
            work,
        );
    }
    let plant = if opportunistic {
        super::plant_tactic::certified_plant_candidates(
            observation,
            history,
            settled_stop_tick,
            memory,
            ordinary,
            strategic,
            support,
            work,
        )
    } else {
        super::plant_tactic::certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            ordinary,
            true,
            support,
            work,
        )
    };
    plant.and_then(|plant| authorize_lair_plant(observation, plant, false, work))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlantContactCertificate {
    plant: PlantCertificate,
    proof: PlantContactProof,
}

pub(super) fn authorize_lair_plant(
    observation: &Observation,
    mut plant: SurvivalPlan,
    topology: bool,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let ExecutionCertificate::Plant(certificate) = plant.execution.as_ref()? else {
        return None;
    };
    match crate::cpu::MobContactLease::certify_route(observation, certificate.route(), work) {
        Ok(contact) => Some(LairSurvivalPlan {
            survival: plant,
            contact: Some(contact),
        }),
        Err(crate::cpu::routing::RouteFailure::NoRoute) if topology => {
            let proof = PlantContactProof::certify(observation, certificate, work)?;
            let Some(ExecutionCertificate::Plant(certificate)) = plant.execution.take() else {
                return None;
            };
            plant.execution = Some(ExecutionCertificate::ContactPlant(Box::new(
                PlantContactCertificate::new(*certificate, proof),
            )));
            Some(LairSurvivalPlan {
                survival: plant,
                contact: None,
            })
        }
        Err(_) => None,
    }
}

impl PlantContactCertificate {
    pub(crate) const fn new(plant: PlantCertificate, proof: PlantContactProof) -> Self {
        Self { plant, proof }
    }

    #[must_use]
    pub const fn plant(&self) -> &PlantCertificate {
        &self.plant
    }

    #[must_use]
    pub fn route(&self) -> &RouteCertificate {
        self.plant.route()
    }

    pub(crate) fn into_parts(self) -> (PlantCertificate, PlantContactProof) {
        (self.plant, self.proof)
    }
}

pub(super) fn contact_scene_candidate(
    basis: &Observation,
    current: &Observation,
    plant: &PlantCertificate,
    proof: &PlantContactProof,
    established: PlantPresence,
) -> Option<PlantPresence> {
    let mut work = WorkBudget::new(PLANT_CONTACT_RENEWAL_ALLOWANCE);
    let candidate = plant.contact_scene_candidate(basis, current, established, &mut work)?;
    plant_actor_compatible_charged(basis, current, plant, candidate, &mut work)
        .then_some(())
        .and_then(|()| {
            proof
                .observation_fits(current, &mut work)
                .then_some(candidate)
        })
}
