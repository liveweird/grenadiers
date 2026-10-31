use std::collections::BTreeSet;

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::survival::{CommittedRoute, RouteStatus, plan_respawn};
use crate::cpu::{
    BombKnowledge, CpuRng, ManualIntent, MemoryKey, MovementIntent, Observation, ObservationParts,
    ObservedActor, ObservedArena, ObservedBomb, ObservedBombTrigger, ObservedEnvironment,
    ObservedFlame, ObservedOil, ObservedPickup, ObservedRules, UnknownActorFacts, UnknownBombFacts,
    VisibleActor, WorkBudget, WorldId,
};
use crate::game::{
    ActorInit, Arena, BombTrigger, CellId, Direction, EntityId, InputAction, OilBarrelState,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, StormCloudConfig, Terrain, Tick,
    World,
};
use crate::room::{RespawnLairConfig, RoomSettings};

const HUMAN: SeatId = SeatId::One;
const ATTACKER: SeatId = SeatId::Two;
const OPPONENT: SeatId = SeatId::Three;
const WORLD_ID: WorldId = WorldId {
    room_instance: 91,
    physical_round: 1,
};

fn state() -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 3, 60).unwrap();
    let actors = [
        (HUMAN, CellId::new(1, 1), Direction::East),
        (ATTACKER, CellId::new(5, 7), Direction::East),
        (OPPONENT, CellId::new(11, 7), Direction::West),
    ]
    .map(|(seat, cell, heading)| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(cell),
        heading,
        speed: settings.starting_speed(),
    });
    let mut terrain = [Terrain::Floor; crate::game::arena::ARENA_CELL_COUNT];
    for column in 0..crate::game::arena::ARENA_WIDTH {
        terrain[usize::from(column)] = Terrain::Wall;
        terrain[usize::from(crate::game::arena::ARENA_HEIGHT - 1)
            * usize::from(crate::game::arena::ARENA_WIDTH)
            + usize::from(column)] = Terrain::Wall;
    }
    for row in 0..crate::game::arena::ARENA_HEIGHT {
        terrain[usize::from(row) * usize::from(crate::game::arena::ARENA_WIDTH)] = Terrain::Wall;
        terrain[usize::from(row) * usize::from(crate::game::arena::ARENA_WIDTH)
            + usize::from(crate::game::arena::ARENA_WIDTH - 1)] = Terrain::Wall;
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        vec![CellId::new(1, 1), CellId::new(13, 1), CellId::new(1, 11)],
    )
    .unwrap();
    let state = State::new(
        RulesConfig::environment_v9(
            settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
        ),
        World::new(arena),
        actors,
    )
    .unwrap();
    (state, settings)
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
        .collect::<Vec<_>>();
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
                    paused: false,
                },
                BombTrigger::Remote => ObservedBombTrigger::Remote,
                BombTrigger::Contact => ObservedBombTrigger::Contact,
            },
            blast: bomb.blast,
            unknown: UnknownBombFacts::ALL,
        })
        .collect();
    let parts = ObservationParts {
        world: WORLD_ID,
        tick: view.tick,
        recipient: ATTACKER,
        rules: ObservedRules {
            version: 9,
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
                let remaining = u16::try_from(flame.expires_at.0.checked_sub(view.tick.0)?).ok()?;
                (remaining > 0).then_some(ObservedFlame {
                    cell: flame.cell,
                    remaining_ticks: remaining,
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
    Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Disabled,
            None,
            vec![Some(0); state.actors().len()],
        ),
        None,
    )
    .unwrap()
}

fn apply_plan(state: &mut State, movement: MovementIntent, manual: Option<ManualIntent>, hold: u8) {
    let mut inputs = Vec::new();
    if let MovementIntent::Direction(direction) = movement {
        inputs.push(SeatInput {
            seat: ATTACKER,
            action: InputAction::Press(direction),
        });
    }
    if manual == Some(ManualIntent::Bomb) {
        inputs.push(SeatInput {
            seat: ATTACKER,
            action: InputAction::PlaceBomb,
        });
    }
    state.step(&inputs).unwrap();
    for _ in 1..hold {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: ATTACKER,
            action: InputAction::Clear,
        }])
        .unwrap();
    for _ in 0..3 {
        state.step(&[]).unwrap();
    }
}

fn movement_input(movement: MovementIntent) -> Vec<SeatInput> {
    match movement {
        MovementIntent::Direction(direction) => vec![SeatInput {
            seat: ATTACKER,
            action: InputAction::Press(direction),
        }],
        MovementIntent::Stop => vec![SeatInput {
            seat: ATTACKER,
            action: InputAction::Clear,
        }],
    }
}

fn execute_certificate(
    state: &mut State,
    settings: RoomSettings,
    basis: Observation,
    plan: crate::cpu::survival::LairSurvivalPlan,
) -> Option<crate::game::BombView> {
    let control = plan.survival.control;
    let execution = plan.survival.execution.unwrap();
    let manual = execution.manual_intent();
    let admission_tick = execution
        .manual_execution_tick()
        .map_or(basis.parts().tick, |tick| Tick(tick.0 - 1));
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
    if manual == Some(ManualIntent::Bomb) {
        inputs.push(SeatInput {
            seat: ATTACKER,
            action: InputAction::PlaceBomb,
        });
    }
    state.step(&inputs).unwrap();
    let bomb = state
        .public_view()
        .bombs
        .into_iter()
        .find(|bomb| bomb.owner == EntityId(2));
    let mut settled_stop = None;
    for _ in 0..180 {
        if !state.actor(OPPONENT).unwrap().alive() {
            assert!(state.actor(ATTACKER).unwrap().alive());
            return bomb;
        }
        let current = observe(state, settings);
        if route.observe_with_stop(&current, settled_stop) == RouteStatus::Invalidated {
            state
                .step(&[SeatInput {
                    seat: ATTACKER,
                    action: InputAction::Clear,
                }])
                .unwrap();
            return bomb;
        }
        let status = route.poll_command(state.tick(), settled_stop);
        let (inputs, submitted_stop) = match status {
            RouteStatus::Submit(command) => {
                assert!(route.commit_command(command));
                (
                    movement_input(command.movement()),
                    command.movement() == MovementIntent::Stop,
                )
            }
            RouteStatus::Completed => return bomb,
            RouteStatus::Invalidated => panic!("certified attack route invalidated"),
            RouteStatus::Continue | RouteStatus::Wait => (Vec::new(), false),
        };
        state.step(&inputs).unwrap();
        if submitted_stop {
            settled_stop = Some(state.tick());
        } else if matches!(status, RouteStatus::Submit(_)) {
            settled_stop = None;
        }
    }
    panic!("certified route did not complete")
}

#[test]
fn real_rules_v9_cpu_approaches_bombs_a_cpu_rival_and_escapes_after_human_is_out() {
    let (mut state, settings) = state();
    state
        .step(&[SeatInput {
            seat: HUMAN,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    for _ in 0..60 {
        state.step(&[]).unwrap();
    }
    assert!(!state.actor(HUMAN).unwrap().alive());
    assert!(state.actor(ATTACKER).unwrap().alive());
    assert!(state.actor(OPPONENT).unwrap().alive());

    let start = state.actor(ATTACKER).unwrap().position();
    let key = MemoryKey {
        world: WORLD_ID,
        cpu_seat: ATTACKER,
    };
    let mut memory = UtilityMemory::new(key);
    let mut hazards = ObservedHazardMemory::new(key);
    let mut rng = CpuRng::from_seed(key, [91; 32]);
    let mut attack_bomb = None;
    for _ in 0..60 {
        let observation = observe(&state, settings);
        hazards.observe(&observation).unwrap();
        let plan = plan_respawn(
            &observation,
            &memory,
            &hazards,
            observation.parts().tick,
            &mut rng,
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        memory = plan.survival.control.next_memory;
        if plan.survival.execution.is_some() {
            let planted = execute_certificate(&mut state, settings, observation, plan);
            attack_bomb = attack_bomb.or(planted);
            if !state.actor(OPPONENT).unwrap().alive() {
                break;
            }
            continue;
        }
        apply_plan(
            &mut state,
            plan.survival.control.intent.movement,
            plan.survival.control.intent.manual,
            plan.survival.control.hold_ticks,
        );
    }
    let bomb = attack_bomb.expect("CPU plants after reaching a firing position");
    let target = Position::center(CellId::new(11, 7));
    assert!(bomb.cell.column > start.cell().unwrap().column);
    assert_eq!(bomb.cell.row, target.cell().unwrap().row);
    assert!(bomb.cell.column.abs_diff(target.cell().unwrap().column) <= bomb.range);
    assert!(!state.actor(OPPONENT).unwrap().alive());
    assert!(state.actor(ATTACKER).unwrap().alive());
    assert_ne!(
        state.actor(ATTACKER).unwrap().position().cell(),
        Some(bomb.cell)
    );
}
