use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, Observation, ObservationParts, ObservedActor, ObservedArena,
    ObservedBomb, ObservedBombTrigger, ObservedEffect, ObservedFlame, ObservedPickup,
    ObservedRules, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorkBudget, WorldId,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EntityId, InputAction, Position, RulesConfig, SeatId,
    SeatInput, SpeedLevel, State, Terrain, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

use super::*;

pub(crate) fn key() -> MemoryKey {
    MemoryKey {
        world: WorldId {
            room_instance: 4,
            physical_round: 2,
        },
        cpu_seat: SeatId::Two,
    }
}

pub(crate) fn observation(position: CellId) -> Observation {
    let mut terrain = vec![Terrain::Floor; ARENA_CELL_COUNT];
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
    Observation::new(ObservationParts {
        world: key().world,
        tick: Tick(10),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 5,
            tick_rate: 60,
            running_updates: 10,
            remaining_ticks: 1_000,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: ARENA_WIDTH,
            height: ARENA_HEIGHT,
            terrain,
            blocks: Vec::new(),
        },
        actors: [
            ObservedActor::Hidden {
                id: EntityId(1),
                seat: SeatId::One,
                alive: true,
            },
            ObservedActor::Visible(VisibleActor {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(position),
                heading: Direction::West,
                speed: SpeedLevel::Base,
                alive: true,
                bomb_capacity: 1,
                bomb_range: 2,
                live_bomb_count: 0,
                equipment: None,
                weapon: None,
                effects: Vec::new(),
                damage_grace_remaining_ticks: 0,
                voluntary_actions_frozen: false,
                last_teleport_tick: None,
                unknown: UnknownActorFacts::ALL,
            }),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    })
    .unwrap()
}

fn run(observation: &Observation, memory: UtilityMemory, work: u32) -> ControlPlan {
    let mut rng = CpuRng::from_seed(key(), [8; 32]);
    plan(observation, &memory, &mut rng, &mut WorkBudget::new(work)).unwrap()
}

#[test]
fn work_exhaustion_stops_without_manual_action() {
    let plan = run(
        &observation(CellId::new(7, 7)),
        UtilityMemory::new(key()),
        0,
    );
    assert_eq!(plan.intent, crate::cpu::Intent::STOP);
    assert_eq!(plan.hold_ticks, 1);
}

#[test]
fn identity_mismatch_is_rejected_before_planning() {
    let mut wrong = key();
    wrong.world.physical_round += 1;
    let observation = observation(CellId::new(7, 7));
    let mut rng = CpuRng::from_seed(key(), [8; 32]);
    assert_eq!(
        plan(
            &observation,
            &UtilityMemory::new(wrong),
            &mut rng,
            &mut WorkBudget::new(20)
        ),
        Err(PolicyError::IdentityMismatch)
    );
}

#[test]
fn immediate_flame_moves_to_a_safe_neighbor_deterministically() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(7, 7),
        remaining_ticks: 20,
    });
    parts.flames.push(ObservedFlame {
        cell: CellId::new(7, 6),
        remaining_ticks: 20,
    });
    let observation = Observation::new(parts).unwrap();
    let first = run(&observation, UtilityMemory::new(key()), 40);
    let second = run(&observation, UtilityMemory::new(key()), 40);
    assert_eq!(first, second);
    assert_ne!(first.intent, crate::cpu::Intent::STOP);
    assert!((1..=MAX_HOLD_TICKS).contains(&first.hold_ticks));
}

#[test]
fn visible_pickup_is_preferred_over_rival_pressure() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.pickups.push(ObservedPickup {
        cell: CellId::new(8, 7),
        kind: crate::game::PickupKind::BombCapacity,
    });
    let observation = Observation::new(parts).unwrap();
    assert_eq!(
        run(&observation, UtilityMemory::new(key()), 40)
            .intent
            .movement,
        crate::cpu::MovementIntent::Direction(Direction::East)
    );
}

#[test]
fn narrow_clear_escape_allows_bomb_but_effect_or_pickup_suppresses_it() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.arena.blocks.push(CellId::new(8, 7));
    let clear = Observation::new(parts.clone()).unwrap();
    let planted = run(&clear, UtilityMemory::new(key()), 100);
    assert_eq!(planted.intent.manual, Some(crate::cpu::ManualIntent::Bomb));
    assert!(planted.next_memory.route().iter().flatten().count() <= MAX_ROUTE_CELLS);
    let refuge = planted.next_memory.refuge().unwrap();
    assert_ne!(refuge.column, 7);
    assert_ne!(refuge.row, 7);

    let mut pickup_parts = parts.clone();
    let pickup_cell = CellId::new(7, 6);
    pickup_parts.pickups.push(ObservedPickup {
        cell: pickup_cell,
        kind: crate::game::PickupKind::MovementSpeed,
    });
    let around_pickup = run(
        &Observation::new(pickup_parts).unwrap(),
        UtilityMemory::new(key()),
        100,
    );
    assert_eq!(
        around_pickup.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(
        !around_pickup
            .next_memory
            .route()
            .contains(&Some(pickup_cell))
    );

    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.effects.push(ObservedEffect {
        kind: EffectKind::Diarrhea,
        remaining_ticks: 30,
    });
    let modified = Observation::new(parts).unwrap();
    assert_eq!(
        run(&modified, UtilityMemory::new(key()), 100).intent.manual,
        None
    );
}

#[test]
fn hidden_bombs_and_too_little_escape_work_never_place() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.arena.blocks.push(CellId::new(8, 7));
    parts.bombs = BombKnowledge::Unavailable;
    let hidden = Observation::new(parts).unwrap();
    assert_eq!(
        run(&hidden, UtilityMemory::new(key()), 100).intent,
        crate::cpu::Intent::STOP
    );

    let mut clear_parts = observation(CellId::new(7, 7)).into_parts();
    clear_parts.arena.blocks.push(CellId::new(8, 7));
    let clear = Observation::new(clear_parts).unwrap();
    assert_eq!(
        run(&clear, UtilityMemory::new(key()), 2).intent.manual,
        None
    );
}

#[test]
fn unsupported_control_effects_stop_even_when_about_to_expire() {
    for kind in [
        EffectKind::Diarrhea,
        EffectKind::SlowFoot,
        EffectKind::ReverseControls,
    ] {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
            unreachable!()
        };
        actor.effects.push(ObservedEffect {
            kind,
            remaining_ticks: 1,
        });
        let observation = Observation::new(parts).unwrap();
        let plan = run(&observation, UtilityMemory::new(key()), 100);
        assert_eq!(plan.intent, crate::cpu::Intent::STOP, "{kind:?}");
        assert_eq!(plan.hold_ticks, 1);
    }
}

#[test]
fn planting_requires_both_axes_within_the_exact_center_threshold() {
    for (x_offset, y_offset, heading, expected, alignment) in [
        (64, 64, Direction::West, true, None),
        (65, 0, Direction::West, false, Some(Direction::West)),
        (0, 65, Direction::West, false, Some(Direction::North)),
        (1, 65, Direction::West, false, Some(Direction::North)),
        (-1, -65, Direction::West, false, Some(Direction::South)),
        (65, 1, Direction::West, false, Some(Direction::West)),
        (65, 70, Direction::West, false, Some(Direction::West)),
        (65, 70, Direction::North, false, Some(Direction::North)),
    ] {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        parts.arena.blocks.push(CellId::new(8, 7));
        let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
            unreachable!()
        };
        actor.position.x += x_offset;
        actor.position.y += y_offset;
        actor.heading = heading;
        let observation = Observation::new(parts).unwrap();
        let plan = run(&observation, UtilityMemory::new(key()), 100);
        assert_eq!(
            plan.intent.manual.is_some(),
            expected,
            "{x_offset},{y_offset}"
        );
        if let Some(direction) = alignment {
            assert_eq!(
                plan.intent.movement,
                crate::cpu::MovementIntent::Direction(direction)
            );
        }
    }
}

#[test]
fn route_keeps_heading_until_inside_the_narrow_center_window() {
    let target = CellId::new(8, 7);
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position.x += 614;
    actor.heading = Direction::East;
    let observation = Observation::new(parts).unwrap();
    let memory = UtilityMemory::from_route(key(), &[target], false);
    let plan = run(&observation, memory, 40);
    assert_eq!(
        plan.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::East)
    );
    assert!((1..MAX_HOLD_TICKS).contains(&plan.hold_ticks));
}

#[test]
fn planted_route_crosses_the_visible_own_blast_toward_refuge() {
    let origin = CellId::new(7, 7);
    let mut initial_parts = observation(origin).into_parts();
    initial_parts.arena.blocks.push(CellId::new(8, 7));
    let initial = Observation::new(initial_parts).unwrap();
    let planted = run(&initial, UtilityMemory::new(key()), 100);
    assert_eq!(planted.intent.manual, Some(crate::cpu::ManualIntent::Bomb));

    let mut escape_parts = observation(origin).into_parts();
    escape_parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(9),
        placed_at: crate::game::Tick(0),
        cell: origin,
        owner: EntityId(2),
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 170,
            paused: false,
        },
        blast: crate::game::BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    let escape = Observation::new(escape_parts).unwrap();
    let continuing = run(&escape, planted.next_memory, 40);
    assert_ne!(continuing.intent, crate::cpu::Intent::STOP);
    assert_eq!(continuing.intent.manual, None);
}

#[test]
fn conservative_pulses_reach_minimum_turn_window_in_real_simulation() {
    let settings = RoomSettings::new(SpeedLevel::Base, 64, 3, 180).unwrap();
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let mut state = State::new(
        RulesConfig::configurable_v4(settings),
        World::new(arena),
        actors,
    )
    .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    for _ in 1..12 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }])
        .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    for _ in 1..9 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }])
        .unwrap();

    let target = Position::center(CellId::new(spawns[0].column + 1, spawns[0].row));
    let before_turn = state.actors()[0].position();
    assert!(before_turn.x.abs_diff(target.x) <= 64);
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::South),
        }])
        .unwrap();
    assert!(state.actors()[0].position().y > before_turn.y);
}
