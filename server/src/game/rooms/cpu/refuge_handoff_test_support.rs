use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena, ObservedBomb,
    ObservedBombTrigger, ObservedEnvironment, ObservedFlame, ObservedOil, ObservedPickup,
    ObservedRules, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use grenadiers_core::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, InputAction, Position, RoundAssignment,
    RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick,
    World, fixed_arena,
};
use grenadiers_core::room::{CpuProfile, RespawnLairConfig, RoomSettings, SpikePitConfig};

use super::{CpuController, CpuPlanner};

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const EPOCH: u64 = 4;
const WORLD: WorldId = WorldId {
    room_instance: 4_030,
    physical_round: 1,
};

pub(super) struct RefugeFixture {
    pub controller: CpuController,
    pub planner: CpuPlanner,
    pub state: State,
    settings: RoomSettings,
}

impl RefugeFixture {
    pub fn publication(&self) -> Observation {
        observe(&self.state, self.settings)
    }

    pub fn advance_publication(&mut self) -> Observation {
        self.state.step(&[]).unwrap();
        let observation = self.publication();
        assert!(self.controller.accept_observation(&observation));
        assert_ne!(
            self.controller.observe_route(&observation),
            Some(grenadiers_core::cpu::survival::RouteStatus::Invalidated)
        );
        observation
    }
}

pub(super) fn queued_refuge() -> RefugeFixture {
    let (mut state, settings) = state();
    for _ in 0..48 {
        state.step(&input(Some(Direction::North), false)).unwrap();
    }
    state.step(&input(None, false)).unwrap();
    let basis = observe(&state, settings);
    let planner = CpuPlanner::without_workers();
    let mut controller = CpuController::new(
        planner.clone(),
        CPU,
        WORLD.room_instance,
        [30; 32],
        CpuProfile::HARD_V35,
    );
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let key = controller.memory.unwrap().key();
    let mut history = grenadiers_core::cpu::forecast::ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut rng = grenadiers_core::cpu::CpuRng::from_seed(key, [30; 32]);
    let first = grenadiers_core::cpu::survival::plan_counter_pressure_v29_with_origin(
        &basis,
        &controller.memory.unwrap(),
        &history,
        basis.parts().tick,
        None,
        &mut rng,
        &mut grenadiers_core::cpu::WorkBudget::new(65_536),
    )
    .unwrap();
    let execution = first.survival.execution.as_ref().unwrap();
    let admission = Tick(execution.manual_execution_tick().unwrap().0 - 1);
    while state.tick() < admission {
        state.step(&[]).unwrap();
    }
    let current = observe(&state, settings);
    assert!(controller.accept_observation(&current));
    controller.inject_plan_result_for_test(basis, EPOCH, first.survival);
    let accepted = controller.poll(admission, EPOCH).unwrap();
    let actual = accepted.intent;
    if actual.movement == grenadiers_core::cpu::MovementIntent::Stop {
        controller.on_stop_admitted(admission);
    } else {
        controller.on_movement_admitted();
    }
    controller.commit(accepted, actual);
    state
        .step(&input(movement_direction(actual.movement), true))
        .unwrap();
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
                if command.movement() == grenadiers_core::cpu::MovementIntent::Stop {
                    controller.on_stop_admitted(state.tick());
                } else {
                    controller.on_movement_admitted();
                }
                state
                    .step(&input(movement_direction(command.movement()), false))
                    .unwrap();
            }
            Some(grenadiers_core::cpu::survival::RouteStatus::Wait)
                if controller.submit_refuge_handoff(observation, EPOCH) =>
            {
                return RefugeFixture {
                    controller,
                    planner,
                    state,
                    settings,
                };
            }
            Some(
                grenadiers_core::cpu::survival::RouteStatus::Continue
                | grenadiers_core::cpu::survival::RouteStatus::Wait,
            ) => {
                state.step(&[]).unwrap();
            }
            status => panic!("unexpected route status: {status:?}"),
        }
    }
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

fn movement_direction(movement: grenadiers_core::cpu::MovementIntent) -> Option<Direction> {
    match movement {
        grenadiers_core::cpu::MovementIntent::Direction(direction) => Some(direction),
        grenadiers_core::cpu::MovementIntent::Stop => None,
    }
}

fn state() -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 240)
        .unwrap()
        .with_enabled_items(EnabledItems::TELEPORTER);
    let fixed = fixed_arena();
    let mut terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    terrain[4 * 15 + 6] = grenadiers_core::game::Terrain::Floor;
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11), CellId::new(13, 1)],
    )
    .unwrap();
    let exposed = BTreeMap::from([
        (
            CellId::new(5, 8),
            grenadiers_core::game::PickupKind::BlastRange,
        ),
        (
            CellId::new(5, 7),
            grenadiers_core::game::PickupKind::BlastRange,
        ),
        (
            CellId::new(5, 6),
            grenadiers_core::game::PickupKind::BombCapacity,
        ),
    ]);
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    let actors = [
        ActorInit {
            id: grenadiers_core::game::EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(6, 5)),
            heading: Direction::East,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: grenadiers_core::game::EntityId(2),
            seat: CPU,
            position: Position::center(CellId::new(5, 9)),
            heading: Direction::North,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: grenadiers_core::game::EntityId(3),
            seat: SeatId::Three,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
    ];
    (
        State::new_at_tick_with_assignment_and_seed(
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
        .unwrap(),
        settings,
    )
}

fn observe(state: &State, settings: RoomSettings) -> Observation {
    let view = state.public_view();
    let actors = view
        .actors
        .iter()
        .map(|actor| {
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
                grenadiers_core::game::BombTrigger::Timed { detonates_at } => {
                    ObservedBombTrigger::Timed {
                        remaining_ticks: (detonates_at.0 - view.tick.0) as u16,
                        paused: false,
                    }
                }
                grenadiers_core::game::BombTrigger::Remote => ObservedBombTrigger::Remote,
                grenadiers_core::game::BombTrigger::Contact => ObservedBombTrigger::Contact,
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
            version: 10,
            tick_rate: 60,
            running_updates: view.running_updates as u16,
            remaining_ticks: view.remaining_ticks as u16,
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
                (remaining > 0).then_some(ObservedFlame {
                    cell: flame.cell,
                    remaining_ticks: remaining as u16,
                })
            })
            .collect(),
        oil: view
            .oil_barrels
            .into_iter()
            .map(|oil| ObservedOil {
                cell: oil.cell,
                state: oil.state,
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
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 3]),
        None,
        None,
    )
    .unwrap()
}
