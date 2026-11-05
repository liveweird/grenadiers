use crate::cpu::{CpuRng, Observation, ObservedActor, ObservedEnvironment, WorkBudget};
use crate::game::{CellId, EntityId, Position, SeatId, StormCloudConfig, Tick};

use super::tests::{key, observation};
use super::{GoalModes, StrategicObjectiveKind, UtilityMemory};

fn scene(cpu: CellId, rival: CellId, tick: Tick) -> Observation {
    let mut parts = observation(cpu).into_parts();
    parts.tick = tick;
    parts.rules.version = 10;
    parts.rules.running_updates = u16::try_from(tick.0).unwrap();
    parts.arena.blocks = vec![CellId::new(3, 3)];
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.bomb_range = 1;
    let mut opponent = actor.clone();
    opponent.id = EntityId(1);
    opponent.seat = SeatId::One;
    opponent.position = Position::center(rival);
    parts.actors[0] = ObservedActor::Visible(opponent);
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
        None,
    )
    .unwrap()
}

fn select(
    observation: &Observation,
    memory: UtilityMemory,
    modes: GoalModes,
) -> super::goals::GuidanceV24 {
    let ObservedActor::Visible(actor) = &observation.parts().actors[1] else {
        unreachable!()
    };
    super::pit_planner::select_guidance_v59(
        observation,
        actor,
        actor.position.cell().unwrap(),
        &memory,
        &mut WorkBudget::new(65_536),
        true,
        modes,
    )
    .unwrap()
}

#[test]
fn hard69_requires_actor_progress_while_hard67_remains_archived() {
    let initial = scene(CellId::new(5, 7), CellId::new(11, 7), Tick(10));
    let first = select(
        &initial,
        UtilityMemory::new(key()),
        GoalModes::visible_yield_actor_progress(),
    );
    assert_eq!(first.intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(first.intent.approach_cell, Some(CellId::new(10, 7)));
    assert_eq!(first.intent.last_distance, 5);
    let memory = UtilityMemory::new(key()).with_strategic_intent(first.intent);

    let unchanged = select(
        &scene(CellId::new(5, 7), CellId::new(11, 7), Tick(11)),
        memory,
        GoalModes::visible_yield_actor_progress(),
    );
    assert_eq!(unchanged.intent.last_distance, 5);
    assert_eq!(
        unchanged.intent.progress_deadline,
        first.intent.progress_deadline
    );
    assert_eq!(unchanged.failed_target, None);

    let archived = select(
        &scene(
            CellId::new(5, 7),
            CellId::new(9, 7),
            first.intent.progress_deadline,
        ),
        memory,
        GoalModes::visible_yield_contention(),
    );
    assert_eq!(archived.intent.target, first.intent.target);
    assert_eq!(archived.intent.approach_cell, Some(CellId::new(8, 7)));
    assert_eq!(archived.intent.last_distance, 3);
    assert_eq!(
        archived.intent.last_cpu_position,
        first.intent.last_cpu_position
    );
    assert!(archived.intent.progress_deadline > first.intent.progress_deadline);
    assert_eq!(archived.failed_target, None);

    let corrected = select(
        &scene(
            CellId::new(5, 7),
            CellId::new(9, 7),
            first.intent.progress_deadline,
        ),
        memory,
        GoalModes::visible_yield_actor_progress(),
    );
    assert_ne!(corrected.intent.target, first.intent.target);
    assert_eq!(
        corrected.failed_target.map(|(target, _)| target),
        Some(first.intent.target)
    );

    let moving = select(
        &scene(
            CellId::new(6, 7),
            CellId::new(11, 7),
            first.intent.progress_deadline,
        ),
        memory,
        GoalModes::visible_yield_actor_progress(),
    );
    assert_eq!(moving.intent.approach_cell, Some(CellId::new(10, 7)));
    assert_eq!(moving.intent.last_distance, 4);
    assert_ne!(
        moving.intent.last_cpu_position,
        first.intent.last_cpu_position
    );
    assert!(moving.intent.progress_deadline > first.intent.progress_deadline);
    assert_eq!(moving.failed_target, None);

    let stalled = select(
        &scene(
            CellId::new(5, 7),
            CellId::new(11, 7),
            first.intent.progress_deadline,
        ),
        memory,
        GoalModes::visible_yield_actor_progress(),
    );
    assert_ne!(stalled.intent.target, first.intent.target);
    assert_eq!(
        stalled.failed_target.map(|(target, _)| target),
        Some(first.intent.target)
    );
}

#[test]
fn hard69_planner_version_inherits_every_hard66_switch_except_progress() {
    super::planner::version::assert_hard69_inherits_hard66();
}

#[test]
fn hard69_fresh_ordinary_plan_preserves_hard66_output_work_and_rng() {
    let observation = scene(CellId::new(5, 7), CellId::new(11, 7), Tick(10));
    let memory = UtilityMemory::new(key());
    let mut old_rng = CpuRng::from_seed(key(), [69; 32]);
    let mut new_rng = old_rng.clone();
    let mut old_work = WorkBudget::new(65_536);
    let mut new_work = WorkBudget::new(65_536);
    let old =
        super::plan_hard_pickup_contention_v66(&observation, &memory, &mut old_rng, &mut old_work);
    let new = super::plan_hard_progress_v69(&observation, &memory, &mut new_rng, &mut new_work);
    assert_eq!(new, old);
    assert_eq!(new_work.spent(), old_work.spent());
    assert_eq!(new_rng.draws(), old_rng.draws());
}

#[test]
fn hard69_subcell_progress_renews_without_a_cell_distance_change() {
    let initial = scene(CellId::new(5, 7), CellId::new(11, 7), Tick(10));
    let modes = GoalModes::visible_yield_actor_progress();
    let first = select(&initial, UtilityMemory::new(key()), modes);
    let memory = UtilityMemory::new(key()).with_strategic_intent(first.intent);
    let mut parts = scene(
        CellId::new(5, 7),
        CellId::new(11, 7),
        first.intent.progress_deadline,
    )
    .into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position.x += 1;
    assert_eq!(actor.position.cell(), first.intent.last_cpu_position.cell());
    let observation = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
        None,
    )
    .unwrap();
    let progress = select(&observation, memory, modes);
    assert_eq!(progress.intent.target, first.intent.target);
    assert_eq!(progress.intent.last_distance, first.intent.last_distance);
    assert!(progress.intent.progress_deadline > first.intent.progress_deadline);
    assert_eq!(progress.failed_target, None);
}
