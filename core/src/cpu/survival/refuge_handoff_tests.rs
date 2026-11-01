use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{StrategicPhase, StrategicTarget, UtilityMemory};
use crate::cpu::{
    BombKnowledge, CpuRng, ManualIntent, MemoryKey, MovementIntent, Observation,
    ObservedBombTrigger, WorkBudget,
};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, State,
    StormCloudConfig, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RoomSettings, SpikePitConfig};

use super::reliable_v27_oil_tests::observe;
use super::{
    CommittedRoute, LairSurvivalPlan, RouteStatus, WORK_ALLOWANCE,
    plan_counter_pressure_v29_with_origin,
};

const CPU: SeatId = SeatId::Two;
const ORIGIN: CellId = CellId::new(5, 5);
const RIVAL: CellId = CellId::new(6, 5);

fn fixture() -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 240)
        .unwrap()
        .with_enabled_items(EnabledItems::TELEPORTER);
    let fixed = fixed_arena();
    let mut terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new(
                u16::try_from(index % 15).unwrap(),
                u16::try_from(index / 15).unwrap(),
            ))
            .unwrap()
    });
    // Open one fixed wall so the first certified refuge is also an exact
    // off-axis firing origin; this proves conditional handoff, not universal availability.
    terrain[4 * 15 + 6] = crate::game::Terrain::Floor;
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11), CellId::new(13, 1)],
    )
    .unwrap();
    let exposed = BTreeMap::from([
        (CellId::new(5, 8), PickupKind::BlastRange),
        (CellId::new(5, 7), PickupKind::BlastRange),
        (CellId::new(5, 6), PickupKind::BombCapacity),
    ]);
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(RIVAL),
            heading: Direction::East,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(CellId::new(5, 9)),
            heading: Direction::North,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(3),
            seat: SeatId::Three,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
    ];
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        Tick(30),
        RoundAssignment::first(),
        SimulationSeed::new([30; 32]),
    )
    .unwrap();
    (state, settings)
}

fn plan(observation: &Observation, memory: &UtilityMemory) -> (LairSurvivalPlan, u32) {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let mut rng = CpuRng::from_seed(key, [30; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let result = plan_counter_pressure_v29_with_origin(
        observation,
        memory,
        &history,
        observation.parts().tick,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    (result, work.spent())
}

fn input(movement: MovementIntent, bomb: bool) -> Vec<SeatInput> {
    let mut inputs = vec![SeatInput {
        seat: CPU,
        action: match movement {
            MovementIntent::Direction(direction) => InputAction::Press(direction),
            MovementIntent::Stop => InputAction::Clear,
        },
    }];
    if bomb {
        inputs.push(SeatInput {
            seat: CPU,
            action: InputAction::PlaceBomb,
        });
    }
    inputs
}

fn reach_terminal_refuge(
    state: &mut State,
    settings: RoomSettings,
    basis: Observation,
    planned: &LairSurvivalPlan,
) -> (CommittedRoute, Observation, Tick) {
    let control = &planned.survival.control;
    let execution = planned.survival.execution.clone().unwrap();
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
    }
    let current = observe(state, settings);
    let mut route = CommittedRoute::admit_execution(
        execution,
        basis,
        &current,
        admission,
        control.intent.movement,
    )
    .unwrap();
    state.step(&input(control.intent.movement, true)).unwrap();
    let mut settled = (control.intent.movement == MovementIntent::Stop).then_some(state.tick());
    let mut first_publication = true;
    loop {
        let current = observe(state, settings);
        assert_ne!(
            route.observe_with_stop(&current, settled),
            RouteStatus::Invalidated
        );
        if first_publication {
            assert!(route.terminal_refuge_handoff(&current, settled).is_none());
            first_publication = false;
        }
        match route.poll_command(state.tick(), settled) {
            RouteStatus::Submit(command) => {
                assert!(route.commit_command(command));
                state.step(&input(command.movement(), false)).unwrap();
                settled = (command.movement() == MovementIntent::Stop).then_some(state.tick());
            }
            RouteStatus::Wait if state.tick() >= final_stop => {
                return (route, current, settled.unwrap());
            }
            RouteStatus::Continue | RouteStatus::Wait => {
                state.step(&[]).unwrap();
            }
            status => panic!("unexpected route status at {:?}: {status:?}", state.tick()),
        }
    }
}

#[test]
fn live_first_bomb_can_yield_a_second_full_v29_plant_certificate() {
    let (mut state, settings) = fixture();
    for _ in 0..48 {
        state
            .step(&input(MovementIntent::Direction(Direction::North), false))
            .unwrap();
    }
    state.step(&input(MovementIntent::Stop, false)).unwrap();
    assert_eq!(state.actor(CPU).unwrap().position().cell(), Some(ORIGIN));
    assert_eq!(state.actor(CPU).unwrap().bomb_capacity(), 2);
    assert_eq!(state.actor(CPU).unwrap().bomb_range(), 4);
    let basis = observe(&state, settings);
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: CPU,
    };
    let (first, first_work) = plan(&basis, &UtilityMemory::new(key));
    assert_eq!(
        first.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert!(first_work <= WORK_ALLOWANCE);
    assert_eq!(
        first
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap()
            .target,
        StrategicTarget::Opponent { seat: SeatId::One }
    );
    let first_deadline = first
        .survival
        .execution
        .as_ref()
        .unwrap()
        .route()
        .survive_until();
    let (first_route, refuge, settled) = reach_terminal_refuge(&mut state, settings, basis, &first);
    let BombKnowledge::Visible(bombs) = &refuge.parts().bombs else {
        unreachable!()
    };
    assert_eq!(
        bombs
            .iter()
            .filter(|bomb| bomb.owner == EntityId(2))
            .count(),
        1
    );
    let first_detonation = bombs
        .iter()
        .find(|bomb| bomb.owner == EntityId(2))
        .and_then(|bomb| match bomb.trigger {
            ObservedBombTrigger::Timed {
                remaining_ticks,
                paused: false,
            } => Some(Tick(refuge.parts().tick.0 + u64::from(remaining_ticks))),
            _ => None,
        })
        .unwrap();
    let refuge_cell = state.actor(CPU).unwrap().position().cell().unwrap();
    assert_ne!(refuge_cell.column, ORIGIN.column);
    assert_ne!(refuge_cell.row, ORIGIN.row);
    assert!(first_route.terminal_refuge_handoff(&refuge, None).is_none());
    assert!(
        first_route
            .terminal_refuge_handoff(&refuge, Some(Tick(settled.0 - 1)))
            .is_none()
    );
    let handoff = first_route
        .terminal_refuge_handoff(&refuge, Some(settled))
        .unwrap();
    assert_eq!(handoff.key().world, refuge.parts().world);
    assert_eq!(handoff.refuge(), state.actor(CPU).unwrap().position());
    assert_eq!(handoff.checkpoint(), &refuge);
    assert_eq!(handoff.old_survive_until(), first_deadline);
    assert_eq!(handoff.bomb_id(), bombs[0].id);

    for changed in [
        refuge
            .clone()
            .map_parts(|parts| parts.world.room_instance += 1)
            .unwrap(),
        refuge
            .clone()
            .map_parts(|parts| parts.world.physical_round += 1)
            .unwrap(),
        refuge
            .clone()
            .map_parts(|parts| {
                let actor = parts
                    .actors
                    .iter_mut()
                    .find(|actor| actor.seat() == CPU)
                    .unwrap();
                let crate::cpu::ObservedActor::Visible(actor) = actor else {
                    unreachable!()
                };
                actor.bomb_capacity = actor.live_bomb_count;
            })
            .unwrap(),
        refuge
            .clone()
            .map_parts(|parts| parts.tick = Tick(parts.tick.0 + 1))
            .unwrap(),
        refuge
            .clone()
            .map_parts(|parts| {
                let actor = parts
                    .actors
                    .iter_mut()
                    .find(|actor| actor.seat() == CPU)
                    .unwrap();
                let crate::cpu::ObservedActor::Visible(actor) = actor else {
                    unreachable!()
                };
                actor.position = Position::center(ORIGIN);
            })
            .unwrap(),
        refuge
            .clone()
            .map_parts(|parts| {
                let rival = parts
                    .actors
                    .iter_mut()
                    .find(|actor| actor.seat() == SeatId::One)
                    .unwrap();
                let crate::cpu::ObservedActor::Visible(rival) = rival else {
                    unreachable!()
                };
                rival.heading = Direction::West;
            })
            .unwrap(),
    ] {
        assert!(
            first_route
                .terminal_refuge_handoff(&changed, Some(settled))
                .is_none()
        );
    }

    let memory = first
        .survival
        .control
        .next_memory
        .mark_strategic_phase(
            StrategicPhase::Reengage,
            crate::cpu::policy::StrategicSwitchReason::Resumed,
            refuge.parts().tick,
        )
        .without_execution_authority();
    let (second, second_work) = plan(&refuge, &memory);
    assert_eq!(
        second.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert!(second_work <= WORK_ALLOWANCE);
    assert!(
        second
            .survival
            .execution
            .as_ref()
            .unwrap()
            .route()
            .survive_until()
            >= first_deadline
    );
    assert!(handoff.accepts_replacement(second.survival.execution.as_ref().unwrap()));
    let nonplant = super::ExecutionCertificate::Route(Box::new(
        second.survival.execution.as_ref().unwrap().route().clone(),
    ));
    assert!(!handoff.accepts_replacement(&nonplant));
    assert!(
        second
            .survival
            .execution
            .as_ref()
            .unwrap()
            .manual_execution_tick()
            .unwrap()
            < first_detonation
    );
    let second_deadline = second
        .survival
        .execution
        .as_ref()
        .unwrap()
        .route()
        .survive_until();
    let (mut second_route, _, second_settled) =
        reach_terminal_refuge(&mut state, settings, refuge, &second);
    while state.tick() < second_deadline && state.result().is_none() {
        state.step(&[]).unwrap();
        if state.result().is_some() {
            break;
        }
        let current = observe(&state, settings);
        assert_ne!(
            second_route.observe_with_stop(&current, Some(second_settled)),
            RouteStatus::Invalidated
        );
    }
    assert!(state.actor(CPU).unwrap().alive());
    assert!(state.tick() >= second_deadline || state.result().is_some());
    let _ = (first_route, settled);
}
