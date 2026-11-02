use std::collections::{BTreeMap, BTreeSet};

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, OilState, Position,
    PunchActivationPolicy, RulesConfig, SeatId, SpeedLevel, Tick, World,
};
use crate::room::{RespawnLairConfig, RoomSettings};

fn arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
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

fn tangent_west(cell: CellId) -> Position {
    Position {
        x: i32::from(cell.column) * crate::game::types::UNITS_PER_CELL
            - crate::game::types::HALF_FOOTPRINT,
        y: Position::center(cell).y,
    }
}

fn bomb(cell: CellId) -> Bomb {
    Bomb::new(
        EntityId(9),
        cell,
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap()
}

fn actors(one: Position, two: Position) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: one,
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: two,
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn rules_v5() -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL))
        .with_punch_activation_policy(PunchActivationPolicy::ContactV2)
}

fn east() -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(Direction::East),
    }
}

#[test]
fn stationary_tangency_is_not_an_attempt_and_unlit_oil_is_skipped_for_landing() {
    let source = CellId::new(5, 5);
    let first_landing = CellId::new(8, 5);
    let world = World::with_pickups_and_oil(
        arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(first_landing, OilState::Unlit)]),
    )
    .unwrap();
    let mut state = State::new(
        rules_v5(),
        world,
        actors(tangent_west(source), Position::center(CellId::new(13, 11))),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.world.bombs.insert(EntityId(9), bomb(source));

    state.step(&[]).unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert!(state.actors[0].contact_punch_latches.is_empty());

    state.step(&[east()]).unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(11, 5));
}

#[test]
fn an_exact_endpoint_player_bomb_tie_suppresses_contact_punch() {
    let source = CellId::new(5, 5);
    let tangent = tangent_west(source);
    let start = Position {
        x: tangent.x - 51,
        ..tangent
    };
    let blocker = Position {
        x: tangent.x + 2 * crate::game::types::HALF_FOOTPRINT,
        y: start.y,
    };
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::PUNCH);
    let rules = RulesConfig::environment_v9(
        settings,
        crate::game::StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
    )
    .with_punch_activation_policy(PunchActivationPolicy::ContactV2);
    let mut state = State::new(rules, World::new(arena()), actors(start, blocker)).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let planted = state
        .world
        .bombs
        .values()
        .find(|bomb| bomb.cell() == source)
        .map(|bomb| bomb.id())
        .unwrap();
    assert!(state.actors[1].bomb_grace.contains(&planted));

    state.step(&[east()]).unwrap();

    assert_eq!(state.actors[0].position, tangent);
    assert_eq!(state.world.bombs[&planted].cell(), source);
    assert!(state.actors[0].contact_punch_latches.is_empty());
}
