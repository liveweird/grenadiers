use grenadiers_core::cpu::forecast::ObservedHazardMemory;
use grenadiers_core::cpu::policy::UtilityMemory;
use grenadiers_core::cpu::survival::{PulseStatus, SurvivalPlan, WORK_ALLOWANCE, plan};
use grenadiers_core::cpu::{CpuRng, MemoryKey, Observation, ObservedActor, WorkBudget};
use grenadiers_core::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, Position, SeatId, SpeedLevel, Tick,
};

use super::CpuController;
use crate::game::rooms::cpu::worker::CpuPlanner;

#[test]
fn controller_rejects_a_guarded_bracelet_pulse_after_the_rival_can_countershove() {
    let basis = bracelet_basis();
    let plan = bracelet_plan(&basis);
    assert!(plan.lease.is_some());
    assert_eq!(
        plan.control.intent.movement,
        grenadiers_core::cpu::MovementIntent::Direction(Direction::East)
    );
    let first_pose = plan.lease.as_ref().unwrap().pose_after(1).min();

    let current = later_with_rival_equipment(&basis, None);
    let mut accepted = controller(&basis, &current);
    accepted.inject_plan_result_for_test(basis.clone(), 4, plan.clone());
    let admitted = accepted.poll(Tick(21), 4).unwrap();
    let intent = admitted.intent;
    accepted.commit(admitted, intent);
    let countershove_during_execution = execution_observation(&basis, first_pose);
    assert_eq!(
        accepted.validate_pulse(&countershove_during_execution, Tick(22)),
        Some(PulseStatus::Invalidated)
    );

    let countershove = later_with_rival_equipment(&basis, Some(Equipment::Bracelet));
    let mut rejected = controller(&basis, &countershove);
    rejected.inject_plan_result_for_test(basis, 4, plan);
    assert!(rejected.poll(Tick(21), 4).is_none());
    assert_eq!(rejected.rng_draws(), 0);
}

fn bracelet_basis() -> Observation {
    let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
    parts.arena.blocks.clear();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::BRACELET);
    parts.flames.push(grenadiers_core::cpu::ObservedFlame {
        cell: CellId::new(9, 7),
        remaining_ticks: 40,
    });
    parts.actors[0] = ObservedActor::Visible(grenadiers_core::cpu::VisibleActor {
        id: EntityId(1),
        seat: SeatId::One,
        position: Position::center(CellId::new(8, 7)),
        heading: Direction::West,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: grenadiers_core::cpu::UnknownActorFacts::ALL,
    });
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.equipment = Some(Equipment::Bracelet);
    cpu.heading = Direction::East;
    Observation::new(parts).unwrap()
}

fn bracelet_plan(basis: &Observation) -> SurvivalPlan {
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(basis).unwrap();
    plan(
        basis,
        &UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        &mut CpuRng::from_seed(key, [5; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap()
}

fn later_with_rival_equipment(basis: &Observation, equipment: Option<Equipment>) -> Observation {
    let mut parts = basis.clone().into_parts();
    parts.tick = Tick(21);
    parts.rules.running_updates += 1;
    parts.rules.remaining_ticks -= 1;
    parts.flames[0].remaining_ticks -= 1;
    let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
        unreachable!()
    };
    rival.equipment = equipment;
    Observation::new(parts).unwrap()
}

fn execution_observation(basis: &Observation, cpu_position: Position) -> Observation {
    let mut parts = basis.clone().into_parts();
    parts.tick = Tick(22);
    parts.rules.running_updates += 2;
    parts.rules.remaining_ticks -= 2;
    parts.flames[0].remaining_ticks -= 2;
    let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
        unreachable!()
    };
    rival.equipment = Some(Equipment::Bracelet);
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.position = cpu_position;
    cpu.heading = Direction::East;
    Observation::new(parts).unwrap()
}

fn controller(basis: &Observation, current: &Observation) -> CpuController {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        grenadiers_core::game::SeatId::Two,
        7,
        [5; 32],
        grenadiers_core::room::CpuProfile::CURRENT,
    );
    controller.reset_world(3);
    assert!(controller.accept_observation(basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    assert!(controller.accept_observation(current));
    controller
}
