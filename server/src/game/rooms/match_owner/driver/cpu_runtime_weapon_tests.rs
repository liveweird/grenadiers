use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{
        BombKnowledge, Intent, ManualIntent, MemoryKey, MovementIntent, Observation, ObservedActor,
        WorkBudget,
        forecast::ObservedHazardMemory,
        policy::{ControlPlan, UtilityMemory},
        routing::{PlantPresence, PlantRequest, RouteLimits, plan_timed_weapon_plant},
        survival::{ExecutionCertificate, SurvivalPlan, WORK_ALLOWANCE},
    },
    game::{
        ActorInit, Arena, BlastKind, CellId, Direction, EnabledItems, EntityId, PickupKind,
        Position, RulesConfig, SeatId, SpecialWeapon, SpeedLevel, State, Terrain, Tick, World,
    },
    room::RoomSettings,
};
use tokio::time::Instant;

use super::tests::{publish_next_cpu_observation, running_driver_with_settings};
use crate::game::rooms::match_runtime::MatchAction;

#[test]
fn acquired_timed_weapons_are_consumed_once_and_escape_without_workers() {
    for weapon in [SpecialWeapon::Piercing, SpecialWeapon::Power] {
        let mut driver = prepared_driver(weapon, false);
        let basis = acquired_basis(&mut driver, weapon);
        let (plan, placement) = timed_plan(&basis);
        let certificate = match &plan.execution {
            Some(ExecutionCertificate::Plant(certificate)) => certificate.as_ref().clone(),
            _ => unreachable!(),
        };
        let admission = Tick(basis.parts().tick.0 + 6);
        driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
            basis.clone(),
            driver.runtime.lifecycle().round_attempt(),
            plan,
        );
        settle_to_admission(&mut driver, admission);
        driver.poll_cpu_result().unwrap();
        let manual_after = driver.runtime.next_cpu_manual_action(SeatId::Two);

        let expected_blast = match weapon {
            SpecialWeapon::Piercing => BlastKind::Piercing,
            SpecialWeapon::Power => BlastKind::Power,
            _ => unreachable!(),
        };
        let observe_clear_by = Tick(placement.detonates_at.0 + 33);
        let mut confirmed = false;
        let mut observed_id = None;
        let mut saw_flame = false;
        let mut alive_after_clear = false;
        let mut last_observation_tick = basis.parts().tick;
        while driver.runtime.lifecycle().tick() <= observe_clear_by {
            driver.tick(Instant::now()).unwrap();
            let Some(observation) = driver.last.cpu_observation.as_ref() else {
                continue;
            };
            if observation.parts().tick <= last_observation_tick {
                continue;
            }
            last_observation_tick = observation.parts().tick;
            let actor = cpu_actor(observation);
            if observation.parts().tick >= placement.placed_at && !confirmed {
                let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
                    panic!("timed weapon bomb must be visible to its owner")
                };
                let bomb = bombs
                    .iter()
                    .find(|bomb| bomb.placed_at == placement.placed_at)
                    .expect("successful timed placement is published");
                assert_eq!(bomb.blast, expected_blast, "{weapon:?}");
                assert_eq!(actor.weapon, None, "{weapon:?} is consumed once");
                assert_eq!(
                    driver.cpu.as_ref().unwrap().plant_presence(),
                    Some(PlantPresence::Present(bomb.id))
                );
                observed_id = Some(bomb.id);
                confirmed = true;
            }
            if !observation.parts().flames.is_empty() {
                saw_flame = true;
                assert!(
                    driver.cpu.as_ref().unwrap().has_committed_route(),
                    "{weapon:?} route ended with flames at {:?}; survives {:?}; scene {}",
                    observation.parts().tick,
                    certificate.route().survive_until(),
                    certificate.matches_scene(
                        &basis,
                        observation,
                        PlantPresence::Present(observed_id.unwrap())
                    )
                );
                assert!(!driver.cpu.as_ref().unwrap().has_pending());
            }
            if saw_flame && observation.parts().flames.is_empty() {
                assert!(
                    matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()),
                    "{weapon:?}"
                );
                alive_after_clear = actor.alive;
            }
        }
        assert!(confirmed, "{weapon:?} placement was never confirmed");
        assert!(saw_flame, "{weapon:?} blast was never published");
        assert!(alive_after_clear, "{weapon:?} route survives flame expiry");
        assert_eq!(
            driver.runtime.next_cpu_manual_action(SeatId::Two),
            manual_after
        );
    }
}

#[test]
fn timed_weapon_same_cell_veto_retains_charge_and_never_retries() {
    for weapon in [SpecialWeapon::Piercing, SpecialWeapon::Power] {
        let mut driver = prepared_driver(weapon, true);
        let basis = acquired_basis(&mut driver, weapon);
        let (plan, placement) = timed_plan(&basis);
        let admission = Tick(basis.parts().tick.0 + 6);
        driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
            basis,
            driver.runtime.lifecycle().round_attempt(),
            plan,
        );
        settle_to_admission(&mut driver, admission);
        let epoch = driver.runtime.lifecycle().round_attempt();
        driver
            .runtime
            .action(SeatId::One, epoch, MatchAction::Bomb(1))
            .unwrap();
        driver.poll_cpu_result().unwrap();
        let manual_after = driver.runtime.next_cpu_manual_action(SeatId::Two);

        let observation = first_placement_observation(&mut driver, placement.placed_at);
        assert!(
            matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        );
        assert_eq!(cpu_actor(&observation).weapon, Some(weapon));
        assert_eq!(
            driver.cpu.as_ref().unwrap().plant_presence(),
            Some(PlantPresence::Absent)
        );
        assert!(driver.cpu.as_ref().unwrap().has_committed_route());
        for _ in 0..3 {
            driver.tick(Instant::now()).unwrap();
        }
        assert_eq!(
            driver.runtime.next_cpu_manual_action(SeatId::Two),
            manual_after
        );
    }
}

#[test]
fn timed_weapon_api_rejection_retains_charge_selects_absent_and_never_retries() {
    for weapon in [SpecialWeapon::Piercing, SpecialWeapon::Power] {
        let mut driver = prepared_driver(weapon, false);
        let basis = acquired_basis(&mut driver, weapon);
        let (plan, placement) = timed_plan(&basis);
        let admission = Tick(basis.parts().tick.0 + 6);
        driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
            basis,
            driver.runtime.lifecycle().round_attempt(),
            plan,
        );
        settle_to_admission(&mut driver, admission);
        let epoch = driver.runtime.lifecycle().round_attempt();
        for _ in 0..7 {
            driver
                .runtime
                .action(SeatId::Two, epoch, MatchAction::Movement(None))
                .unwrap();
        }
        let manual_before = driver.runtime.next_cpu_manual_action(SeatId::Two);
        driver.poll_cpu_result().unwrap();

        assert_eq!(
            driver.runtime.next_cpu_manual_action(SeatId::Two),
            manual_before
        );
        assert_eq!(
            driver.cpu.as_ref().unwrap().plant_presence(),
            Some(PlantPresence::Absent)
        );
        let observation = first_placement_observation(&mut driver, placement.placed_at);
        assert_eq!(cpu_actor(&observation).weapon, Some(weapon));
        assert!(
            matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        );
        for _ in 0..3 {
            driver.tick(Instant::now()).unwrap();
        }
        assert_eq!(
            driver.runtime.next_cpu_manual_action(SeatId::Two),
            manual_before
        );
    }
}

fn acquired_basis(driver: &mut super::super::Driver, weapon: SpecialWeapon) -> Observation {
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(driver);
    driver.cpu.as_mut().unwrap().cancel();
    assert_eq!(cpu_actor(&basis).weapon, Some(weapon));
    basis
}

fn timed_plan(basis: &Observation) -> (SurvivalPlan, grenadiers_core::cpu::routing::PlannedBomb) {
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(basis).unwrap();
    let movement = MovementIntent::Direction(Direction::West);
    let certificate = plan_timed_weapon_plant(
        basis,
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
    (
        SurvivalPlan {
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
        },
        placement,
    )
}

fn settle_to_admission(driver: &mut super::super::Driver, admission: Tick) {
    while driver.runtime.lifecycle().tick() < admission {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            driver.publish(notice);
        }
    }
}

fn first_placement_observation(driver: &mut super::super::Driver, placed_at: Tick) -> Observation {
    for _ in 0..4 {
        driver.tick(Instant::now()).unwrap();
        if let Some(observation) = driver.last.cpu_observation.as_ref()
            && observation.parts().tick >= placed_at
        {
            return observation.clone();
        }
    }
    panic!("expected first post-placement observation")
}

fn cpu_actor(observation: &Observation) -> &grenadiers_core::cpu::VisibleActor {
    match &observation.parts().actors[SeatId::Two.index()] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => panic!("CPU observes itself"),
    }
}

fn prepared_driver(weapon: SpecialWeapon, shared_cell: bool) -> super::super::Driver {
    let enabled = enabled_item(weapon);
    let mut driver = running_driver_with_settings(
        23,
        RoomSettings::new(SpeedLevel::Four, 64, 1, 60)
            .unwrap()
            .with_enabled_items(enabled),
    );
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(weapon_fixture(tick, weapon, shared_cell));
    driver
}

fn weapon_fixture(tick: Tick, weapon: SpecialWeapon, shared_cell: bool) -> State {
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
    let cpu_cell = CellId::new(7, 7);
    let arena = Arena::new(
        terrain,
        BTreeSet::from([CellId::new(8, 7)]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let pickup = match weapon {
        SpecialWeapon::Piercing => PickupKind::PiercingBomb,
        SpecialWeapon::Power => PickupKind::PowerBomb,
        _ => unreachable!(),
    };
    let enabled = enabled_item(weapon);
    let mut human_position = Position::center(if shared_cell {
        cpu_cell
    } else {
        CellId::new(1, 1)
    });
    if shared_cell {
        human_position.x += 1;
    }
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: if seat == SeatId::Two {
            Position::center(cpu_cell)
        } else {
            human_position
        },
        heading: Direction::West,
        speed: SpeedLevel::Four,
    });
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::new(SpeedLevel::Four, 64, 1, 60)
                .unwrap()
                .with_enabled_items(enabled),
        ),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(cpu_cell, pickup)]),
        )
        .unwrap(),
        actors,
        tick,
    )
    .unwrap()
}

fn enabled_item(weapon: SpecialWeapon) -> EnabledItems {
    match weapon {
        SpecialWeapon::Piercing => EnabledItems::PIERCING_BOMB,
        SpecialWeapon::Power => EnabledItems::POWER_BOMB,
        _ => unreachable!(),
    }
}
