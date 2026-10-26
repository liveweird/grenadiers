use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::TickWindow;
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedOil, UnknownBombFacts, WorkBudget,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{
    ActorInit, Arena, Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment,
    InputAction, OilBarrelState, OilState, PickupKind, Position, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, Terrain, Tick, World,
};
use crate::room::RoomSettings;

use super::motion::{Motion, advance};
use super::tests::{forecast_for, observation, request};
use super::validation::validate;
use super::{RouteFailure, certify_pulse};

const START: CellId = CellId::new(2, 1);
const OBSTACLE: CellId = CellId::new(3, 1);

fn observed_with(block: bool, oil: bool, wall_pass: bool, bomb: bool) -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    if block {
        parts.arena.blocks.push(OBSTACLE);
    }
    if oil {
        parts.oil.push(ObservedOil {
            cell: OBSTACLE,
            state: OilBarrelState::Unlit,
        });
    }
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = Position::center(START);
    actor.heading = Direction::East;
    actor.speed = SpeedLevel::Base;
    actor.equipment = wall_pass.then_some(Equipment::WallPass);
    if bomb {
        parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(90),
            cell: CellId::new(5, 1),
            owner: EntityId(1),
            range: 2,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: 8,
                paused: false,
            },
            blast: crate::game::BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]);
    }
    Observation::new(parts).unwrap()
}

fn modeled_endpoint(observed: &Observation, updates: u8) -> Position {
    let forecast = forecast_for(observed);
    let context = validate(
        observed,
        &forecast,
        TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        Tick(23),
    )
    .unwrap();
    let mut motion = Motion::stopped(&context, Tick(0));
    for tick in 1..=updates {
        advance(
            &context,
            &mut motion,
            MovementIntent::Direction(Direction::East),
            Tick(u64::from(tick)),
        )
        .unwrap();
    }
    motion.position
}

fn arena(blocks: BTreeSet<CellId>, internal_wall: bool) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row + 1 == ARENA_HEIGHT || column + 1 == ARENA_WIDTH {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    if internal_wall {
        terrain
            [usize::from(OBSTACLE.row) * usize::from(ARENA_WIDTH) + usize::from(OBSTACLE.column)] =
            Terrain::Wall;
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(START),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
    ]
}

fn state(block: bool, oil: bool, wall_pass: bool, bomb: bool, internal_wall: bool) -> State {
    let arena = arena(
        block.then_some(OBSTACLE).into_iter().collect(),
        internal_wall,
    );
    let oil_sites = oil
        .then_some((OBSTACLE, OilState::Unlit))
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let mut world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        wall_pass
            .then_some((START, PickupKind::WallPass))
            .into_iter()
            .collect(),
        oil_sites,
    )
    .unwrap();
    if bomb {
        world.bombs.insert(
            EntityId(90),
            Bomb::new(
                EntityId(90),
                CellId::new(5, 1),
                EntityId(1),
                2,
                Tick(0),
                BombTrigger::Timed {
                    detonates_at: Tick(8),
                },
            )
            .unwrap(),
        );
    }
    let enabled = EnabledItems::from_bits(
        EnabledItems::WALL_PASS.bits()
            | if oil {
                EnabledItems::OIL_BARREL.bits()
            } else {
                0
            },
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let mut state = State::new(RulesConfig::expanded_v5(settings), world, actors()).unwrap();
    if wall_pass {
        state.step(&[]).unwrap();
        assert_eq!(
            state.actors()[SeatId::Two.index()].inventory().equipment(),
            Some(Equipment::WallPass)
        );
    }
    state
}

fn held_state(mut state: State, updates: u8) -> State {
    for _ in 0..updates {
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            }])
            .unwrap();
    }
    state
}

fn held_endpoint(state: State, updates: u8) -> Position {
    held_state(state, updates).actors()[SeatId::Two.index()].position()
}

fn assert_pulse_rejected(observed: &Observation) {
    let forecast = forecast_for(observed);
    assert_eq!(
        certify_pulse(
            observed,
            &forecast,
            request(MovementIntent::Direction(Direction::East), 12),
            &mut WorkBudget::new(65_536),
        ),
        Err(RouteFailure::NoRoute)
    );
}

#[test]
fn wall_pass_motion_ignores_soft_blocks_and_unlit_oil_but_certification_rejects_them() {
    for (block, oil) in [(true, false), (false, true)] {
        let observed = observed_with(block, oil, true, false);
        let modeled = modeled_endpoint(&observed, 12);
        let actual = held_endpoint(state(block, oil, true, false, false), 12);
        assert_eq!(modeled, actual);
        assert!(modeled.x > Position::center(START).x);
        assert_pulse_rejected(&observed);
    }
}

#[test]
fn known_occupied_starts_remain_rejected() {
    for (block, oil) in [(true, false), (false, true)] {
        let mut parts = observed_with(block, oil, true, false).into_parts();
        let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
            unreachable!()
        };
        actor.position = Position::center(OBSTACLE);
        let observed = Observation::new(parts).unwrap();
        let forecast = forecast_for(&observed);
        assert_eq!(
            certify_pulse(
                &observed,
                &forecast,
                request(MovementIntent::Direction(Direction::East), 1),
                &mut WorkBudget::new(65_536),
            ),
            Err(RouteFailure::Unsupported)
        );
    }
}

#[test]
fn disappearing_soft_block_does_not_create_a_false_pulse_certificate() {
    let observed = observed_with(true, false, false, true);
    let modeled = modeled_endpoint(&observed, 12);
    let actual_state = held_state(state(true, false, false, true, false), 12);
    let actual = actual_state.actors()[SeatId::Two.index()].position();
    let old_clipped_x = i32::from(OBSTACLE.column) * UNITS_PER_CELL - HALF_FOOTPRINT;
    assert!(!actual_state.world().arena().blocks().contains(&OBSTACLE));
    assert!(actual.x > old_clipped_x);
    assert!(actual.x < modeled.x);
    assert_pulse_rejected(&observed);
}

#[test]
fn igniting_unlit_oil_does_not_create_a_false_pulse_certificate() {
    let observed = observed_with(false, true, false, true);
    let modeled = modeled_endpoint(&observed, 12);
    let actual_state = held_state(state(false, true, false, true, false), 12);
    let actual = actual_state.actors()[SeatId::Two.index()].position();
    let old_clipped_x = i32::from(OBSTACLE.column) * UNITS_PER_CELL - HALF_FOOTPRINT;
    assert_eq!(
        actual_state
            .world()
            .oil_barrels()
            .find(|(cell, _)| *cell == OBSTACLE),
        Some((OBSTACLE, OilState::Burning))
    );
    assert!(actual.x > old_clipped_x);
    assert!(actual.x < modeled.x);
    assert_pulse_rejected(&observed);
}

#[test]
fn permanent_wall_clipping_and_open_corridor_motion_still_match_state() {
    let open = observed_with(false, false, false, false);
    assert_eq!(
        modeled_endpoint(&open, 12),
        held_endpoint(state(false, false, false, false, false), 12)
    );

    let mut wall_parts = open.into_parts();
    let index = usize::from(OBSTACLE.row) * usize::from(ARENA_WIDTH) + usize::from(OBSTACLE.column);
    wall_parts.arena.terrain[index] = Terrain::Wall;
    let wall = Observation::new(wall_parts).unwrap();
    assert_eq!(
        modeled_endpoint(&wall, 12),
        held_endpoint(state(false, false, false, false, true), 12)
    );
}
