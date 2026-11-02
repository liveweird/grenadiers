use std::collections::BTreeSet;

use crate::cpu::*;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    ActorInit, BlastKind, Bomb, BombTrigger, CellId, Direction, EntityId, OilBarrelState, Position,
    RulesConfig, SeatId, SpeedLevel, State, Tick, World,
};
use crate::room::RoomSettings;

use super::*;

pub(super) fn key() -> MemoryKey {
    MemoryKey {
        world: WorldId {
            room_instance: 1,
            physical_round: 2,
        },
        cpu_seat: SeatId::Two,
    }
}

fn actor(seat: SeatId, id: u32) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position: Position::center(CellId::new(1 + seat.index() as u16, 1)),
        heading: Direction::East,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 2,
        bomb_range: 3,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

pub(super) fn observation(tick: u64, bombs: BombKnowledge) -> Observation {
    let mut terrain = vec![Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row + 1 == ARENA_HEIGHT || column + 1 == ARENA_WIDTH {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Observation::new(ObservationParts {
        world: key().world,
        tick: Tick(tick),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 5,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: u16::try_from(tick).unwrap(),
            remaining_ticks: 1_000,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: ARENA_WIDTH,
            height: ARENA_HEIGHT,
            terrain,
            blocks: Vec::new(),
        },
        actors: vec![actor(SeatId::One, 10), actor(SeatId::Two, 20)],
        bombs,
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    })
    .unwrap()
}

pub(super) fn timed(id: u32, cell: CellId, remaining: u16, blast: BlastKind) -> ObservedBomb {
    ObservedBomb {
        id: EntityId(id),
        placed_at: crate::game::Tick(0),
        cell,
        owner: EntityId(10),
        range: 3,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: remaining,
            paused: false,
        },
        blast,
        unknown: UnknownBombFacts::ALL,
    }
}

pub(super) fn run(
    observation: &Observation,
    limits: ForecastLimits,
) -> Result<HazardForecast, ForecastFailure> {
    let memory = ObservedHazardMemory::new(key());
    forecast(
        observation,
        &memory,
        None,
        limits,
        &mut WorkBudget::new(65_536),
    )
}

#[test]
fn stationary_bombs_preserve_two_burns_and_the_safe_gap() {
    let target = CellId::new(7, 6);
    let observed = observation(
        0,
        BombKnowledge::Visible(vec![
            timed(1, CellId::new(7, 3), 2, BlastKind::Standard),
            timed(2, CellId::new(4, 6), 40, BlastKind::Standard),
        ]),
    );
    let result = run(&observed, ForecastLimits::default()).unwrap();

    assert_eq!(
        result.cell(target).unwrap().lethal,
        vec![
            TickWindow {
                start: Tick(2),
                end: Tick(32)
            },
            TickWindow {
                start: Tick(40),
                end: Tick(70)
            }
        ]
    );
    assert!(!result.cell(target).unwrap().lethal[0].contains(Tick(32)));
}

#[test]
fn stationary_timeline_matches_state_steps_at_window_boundaries() {
    let target = CellId::new(7, 6);
    let arena = open_arena();
    let spawns = arena.spawns().to_vec();
    let mut world = World::new(arena);
    for (id, cell, deadline) in [(1, CellId::new(7, 3), 2), (2, CellId::new(4, 6), 40)] {
        world.bombs.insert(
            EntityId(id),
            Bomb::new(
                EntityId(id),
                cell,
                EntityId(10),
                3,
                Tick(0),
                BombTrigger::Timed {
                    detonates_at: Tick(deadline),
                },
            )
            .unwrap(),
        );
    }
    let rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let mut state = State::new(rules, world, actors).unwrap();
    let observed = observation(
        0,
        BombKnowledge::Visible(vec![
            timed(1, CellId::new(7, 3), 2, BlastKind::Standard),
            timed(2, CellId::new(4, 6), 40, BlastKind::Standard),
        ]),
    );
    let forecast = run(&observed, ForecastLimits::default()).unwrap();
    for tick in 1..=70 {
        state.step(&[]).unwrap();
        assert_eq!(
            forecast
                .cell(target)
                .unwrap()
                .lethal
                .iter()
                .any(|window| window.contains(Tick(tick))),
            state.world().has_fire(target, Tick(tick)),
            "tick {tick}"
        );
    }
}

#[test]
fn burning_oil_has_no_invented_expiry() {
    let oil = CellId::new(5, 5);
    let mut observed = observation(9, BombKnowledge::Visible(Vec::new())).into_parts();
    observed.oil.push(ObservedOil {
        cell: oil,
        state: OilBarrelState::Burning,
    });
    let observed = Observation::new(observed).unwrap();
    let result = run(&observed, ForecastLimits::default()).unwrap();
    assert_eq!(
        result.cell(oil).unwrap().lethal,
        [TickWindow {
            start: Tick(9),
            end: Tick(849)
        }]
    );
}

#[test]
fn unknown_motion_and_trigger_never_look_safe() {
    let mut bomb = timed(7, CellId::new(7, 6), 20, BlastKind::Piercing);
    bomb.trigger = ObservedBombTrigger::Remote;
    let observed = observation(0, BombKnowledge::Visible(vec![bomb]));
    let result = run(&observed, ForecastLimits::default()).unwrap();
    assert!(
        result
            .cell(CellId::new(9, 6))
            .unwrap()
            .uncertainty
            .iter()
            .any(|value| matches!(value.reason, UncertaintyReason::BombMotion(EntityId(7))))
    );
    assert!(
        result
            .cell(CellId::new(7, 6))
            .unwrap()
            .uncertainty
            .iter()
            .any(|value| matches!(value.reason, UncertaintyReason::Trigger(EntityId(7))))
    );
}

#[test]
fn exhaustion_unavailable_bombs_and_clock_overflow_fail_closed() {
    let observed = observation(
        0,
        BombKnowledge::Visible(vec![timed(1, CellId::new(7, 6), 2, BlastKind::Power)]),
    );
    let memory = ObservedHazardMemory::new(key());
    assert_eq!(
        forecast(
            &observed,
            &memory,
            None,
            ForecastLimits::default(),
            &mut WorkBudget::new(1)
        ),
        Err(ForecastFailure::WorkExhausted)
    );
    assert_eq!(
        run(
            &observation(0, BombKnowledge::Unavailable),
            ForecastLimits::default()
        ),
        Err(ForecastFailure::BombKnowledgeUnavailable)
    );

    let mut overflow = observation(0, BombKnowledge::Visible(Vec::new())).into_parts();
    overflow.tick = Tick(u64::MAX);
    overflow.rules.running_updates = 0;
    let overflow = Observation::new(overflow).unwrap();
    assert_eq!(
        run(&overflow, ForecastLimits::default()),
        Err(ForecastFailure::ClockOverflow)
    );
}

#[test]
fn history_keeps_two_samples_and_displacement_sticky() {
    let mut memory = ObservedHazardMemory::new(key());
    let first = observation(
        1,
        BombKnowledge::Visible(vec![timed(9, CellId::new(3, 3), 20, BlastKind::Standard)]),
    );
    memory.observe(&first).unwrap();
    let mut moved = first.into_parts();
    moved.tick = Tick(2);
    moved.rules.running_updates = 2;
    let BombKnowledge::Visible(bombs) = &mut moved.bombs else {
        unreachable!()
    };
    bombs[0].cell = CellId::new(4, 3);
    let moved = Observation::new(moved).unwrap();
    memory.observe(&moved).unwrap();
    assert_eq!(memory.retained_samples(EntityId(9)), 2);
    assert!(
        !run_with_memory(&moved, &memory)
            .stationary_assumptions()
            .contains(&EntityId(9))
    );
}

#[test]
fn stationary_primary_layer_reuses_all_three_blast_geometries() {
    let source = CellId::new(3, 5);
    let behind = CellId::new(5, 5);
    let block = CellId::new(4, 5);
    for (blast, burns_behind) in [(BlastKind::Standard, false), (BlastKind::Piercing, true)] {
        let mut parts =
            observation(0, BombKnowledge::Visible(vec![timed(1, source, 2, blast)])).into_parts();
        parts.arena.blocks.push(block);
        let result = run(&Observation::new(parts).unwrap(), ForecastLimits::default()).unwrap();
        assert_eq!(
            !result.cell(behind).unwrap().lethal.is_empty(),
            burns_behind
        );
    }

    let mut power = observation(
        0,
        BombKnowledge::Visible(vec![timed(1, source, 2, BlastKind::Power)]),
    )
    .into_parts();
    power.arena.terrain
        [usize::from(block.row) * usize::from(ARENA_WIDTH) + usize::from(block.column)] =
        Terrain::Wall;
    let result = run(&Observation::new(power).unwrap(), ForecastLimits::default()).unwrap();
    assert!(!result.cell(behind).unwrap().lethal.is_empty());
    assert!(result.cell(block).unwrap().lethal.is_empty());
}

#[test]
fn forecast_ignition_turns_oil_into_persistent_fire() {
    let oil = CellId::new(4, 5);
    let mut parts = observation(
        0,
        BombKnowledge::Visible(vec![timed(1, CellId::new(3, 5), 2, BlastKind::Standard)]),
    )
    .into_parts();
    parts.oil.push(ObservedOil {
        cell: oil,
        state: OilBarrelState::Unlit,
    });
    let limits = ForecastLimits {
        horizon_ticks: 100,
        ..ForecastLimits::default()
    };
    let result = run(&Observation::new(parts).unwrap(), limits).unwrap();
    assert_eq!(
        result.cell(oil).unwrap().lethal,
        [TickWindow {
            start: Tick(2),
            end: Tick(100)
        }]
    );
    assert_eq!(
        result.cell(oil).unwrap().blocking,
        [TickWindow {
            start: Tick(0),
            end: Tick(3)
        }]
    );
}

fn run_with_memory(observation: &Observation, memory: &ObservedHazardMemory) -> HazardForecast {
    forecast(
        observation,
        memory,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

pub(super) fn open_arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row + 1 == ARENA_HEIGHT || column + 1 == ARENA_WIDTH {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}
