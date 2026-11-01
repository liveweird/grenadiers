use std::sync::Arc;

use grenadiers_core::cpu::{MovementIntent, Observation, WorldId};
use grenadiers_core::game::{
    CellId, Direction, InputAction, MobNavigationPolicy, SeatId, SeatInput, SimulationSeed, State,
    StormCloudConfig, Tick,
};
use grenadiers_core::protocol::{RoomCode, RoomId};
use grenadiers_core::room::{
    BoardSize, CpuDifficulty, CpuProfile, RespawnLairTuning, RoomConfiguration, RoomSettings,
    SeatControl, SpikePitTuning,
};
use tokio::{sync::watch, time::Instant};

use super::{
    Driver,
    refuge_handoff_observation_support::{observe, state},
};
use crate::game::rooms::RandomFill;
use crate::game::rooms::cpu::{AcceptedPlan, CpuController, CpuPlanner};
use crate::game::rooms::roster_owner::RosterOwnerNotice;
use crate::game::rooms::roster_runtime::{RosterAction, RosterIdentity, RosterRuntime};
use crate::round_seed::RoundSeedSource;

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const EPOCH: u64 = 4;
pub(super) const WORLD: WorldId = WorldId {
    room_instance: 4_030,
    physical_round: 1,
};

pub(super) struct TerminalFixture {
    pub controller: CpuController,
    pub planner: CpuPlanner,
    pub state: State,
    pub settings: RoomSettings,
    pub observation: Observation,
}

pub(super) fn accepted_refuge() -> (TerminalFixture, AcceptedPlan) {
    let mut fixture = terminal_refuge();
    assert!(
        fixture
            .controller
            .submit_refuge_handoff(fixture.observation.clone(), EPOCH)
    );
    assert_eq!(fixture.planner.pump_for_test(1).delivered, 1);
    loop {
        if let Some(accepted) = fixture.controller.poll(fixture.state.tick(), EPOCH) {
            return (fixture, accepted);
        }
        fixture.state.step(&[]).unwrap();
        let observation = observe(&fixture.state, fixture.settings);
        assert!(fixture.controller.accept_observation(&observation));
        assert_ne!(
            fixture.controller.observe_route(&observation),
            Some(grenadiers_core::cpu::survival::RouteStatus::Invalidated)
        );
        fixture.observation = observation;
    }
}

pub(super) fn terminal_refuge() -> TerminalFixture {
    let (mut state, settings) = state();
    for _ in 0..48 {
        state.step(&input(Some(Direction::North), false)).unwrap();
    }
    let planner = CpuPlanner::without_workers();
    let mut controller = CpuController::new(
        planner.clone(),
        CPU,
        WORLD.room_instance,
        [30; 32],
        CpuProfile::HARD_V35,
    );
    let before_stop = observe(&state, settings);
    assert!(controller.accept_observation(&before_stop));
    controller.on_stop_admitted(state.tick());
    state.step(&input(None, false)).unwrap();
    let basis = observe(&state, settings);
    assert!(controller.accept_observation(&basis));
    let key = grenadiers_core::cpu::MemoryKey {
        world: basis.parts().world,
        cpu_seat: CPU,
    };
    let mut history = grenadiers_core::cpu::forecast::ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut rng = grenadiers_core::cpu::CpuRng::from_seed(key, [30; 32]);
    let first = grenadiers_core::cpu::survival::plan_counter_pressure_v29_with_origin(
        &basis,
        &grenadiers_core::cpu::policy::UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        None,
        &mut rng,
        &mut grenadiers_core::cpu::WorkBudget::new(65_536),
    )
    .unwrap();
    let execution = first.survival.execution.as_ref().unwrap();
    let first_update = execution.manual_execution_tick().unwrap();
    let final_stop = execution
        .route()
        .lane(first_update)
        .unwrap()
        .stop_update(execution.route().steps().len() - 1)
        .unwrap();
    let admission = Tick(first_update.0 - 1);
    while state.tick() < admission {
        state.step(&[]).unwrap();
        assert!(controller.accept_observation(&observe(&state, settings)));
    }
    controller.inject_plan_result_for_test(basis, EPOCH, first.survival);
    let accepted = controller.poll(admission, EPOCH).unwrap();
    let actual = accepted.intent;
    if actual.movement == MovementIntent::Stop {
        controller.on_stop_admitted(admission);
    } else {
        controller.on_movement_admitted();
    }
    controller.commit(accepted, actual);
    step_with_rival(
        &mut state,
        input(movement(actual.movement), true),
        first_update,
        true,
    );
    loop {
        let observation = observe(&state, settings);
        assert!(controller.accept_observation(&observation));
        assert_ne!(
            controller.observe_route(&observation),
            Some(grenadiers_core::cpu::survival::RouteStatus::Invalidated)
        );
        match controller.poll_route_command(state.tick(), EPOCH) {
            Some(grenadiers_core::cpu::survival::RouteStatus::Submit(command)) => {
                assert!(controller.commit_route_command(command));
                if command.movement() == MovementIntent::Stop {
                    controller.on_stop_admitted(state.tick());
                } else {
                    controller.on_movement_admitted();
                }
                step_with_rival(
                    &mut state,
                    input(movement(command.movement()), false),
                    first_update,
                    true,
                );
            }
            Some(grenadiers_core::cpu::survival::RouteStatus::Wait)
                if state.tick() >= final_stop =>
            {
                assert_eq!(
                    state.actor(CPU).unwrap().position().cell(),
                    Some(CellId::new(6, 3))
                );
                return TerminalFixture {
                    controller,
                    planner,
                    state,
                    settings,
                    observation,
                };
            }
            Some(
                grenadiers_core::cpu::survival::RouteStatus::Continue
                | grenadiers_core::cpu::survival::RouteStatus::Wait,
            ) => step_with_rival(&mut state, Vec::new(), first_update, true),
            status => panic!("unexpected route status: {status:?}"),
        }
    }
}

pub(super) fn driver(controller: CpuController) -> Driver {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: WORLD.room_instance,
            configuration,
        },
        RoomSettings::DEFAULT,
        StormCloudConfig::Single,
        RespawnLairTuning::DEFAULT,
        SpikePitTuning::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(30);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap();
    let admission = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(admission);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let notice = runtime.current_notice().unwrap();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(30);
        Ok(())
    });
    Driver::new(
        runtime,
        notice,
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        vec![controller],
    )
}

fn input(direction: Option<Direction>, bomb: bool) -> Vec<SeatInput> {
    let mut inputs = vec![SeatInput {
        seat: CPU,
        action: direction.map_or(InputAction::Clear, InputAction::Press),
    }];
    if bomb {
        inputs.push(SeatInput {
            seat: CPU,
            action: InputAction::PlaceBomb,
        });
    }
    inputs
}

fn movement(intent: MovementIntent) -> Option<Direction> {
    match intent {
        MovementIntent::Direction(direction) => Some(direction),
        MovementIntent::Stop => None,
    }
}

fn step_with_rival(
    state: &mut State,
    mut inputs: Vec<SeatInput>,
    first_update: Tick,
    move_rival: bool,
) {
    if move_rival {
        let offset = state.tick().0 + 1 - first_update.0;
        let action = match offset {
            0..=11 => InputAction::Press(Direction::East),
            12..=35 => InputAction::Press(Direction::North),
            _ => InputAction::Clear,
        };
        inputs.push(SeatInput {
            seat: SeatId::One,
            action,
        });
    }
    state.step(&inputs).unwrap();
}
