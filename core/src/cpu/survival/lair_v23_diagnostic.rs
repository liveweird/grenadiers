use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{GoalModes, PolicyError, UtilityMemory};
use crate::cpu::routing::{PlannedBomb, PulseRequest, RouteFailure, RouteMode, certify_lair_pulse};
use crate::cpu::{CpuRng, MobContactLease, Observation, WorkBudget};
use crate::game::{CellId, PunchActivationPolicy, Tick};

use super::super::plant_tactic::{
    PlantCandidateDiagnostic, PlantCandidateOutcome, PlantSupport, diagnose_candidates,
    diagnose_demolition_candidate,
};
use super::super::{MAX_PLAN_AGE_TICKS, compatibility::actor};

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardPlantDiagnostic {
    pub initial_allowance: u32,
    pub total_spent: u32,
    pub opening_spent_before: u32,
    pub opening_remaining_before: u32,
    pub tactic_allowance: u32,
    pub tactic_spent: u32,
    pub prefilter_allowance: u32,
    pub prefilter_spent: u32,
    pub candidates: Vec<PlantCandidateDiagnostic>,
    pub demolition: Option<DemolitionCounterfactual>,
    pub outcome: HardPlantOutcome,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemolitionCounterfactual {
    pub block: CellId,
    pub visible_yield: u8,
    pub movement: crate::cpu::MovementIntent,
    pub original_slice: u32,
    pub child_spent: u32,
    pub outcome: PlantCandidateOutcome,
    pub useful: Option<bool>,
    pub placement: Option<PlannedBomb>,
    pub survive_until: Option<Tick>,
    pub contact: Result<(), RouteFailure>,
    pub total_spent: u32,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardPlantOutcome {
    NotAttempted,
    PrefilterRejected,
    CandidateRefused,
    MobContact(RouteFailure),
    Productive,
}

/// Replays the hard-v61 pre-opening pipeline without granting authority or
/// changing live planner state. Its allowance is the worker allowance after
/// any server-side continuation prepass.
#[doc(hidden)]
pub fn diagnose_hard_v61_plant(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    rng: &mut CpuRng,
    initial_allowance: u32,
    demolition_enabled: bool,
) -> Result<HardPlantDiagnostic, PolicyError> {
    let mut work = WorkBudget::new(initial_allowance);
    let parts = observation.parts();
    if parts.rules.version != 10
        || parts.rules.punch_activation_policy != PunchActivationPolicy::ContactV2
        || observation.respawn_lairs().is_none()
        || memory.key().world != parts.world
        || memory.key().cpu_seat != parts.recipient
        || history.key() != memory.key()
        || rng.key() != memory.key()
        || settled_stop_tick > parts.tick
        || actor(observation).is_none_or(|actor| !actor.alive || actor.voluntary_actions_frozen)
    {
        return Ok(not_attempted(initial_allowance, &work));
    }
    let ordinary = super::super::ordinary_planner::plan(
        observation,
        memory,
        rng,
        &mut work,
        true,
        true,
        true,
        true,
        GoalModes::visible_yield(),
        true,
        true,
    )?;
    let hazards = match forecast(
        observation,
        history,
        None,
        ForecastLimits::default(),
        &mut work,
    ) {
        Ok(hazards) => hazards,
        Err(_) => return Ok(not_attempted(initial_allowance, &work)),
    };
    let Some(end) = parts.tick.0.checked_add(MAX_PLAN_AGE_TICKS + 2) else {
        return Ok(not_attempted(initial_allowance, &work));
    };
    let Some(first) = parts.tick.0.checked_add(1) else {
        return Ok(not_attempted(initial_allowance, &work));
    };
    let start_window = TickWindow {
        start: Tick(first),
        end: Tick(end),
    };
    let ordinary_clear = super::super::deadline::ordinary_movement_is_clear(
        observation,
        ordinary.intent.movement != crate::cpu::MovementIntent::Stop,
        &mut work,
    );
    let recovering =
        super::super::strategic_recovery::attained_firing_position(observation, &ordinary);
    let support = PlantSupport::Strict;
    let _ = super::super::joint_refuge::eligible_failed_farm_approach(
        observation,
        settled_stop_tick,
        &ordinary,
        support,
        &mut work,
    );
    if super::super::joint_refuge::certified_farm_plant(
        observation,
        history,
        settled_stop_tick,
        memory,
        &ordinary,
        support,
        super::super::joint_refuge::FarmRefugeMode::BlockedAway,
        &mut work,
    )
    .is_some()
        || recovering
    {
        return Ok(not_attempted(initial_allowance, &work));
    }
    let opening_spent_before = work.spent();
    let opening_remaining_before = work.remaining();
    let hold_ticks = ordinary.hold_ticks.clamp(1, 12);
    let stopped_safe_until = Tick(end + u64::from(hold_ticks) + 9);
    let certified = certify_lair_pulse(
        observation,
        &hazards,
        PulseRequest {
            movement: ordinary.intent.movement,
            hold_ticks,
            start_window,
            stopped_safe_until,
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick,
        },
        &mut work,
    );
    let fallback_available = if let Ok((lease, _)) = certified
        && crate::cpu::routing::pit_clear(observation, &lease, &mut work).unwrap_or(false)
    {
        true
    } else {
        super::super::pickup_handoff::lair(
            observation,
            &hazards,
            ordinary,
            ordinary_clear,
            start_window,
            end,
            settled_stop_tick,
            true,
            true,
            &mut work,
        )
        .is_some()
    };
    if !fallback_available {
        return Ok(not_attempted(initial_allowance, &work));
    }
    let tactic_allowance = work.remaining().saturating_mul(7) / 8;
    let work_before_tactic = work;
    let mut tactic = WorkBudget::new(tactic_allowance);
    let probe = diagnose_candidates(
        observation,
        history,
        settled_stop_tick,
        memory,
        &ordinary,
        true,
        support,
        &mut tactic,
    );
    let _ = work.try_take(tactic.spent());
    let demolition = demolition_enabled
        .then(|| {
            demolition_counterfactual(
                observation,
                history,
                settled_stop_tick,
                work_before_tactic,
                tactic_allowance,
                &probe.candidates,
            )
        })
        .flatten();
    let outcome = if probe.prefilter_useful != Some(true) {
        HardPlantOutcome::PrefilterRejected
    } else if let Some(certificate) = probe.certificate {
        match MobContactLease::certify_route(observation, certificate.route(), &mut work) {
            Ok(_) => HardPlantOutcome::Productive,
            Err(error) => HardPlantOutcome::MobContact(error),
        }
    } else {
        HardPlantOutcome::CandidateRefused
    };
    Ok(HardPlantDiagnostic {
        initial_allowance,
        total_spent: work.spent(),
        opening_spent_before,
        opening_remaining_before,
        tactic_allowance,
        tactic_spent: tactic.spent(),
        prefilter_allowance: probe.prefilter_allowance,
        prefilter_spent: probe.prefilter_spent,
        candidates: probe.candidates,
        demolition,
        outcome,
    })
}

fn not_attempted(initial_allowance: u32, work: &WorkBudget) -> HardPlantDiagnostic {
    HardPlantDiagnostic {
        initial_allowance,
        total_spent: work.spent(),
        opening_spent_before: work.spent(),
        opening_remaining_before: work.remaining(),
        tactic_allowance: 0,
        tactic_spent: 0,
        prefilter_allowance: 0,
        prefilter_spent: 0,
        candidates: Vec::new(),
        demolition: None,
        outcome: HardPlantOutcome::NotAttempted,
    }
}

fn demolition_counterfactual(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    mut parent: WorkBudget,
    tactic_allowance: u32,
    candidates: &[PlantCandidateDiagnostic],
) -> Option<DemolitionCounterfactual> {
    let first = candidates.first()?;
    if first.outcome != PlantCandidateOutcome::UsefulTargetRejected {
        return None;
    }
    let block = first.visible_soft_block?;
    let visible_yield = first.visible_soft_block_yield?;
    let mut tactic = WorkBudget::new(tactic_allowance);
    let mut prefilter = WorkBudget::new(tactic.remaining() / 8);
    if !super::super::plant_value::could_be_useful(observation, &mut prefilter)
        || tactic.try_take(prefilter.spent()).is_err()
    {
        return None;
    }
    let total = tactic.remaining();
    let reserve = total / 10;
    let original_slice = total.saturating_sub(reserve.saturating_mul(4));
    debug_assert_eq!(original_slice, first.allowance);
    let probe = diagnose_demolition_candidate(
        observation,
        history,
        settled_stop_tick,
        first.movement,
        block,
        PlantSupport::Strict,
        original_slice,
    );
    tactic.try_take(probe.spent).ok()?;
    parent.try_take(tactic.spent()).ok()?;
    let (placement, survive_until, contact) = if let Some(certificate) = probe.certificate {
        let placement = Some(*certificate.placement());
        let survive_until = Some(certificate.route().survive_until());
        let contact = MobContactLease::certify_route(observation, certificate.route(), &mut parent)
            .map(|_| ());
        (placement, survive_until, contact)
    } else {
        (None, None, Err(RouteFailure::NoRoute))
    };
    Some(DemolitionCounterfactual {
        block,
        visible_yield,
        movement: first.movement,
        original_slice,
        child_spent: probe.spent,
        outcome: probe.outcome,
        useful: probe.useful,
        placement,
        survive_until,
        contact,
        total_spent: parent.spent(),
    })
}
