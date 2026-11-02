use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena, ObservedBomb,
    ObservedBombTrigger, ObservedEnvironment, ObservedFlame, ObservedOil, ObservedPickup,
    ObservedRules, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use grenadiers_core::game::{
    ActorInit, Arena, BombTrigger, CellId, Direction, EnabledItems, EntityId, InputAction,
    OilBarrelState, PickupKind, Position, RulesConfig, SeatId, SeatInput, SimulationSeed,
    SpeedLevel, State, StormCloudConfig, Terrain, Tick, World,
};
use grenadiers_core::room::{CpuProfile, RespawnLairConfig, RoomSettings, SpikePitConfig};

use super::{CpuController, CpuPlanner};

const CPU: SeatId = SeatId::Two;
const WORLD: WorldId = WorldId {
    room_instance: 27,
    physical_round: 1,
};

#[test]
fn cpu26_certifies_the_safe_detour_around_a_bomb_obstructed_refuge() {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let mut state = obstructed_refuge_state(settings);
    state.step(&[]).unwrap();
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    for _ in 0..12 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }])
        .unwrap();
    assert!(
        state
            .public_view()
            .pickups
            .iter()
            .all(|pickup| pickup.kind != PickupKind::Stopwatch)
    );

    let planner = CpuPlanner::without_workers();
    let mut controller = CpuController::new(
        planner.clone(),
        CPU,
        WORLD.room_instance,
        [27; 32],
        CpuProfile::SPIKE_PITS_STANDARD,
    );
    let basis = observe(&state, settings);
    assert_eq!(state.public_view().bombs[0].range, 8);
    let before_deadline = timed_remaining(&basis);
    assert!(before_deadline > 120);
    assert!(controller.accept_observation(&basis));
    controller.on_stop_admitted(basis.parts().tick);
    state.step(&[]).unwrap();
    let stopped = observe(&state, settings);
    assert_eq!(timed_remaining(&stopped), before_deadline);
    assert!(controller.accept_observation(&stopped));
    assert!(controller.submit(stopped.clone(), 4));
    assert_eq!(planner.pump_for_test(1).delivered, 1);

    let accepted = controller.poll(stopped.parts().tick, 4).unwrap();
    assert_eq!(
        accepted.intent.movement,
        grenadiers_core::cpu::MovementIntent::Direction(Direction::West),
        "hold={} certificate={}",
        accepted.hold_ticks,
        accepted.has_certificate_for_test()
    );
    assert!(accepted.has_certificate_for_test());
}

fn timed_remaining(observation: &Observation) -> u16 {
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        panic!("fixture bomb is visible")
    };
    let ObservedBombTrigger::Timed {
        remaining_ticks,
        paused: true,
    } = bombs[0].trigger
    else {
        panic!("fixture bomb is a paused timer")
    };
    remaining_ticks
}

fn obstructed_refuge_state(settings: RoomSettings) -> State {
    use grenadiers_core::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};

    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for column in 0..ARENA_WIDTH {
        terrain[usize::from(column)] = Terrain::Wall;
        terrain[usize::from(ARENA_HEIGHT - 1) * usize::from(ARENA_WIDTH) + usize::from(column)] =
            Terrain::Wall;
    }
    for row in 0..ARENA_HEIGHT {
        terrain[usize::from(row) * usize::from(ARENA_WIDTH)] = Terrain::Wall;
        terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(ARENA_WIDTH - 1)] =
            Terrain::Wall;
    }
    let cpu_cell = CellId::new(10, 3);
    let bomb_cell = CellId::new(11, 3);
    for cell in [CellId::new(10, 2), CellId::new(10, 4)] {
        terrain[usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column)] =
            Terrain::Wall;
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        vec![CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(bomb_cell),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(cpu_cell),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ];
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
            SpikePitConfig::Disabled,
        ),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([
                (bomb_cell, PickupKind::GoldenFlame),
                (CellId::new(12, 3), PickupKind::Stopwatch),
            ]),
        )
        .unwrap(),
        actors,
        Tick(20),
        grenadiers_core::game::RoundAssignment::first(),
        SimulationSeed::new([27; 32]),
    )
    .unwrap()
}

fn observe(state: &State, settings: RoomSettings) -> Observation {
    let view = state.public_view();
    let actors = view
        .actors
        .iter()
        .map(|actor| {
            if actor.seat != CPU {
                return ObservedActor::Hidden {
                    id: actor.id,
                    seat: actor.seat,
                    alive: actor.alive,
                };
            }
            ObservedActor::Visible(VisibleActor {
                id: actor.id,
                seat: actor.seat,
                position: actor.position,
                heading: actor.heading,
                speed: actor.speed,
                alive: actor.alive,
                bomb_capacity: actor.bomb_capacity,
                bomb_range: actor.bomb_range,
                live_bomb_count: actor.live_bomb_count as u16,
                equipment: None,
                weapon: None,
                effects: Vec::new(),
                damage_grace_remaining_ticks: 0,
                voluntary_actions_frozen: false,
                last_teleport_tick: None,
                unknown: UnknownActorFacts::ALL,
            })
        })
        .collect();
    let bombs = view
        .bombs
        .iter()
        .map(|bomb| ObservedBomb {
            id: bomb.id,
            placed_at: bomb.placed_at,
            cell: bomb.cell,
            owner: bomb.owner,
            range: bomb.range,
            trigger: match bomb.trigger {
                BombTrigger::Timed { detonates_at } => ObservedBombTrigger::Timed {
                    remaining_ticks: u16::try_from(detonates_at.0 - view.tick.0).unwrap(),
                    paused: true,
                },
                BombTrigger::Remote => ObservedBombTrigger::Remote,
                BombTrigger::Contact => ObservedBombTrigger::Contact,
            },
            blast: bomb.blast,
            unknown: UnknownBombFacts::ALL,
        })
        .collect();
    let parts = ObservationParts {
        world: WORLD,
        tick: view.tick,
        recipient: CPU,
        rules: ObservedRules {
            punch_activation_policy: grenadiers_core::game::PunchActivationPolicy::ManualV1,
            version: 10,
            tick_rate: 60,
            running_updates: u16::try_from(view.running_updates).unwrap(),
            remaining_ticks: u16::try_from(view.remaining_ticks).unwrap(),
            settings,
        },
        arena: ObservedArena {
            width: view.arena.width,
            height: view.arena.height,
            terrain: view.arena.terrain,
            blocks: view.arena.blocks,
        },
        actors,
        bombs: BombKnowledge::Visible(bombs),
        flames: view
            .flames
            .into_iter()
            .filter_map(|flame| {
                let remaining = flame.expires_at.0.checked_sub(view.tick.0)?;
                Some(ObservedFlame {
                    cell: flame.cell,
                    remaining_ticks: u16::try_from(remaining).ok()?,
                })
            })
            .collect(),
        oil: view
            .oil_barrels
            .into_iter()
            .map(|oil| ObservedOil {
                cell: oil.cell,
                state: match oil.state {
                    OilBarrelState::Unlit => OilBarrelState::Unlit,
                    OilBarrelState::Burning => OilBarrelState::Burning,
                },
            })
            .collect(),
        pickups: view
            .pickups
            .into_iter()
            .map(|pickup| ObservedPickup {
                cell: pickup.cell,
                kind: pickup.kind,
            })
            .collect(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![None, Some(0)]),
        None,
        None,
    )
    .unwrap()
}
