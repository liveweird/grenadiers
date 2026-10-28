use std::collections::BTreeSet;

use super::{ActorInit, RoundActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, DrawReason, EffectKind, EnabledItems, EntityId,
    Equipment, EventAudience, HeldDirections, Inventory, PlayerCount, Position, RoundAssignment,
    RoundResult, RulesConfig, SeatId, SimulationSeed, SpeedLevel, StateError, StepError,
    StepEventKind, Tick, World,
};
use crate::room::RoomSettings;

const CORNERS: [CellId; 4] = [
    CellId::new(1, 1),
    CellId::new(13, 1),
    CellId::new(1, 11),
    CellId::new(13, 11),
];

fn arena(count: PlayerCount) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row == ARENA_HEIGHT - 1 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    let spawns = if count == PlayerCount::Two {
        vec![CORNERS[0], CORNERS[3]]
    } else {
        CORNERS[..count.get()].to_vec()
    };
    Arena::new(terrain, BTreeSet::new(), spawns).unwrap()
}

fn init(seat: SeatId) -> ActorInit {
    ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(CORNERS[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn roster(count: PlayerCount, participating: &[SeatId]) -> Vec<RoundActorInit> {
    count
        .seats()
        .iter()
        .rev()
        .map(|&seat| RoundActorInit {
            actor: init(seat),
            participating: participating.contains(&seat),
        })
        .collect()
}

fn rules(items: EnabledItems) -> RulesConfig {
    RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(items))
}

fn round_state(count: PlayerCount, participating: &[SeatId]) -> State {
    State::new_round(
        rules(EnabledItems::EMPTY),
        World::new(arena(count)),
        roster(count, participating),
        Tick(7),
        RoundAssignment::first(),
        None,
    )
    .unwrap()
}

#[test]
fn shuffled_three_and_four_seat_rosters_apply_participation_by_seat() {
    for (count, participating) in [
        (PlayerCount::Three, vec![SeatId::One, SeatId::Three]),
        (PlayerCount::Four, vec![SeatId::Two, SeatId::Four]),
    ] {
        let game = round_state(count, &participating);
        assert_eq!(game.actors.len(), count.get());
        for &seat in count.seats() {
            let actor = &game.actors[seat.index()];
            assert_eq!(actor.seat, seat);
            assert_eq!(actor.alive, participating.contains(&seat));
            if !actor.alive {
                assert_eq!(actor.held, HeldDirections::default());
                assert_eq!(actor.inventory, Inventory::default());
                assert_eq!(actor.effects, crate::game::TimedEffects::default());
                assert!(actor.bomb_grace.is_empty());
                assert!(actor.soft_wall_grace.is_empty());
                assert!(actor.unlit_oil_grace.is_empty());
            }
        }
    }
}

#[test]
fn insufficient_zero_or_one_participants_is_rejected_after_roster_validation() {
    for participating in [vec![], vec![SeatId::Three]] {
        assert_eq!(
            State::new_round(
                rules(EnabledItems::EMPTY),
                World::new(arena(PlayerCount::Three)),
                roster(PlayerCount::Three, &participating),
                Tick(0),
                RoundAssignment::first(),
                None,
            ),
            Err(StateError::InsufficientParticipants(participating.len()))
        );
    }
}

#[test]
fn inactive_contact_mine_grace_is_explicitly_cleared_after_common_initialization() {
    let mut world = World::new(arena(PlayerCount::Three));
    let mine = Bomb::new(
        EntityId(90),
        CORNERS[2],
        EntityId(12),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    world.bombs.insert(mine.id(), mine);
    let game = State::new_round(
        rules(EnabledItems::LANDMINE),
        world,
        roster(PlayerCount::Three, &[SeatId::One, SeatId::Two]),
        Tick(0),
        RoundAssignment::first(),
        None,
    )
    .unwrap();
    assert!(!game.actors[2].alive);
    assert!(game.actors[2].bomb_grace.is_empty());
}

#[test]
fn all_true_legacy_round_constructor_equals_the_existing_constructor() {
    let actor_inits: Vec<_> = PlayerCount::Two
        .seats()
        .iter()
        .map(|&seat| init(seat))
        .collect();
    let old = State::new_at_tick_with_assignment(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT),
        World::new(arena(PlayerCount::Two)),
        actor_inits.clone(),
        Tick(9),
        RoundAssignment::first(),
    )
    .unwrap();
    let new = State::new_round(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT),
        World::new(arena(PlayerCount::Two)),
        actor_inits
            .into_iter()
            .map(|actor| RoundActorInit {
                actor,
                participating: true,
            })
            .collect::<Vec<_>>(),
        Tick(9),
        RoundAssignment::first(),
        None,
    )
    .unwrap();
    assert_eq!(new, old);
}

#[test]
fn departure_batch_can_leave_two_players_running_then_choose_one_winner() {
    let mut game = round_state(PlayerCount::Four, &SeatId::ALL);
    let events = game.eliminate_departed([true, true, false, false]).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(game.result, None);
    assert_eq!(game.actors.iter().filter(|actor| actor.alive).count(), 2);
    let after_first = game.clone();
    assert!(
        game.eliminate_departed([true, true, false, false])
            .unwrap()
            .is_empty()
    );
    assert_eq!(game, after_first);

    let events = game
        .eliminate_departed([false, false, true, false])
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(game.result, Some(RoundResult::Winner(SeatId::Four)));
    assert!(matches!(
        events.last().unwrap().kind,
        StepEventKind::RoundEnded(RoundResult::Winner(SeatId::Four))
    ));
}

#[test]
fn all_living_departures_are_one_canonical_batch_and_end_in_a_draw() {
    let mut game = round_state(PlayerCount::Three, &SeatId::ALL[..3]);
    let events = game.eliminate_departed([true, true, true, false]).unwrap();
    assert_eq!(events.len(), 4);
    assert_eq!(
        game.result,
        Some(RoundResult::Draw(DrawReason::MutualElimination))
    );
    assert_eq!(
        events
            .iter()
            .map(|event| event.tick)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([Tick(7)])
    );
    assert!(
        events
            .iter()
            .all(|event| event.audience == EventAudience::all(PlayerCount::Three))
    );
}

#[test]
fn departure_bypasses_protection_clears_control_state_and_retains_bombs_and_bomb_grace() {
    let items =
        EnabledItems::from_bits(EnabledItems::HEART.bits() | EnabledItems::SHIELD.bits()).unwrap();
    let mut game = State::new_round(
        rules(items),
        World::new(arena(PlayerCount::Three)),
        roster(PlayerCount::Three, &SeatId::ALL[..3]),
        Tick(0),
        RoundAssignment::first(),
        None,
    )
    .unwrap();
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    game.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(0));
    game.actors[0].held.press(Direction::East);
    game.step(&[super::SeatInput {
        seat: SeatId::One,
        action: super::InputAction::PlaceBomb,
    }])
    .unwrap();
    let bomb_id = game
        .world
        .bombs
        .values()
        .find(|bomb| bomb.owner() == game.actors[0].id)
        .unwrap()
        .id();
    assert!(game.actors[0].bomb_grace.contains(&bomb_id));

    game.eliminate_departed([true, false, false, false])
        .unwrap();

    let actor = &game.actors[0];
    assert!(!actor.alive);
    assert_eq!(actor.inventory, Inventory::default());
    assert_eq!(actor.effects, crate::game::TimedEffects::default());
    assert_eq!(actor.held, HeldDirections::default());
    assert!(actor.bomb_grace.contains(&bomb_id));
    assert!(game.world.bombs.contains_key(&bomb_id));
}

#[test]
fn invalid_tail_is_atomic_and_completed_or_repeated_elimination_freezes_state() {
    let mut game = round_state(PlayerCount::Three, &SeatId::ALL[..3]);
    let before = game.clone();
    assert_eq!(
        game.eliminate_departed([true, false, false, true]),
        Err(StepError::SeatNotParticipating(SeatId::Four))
    );
    assert_eq!(game, before);

    game.eliminate_departed([true, true, false, false]).unwrap();
    assert!(game.result.is_some());
    let completed = game.clone();
    assert!(
        game.eliminate_departed([false, false, true, false])
            .unwrap()
            .is_empty()
    );
    assert_eq!(game, completed);
}

#[test]
fn elimination_does_not_advance_clock_updates_world_or_seeded_stream_and_events_merge() {
    let mut game = State::new_round(
        rules(EnabledItems::TELEPORTER),
        World::new(arena(PlayerCount::Four)),
        roster(PlayerCount::Four, &SeatId::ALL),
        Tick(4),
        RoundAssignment::first(),
        Some(SimulationSeed::new([4; 32])),
    )
    .unwrap();
    let tick = game.tick;
    let updates = game.running_updates;
    let world = game.world.clone();
    let randomness = game.randomness.clone();
    let mut report = game.step(&[]).unwrap();
    let completed_tick = game.tick;
    let completed_updates = game.running_updates;
    let events = game
        .eliminate_departed([true, false, false, false])
        .unwrap();
    report.events.extend(events);

    assert_eq!(tick, Tick(4));
    assert_eq!(updates, 0);
    assert_eq!(game.tick, completed_tick);
    assert_eq!(game.running_updates, completed_updates);
    assert_eq!(game.world, world);
    assert_eq!(game.randomness, randomness);
    assert!(report.events.iter().all(|event| event.tick == report.tick));
}
