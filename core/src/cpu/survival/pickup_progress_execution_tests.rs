use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, WorkBudget};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, SpikePitPhase,
    State, StormCloudConfig, Terrain, Tick, World, fixed_arena, spike_pit_layout_v1,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

use super::reliable_v27_oil_tests::observe;
use super::{
    WORK_ALLOWANCE, plan_hard_pickup_progress_v32_with_origin, plan_pickup_progress_v31_with_origin,
};

const CPU: SeatId = SeatId::Two;
const PICKUP: CellId = CellId::new(4, 5);

fn arena() -> Arena {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        let cell = CellId::new(
            u16::try_from(index % 15).unwrap(),
            u16::try_from(index / 15).unwrap(),
        );
        fixed.terrain(cell).unwrap_or(Terrain::Wall)
    });
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn state(kind: PickupKind, lairs: bool) -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::ALL);
    let world = World::with_pickups(
        arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(PICKUP, kind)]),
    )
    .unwrap();
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let mut state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            lairs,
            SpikePitConfig::Disabled,
        ),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: settings.starting_speed(),
            },
            ActorInit {
                id: EntityId(2),
                seat: CPU,
                position: Position::center(CellId::new(5, 5)),
                heading: Direction::West,
                speed: settings.starting_speed(),
            },
        ],
        Tick(30),
        RoundAssignment::first(),
        SimulationSeed::new([31; 32]),
    )
    .unwrap();
    let pickup = Position::center(PICKUP);
    state.set_test_actor_pose(
        CPU,
        Position {
            x: pickup.x + 964,
            y: pickup.y,
        },
        Direction::West,
    );
    (state, settings)
}

#[test]
fn direct_state_updates_collect_real_capacity_and_range_under_one_tick_certificates() {
    for (kind, lairs, hard) in [
        (PickupKind::BombCapacity, false, false),
        (PickupKind::BlastRange, true, true),
    ] {
        let (mut state, settings) = state(kind, lairs);
        let first = observe(&state, settings);
        let key = MemoryKey {
            world: first.parts().world,
            cpu_seat: CPU,
        };
        let mut history = ObservedHazardMemory::new(key);
        let mut memory = UtilityMemory::new(key);
        let mut rng = CpuRng::from_seed(key, [31; 32]);
        let planner = if hard {
            plan_hard_pickup_progress_v32_with_origin
        } else {
            plan_pickup_progress_v31_with_origin
        };
        let mut updates = 0;
        while state.world().exposed_pickups().contains_key(&PICKUP) && updates < 32 {
            let observed = observe(&state, settings);
            history.observe(&observed).unwrap();
            let mut work = WorkBudget::new(WORK_ALLOWANCE);
            let plan = planner(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut rng,
                &mut work,
            )
            .unwrap();
            assert_eq!(
                plan.survival.control.intent.movement,
                MovementIntent::Direction(Direction::West),
                "{kind:?} update {updates}"
            );
            assert_eq!(plan.survival.control.hold_ticks, 1, "{kind:?}");
            assert_eq!(plan.contact.is_some(), lairs, "{kind:?}");
            assert!(work.spent() <= WORK_ALLOWANCE);
            memory = plan.survival.control.next_memory;
            state
                .step(&[SeatInput {
                    seat: CPU,
                    action: InputAction::Press(Direction::West),
                }])
                .unwrap();
            let stopped_at = state.actor(CPU).unwrap().position();
            state
                .step(&[SeatInput {
                    seat: CPU,
                    action: InputAction::Release(Direction::West),
                }])
                .unwrap();
            assert_eq!(state.actor(CPU).unwrap().position(), stopped_at);
            updates += 1;
        }
        assert!(!state.world().exposed_pickups().contains_key(&PICKUP));
        assert!(updates < 32, "{kind:?} needed {updates} updates");
        let actor = state.actor(CPU).unwrap();
        match kind {
            PickupKind::BombCapacity => assert_eq!(actor.bomb_capacity(), 2),
            PickupKind::BlastRange => assert_eq!(actor.bomb_range(), 3),
            _ => unreachable!(),
        }
    }
}

#[test]
fn pickup_progress_does_not_bypass_teleporter_or_diarrhea_risk() {
    for kind in [PickupKind::Teleporter, PickupKind::Diarrhea] {
        let (state, settings) = state(kind, true);
        let observed = observe(&state, settings);
        let key = MemoryKey {
            world: observed.parts().world,
            cpu_seat: CPU,
        };
        let mut history = ObservedHazardMemory::new(key);
        history.observe(&observed).unwrap();
        for planner in [
            plan_pickup_progress_v31_with_origin,
            plan_hard_pickup_progress_v32_with_origin,
        ] {
            let plan = planner(
                &observed,
                &UtilityMemory::new(key),
                &history,
                observed.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [32; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
            .unwrap();
            assert_ne!(
                plan.survival.control.intent.movement,
                MovementIntent::Direction(Direction::West),
                "{kind:?}"
            );
        }
    }
}

#[test]
fn pickup_progress_does_not_enter_warning_or_raised_spike_pit() {
    let pit = spike_pit_layout_v1(&fixed_arena(), [])[0];
    let target = CellId::new(pit.column - 1, pit.row);
    for (phase, remaining) in [(SpikePitPhase::Warning, 6), (SpikePitPhase::Raised, 60)] {
        let observed = super::spike_pit_execution_tests::observed(0, phase, remaining)
            .map_parts(|parts| {
                let actor = parts
                    .actors
                    .iter_mut()
                    .find_map(|actor| match actor {
                        crate::cpu::ObservedActor::Visible(actor) if actor.seat == CPU => {
                            Some(actor)
                        }
                        _ => None,
                    })
                    .unwrap();
                let center = Position::center(pit);
                actor.position = Position {
                    x: center.x + 832,
                    y: center.y,
                };
                actor.heading = Direction::West;
                parts.arena.blocks.clear();
                parts.pickups = vec![crate::cpu::ObservedPickup {
                    cell: target,
                    kind: PickupKind::BombCapacity,
                }];
            })
            .unwrap();
        let key = MemoryKey {
            world: observed.parts().world,
            cpu_seat: CPU,
        };
        let mut history = ObservedHazardMemory::new(key);
        history.observe(&observed).unwrap();
        for planner in [
            plan_pickup_progress_v31_with_origin,
            plan_hard_pickup_progress_v32_with_origin,
        ] {
            let plan = planner(
                &observed,
                &UtilityMemory::new(key),
                &history,
                observed.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [33; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
            .unwrap();
            assert_ne!(
                plan.survival.control.intent.movement,
                MovementIntent::Direction(Direction::West),
                "{phase:?}"
            );
        }
    }
}
