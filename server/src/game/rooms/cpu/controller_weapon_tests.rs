use grenadiers_core::{
    cpu::{
        BombKnowledge, Intent, ManualIntent, MemoryKey, MovementIntent, Observation, ObservedActor,
        ObservedBomb, ObservedBombTrigger, UnknownBombFacts, WorkBudget,
        forecast::ObservedHazardMemory,
        policy::{ControlPlan, UtilityMemory},
        routing::{PlantPresence, PlantRequest, RouteLimits, plan_timed_weapon_plant},
        survival::{ExecutionCertificate, RouteStatus, SurvivalPlan, WORK_ALLOWANCE},
    },
    game::{EnabledItems, EntityId, SeatId, SpecialWeapon, Tick},
};

use super::CpuController;
use crate::game::rooms::cpu::worker::CpuPlanner;

#[test]
fn timed_weapon_transition_is_checked_at_admission_and_plant_confirmation() {
    for weapon in [SpecialWeapon::Piercing, SpecialWeapon::Power] {
        let (basis, plan, fixture) = fixture(weapon);
        let mut admission = prepared(&basis);
        admission.inject_plan_result_for_test(basis.clone(), 4, plan.clone());
        let exact = advanced(&basis, Tick(basis.parts().tick.0 + 6));
        assert!(admission.accept_observation(&exact));
        let accepted = admission
            .poll(exact.parts().tick, 4)
            .expect("matching weapon");
        assert_eq!(accepted.intent.manual, Some(ManualIntent::Bomb));

        for case in [
            WeaponAfter::AbsentRetained,
            WeaponAfter::PresentConsumed,
            WeaponAfter::PresentRetained,
            WeaponAfter::PresentReplacement,
        ] {
            let mut controller = prepared(&basis);
            controller.inject_plan_result_for_test(basis.clone(), 4, plan.clone());
            let exact = advanced(&basis, Tick(basis.parts().tick.0 + 6));
            assert!(controller.accept_observation(&exact));
            let accepted = controller.poll(exact.parts().tick, 4).unwrap();
            controller.on_movement_admitted();
            controller.commit(
                accepted,
                Intent {
                    movement: fixture.movement,
                    manual: Some(ManualIntent::Bomb),
                },
            );
            let observation = placement_observation(&basis, &fixture, weapon, case);
            assert!(controller.accept_observation(&observation));
            let status = controller.observe_route(&observation);
            match case {
                WeaponAfter::AbsentRetained | WeaponAfter::PresentConsumed => {
                    assert_ne!(
                        status,
                        Some(RouteStatus::Invalidated),
                        "{weapon:?} {case:?}"
                    );
                    assert_eq!(
                        controller.plant_presence(),
                        Some(match case {
                            WeaponAfter::AbsentRetained => PlantPresence::Absent,
                            WeaponAfter::PresentConsumed => {
                                PlantPresence::Present(EntityId(91))
                            }
                            _ => unreachable!(),
                        })
                    );
                }
                WeaponAfter::PresentRetained | WeaponAfter::PresentReplacement => {
                    assert_eq!(
                        status,
                        Some(RouteStatus::Invalidated),
                        "{weapon:?} {case:?}"
                    );
                    assert!(!controller.has_committed_route());
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum WeaponAfter {
    AbsentRetained,
    PresentConsumed,
    PresentRetained,
    PresentReplacement,
}

#[derive(Clone, Copy)]
struct Fixture {
    placement: grenadiers_core::cpu::routing::PlannedBomb,
    movement: MovementIntent,
    position: grenadiers_core::game::Position,
    heading: grenadiers_core::game::Direction,
}

fn fixture(weapon: SpecialWeapon) -> (Observation, SurvivalPlan, Fixture) {
    let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
    let enabled = match weapon {
        SpecialWeapon::Piercing => EnabledItems::PIERCING_BOMB,
        SpecialWeapon::Power => EnabledItems::POWER_BOMB,
        _ => unreachable!(),
    };
    parts.rules.settings = parts.rules.settings.with_enabled_items(enabled);
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.weapon = Some(weapon);
    let basis = Observation::new(parts).unwrap();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let movement = MovementIntent::Direction(grenadiers_core::game::Direction::West);
    let certificate = plan_timed_weapon_plant(
        &basis,
        &history,
        PlantRequest {
            execution_tick: Tick(basis.parts().tick.0 + 7),
            settled_stop_tick: basis.parts().tick,
            first_movement: movement,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let placement = *certificate.placement();
    let (position, heading) = certificate
        .route()
        .lane(placement.placed_at)
        .and_then(|lane| lane.pose_at(placement.placed_at))
        .unwrap();
    let plan = SurvivalPlan {
        control: ControlPlan {
            intent: Intent {
                movement,
                manual: Some(ManualIntent::Bomb),
            },
            hold_ticks: 1,
            next_memory: UtilityMemory::new(key),
        },
        lease: None,
        execution: Some(ExecutionCertificate::Plant(Box::new(certificate))),
    };
    (
        basis,
        plan,
        Fixture {
            placement,
            movement,
            position,
            heading,
        },
    )
}

fn prepared(basis: &Observation) -> CpuController {
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
    controller
}

fn advanced(basis: &Observation, tick: Tick) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    Observation::new(parts).unwrap()
}

fn placement_observation(
    basis: &Observation,
    fixture: &Fixture,
    weapon: SpecialWeapon,
    case: WeaponAfter,
) -> Observation {
    let mut parts = advanced(basis, fixture.placement.placed_at).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.position = fixture.position;
    cpu.heading = fixture.heading;
    cpu.weapon = match case {
        WeaponAfter::AbsentRetained | WeaponAfter::PresentRetained => Some(weapon),
        WeaponAfter::PresentConsumed => None,
        WeaponAfter::PresentReplacement => Some(match weapon {
            SpecialWeapon::Piercing => SpecialWeapon::Power,
            SpecialWeapon::Power => SpecialWeapon::Piercing,
            _ => unreachable!(),
        }),
    };
    if matches!(
        case,
        WeaponAfter::PresentConsumed
            | WeaponAfter::PresentRetained
            | WeaponAfter::PresentReplacement
    ) {
        cpu.live_bomb_count = 1;
        parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(91),
            placed_at: fixture.placement.placed_at,
            cell: fixture.placement.cell,
            owner: fixture.placement.owner,
            range: fixture.placement.range,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: u16::try_from(
                    fixture.placement.detonates_at.0 - fixture.placement.placed_at.0,
                )
                .unwrap(),
                paused: false,
            },
            blast: fixture.placement.blast,
            unknown: UnknownBombFacts::ALL,
        }]);
    }
    Observation::new(parts).unwrap()
}
