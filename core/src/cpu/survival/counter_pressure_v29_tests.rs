use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, ObservedEffect, WorkBudget};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, InputAction, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, State,
    StormCloudConfig, Terrain, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

use super::reliable_v27_oil_tests::observe;
use super::{
    CommittedRoute, LairSurvivalPlan, RouteStatus, WORK_ALLOWANCE,
    plan_counter_pressure_v29_with_origin, plan_hard_v28_with_origin,
    plan_reliable_v27_with_origin,
};

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const CPU_CELL: CellId = CellId::new(5, 5);
pub(super) const RIVAL_CELL: CellId = CellId::new(7, 5);

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let fixed = fixed_arena();
    let mut terrain = std::array::from_fn(|index| {
        let column = u16::try_from(index % 15).unwrap();
        let row = u16::try_from(index / 15).unwrap();
        fixed.terrain(CellId::new(column, row)).unwrap()
    });
    terrain[4 * 15 + 4] = Terrain::Floor;
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

pub(super) fn state(lairs: bool, pending_teleporter: bool) -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 180)
        .unwrap()
        .with_enabled_items(
            EnabledItems::from_bits(
                EnabledItems::TELEPORTER.bits() | EnabledItems::SHORT_FLAME.bits(),
            )
            .unwrap(),
        );
    let pending = pending_teleporter
        .then_some((CellId::new(3, 3), PickupKind::Teleporter))
        .into_iter()
        .collect();
    let exposed = (!pending_teleporter)
        .then_some((CPU_CELL, PickupKind::ShortFlame))
        .into_iter()
        .collect();
    let world =
        World::with_pickups(arena(BTreeSet::new()), BTreeMap::new(), pending, exposed).unwrap();
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let actors = [
        ActorInit {
            id: crate::game::EntityId(1),
            seat: SeatId::One,
            position: Position::center(RIVAL_CELL),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: crate::game::EntityId(2),
            seat: CPU,
            position: Position::center(CPU_CELL),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
    ];
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            lairs,
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        Tick(30),
        RoundAssignment::first(),
        SimulationSeed::new([29; 32]),
    )
    .unwrap();
    (state, settings)
}

pub(super) fn ordinary_counter_state(lairs: bool) -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::TELEPORTER);
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let actors = [
        ActorInit {
            id: crate::game::EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(9, 7)),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: crate::game::EntityId(2),
            seat: CPU,
            position: Position::center(CPU_CELL),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
    ];
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            lairs,
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        Tick(30),
        RoundAssignment::first(),
        SimulationSeed::new([29; 32]),
    )
    .unwrap();
    (state, settings)
}

pub(super) fn power_counter_state(lairs: bool) -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 180)
        .unwrap()
        .with_enabled_items(
            EnabledItems::from_bits(
                EnabledItems::TELEPORTER.bits() | EnabledItems::POWER_BOMB.bits(),
            )
            .unwrap(),
        );
    let exposed = BTreeMap::from([
        (CellId::new(7, 7), PickupKind::PowerBomb),
        (CellId::new(7, 6), PickupKind::BlastRange),
    ]);
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        exposed,
    )
    .unwrap();
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let actors = [
        ActorInit {
            id: crate::game::EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(7, 7)),
            heading: Direction::North,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: crate::game::EntityId(2),
            seat: CPU,
            position: Position::center(CPU_CELL),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
    ];
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            lairs,
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        Tick(30),
        RoundAssignment::first(),
        SimulationSeed::new([29; 32]),
    )
    .unwrap();
    (state, settings)
}

pub(super) fn observe_state(state: &State, settings: RoomSettings) -> crate::cpu::Observation {
    let tick = state.tick();
    observe(state, settings)
        .map_parts(|parts| {
            let actor = parts
                .actors
                .iter_mut()
                .find_map(|actor| match actor {
                    crate::cpu::ObservedActor::Visible(actor) if actor.seat == CPU => Some(actor),
                    _ => None,
                })
                .unwrap();
            actor.effects = state
                .actor(CPU)
                .unwrap()
                .effects()
                .observations()
                .filter_map(|(kind, expires_at)| {
                    Some(ObservedEffect {
                        kind,
                        remaining_ticks: u16::try_from(expires_at.0.checked_sub(tick.0)?).ok()?,
                    })
                })
                .collect();
        })
        .unwrap()
}

pub(super) fn plan(observation: &crate::cpu::Observation, version: u8) -> (LairSurvivalPlan, u32) {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    let _ = history.observe(observation);
    let mut rng = CpuRng::from_seed(key, [29; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = match version {
        27 => plan_reliable_v27_with_origin,
        28 => plan_hard_v28_with_origin,
        29 => plan_counter_pressure_v29_with_origin,
        _ => unreachable!(),
    }(
        observation,
        &UtilityMemory::new(key),
        &history,
        observation.parts().tick,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    (planned, work.spent())
}

fn movement_input(movement: MovementIntent) -> Vec<SeatInput> {
    let action = match movement {
        MovementIntent::Direction(direction) => InputAction::Press(direction),
        MovementIntent::Stop => InputAction::Clear,
    };
    vec![SeatInput { seat: CPU, action }]
}

pub(super) fn execute(
    state: &mut State,
    settings: RoomSettings,
    basis: crate::cpu::Observation,
    plan: LairSurvivalPlan,
) {
    let finish_tick = Tick(basis.parts().tick.0 + 230);
    let control = plan.survival.control;
    let execution = plan.survival.execution.unwrap();
    let admission_tick = Tick(execution.manual_execution_tick().unwrap().0 - 1);
    while state.tick() < admission_tick {
        state.step(&[]).unwrap();
    }
    let current = observe_state(state, settings);
    let mut route = CommittedRoute::admit_execution(
        execution,
        basis,
        &current,
        admission_tick,
        control.intent.movement,
    )
    .unwrap();
    let mut inputs = movement_input(control.intent.movement);
    inputs.push(SeatInput {
        seat: CPU,
        action: InputAction::PlaceBomb,
    });
    state.step(&inputs).unwrap();
    assert_eq!(state.public_view().bombs.len(), 2);
    let mut settled = (control.intent.movement == MovementIntent::Stop).then_some(state.tick());
    for _ in 0..300 {
        let current = observe_state(state, settings);
        assert_ne!(
            route.observe_with_stop(&current, settled),
            RouteStatus::Invalidated
        );
        let status = route.poll_command(state.tick(), settled);
        let (inputs, stopped) = match status {
            RouteStatus::Submit(command) => {
                assert!(route.commit_command(command));
                (
                    movement_input(command.movement()),
                    command.movement() == MovementIntent::Stop,
                )
            }
            RouteStatus::Invalidated => panic!("counter-pressure route invalidated"),
            RouteStatus::Completed => {
                while state.tick() <= finish_tick && state.result().is_none() {
                    state.step(&[]).unwrap();
                }
                break;
            }
            RouteStatus::Continue | RouteStatus::Wait => (Vec::new(), false),
        };
        state.step(&inputs).unwrap();
        settled = if stopped {
            Some(state.tick())
        } else if matches!(status, RouteStatus::Submit(_)) {
            None
        } else {
            settled
        };
        if state.tick() > finish_tick || state.result().is_some() {
            break;
        }
    }
    assert!(state.actor(CPU).unwrap().alive());
    assert!(!state.actor(SeatId::One).unwrap().alive());
    assert!(state.result().is_some() || state.public_view().bombs.is_empty());
}

#[test]
fn motion_uncertainty_over_the_cpu_keeps_counter_pressure_refused() {
    let (mut state, settings) = state(false, false);
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    for _ in 0..60 {
        state.step(&[]).unwrap();
    }
    let basis = observe_state(&state, settings);
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let execution = Tick(basis.parts().tick.0 + 7);
    let mut forecast_work = WorkBudget::new(WORK_ALLOWANCE);
    let forecast = crate::cpu::forecast::forecast(
        &basis,
        &history,
        None,
        crate::cpu::forecast::ForecastLimits::default(),
        &mut forecast_work,
    )
    .unwrap();
    assert!(
        forecast
            .cell(CPU_CELL)
            .unwrap()
            .uncertainty
            .iter()
            .any(|uncertain| {
                uncertain.kind == crate::cpu::forecast::HazardKind::Blocking
                    && uncertain.window.start <= basis.parts().tick
                    && uncertain.window.end > execution
            })
    );
    let mut direct_work = WorkBudget::new(WORK_ALLOWANCE);
    assert!(matches!(
        crate::cpu::routing::plan_lair_ordinary_plant(
            &basis,
            &history,
            crate::cpu::routing::PlantRequest {
                execution_tick: execution,
                settled_stop_tick: basis.parts().tick,
                first_movement: MovementIntent::Stop,
                limits: crate::cpu::routing::RouteLimits::default(),
            },
            &mut direct_work,
        ),
        Err(crate::cpu::routing::PlantFailure::Route(
            crate::cpu::routing::RouteFailure::NoRoute
        ))
    ));
    assert_eq!(plan(&basis, 29).0.survival.control.intent.manual, None);
}
