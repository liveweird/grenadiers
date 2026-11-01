use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::{CpuRng, ManualIntent, MemoryKey, MovementIntent, WorkBudget};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EntityId, InputAction, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, State, StormCloudConfig, Tick,
    World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

use super::reliable_v27_oil_tests::observe;
use super::{
    CommittedRoute, LairSurvivalPlan, RouteStatus, WORK_ALLOWANCE, plan_hard_v28_with_origin,
    plan_reliable_v27_with_origin, plan_spike_pits_v26_with_origin, strategic_plant_still_targets,
};

const CPU: SeatId = SeatId::Two;
const START: CellId = CellId::new(5, 7);
const ORIGIN: CellId = CellId::new(7, 7);
const RIVAL: CellId = CellId::new(10, 7);
const FARM: CellId = CellId::new(3, 3);

fn arena() -> Arena {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        let column = u16::try_from(index % 15).unwrap();
        let row = u16::try_from(index / 15).unwrap();
        fixed.terrain(CellId::new(column, row)).unwrap()
    });
    Arena::new(
        terrain,
        BTreeSet::from([FARM]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn state(lairs: bool) -> (State, RoomSettings) {
    let settings = RoomSettings::DEFAULT;
    let pickups = BTreeMap::from([
        (CellId::new(6, 7), PickupKind::BlastRange),
        (ORIGIN, PickupKind::BlastRange),
    ]);
    let world = World::with_pickups(arena(), BTreeMap::new(), BTreeMap::new(), pickups).unwrap();
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let rules = RulesConfig::environment_v10(
        settings,
        StormCloudConfig::Disabled,
        lairs,
        SpikePitConfig::Disabled,
    );
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(RIVAL),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(START),
            heading: Direction::East,
            speed: settings.starting_speed(),
        },
    ];
    let state = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([28; 32]),
    )
    .unwrap();
    (state, settings)
}

fn upgraded(lairs: bool) -> (State, RoomSettings) {
    let (mut state, settings) = state(lairs);
    for tick in 0..40 {
        let inputs = (tick == 0)
            .then_some(SeatInput {
                seat: CPU,
                action: InputAction::Press(Direction::East),
            })
            .into_iter()
            .collect::<Vec<_>>();
        state.step(&inputs).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: CPU,
            action: InputAction::Clear,
        }])
        .unwrap();
    let actor = state.actor(CPU).unwrap();
    assert_eq!(actor.position(), Position::center(ORIGIN));
    assert_eq!(actor.bomb_range(), 4);
    (state, settings)
}

fn plan(
    state: &State,
    settings: RoomSettings,
    version: u8,
) -> (crate::cpu::Observation, LairSurvivalPlan, u32) {
    let observation = observe(state, settings);
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut hazards = ObservedHazardMemory::new(key);
    hazards.observe(&observation).unwrap();
    let mut rng = CpuRng::from_seed(key, [28; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = match version {
        26 => plan_spike_pits_v26_with_origin(
            &observation,
            &UtilityMemory::new(key),
            &hazards,
            observation.parts().tick,
            None,
            &mut rng,
            &mut work,
        ),
        27 => plan_reliable_v27_with_origin(
            &observation,
            &UtilityMemory::new(key),
            &hazards,
            observation.parts().tick,
            None,
            &mut rng,
            &mut work,
        ),
        28 => plan_hard_v28_with_origin(
            &observation,
            &UtilityMemory::new(key),
            &hazards,
            observation.parts().tick,
            None,
            &mut rng,
            &mut work,
        ),
        _ => unreachable!(),
    }
    .unwrap();
    (observation, planned, work.spent())
}

fn movement_input(movement: MovementIntent) -> Vec<SeatInput> {
    match movement {
        MovementIntent::Direction(direction) => vec![SeatInput {
            seat: CPU,
            action: InputAction::Press(direction),
        }],
        MovementIntent::Stop => vec![SeatInput {
            seat: CPU,
            action: InputAction::Clear,
        }],
    }
}

fn execute_attack(
    state: &mut State,
    settings: RoomSettings,
    basis: crate::cpu::Observation,
    planned: LairSurvivalPlan,
) {
    let control = planned.survival.control;
    let execution = planned.survival.execution.unwrap();
    let admission_tick = Tick(execution.manual_execution_tick().unwrap().0 - 1);
    while state.tick() < admission_tick {
        state.step(&[]).unwrap();
    }
    let current = observe(state, settings);
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
    let mut settled = (control.intent.movement == MovementIntent::Stop).then_some(state.tick());
    for _ in 0..300 {
        if !state.actor(SeatId::One).unwrap().alive() {
            for _ in 0..40 {
                state.step(&[]).unwrap();
            }
            assert!(state.actor(CPU).unwrap().alive());
            return;
        }
        let current = observe(state, settings);
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
            RouteStatus::Invalidated => panic!("hard attack route invalidated"),
            RouteStatus::Completed | RouteStatus::Continue | RouteStatus::Wait => {
                (Vec::new(), false)
            }
        };
        state.step(&inputs).unwrap();
        if stopped {
            settled = Some(state.tick());
        } else if matches!(status, RouteStatus::Submit(_)) {
            settled = None;
        }
    }
    panic!("hard attack did not cover the rival")
}

#[test]
fn authoritative_range_upgrade_makes_only_v28_attack_plant_and_escape() {
    for lairs in [false, true] {
        let (mut state, settings) = upgraded(lairs);
        let mut archived_control = None;
        let mut archived_work = None;
        for version in [26, 27] {
            let (_, archived, spent) = plan(&state, settings, version);
            let intent = archived
                .survival
                .control
                .next_memory
                .strategic_intent()
                .unwrap();
            assert_eq!(intent.objective, StrategicObjectiveKind::Farm);
            assert_eq!(intent.target, StrategicTarget::Block { cell: FARM });
            assert!(spent <= WORK_ALLOWANCE);
            if let Some(control) = archived_control {
                assert_eq!(archived.survival.control, control);
                assert_eq!(Some(spent), archived_work);
            } else {
                archived_control = Some(archived.survival.control);
                archived_work = Some(spent);
            }
        }

        let (basis, hard, spent) = plan(&state, settings, 28);
        let intent = hard
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap();
        assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
        assert_eq!(
            intent.target,
            StrategicTarget::Opponent { seat: SeatId::One }
        );
        assert_eq!(intent.approach_cell, Some(ORIGIN));
        assert_eq!(
            hard.survival.control.intent.manual,
            Some(ManualIntent::Bomb)
        );
        assert_eq!(hard.contact.is_some(), lairs);
        assert!(spent <= WORK_ALLOWANCE);
        let certificate = hard.survival.execution.as_ref().unwrap().plant().unwrap();
        assert!(strategic_plant_still_targets(
            &basis,
            intent,
            certificate,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ));
        execute_attack(&mut state, settings, basis, hard);
        assert!(state.actor(CPU).unwrap().alive());
        assert!(!state.actor(SeatId::One).unwrap().alive());
    }
}

#[test]
fn rival_motion_before_admission_rejects_the_attack_certificate() {
    let (mut state, settings) = upgraded(false);
    let (_basis, hard, _) = plan(&state, settings, 28);
    let control = hard.survival.control;
    let execution = hard.survival.execution.unwrap();
    let admission_tick = Tick(execution.manual_execution_tick().unwrap().0 - 1);
    while state.tick() < admission_tick {
        state.step(&[]).unwrap();
    }
    let changed = observe(&state, settings)
        .map_parts(|parts| {
            let rival = parts
                .actors
                .iter_mut()
                .find_map(|actor| match actor {
                    crate::cpu::ObservedActor::Visible(actor) if actor.seat == SeatId::One => {
                        Some(actor)
                    }
                    _ => None,
                })
                .unwrap();
            rival.position = Position::center(CellId::new(10, 8));
        })
        .unwrap();
    let intent = control.next_memory.strategic_intent().unwrap();
    let certificate = execution.plant().unwrap();
    assert!(!strategic_plant_still_targets(
        &changed,
        intent,
        certificate,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    ));
}

#[test]
fn hard_v28_entry_rejects_non_rules10_before_selection() {
    let observation = super::rules_v9_occupancy_tests::observed(CellId::new(7, 7), None);
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    assert!(matches!(
        plan_hard_v28_with_origin(
            &observation,
            &UtilityMemory::new(key),
            &ObservedHazardMemory::new(key),
            observation.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [28; 32]),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ),
        Err(crate::cpu::policy::PolicyError::InvalidObservation)
    ));
}
