use super::contract::ForcedMovement;
use std::collections::{BTreeMap, BTreeSet};

use super::State;
use super::storm_test_support::{actors, arena_with_blocks, rules, state};
use crate::game::{
    CellId, EnabledItems, EntityId, MovementSegment, MovementTrace, OilState, PlayerCount,
    Position, SeatId, SimulationSeed, StormCloudConfig, Tick, World,
};
use crate::room::BoardSize;

#[test]
fn cloud_crosses_walls_blocks_bombs_flames_and_oil_without_interaction() {
    let board = BoardSize::FifteenByThirteen;
    let count = PlayerCount::Two;
    let blocked = arena_with_blocks(board, count, BTreeSet::from([CellId::new(6, 6)]));
    let mut blocked_game = State::new_at_tick_with_assignment_and_seed(
        rules(EnabledItems::EMPTY, StormCloudConfig::Single),
        World::new(blocked),
        actors(board, count),
        Tick(0),
        crate::game::RoundAssignment::first(),
        SimulationSeed::new([0; 32]),
    )
    .unwrap();
    let world = World::with_pickups_and_oil(
        arena_with_blocks(board, count, BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(6, 6), OilState::Unlit)]),
    )
    .unwrap();
    let mut game = State::new_at_tick_with_assignment_and_seed(
        rules(EnabledItems::OIL_BARREL, StormCloudConfig::Single),
        world,
        actors(board, count),
        Tick(0),
        crate::game::RoundAssignment::first(),
        SimulationSeed::new([0; 32]),
    )
    .unwrap();
    game.insert_test_flame(CellId::new(6, 6), Tick(500));
    game.insert_test_bomb(
        EntityId(90),
        CellId::new(6, 6),
        game.actors[0].id,
        1,
        Tick(0),
        crate::game::BombTrigger::Timed {
            detonates_at: Tick(500),
        },
    )
    .unwrap();
    for _ in 0..128 {
        game.step(&[]).unwrap();
        blocked_game.step(&[]).unwrap();
    }
    assert_eq!(game.storm_cloud(), blocked_game.storm_cloud());
    assert_eq!(
        game.storm_cloud().unwrap().position,
        Position::center(CellId::new(6, 6))
    );
}

#[test]
fn cloud_sweep_contacts_an_actor_from_either_side_of_motion() {
    let mut game = state([0; 32]);
    let cloud = game.storm_cloud().unwrap().position;
    game.actors[0].position = Position {
        x: cloud.x - 512 - 320,
        y: cloud.y,
    };
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].storm_slow_until(), Some(Tick(181)));
}

#[test]
fn ordinary_actor_sweep_contacts_the_final_cloud_footprint() {
    let game = state([0; 32]);
    let cloud = game.storm_cloud().unwrap().position;
    let start = Position {
        x: cloud.x - 1_200,
        y: cloud.y,
    };
    let mut crossing = MovementTrace::new(start);
    crossing.push(MovementSegment::new(
        start,
        Position {
            x: cloud.x + 1_200,
            y: cloud.y,
        },
    ));
    let stationary = MovementTrace::new(game.actors[1].position);
    let mut contacts = vec![false; 2];
    game.include_storm_movement_contacts(&mut contacts, &[crossing, stationary], &[]);
    assert_eq!(contacts, [true, false]);
}

#[test]
fn accepted_forced_sweep_contacts_the_final_cloud_footprint() {
    let game = state([0; 32]);
    let cloud = game.storm_cloud().unwrap().position;
    let segment = MovementSegment::new(
        Position {
            x: cloud.x,
            y: cloud.y - 1_200,
        },
        Position {
            x: cloud.x,
            y: cloud.y + 1_200,
        },
    );
    let forced = ForcedMovement {
        pusher: SeatId::Two,
        seat: SeatId::One,
        segment,
    };
    let traces = game
        .actors
        .iter()
        .map(|actor| MovementTrace::new(actor.position))
        .collect::<Vec<_>>();
    let mut contacts = vec![false; 2];
    game.include_storm_movement_contacts(&mut contacts, &traces, &[forced]);
    assert_eq!(contacts, [true, false]);
}

#[test]
fn final_endpoint_contact_is_included_without_a_path_sweep() {
    let mut game = state([0; 32]);
    game.actors[0].position = game.storm_cloud().unwrap().position;
    let mut contacts = vec![false; 2];
    game.include_storm_endpoint_contacts(&mut contacts);
    assert_eq!(contacts, [true, false]);
}
