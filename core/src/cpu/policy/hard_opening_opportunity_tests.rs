use crate::cpu::{
    CpuRng, Observation, ObservedActor, ObservedEnvironment, ObservedSpikePits, WorkBudget,
};
use crate::game::{
    CellId, EntityId, Position, SeatId, SpikePitPhase, SpikePitView, StormCloudConfig, Terrain,
    Tick,
};
use crate::room::{SpikePitConfig, SpikePitSettings};

use super::tests::{key, observation};
use super::{
    GoalModes, STRATEGIC_COMMITMENT_TICKS, StrategicObjectiveKind, StrategicSwitchReason,
    UtilityMemory,
};

const CPU: CellId = CellId::new(1, 11);
const RIVAL: CellId = CellId::new(13, 1);
const BLOCK: CellId = CellId::new(1, 8);

fn scene_with_rival(tick: Tick, pits: &[CellId], rival_visible: bool) -> Observation {
    let mut parts = observation(CPU).into_parts();
    parts.tick = tick;
    parts.rules.version = 10;
    parts.rules.running_updates = u16::try_from(tick.0).unwrap();
    parts.arena.blocks = vec![BLOCK];
    if !pits.is_empty() {
        for cell in [CellId::new(1, 7), CellId::new(2, 8)] {
            let index =
                usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
            parts.arena.terrain[index] = Terrain::Wall;
        }
    }
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.bomb_range = 1;
    if rival_visible {
        let mut rival = cpu.clone();
        rival.id = EntityId(1);
        rival.seat = SeatId::One;
        rival.position = Position::center(RIVAL);
        parts.actors[0] = ObservedActor::Visible(rival);
    }
    let environment = ObservedEnvironment::new(
        StormCloudConfig::Disabled,
        None,
        parts
            .actors
            .iter()
            .map(|actor| matches!(actor, ObservedActor::Visible(_)).then_some(0))
            .collect(),
    );
    let pits = (!pits.is_empty()).then(|| {
        let mut views = pits
            .iter()
            .copied()
            .map(|cell| SpikePitView {
                cell,
                phase: SpikePitPhase::Idle,
                remaining_ticks: 0,
            })
            .collect::<Vec<_>>();
        views.sort_unstable_by_key(|pit| pit.cell);
        ObservedSpikePits::new(SpikePitConfig::Enabled(SpikePitSettings::DEFAULT), views)
    });
    Observation::new_rules_v10(parts, environment, None, pits).unwrap()
}

fn scene(tick: Tick, pits: &[CellId]) -> Observation {
    scene_with_rival(tick, pits, true)
}

fn hidden_rival_scene(tick: Tick) -> Observation {
    scene_with_rival(tick, &[], false)
}

fn select(
    observation: &Observation,
    memory: UtilityMemory,
    modes: GoalModes,
) -> super::goals::GuidanceV24 {
    select_measured(observation, memory, modes).0
}

fn select_measured(
    observation: &Observation,
    memory: UtilityMemory,
    modes: GoalModes,
) -> (super::goals::GuidanceV24, u32) {
    let ObservedActor::Visible(actor) = &observation.parts().actors[1] else {
        unreachable!()
    };
    let mut work = WorkBudget::new(65_536);
    let guidance = super::pit_planner::select_guidance_v59(
        observation,
        actor,
        actor.position.cell().unwrap(),
        &memory,
        &mut work,
        true,
        modes,
    )
    .unwrap();
    (guidance, work.spent())
}

#[test]
fn fresh_far_attack_promotes_near_farm_and_commitment_retains_it() {
    let observed = scene(Tick(10), &[]);
    let archived = select(
        &observed,
        UtilityMemory::new(key()),
        GoalModes::visible_yield_actor_progress(),
    );
    assert_eq!(archived.intent.objective, StrategicObjectiveKind::Attack);

    let promoted = select(
        &observed,
        UtilityMemory::new(key()),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(promoted.intent.objective, StrategicObjectiveKind::Farm);
    assert_eq!(promoted.intent.last_distance, 2);
    assert!(archived.intent.last_distance >= 16);
    assert_eq!(
        promoted.intent.committed_until,
        Tick(observed.parts().tick.0 + STRATEGIC_COMMITMENT_TICKS)
    );

    let retained = select(
        &scene(Tick(11), &[]),
        UtilityMemory::new(key()).with_strategic_intent(promoted.intent),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(retained.intent.target, promoted.intent.target);
    assert_eq!(
        retained.intent.committed_until,
        promoted.intent.committed_until
    );
    assert_eq!(
        retained.intent.last_switch_reason,
        promoted.intent.last_switch_reason
    );
}

#[test]
fn expired_commitment_uses_legacy_better_opportunity_without_repromoting_farm() {
    let first = select(
        &scene(Tick(10), &[]),
        UtilityMemory::new(key()),
        GoalModes::visible_yield_farm_opportunity(),
    );
    let switched = select(
        &scene(first.intent.committed_until, &[]),
        UtilityMemory::new(key()).with_strategic_intent(first.intent),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(switched.intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(
        switched.intent.last_switch_reason,
        StrategicSwitchReason::BetterOpportunity
    );
    assert_ne!(switched.intent.target, first.intent.target);
}

#[test]
fn stalled_alternate_excludes_previous_farm_at_the_selector_call_site() {
    let farm = select(
        &hidden_rival_scene(Tick(10)),
        UtilityMemory::new(key()),
        GoalModes::visible_yield_actor_progress(),
    );
    assert_eq!(farm.intent.objective, StrategicObjectiveKind::Farm);

    let stalled = select(
        &scene(farm.intent.progress_deadline, &[]),
        UtilityMemory::new(key()).with_strategic_intent(farm.intent),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(stalled.intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(
        stalled.intent.last_switch_reason,
        StrategicSwitchReason::Stalled
    );
    assert_eq!(
        stalled.failed_target.map(|(target, _)| target),
        Some(farm.intent.target)
    );
}

#[test]
fn stalled_without_an_alternate_preserves_v69_fallback_and_work() {
    let first = select(
        &hidden_rival_scene(Tick(10)),
        UtilityMemory::new(key()),
        GoalModes::visible_yield_actor_progress(),
    );
    assert_eq!(first.intent.objective, StrategicObjectiveKind::Farm);
    let observed = hidden_rival_scene(first.intent.progress_deadline);
    let memory = UtilityMemory::new(key()).with_strategic_intent(first.intent);
    let old = select_measured(&observed, memory, GoalModes::visible_yield_actor_progress());
    let new = select_measured(
        &observed,
        memory,
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(new, old);
    assert_eq!(new.0.intent.target, first.intent.target);
    assert_eq!(
        new.0.intent.last_switch_reason,
        StrategicSwitchReason::Stalled
    );
}

#[test]
fn failed_or_pit_only_farm_is_filtered_before_the_opening_comparison() {
    let first = select(
        &scene(Tick(10), &[]),
        UtilityMemory::new(key()),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(first.intent.objective, StrategicObjectiveKind::Farm);
    let failed = select(
        &scene(Tick(11), &[]),
        UtilityMemory::new(key()).with_failed_target(Some((first.intent.target, Tick(100)))),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(failed.intent.objective, StrategicObjectiveKind::Attack);

    let blocked_approaches = [
        CellId::new(1, 3),
        CellId::new(1, 9),
        CellId::new(13, 3),
        CellId::new(13, 9),
    ];
    let pit_filtered = select(
        &scene(Tick(10), &blocked_approaches),
        UtilityMemory::new(key()),
        GoalModes::visible_yield_farm_opportunity(),
    );
    assert_eq!(
        pit_filtered.intent.objective,
        StrategicObjectiveKind::Attack
    );
}

#[test]
fn hard70_planner_version_inherits_hard69_except_for_opening_mode() {
    super::planner::version::assert_hard70_inherits_hard69();
}

#[test]
fn hard70_ordinary_dispatch_reaches_the_opt_in_selector() {
    let observed = scene(Tick(10), &[]);
    let memory = UtilityMemory::new(key());
    let mut old_rng = CpuRng::from_seed(key(), [70; 32]);
    let mut new_rng = old_rng.clone();
    let old = super::plan_hard_progress_v69(
        &observed,
        &memory,
        &mut old_rng,
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let new = super::plan_hard_opportunity_v70(
        &observed,
        &memory,
        &mut new_rng,
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        old.next_memory.strategic_intent().unwrap().objective,
        StrategicObjectiveKind::Attack
    );
    assert_eq!(
        new.next_memory.strategic_intent().unwrap().objective,
        StrategicObjectiveKind::Farm
    );
    assert_eq!(new_rng.draws(), old_rng.draws());
}
