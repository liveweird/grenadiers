use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use super::{
    ActorInit, DrawReason, InputAction, RoundResult, SeatInput, State, StateError, StepError,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::{
    CellId, Direction, EXPANDED_GENERATOR_VERSION, EnabledItems, EntityId, EventAudience,
    GENERATOR_VERSION, PickupKind, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId,
    SpeedLevel, StepEventKind, Tick, World, fixed_arena, generate_arena, generate_expanded_arena,
};
use crate::room::RoomSettings;

const CORNERS: [CellId; 4] = [
    CellId::new(1, 1),
    CellId::new(13, 1),
    CellId::new(1, 11),
    CellId::new(13, 11),
];

fn arena(count: usize) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    let spawns = if count == 2 {
        vec![CORNERS[0], CORNERS[3]]
    } else {
        CORNERS[..count].to_vec()
    };
    Arena::new(terrain, BTreeSet::new(), spawns).unwrap()
}

fn actors(count: PlayerCount) -> Vec<ActorInit> {
    count
        .seats()
        .iter()
        .map(|&seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(CORNERS[seat.index()]),
            heading: if matches!(seat, SeatId::One | SeatId::Three) {
                Direction::East
            } else {
                Direction::West
            },
            speed: SpeedLevel::Base,
        })
        .collect()
}

fn state(count: PlayerCount) -> State {
    State::new(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
        World::new(arena(count.get())),
        actors(count),
    )
    .unwrap()
}

#[test]
fn constructs_three_and_four_players_and_moves_every_participant() {
    for count in [PlayerCount::Three, PlayerCount::Four] {
        let mut game = state(count);
        let before: Vec<_> = game.actors().iter().map(|actor| actor.position()).collect();
        let inputs: Vec<_> = count
            .seats()
            .iter()
            .map(|&seat| SeatInput {
                seat,
                action: InputAction::Press(if matches!(seat, SeatId::One | SeatId::Three) {
                    Direction::East
                } else {
                    Direction::West
                }),
            })
            .collect();

        let report = game.step(&inputs).unwrap();

        assert_eq!(game.player_count(), count);
        assert_eq!(game.public_view().actors.len(), count.get());
        assert_eq!(report.traces.len(), count.get());
        for (&seat, old) in count.seats().iter().zip(before) {
            assert_ne!(game.actor(seat).unwrap().position(), old);
        }
    }
}

#[test]
fn out_of_order_actor_input_is_stored_in_canonical_seat_order() {
    let mut shuffled = actors(PlayerCount::Four);
    shuffled.rotate_right(2);
    let game = State::new(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
        World::new(arena(4)),
        shuffled,
    )
    .unwrap();

    assert_eq!(
        game.actors()
            .iter()
            .map(|actor| actor.seat())
            .collect::<Vec<_>>(),
        SeatId::ALL
    );
}

#[test]
fn rejects_every_nonparticipant_input_without_changing_any_state() {
    let mut game = state(PlayerCount::Three);
    let snapshot = game.clone();

    assert_eq!(
        game.step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
            SeatInput {
                seat: SeatId::Four,
                action: InputAction::PlaceBomb,
            },
        ]),
        Err(StepError::SeatNotParticipating(SeatId::Four))
    );
    assert_eq!(game, snapshot);
}

#[test]
fn contested_placements_are_all_rejected_while_independent_placement_succeeds() {
    let mut game = state(PlayerCount::Four);
    game.actors[0].position = Position::center(CellId::new(5, 5));
    game.actors[1].position = Position::center(CellId::new(5, 5));
    game.actors[2].position = Position::center(CellId::new(7, 5));

    game.step(&[
        SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        },
        SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        },
        SeatInput {
            seat: SeatId::Three,
            action: InputAction::PlaceBomb,
        },
    ])
    .unwrap();

    assert_eq!(game.world().bombs().len(), 1);
    assert_eq!(
        game.world().bombs().values().next().unwrap().owner(),
        EntityId(12)
    );
}

#[test]
fn flames_support_multiplayer_elimination_winner_draw_and_broadcast_audiences() {
    let mut winner = state(PlayerCount::Four);
    let eliminated: Vec<_> = winner.actors[..3]
        .iter()
        .map(|actor| actor.position.cell().unwrap())
        .collect();
    for cell in eliminated {
        winner.world.hazards.insert(cell, Tick(10));
    }
    let report = winner.step(&[]).unwrap();
    assert_eq!(winner.result(), Some(RoundResult::Winner(SeatId::Four)));
    assert!(
        report
            .events
            .iter()
            .all(|event| { event.audience == EventAudience::all(PlayerCount::Four) })
    );

    let mut draw = state(PlayerCount::Three);
    let eliminated: Vec<_> = draw
        .actors
        .iter()
        .map(|actor| actor.position.cell().unwrap())
        .collect();
    for cell in eliminated {
        draw.world.hazards.insert(cell, Tick(10));
    }
    draw.step(&[]).unwrap();
    assert_eq!(
        draw.result(),
        Some(RoundResult::Draw(DrawReason::MutualElimination))
    );
}

#[test]
fn ordinary_multiplayer_detonation_is_broadcast_to_every_participant() {
    let mut game = state(PlayerCount::Four);
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(90),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();

    let report = game.step(&[]).unwrap();
    let detonation = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
        .unwrap();
    assert_eq!(detonation.audience, EventAudience::all(PlayerCount::Four));
}

#[test]
fn pickup_tie_uses_next_contender_in_rotated_priority_order() {
    let pickup_cell = CellId::new(7, 5);
    let world = World::with_pickups(
        arena(4),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup_cell, PickupKind::BombCapacity)]),
    )
    .unwrap();
    let mut actor_inits = actors(PlayerCount::Four);
    actor_inits[SeatId::One.index()].position = Position::center(pickup_cell);
    actor_inits[SeatId::Three.index()].position = Position::center(pickup_cell);
    let assignment = RoundAssignment::new(NonZeroU32::new(2).unwrap());
    assert_eq!(
        assignment
            .priority_order(PlayerCount::Four)
            .collect::<Vec<_>>(),
        [SeatId::Two, SeatId::Three, SeatId::Four, SeatId::One]
    );
    let mut game = State::new_at_tick_with_assignment(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
        world,
        actor_inits,
        Tick(0),
        assignment,
    )
    .unwrap();

    let report = game.step(&[]).unwrap();

    assert_eq!(game.actor(SeatId::One).unwrap().bomb_capacity(), 1);
    assert_eq!(game.actor(SeatId::Three).unwrap().bomb_capacity(), 2);
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::PickupCollected {
            seat: SeatId::Three,
            ..
        }
    )));
}

#[test]
fn multiplayer_timeout_and_base_upgrade_collection_remain_supported() {
    let mut timed = state(PlayerCount::Three);
    timed.running_updates = timed.rules.round_ticks() - 1;
    timed.step(&[]).unwrap();
    assert_eq!(timed.result(), Some(RoundResult::Draw(DrawReason::Timeout)));

    let pickup_cell = CORNERS[2];
    let world = World::with_pickups(
        arena(3),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup_cell, PickupKind::BombCapacity)]),
    )
    .unwrap();
    let mut upgraded = State::new(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
        world,
        actors(PlayerCount::Three),
    )
    .unwrap();
    upgraded.step(&[]).unwrap();
    assert_eq!(upgraded.actor(SeatId::Three).unwrap().bomb_capacity(), 2);
}

#[test]
fn construction_enforces_profile_item_count_spawn_and_canonical_seat_boundaries() {
    assert_eq!(
        State::new(
            RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
            World::new(fixed_arena()),
            vec![actors(PlayerCount::Two)[0]],
        ),
        Err(StateError::InvalidPlayerCount(1))
    );
    assert_eq!(
        State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT),
            World::new(arena(3)),
            actors(PlayerCount::Three),
        ),
        Err(StateError::UnsupportedPlayerCount {
            version: 5,
            count: PlayerCount::Three,
        })
    );
    let enabled = EnabledItems::HEART;
    assert_eq!(
        State::new(
            RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(enabled)),
            World::new(arena(3)),
            actors(PlayerCount::Three),
        ),
        Err(StateError::UnsupportedEnabledItems(enabled))
    );
    assert!(matches!(
        State::new(
            RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
            World::new(arena(2)),
            actors(PlayerCount::Three),
        ),
        Err(StateError::SpawnCountMismatch {
            actors: 3,
            spawns: 2
        })
    ));
    let mut skipped = actors(PlayerCount::Three);
    skipped[1].seat = SeatId::Four;
    assert_eq!(
        State::new(
            RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
            World::new(arena(3)),
            skipped,
        ),
        Err(StateError::NonCanonicalSeats)
    );
}

#[test]
fn legacy_duel_trace_parity_is_preserved_by_vector_storage() {
    let arena = fixed_arena();
    let spawns = arena.spawns().to_vec();
    let duel: Vec<_> = SeatId::DUEL
        .into_iter()
        .map(|seat| ActorInit {
            id: EntityId(20 + seat.index() as u32),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect();
    let start = duel[0].position;
    let mut game = State::new(RulesConfig::classic_v1(), World::new(arena), duel).unwrap();

    let report = game
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert_eq!(report.traces.len(), 2);
    assert_eq!(report.traces[0].start(), start);
    assert_eq!(
        report.traces[0].segments().next().unwrap().end().x,
        start.x + 51
    );
    assert!(report.traces[1].is_empty());
}

#[test]
fn legacy_generator_parity_retains_versions_and_exact_duel_spawns() {
    let classic = generate_arena([0; 32]);
    let expanded = generate_expanded_arena([0; 32], EnabledItems::HEART);
    let expected = [CellId::new(1, 1), CellId::new(13, 11)];

    assert_eq!(classic.record().version(), GENERATOR_VERSION);
    assert_eq!(expanded.record().version(), EXPANDED_GENERATOR_VERSION);
    assert_eq!(classic.arena().spawns(), expected);
    assert_eq!(expanded.arena().spawns(), expected);
}
