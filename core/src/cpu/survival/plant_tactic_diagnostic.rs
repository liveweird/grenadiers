use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{ControlPlan, StrategicTarget, UtilityMemory};
use crate::cpu::routing::{
    PlannedBomb, PlantCertificate, PlantFailure, PlantRequest, RouteLimits,
    plan_lair_ordinary_plant, plan_lair_timed_weapon_plant, plan_ordinary_plant,
    plan_timed_weapon_plant,
};
use crate::cpu::{Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, Direction, Position, Tick};

use super::demolition_diagnostic::visible_soft_block_yield;
use super::{PlantSupport, actor, teleporter_plant_supported};
use crate::cpu::survival::{ExecutionCertificate, SurvivalPlan};

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlantCandidateDiagnostic {
    pub movement: MovementIntent,
    pub support: PlantSupportDiagnostic,
    pub allowance: u32,
    pub spent: u32,
    pub route_spent: u32,
    pub usefulness_spent: u32,
    pub placement_derivation_spent: u32,
    pub placement: Option<PlannedBomb>,
    pub placement_failure: Option<PlantFailure>,
    pub target: Option<StrategicTarget>,
    pub target_position: Option<Position>,
    pub usefulness_probe_spent: u32,
    pub derived_exact_usefulness: Option<bool>,
    pub visible_soft_block: Option<CellId>,
    pub visible_soft_block_yield: Option<u8>,
    pub visible_yield_spent: u32,
    pub exact_usefulness: Option<bool>,
    pub outcome: PlantCandidateOutcome,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantSupportDiagnostic {
    Strict,
    PersistentBurningOil,
    ClearArenaCounterPressure,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantCandidateOutcome {
    TeleporterSupport(TeleporterSupportFailure),
    LairPlant(PlantFailure),
    UsefulTargetWork,
    UsefulTargetRejected,
    Certified,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleporterSupportFailure {
    VisiblePickup,
    ActiveFlame,
    HiddenBombs,
    VisibleBombs,
    BurningOil,
    Other,
}

pub(in crate::cpu::survival) struct CandidateProbe {
    pub plan: Option<SurvivalPlan>,
    pub certificate: Option<PlantCertificate>,
    pub prefilter_allowance: u32,
    pub prefilter_spent: u32,
    pub prefilter_useful: Option<bool>,
    pub candidates: Vec<PlantCandidateDiagnostic>,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cpu::survival) fn diagnose_candidates(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    strategic: bool,
    support: PlantSupport,
    work: &mut WorkBudget,
) -> CandidateProbe {
    let mut probe = CandidateProbe {
        plan: None,
        certificate: None,
        prefilter_allowance: 0,
        prefilter_spent: 0,
        prefilter_useful: None,
        candidates: Vec::new(),
    };
    let selected = ordinary.next_memory.strategic_intent();
    if strategic && selected.is_none()
        || strategic
            && selected.is_some_and(|intent| {
                !matches!(
                    intent.phase,
                    crate::cpu::policy::StrategicPhase::Approach
                        | crate::cpu::policy::StrategicPhase::Reengage
                )
            })
        || selected.is_some_and(|intent| {
            !matches!(
                intent.target,
                crate::cpu::policy::StrategicTarget::Block { .. }
                    | crate::cpu::policy::StrategicTarget::Opponent { .. }
            )
        })
    {
        return probe;
    }
    probe.prefilter_allowance = work.remaining() / 8;
    let mut prefilter = WorkBudget::new(probe.prefilter_allowance);
    let useful = super::super::plant_value::could_be_useful(observation, &mut prefilter);
    probe.prefilter_spent = prefilter.spent();
    probe.prefilter_useful = Some(useful);
    if work.try_take(prefilter.spent()).is_err() || !useful {
        return probe;
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
    let count = u32::try_from(movements.len()).unwrap_or(1);
    let reserve = total / count.saturating_mul(2);
    for (index, movement) in movements.into_iter().enumerate() {
        let allowance = if index == 0 {
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
        let mut child = WorkBudget::new(allowance);
        let target = candidate
            .next_memory
            .strategic_intent()
            .or(memory.strategic_intent())
            .map(|intent| intent.target);
        let mut diagnostic = PlantCandidateDiagnostic {
            movement,
            support: support.into(),
            allowance,
            spent: 0,
            route_spent: 0,
            usefulness_spent: 0,
            placement_derivation_spent: 0,
            placement: None,
            placement_failure: None,
            target,
            target_position: target.and_then(|target| target_position(observation, target)),
            usefulness_probe_spent: 0,
            derived_exact_usefulness: None,
            visible_soft_block: None,
            visible_soft_block_yield: None,
            visible_yield_spent: 0,
            exact_usefulness: None,
            outcome: PlantCandidateOutcome::UsefulTargetWork,
        };
        let result = diagnose_candidate(
            observation,
            history,
            settled_stop_tick,
            memory,
            &candidate,
            support,
            &mut child,
            &mut diagnostic,
        );
        let spent = child.spent();
        let outcome = match &result {
            Ok((_, _, true)) => PlantCandidateOutcome::Certified,
            Ok((_, _, false)) => PlantCandidateOutcome::UsefulTargetRejected,
            Err(outcome) => *outcome,
        };
        diagnostic.spent = spent;
        diagnostic.outcome = outcome;
        if diagnostic.derived_exact_usefulness == Some(false)
            && let Some(placement) = diagnostic.placement
        {
            if let Some(previous) = probe.candidates.iter().find(|candidate| {
                candidate.placement.is_some_and(|prior| {
                    prior.cell == placement.cell
                        && prior.range == placement.range
                        && prior.blast == placement.blast
                })
            }) {
                diagnostic.visible_soft_block = previous.visible_soft_block;
                diagnostic.visible_soft_block_yield = previous.visible_soft_block_yield;
                diagnostic.visible_yield_spent = previous.visible_yield_spent;
            } else {
                let (block, yield_count, spent) = visible_soft_block_yield(observation, placement);
                diagnostic.visible_soft_block = block;
                diagnostic.visible_soft_block_yield = yield_count;
                diagnostic.visible_yield_spent = spent;
            }
        }
        probe.candidates.push(diagnostic);
        if work.try_take(spent).is_err() {
            return probe;
        }
        if let Ok((certificate, plan, true)) = result {
            probe.certificate = Some(certificate);
            probe.plan = Some(plan);
            return probe;
        }
    }
    probe
}

#[allow(clippy::too_many_arguments)]
fn diagnose_candidate(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    support: PlantSupport,
    work: &mut WorkBudget,
    diagnostic: &mut PlantCandidateDiagnostic,
) -> Result<(PlantCertificate, SurvivalPlan, bool), PlantCandidateOutcome> {
    let execution_tick = observation
        .parts()
        .tick
        .0
        .checked_add(7)
        .map(Tick)
        .ok_or(PlantCandidateOutcome::UsefulTargetWork)?;
    let request = PlantRequest {
        execution_tick,
        settled_stop_tick,
        first_movement: ordinary.intent.movement,
        limits: RouteLimits::default(),
    };
    let mut placement_work = WorkBudget::new(crate::cpu::survival::WORK_ALLOWANCE);
    match crate::cpu::routing::derive_lair_plant_placement(
        observation,
        history,
        request,
        &mut placement_work,
    ) {
        Ok(placement) => diagnostic.placement = Some(placement),
        Err(error) => diagnostic.placement_failure = Some(error),
    }
    diagnostic.placement_derivation_spent = placement_work.spent();
    if let (Some(placement), Some(target)) = (diagnostic.placement, diagnostic.target) {
        let mut usefulness = WorkBudget::new(crate::cpu::survival::WORK_ALLOWANCE);
        diagnostic.derived_exact_usefulness = super::super::plant_value::useful_plant_for(
            observation,
            &placement,
            target,
            &mut usefulness,
        );
        diagnostic.usefulness_probe_spent = usefulness.spent();
    }
    if !teleporter_plant_supported(observation, support, work) {
        return Err(PlantCandidateOutcome::TeleporterSupport(
            teleporter_failure(observation),
        ));
    }
    let actor = actor(observation).ok_or(PlantCandidateOutcome::UsefulTargetWork)?;
    let lairs = matches!(observation.parts().rules.version, 8..=10);
    let route_start = work.spent();
    let certificate = if actor.weapon.is_some() && lairs {
        plan_lair_timed_weapon_plant(observation, history, request, work)
    } else if actor.weapon.is_some() {
        plan_timed_weapon_plant(observation, history, request, work)
    } else if lairs {
        plan_lair_ordinary_plant(observation, history, request, work)
    } else {
        plan_ordinary_plant(observation, history, request, work)
    };
    diagnostic.route_spent = work.spent().saturating_sub(route_start);
    let certificate = certificate.map_err(PlantCandidateOutcome::LairPlant)?;
    debug_assert_eq!(diagnostic.placement, Some(*certificate.placement()));
    let strategic = ordinary
        .next_memory
        .strategic_intent()
        .or(memory.strategic_intent());
    let useful_start = work.spent();
    let useful = if let Some(intent) = strategic {
        super::super::plant_value::useful_plant_for(
            observation,
            certificate.placement(),
            intent.target,
            work,
        )
    } else {
        super::super::plant_value::useful_plant(observation, certificate.placement(), work)
    };
    diagnostic.usefulness_spent = work.spent().saturating_sub(useful_start);
    let useful = useful.ok_or(PlantCandidateOutcome::UsefulTargetWork)?;
    diagnostic.exact_usefulness = Some(useful);
    debug_assert_eq!(diagnostic.derived_exact_usefulness, Some(useful));
    let plan = SurvivalPlan {
        control: ControlPlan {
            intent: Intent {
                movement: ordinary.intent.movement,
                manual: Some(ManualIntent::Bomb),
            },
            hold_ticks: 1,
            next_memory: ordinary.next_memory,
        },
        lease: None,
        execution: Some(ExecutionCertificate::Plant(Box::new(certificate.clone()))),
    };
    Ok((certificate, plan, useful))
}

fn target_position(observation: &Observation, target: StrategicTarget) -> Option<Position> {
    let StrategicTarget::Opponent { seat } = target else {
        return None;
    };
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor) if actor.seat == seat && actor.alive => {
                Some(actor.position)
            }
            _ => None,
        })
}

impl From<PlantSupport> for PlantSupportDiagnostic {
    fn from(value: PlantSupport) -> Self {
        match value {
            PlantSupport::Strict => Self::Strict,
            PlantSupport::PersistentBurningOil => Self::PersistentBurningOil,
            PlantSupport::ClearArenaCounterPressure => Self::ClearArenaCounterPressure,
        }
    }
}

fn teleporter_failure(observation: &Observation) -> TeleporterSupportFailure {
    let parts = observation.parts();
    if parts
        .pickups
        .iter()
        .any(|pickup| pickup.kind == crate::game::PickupKind::Teleporter)
    {
        TeleporterSupportFailure::VisiblePickup
    } else if !parts.flames.is_empty() {
        TeleporterSupportFailure::ActiveFlame
    } else if !matches!(parts.bombs, crate::cpu::BombKnowledge::Visible(_)) {
        TeleporterSupportFailure::HiddenBombs
    } else if matches!(&parts.bombs, crate::cpu::BombKnowledge::Visible(bombs) if !bombs.is_empty())
    {
        TeleporterSupportFailure::VisibleBombs
    } else if parts
        .oil
        .iter()
        .any(|oil| oil.state == crate::game::OilBarrelState::Burning)
    {
        TeleporterSupportFailure::BurningOil
    } else {
        TeleporterSupportFailure::Other
    }
}
