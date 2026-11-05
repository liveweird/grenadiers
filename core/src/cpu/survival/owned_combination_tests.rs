use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::{BombKnowledge, ObservedActor, WorkBudget};
use crate::game::{EntityId, Tick};

use super::owned_combination_fixture::{CombinationBasis, basis, replace_observation};
use super::{TerminalRefugeHandoff, WORK_ALLOWANCE, plan_hard_combination_v67_with_handoff};

fn plan(value: &CombinationBasis, allowance: u32) -> Option<super::LairSurvivalPlan> {
    let mut work = WorkBudget::new(allowance);
    plan_hard_combination_v67_with_handoff(
        &value.observation,
        &value.memory,
        &value.history,
        value.settled,
        &value.handoff,
        &mut work,
    )
    .unwrap()
}

#[test]
fn old_settled_stop_can_authorize_one_sealed_useful_combination() {
    let value = basis();
    assert!(
        super::reactive_assumptions::ReactivePlantAssumptions::certify_owned_combination(
            &value.observation,
            &value.history,
            value.settled,
            &value.handoff,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_some()
    );
    let planned = plan(&value, WORK_ALLOWANCE).expect("curated combination");
    let execution = planned.survival.execution.as_ref().unwrap();
    assert!(execution.requires_owned_combination_support());
    assert!(execution.requires_reactive_support());
    assert!(value.handoff.accepts_replacement(execution));
    assert_eq!(
        planned.survival.control.intent.movement,
        crate::cpu::MovementIntent::Stop
    );
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(execution.reactive_block_contribution().is_some());
    assert!(
        execution.owned_combination_handoff_matches(
            &value.handoff,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
    );
    let admission_tick = Tick(execution.manual_execution_tick().unwrap().0 - 1);
    let elapsed = admission_tick.0 - value.observation.parts().tick.0;
    let current = value
        .observation
        .clone()
        .map_parts(|parts| {
            parts.tick = admission_tick;
            parts.rules.running_updates += u16::try_from(elapsed).unwrap();
            parts.rules.remaining_ticks -= u16::try_from(elapsed).unwrap();
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            let crate::cpu::ObservedBombTrigger::Timed {
                remaining_ticks, ..
            } = &mut bombs[0].trigger
            else {
                unreachable!()
            };
            *remaining_ticks -= u16::try_from(elapsed).unwrap();
        })
        .unwrap();
    let admitted = super::CommittedRoute::admit_owned_combination_execution(
        execution.clone(),
        value.observation.clone(),
        &current,
        admission_tick,
        crate::cpu::MovementIntent::Stop,
        &value.handoff,
    )
    .expect("exact handoff admits the combination certificate");
    assert!(admitted.requires_owned_combination_support());

    let (route_tick, first_update) = value.handoff.route_identity();
    let longer_current = TerminalRefugeHandoff::new(
        value.handoff.key(),
        (route_tick, first_update),
        value.handoff.bomb_id(),
        value.handoff.refuge(),
        value.observation.clone(),
        Tick(value.handoff.old_survive_until().0 + 1),
        value.handoff.settled_stop_tick(),
    );
    assert!(
        !execution.owned_combination_handoff_matches(
            &longer_current,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
    );
    let wrong_horizon = super::CommittedRoute::admit_owned_combination_execution(
        execution.clone(),
        value.observation.clone(),
        &value.observation,
        value.observation.parts().tick,
        crate::cpu::MovementIntent::Stop,
        &longer_current,
    );
    assert!(wrong_horizon.is_err());

    let generic = super::CommittedRoute::admit_reactive_execution(
        execution.clone(),
        value.observation.clone(),
        &value.observation,
        value.observation.parts().tick,
        crate::cpu::MovementIntent::Stop,
    );
    assert!(generic.is_err());
}

#[test]
fn sealed_stop_tick_and_checkpoint_scan_are_exact_and_charged() {
    let value = basis();
    assert!(
        plan_hard_combination_v67_with_handoff(
            &value.observation,
            &value.memory,
            &value.history,
            Tick(value.settled.0 - 1),
            &value.handoff,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .unwrap()
        .is_none()
    );

    let mut work = WorkBudget::new(1);
    let certified =
        super::reactive_assumptions::ReactivePlantAssumptions::certify_owned_combination(
            &value.observation,
            &value.history,
            value.settled,
            &value.handoff,
            &mut work,
        );
    assert!(certified.is_none());
    assert_eq!(work.spent(), 1);
}

#[test]
fn first_bomb_contribution_cannot_be_reused_for_farming() {
    let mut value = basis();
    let planned = plan(&value, WORK_ALLOWANCE).expect("curated combination");
    let execution = planned.survival.execution.as_ref().unwrap();
    let plant = execution.plant().unwrap();
    let redundant = value
        .observation
        .parts()
        .arena
        .blocks
        .iter()
        .copied()
        .find(|block| {
            execution
                .route()
                .evolution()
                .removes_block_by(*block, Tick(plant.placement().detonates_at.0 - 1))
        })
        .expect("first bomb removes a visible block before the new blast");
    let mut intent = value.memory.strategic_intent().unwrap();
    intent.objective = StrategicObjectiveKind::Farm;
    intent.target = StrategicTarget::Block { cell: redundant };
    value.memory = UtilityMemory::new(value.memory.key()).with_strategic_intent(intent);
    assert!(plan(&value, WORK_ALLOWANCE).is_none());
}

#[test]
fn same_tick_first_blast_still_makes_the_second_farm_contribution_redundant() {
    let baseline = basis();
    let planned = plan(&baseline, WORK_ALLOWANCE).expect("curated combination");
    let detonates_at = planned
        .survival
        .execution
        .as_ref()
        .unwrap()
        .plant()
        .unwrap()
        .placement()
        .detonates_at;
    let remaining = u16::try_from(detonates_at.0 - baseline.observation.parts().tick.0).unwrap();
    let observation = baseline
        .observation
        .clone()
        .map_parts(|parts| {
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            let crate::cpu::ObservedBombTrigger::Timed {
                remaining_ticks, ..
            } = &mut bombs[0].trigger
            else {
                unreachable!()
            };
            *remaining_ticks = remaining;
        })
        .unwrap();
    let mut changed = replace_observation(observation);
    let mut intent = changed.memory.strategic_intent().unwrap();
    intent.target = StrategicTarget::Block {
        cell: crate::game::CellId::new(5, 1),
    };
    changed.memory = UtilityMemory::new(changed.memory.key()).with_strategic_intent(intent);
    assert!(
        super::reactive_assumptions::ReactivePlantAssumptions::certify_owned_combination(
            &changed.observation,
            &changed.history,
            changed.settled,
            &changed.handoff,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_some()
    );
    assert!(plan(&changed, WORK_ALLOWANCE).is_none());
}

#[test]
fn fuse_capacity_context_horizon_and_budget_fail_closed() {
    let value = basis();
    let rebuild = replace_observation;
    for remaining in [9, 10] {
        let observation = value
            .observation
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                let crate::cpu::ObservedBombTrigger::Timed {
                    remaining_ticks, ..
                } = &mut bombs[0].trigger
                else {
                    unreachable!()
                };
                *remaining_ticks = remaining;
            })
            .unwrap();
        let changed = rebuild(observation);
        let certified =
            super::reactive_assumptions::ReactivePlantAssumptions::certify_owned_combination(
                &changed.observation,
                &changed.history,
                changed.settled,
                &changed.handoff,
                &mut WorkBudget::new(WORK_ALLOWANCE),
            );
        assert_eq!(certified.is_some(), remaining == 10);
    }
    let no_capacity = rebuild(
        value
            .observation
            .clone()
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[parts.recipient.index()]
                else {
                    unreachable!()
                };
                actor.bomb_capacity = 1;
            })
            .unwrap(),
    );
    assert!(plan(&no_capacity, WORK_ALLOWANCE).is_none());

    let paused = rebuild(
        value
            .observation
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                let crate::cpu::ObservedBombTrigger::Timed { paused, .. } = &mut bombs[0].trigger
                else {
                    unreachable!()
                };
                *paused = true;
            })
            .unwrap(),
    );
    assert!(plan(&paused, WORK_ALLOWANCE).is_none());

    let second_bomb = rebuild(
        value
            .observation
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                let mut other = bombs[0];
                other.id = EntityId(other.id.0 + 100);
                other.owner = EntityId(1);
                other.cell = crate::game::CellId::new(8, 2);
                bombs.push(other);
                let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                    unreachable!()
                };
                actor.live_bomb_count = 1;
            })
            .unwrap(),
    );
    assert!(plan(&second_bomb, WORK_ALLOWANCE).is_none());
    assert!(plan(&value, 0).is_none());

    let wrong = value
        .observation
        .clone()
        .map_parts(|parts| parts.world.physical_round += 1)
        .unwrap();
    assert!(
        plan_hard_combination_v67_with_handoff(
            &wrong,
            &value.memory,
            &value.history,
            value.settled,
            &value.handoff,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_err()
    );

    let mut short = basis();
    let (route_tick, first_update) = short.handoff.route_identity();
    short.handoff = TerminalRefugeHandoff::new(
        short.handoff.key(),
        (route_tick, first_update),
        short.handoff.bomb_id(),
        short.handoff.refuge(),
        short.observation.clone(),
        Tick(u64::MAX),
        short.handoff.settled_stop_tick(),
    );
    assert!(plan(&short, WORK_ALLOWANCE).is_none());
}
