use std::{collections::BTreeSet, sync::Arc};

use grenadiers_core::{
    cpu::{
        Intent, ManualIntent, MemoryKey, MovementIntent, Observation, ObservedActor, WorkBudget,
        forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast},
        policy::{ControlPlan, UtilityMemory},
        routing::{PulseRequest, RouteMode, certify_pulse},
        survival::SurvivalPlan,
    },
    game::{
        ActorInit, Arena, CellId, Direction, EntityId, Position, RulesConfig, SeatId, SpeedLevel,
        State, Terrain, World,
    },
    room::{CpuProfile, MatchPhase, RoomMode, RoomSettings},
};
use tokio::{sync::watch, time::Instant};

use super::super::Driver;
use crate::{
    game::rooms::{
        cpu::CpuPlanner,
        match_owner::{PreparedMatchOwner, test_support::identity},
        match_runtime::MatchAction,
        profile::RoomProfile,
    },
    round_seed::RoundSeedSource,
};

pub(super) fn running_driver(round_seed: u8) -> Driver {
    let (_, shutdown) = watch::channel(false);
    let mut room = identity(RoomProfile::ExpandedV5);
    room.mode = RoomMode::Solo(CpuProfile::CURRENT);
    room.private_cpu_seed = Some([17; 32]);
    let (_, owner) = PreparedMatchOwner::new_with_planner(
        room,
        Instant::now(),
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.fill(round_seed);
            Ok(())
        }),
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(31);
            Ok(())
        }),
        CpuPlanner::without_workers(),
        shutdown,
    )
    .unwrap();
    let mut driver = Driver::new(
        owner.runtime,
        owner.initial,
        owner.notices,
        Instant::now(),
        owner.random_fill,
        owner.cpu,
    );
    let creator = driver.runtime.prepare_solo_creator(SeatId::One).unwrap();
    driver.runtime.commit_admission(creator);
    driver
        .runtime
        .action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::new(SpeedLevel::Four, 64, 1, 60).unwrap(),
            },
        )
        .unwrap();
    driver.runtime.cpu_ready().unwrap();
    driver
        .runtime
        .action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 2,
            },
        )
        .unwrap();
    while driver.runtime.lifecycle().phase() != MatchPhase::Running {
        driver.runtime.settle_boundary([None; 2]).unwrap();
    }
    driver
}

#[test]
fn accepted_bomb_uses_only_committed_escape_until_hazard_clears() {
    let mut driver = running_driver(23);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let observation = driver
        .runtime
        .current_notice()
        .unwrap()
        .cpu_observation
        .unwrap();
    let origin = cpu_position(&observation);
    let mut planning_parts = observation.clone().into_parts();
    planning_parts.actors[SeatId::One.index()] = ObservedActor::Hidden {
        id: EntityId(1),
        seat: SeatId::One,
        alive: true,
    };
    let ObservedActor::Visible(cpu) = &mut planning_parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.speed = SpeedLevel::Base;
    planning_parts.rules.settings = RoomSettings::DEFAULT;
    let planning_observation = Observation::new(planning_parts).unwrap();
    let epoch = driver.runtime.lifecycle().round_attempt();
    let plan = driver
        .cpu
        .as_mut()
        .unwrap()
        .plan_now_for_test(&planning_observation, epoch);
    assert_eq!(plan.intent.manual, Some(ManualIntent::Bomb));
    driver.apply_cpu_plan(plan).unwrap();
    assert!(driver.cpu.as_ref().unwrap().is_escaping());
    let committed_draws = driver.cpu.as_ref().unwrap().rng_draws();

    let planted_at = driver.runtime.lifecycle().tick().0;
    let mut moved = false;
    let mut alive_after_fuse = false;
    while driver.runtime.lifecycle().tick().0 <= planted_at + 65 {
        driver.apply_cpu_stop().unwrap();
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            if let Some(observation) = notice.cpu_observation.as_ref() {
                moved |= cpu_position(observation) != origin;
                if observation.parts().tick.0 >= planted_at + 60 {
                    alive_after_fuse = cpu_alive(observation);
                }
            }
            let was_escaping = driver.cpu.as_ref().unwrap().is_escaping();
            driver.publish(notice);
            if was_escaping {
                assert!(!driver.cpu.as_ref().unwrap().has_pending());
                assert_eq!(driver.cpu.as_ref().unwrap().rng_draws(), committed_draws);
            }
        }
    }

    assert!(
        moved,
        "committed route must move away from the planted cell"
    );
    assert!(
        alive_after_fuse,
        "CPU must survive the shortest supported fuse"
    );
}

#[test]
fn first_cpu_plan_waits_for_an_executed_stop_snapshot() {
    let mut driver = running_driver(23);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let notice = driver.runtime.current_notice().unwrap();

    driver.consider_cpu_observation(&notice);

    assert!(!driver.cpu.as_ref().unwrap().has_pending());
    assert!(driver.cpu_stationary);
    assert!(driver.cpu_stop_at.is_none());

    for _ in 0..3 {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            driver.publish(notice);
            break;
        }
    }

    assert!(
        driver.cpu.as_ref().unwrap().has_pending(),
        "planning starts only after a later snapshot confirms the admitted Stop"
    );
}

#[test]
fn certified_long_pulse_executes_and_replans_after_its_actual_stop_snapshot() {
    let mut driver = running_driver(23);
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
    let forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let admission_tick = driver.runtime.lifecycle().tick();
    let lease = certify_pulse(
        &basis,
        &forecast,
        PulseRequest {
            movement: MovementIntent::Direction(Direction::West),
            hold_ticks: 12,
            start_window: TickWindow {
                start: grenadiers_core::game::Tick(admission_tick.0 + 1),
                end: grenadiers_core::game::Tick(admission_tick.0 + 2),
            },
            stopped_safe_until: grenadiers_core::game::Tick(admission_tick.0 + 40),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: basis.parts().tick,
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis.clone(),
        driver.runtime.lifecycle().round_attempt(),
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent::moving(Direction::West),
                hold_ticks: 12,
                next_memory: UtilityMemory::new(key),
            },
            lease: Some(lease),
        },
    );

    driver.poll_cpu_result().unwrap();
    assert_eq!(
        driver.cpu_stop_at,
        Some(grenadiers_core::game::Tick(admission_tick.0 + 12))
    );
    assert!(!driver.cpu_stationary);

    let origin = cpu_position(&basis);
    let mut saw_non_center = false;
    let mut latest = basis;
    while driver.runtime.lifecycle().tick().0 <= admission_tick.0 + 12 {
        driver.apply_cpu_stop().unwrap();
        latest = publish_next_cpu_observation(&mut driver);
        let position = cpu_position(&latest);
        saw_non_center |= position != Position::center(position.cell().unwrap());
    }

    assert_ne!(cpu_position(&latest), origin);
    assert!(
        saw_non_center,
        "the lease was checked during continuous movement"
    );
    assert!(driver.cpu_stationary);
    assert!(driver.cpu_stop_at.is_none());
    assert!(
        driver.cpu.as_ref().unwrap().has_pending(),
        "the first fresh post-Stop snapshot must settle the new pose and replan"
    );
}

pub(super) fn publish_next_cpu_observation(driver: &mut Driver) -> Observation {
    for _ in 0..3 {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            let observation = notice.cpu_observation.clone().unwrap();
            driver.publish(notice);
            return observation;
        }
    }
    panic!("expected a CPU observation publication");
}

pub(super) fn open_fixture_state(tick: grenadiers_core::game::Tick) -> State {
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
    let cpu_spawn = CellId::new(7, 7);
    let arena = Arena::new(
        terrain,
        BTreeSet::from([CellId::new(8, 7)]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::Two {
            cpu_spawn
        } else {
            CellId::new(9, 7)
        }),
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

#[test]
fn same_tick_cpu_rate_limit_is_nonfatal_and_committed_stop_retries() {
    let mut driver = running_driver(23);
    let epoch = driver.runtime.lifecycle().round_attempt();
    for _ in 0..8 {
        driver
            .runtime
            .action(
                SeatId::Two,
                epoch,
                MatchAction::Movement(Some(Direction::West)),
            )
            .unwrap();
    }
    driver.cpu_stop_at = Some(driver.runtime.lifecycle().tick());
    driver.cpu_stationary = false;

    assert!(driver.apply_cpu_stop().is_ok());
    assert!(driver.cpu_stop_at.is_some());
    assert!(!driver.cpu_stationary);

    driver.runtime.settle_boundary([None; 2]).unwrap();
    assert!(driver.apply_cpu_stop().is_ok());
    assert!(driver.cpu_stop_at.is_none());
    assert!(driver.cpu_stationary);
}

fn cpu_position(observation: &grenadiers_core::cpu::Observation) -> Position {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == SeatId::Two => Some(actor.position),
            _ => None,
        })
        .unwrap()
}

fn cpu_alive(observation: &grenadiers_core::cpu::Observation) -> bool {
    observation.parts().actors.iter().any(|actor| {
        matches!(actor, ObservedActor::Visible(actor) if actor.seat == SeatId::Two && actor.alive)
    })
}
