use grenadiers_core::cpu::forecast::ObservedHazardMemory;
use grenadiers_core::cpu::policy::UtilityMemory;
use grenadiers_core::cpu::routing::PlantPresence;
use grenadiers_core::cpu::survival::{ExecutionCertificate, WORK_ALLOWANCE};
use grenadiers_core::cpu::{BombKnowledge, CpuRng, MemoryKey, WorkBudget};
use grenadiers_core::game::{
    ActorInit, Arena, CellId, Direction, EntityId, Position, RulesConfig, SeatId, SpeedLevel,
    State, Terrain, Tick, World,
};
use grenadiers_core::room::{CpuProfile, RoomSettings};
use std::collections::BTreeSet;
use tokio::time::Instant;

use super::tests::{
    open_fixture_state, publish_next_cpu_observation, running_driver, running_driver_for_profile,
};
use crate::game::rooms::match_runtime::MatchAction;

#[test]
fn real_driver_binds_a_certified_ordinary_plant_without_workers() {
    for profile in [CpuProfile::CURRENT, CpuProfile::CURRENT_EASY] {
        real_driver_plant(profile);
    }
}

fn real_driver_plant(profile: CpuProfile) {
    let mut driver = running_driver_for_profile(23, profile);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut rng = CpuRng::from_seed(key, [8; 32]);
    let plan = grenadiers_core::cpu::survival::plan(
        &basis,
        &UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        &mut rng,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let certificate = match &plan.execution {
        Some(ExecutionCertificate::Plant(certificate)) => certificate.as_ref().clone(),
        _ => panic!("expected certified ordinary plant"),
    };
    let placement = *certificate.placement();
    let admission = Tick(basis.parts().tick.0 + 6);
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis.clone(),
        driver.runtime.lifecycle().round_attempt(),
        plan,
    );

    while driver.runtime.lifecycle().tick() < admission {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            driver.publish(notice);
        }
        driver.poll_cpu_result().unwrap();
    }
    assert!(driver.cpu.as_ref().unwrap().has_committed_route());
    assert!(!driver.cpu.as_ref().unwrap().is_escaping());

    let mut observed_id = None;
    for _ in 0..4 {
        driver.tick(Instant::now()).unwrap();
        if let Some(observation) = driver.last.cpu_observation.as_ref() {
            if observation.parts().tick >= placement.placed_at {
                let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
                    continue;
                };
                observed_id = bombs
                    .iter()
                    .find(|bomb| bomb.placed_at == placement.placed_at)
                    .map(|bomb| bomb.id);
            }
            if let Some(id) = observed_id {
                assert_eq!(
                    certificate.confirm(&basis, observation),
                    Ok(PlantPresence::Present(id))
                );
                let expected_count = certificate.expected_live_bombs(
                    placement.owner,
                    observation.parts().tick,
                    PlantPresence::Present(id),
                );
                let actual_count =
                    observation
                        .parts()
                        .actors
                        .iter()
                        .find_map(|actor| match actor {
                            grenadiers_core::cpu::ObservedActor::Visible(actor)
                                if actor.id == placement.owner =>
                            {
                                Some(actor.live_bomb_count)
                            }
                            _ => None,
                        });
                assert_eq!(actual_count, expected_count);
                let expected_pose = certificate
                    .route()
                    .lane(placement.placed_at)
                    .and_then(|lane| lane.pose_at(observation.parts().tick));
                let actual_pose = observation
                    .parts()
                    .actors
                    .iter()
                    .find_map(|actor| match actor {
                        grenadiers_core::cpu::ObservedActor::Visible(actor)
                            if actor.id == placement.owner =>
                        {
                            Some((actor.position, actor.heading))
                        }
                        _ => None,
                    });
                assert_eq!(actual_pose, expected_pose);
            }
        }
        if observed_id.is_some() {
            break;
        }
    }
    let id = observed_id.expect("placed bomb is recipient-visible");
    assert_eq!(
        driver.cpu.as_ref().unwrap().plant_presence(),
        Some(PlantPresence::Present(id))
    );
}

#[test]
fn rejected_initial_movement_never_submits_the_bomb_input() {
    for profile in [CpuProfile::CURRENT, CpuProfile::CURRENT_EASY] {
        rejected_initial_movement(profile);
    }
}

fn rejected_initial_movement(profile: CpuProfile) {
    let mut driver = running_driver_for_profile(23, profile);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut rng = CpuRng::from_seed(key, [8; 32]);
    let plan = grenadiers_core::cpu::survival::plan(
        &basis,
        &UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        &mut rng,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(matches!(
        &plan.execution,
        Some(ExecutionCertificate::Plant(_))
    ));
    let admission = Tick(basis.parts().tick.0 + 6);
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis,
        driver.runtime.lifecycle().round_attempt(),
        plan,
    );
    while driver.runtime.lifecycle().tick() < admission {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            driver.publish(notice);
        }
    }
    let epoch = driver.runtime.lifecycle().round_attempt();
    for _ in 0..8 {
        driver
            .runtime
            .action(
                SeatId::Two,
                epoch,
                crate::game::rooms::match_runtime::MatchAction::Movement(None),
            )
            .unwrap();
    }
    let manual_before = driver.runtime.next_cpu_manual_action(SeatId::Two);
    driver.poll_cpu_result().unwrap();

    assert_eq!(
        driver.runtime.next_cpu_manual_action(SeatId::Two),
        manual_before
    );
    assert!(!driver.cpu.as_ref().unwrap().has_committed_route());
    assert!(!driver.cpu.as_ref().unwrap().is_escaping());
}

#[test]
fn accepted_bomb_with_same_cell_veto_selects_absent_and_keeps_route() {
    let mut driver = running_driver(23);
    driver
        .runtime
        .replace_state_for_test(shared_cell_fixture(driver.runtime.lifecycle().tick()));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut rng = CpuRng::from_seed(key, [8; 32]);
    let plan = grenadiers_core::cpu::survival::plan(
        &basis,
        &UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        &mut rng,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let placement = match &plan.execution {
        Some(ExecutionCertificate::Plant(certificate)) => *certificate.placement(),
        _ => panic!("expected certified plant for shared-cell fixture"),
    };
    let admission = Tick(basis.parts().tick.0 + 6);
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis.clone(),
        driver.runtime.lifecycle().round_attempt(),
        plan,
    );
    while driver.runtime.lifecycle().tick() < admission {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            driver.publish(notice);
        }
    }

    let epoch = driver.runtime.lifecycle().round_attempt();
    driver
        .runtime
        .action(SeatId::One, epoch, MatchAction::Bomb(1))
        .unwrap();
    let cpu_manual_before = driver.runtime.next_cpu_manual_action(SeatId::Two);
    driver.poll_cpu_result().unwrap();
    let cpu_manual_after = driver.runtime.next_cpu_manual_action(SeatId::Two);
    assert_eq!(
        cpu_manual_after,
        cpu_manual_before.and_then(|id| id.checked_add(1))
    );
    assert!(driver.cpu.as_ref().unwrap().has_committed_route());

    let mut confirmed_absent = false;
    for _ in 0..4 {
        driver.tick(Instant::now()).unwrap();
        let Some(observation) = driver.last.cpu_observation.as_ref() else {
            continue;
        };
        if observation.parts().tick < placement.placed_at {
            continue;
        }
        assert!(
            matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        );
        assert_eq!(
            driver.cpu.as_ref().unwrap().plant_presence(),
            Some(PlantPresence::Absent)
        );
        assert!(driver.cpu.as_ref().unwrap().has_committed_route());
        confirmed_absent = true;
        break;
    }
    assert!(
        confirmed_absent,
        "first post-placement publication confirms absence"
    );
    for _ in 0..2 {
        driver.tick(Instant::now()).unwrap();
    }
    assert_eq!(
        driver.runtime.next_cpu_manual_action(SeatId::Two),
        cpu_manual_after
    );
}

fn shared_cell_fixture(tick: Tick) -> State {
    use grenadiers_core::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || column == ARENA_WIDTH - 1 || row == 0 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let shared = CellId::new(7, 7);
    let arena = Arena::new(
        terrain,
        BTreeSet::from([CellId::new(8, 7)]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(shared),
        heading: Direction::West,
        speed: SpeedLevel::Four,
    });
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::new(SpeedLevel::Four, 64, 1, 60).unwrap()),
        World::new(arena),
        actors,
        tick,
    )
    .unwrap()
}
