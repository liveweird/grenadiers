use std::collections::BTreeSet;
use std::num::NonZeroU32;

use super::{ActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    CellId, Direction, EffectKind, EnabledItems, EntityId, Pickup, PickupKind, PlayerCount,
    Position, RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, StateError,
    StepError, Tick, World,
};
use crate::room::RoomSettings;

const CELLS: [CellId; 4] = [
    CellId::new(2, 2), CellId::new(12, 2), CellId::new(2, 10), CellId::new(12, 10),
];
const SPAWNS: [CellId; 4] = [
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
        vec![SPAWNS[0], SPAWNS[3]]
    } else {
        SPAWNS[..count.get()].to_vec()
    };
    Arena::new(terrain, BTreeSet::new(), spawns).unwrap()
}

fn actors(count: PlayerCount) -> Vec<ActorInit> {
    count.seats().iter().enumerate().map(|(index, &seat)| ActorInit {
        id: EntityId(10 + index as u32), seat, position: Position::center(CELLS[index]),
        heading: Direction::East, speed: SpeedLevel::Base,
    }).collect()
}

fn state(count: PlayerCount, seed: [u8; 32], round: u32) -> State {
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::multiplayer_v6(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER),
        ),
        World::new(arena(count)),
        actors(count),
        Tick(0),
        RoundAssignment::new(NonZeroU32::new(round).unwrap()),
        SimulationSeed::new(seed),
    ).unwrap()
}

fn words(state: &State) -> u64 {
    state.randomness.as_ref().unwrap().words_consumed()
}

fn selected_target(before: &[Position], after: &State, requester: SeatId) -> SeatId {
    after.player_count.seats().iter().copied().find(|&seat| {
        seat != requester && after.actors[seat.index()].position == before[requester.index()]
    }).unwrap()
}

#[test]
fn v6_teleporter_requires_a_seed_even_for_two_players_and_redacts_it() {
    let rules = RulesConfig::multiplayer_v6(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER),
    );
    assert_eq!(
        State::new(rules, World::new(arena(PlayerCount::Two)), actors(PlayerCount::Two)),
        Err(StateError::MissingSimulationRandomness)
    );
    let game = state(PlayerCount::Two, [0x5a; 32], 1);
    let debug = format!("{game:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("5a5a5a5a"));
}

#[test]
fn fixed_seed_pins_target_sequence_and_clone_equality_tracks_stream_position() {
    let mut game = state(PlayerCount::Four, [0; 32], 1);
    let untouched = game.clone();
    assert_eq!(game, untouched);
    let mut targets = Vec::new();
    for tick in 1..=4 {
        let before: Vec<_> = game.actors.iter().map(|actor| actor.position).collect();
        game.commit_teleport([true, false, false, false], Tick(tick));
        targets.push(selected_target(&before, &game, SeatId::One));
    }
    assert_eq!(targets, [SeatId::Two, SeatId::Two, SeatId::Two, SeatId::Four]);
    assert_ne!(game, untouched);
    assert_eq!(game, game.clone());
    assert_eq!(words(&game), 4);
}

#[test]
fn canonical_living_candidates_include_cloaked_and_exclude_dead_actors() {
    let mut reached = BTreeSet::new();
    for byte in 0..64 {
        let mut game = state(PlayerCount::Four, [byte; 32], 1);
        game.apply_test_effect(SeatId::Three, EffectKind::InvisibilityCloak, Tick(0));
        game.actors[3].alive = false;
        let before: Vec<_> = game.actors.iter().map(|actor| actor.position).collect();
        game.commit_teleport([true, false, false, false], Tick(1));
        reached.insert(selected_target(&before, &game, SeatId::One));
    }
    assert_eq!(reached, BTreeSet::from([SeatId::Two, SeatId::Three]));
}

#[test]
fn seeded_target_choice_adds_no_event_or_audience_observation() {
    let mut baseline = None;
    let mut targets = BTreeSet::new();
    for byte in 0..32 {
        let mut game = state(PlayerCount::Three, [byte; 32], 1);
        game.apply_test_effect(SeatId::Three, EffectKind::InvisibilityCloak, Tick(0));
        game.world
            .pickups
            .exposed
            .insert(CELLS[0], Pickup::new(PickupKind::Teleporter));
        let before: Vec<_> = game.actors.iter().map(|actor| actor.position).collect();
        let report = game.step(&[]).unwrap();
        targets.insert(selected_target(&before, &game, SeatId::One));
        if let Some(expected) = &baseline {
            assert_eq!(&report.events, expected);
        } else {
            baseline = Some(report.events);
        }
    }
    assert_eq!(targets, BTreeSet::from([SeatId::Two, SeatId::Three]));
}

#[test]
fn simultaneous_requests_use_actual_round_priority_and_only_one_selection() {
    let mut game = state(PlayerCount::Four, [3; 32], 2);
    let before: Vec<_> = game.actors.iter().map(|actor| actor.position).collect();
    game.commit_teleport([true, true, true, true], Tick(1));
    assert_ne!(game.actors[SeatId::Two.index()].position, before[SeatId::Two.index()]);
    assert_eq!(game.actors[SeatId::One.index()].position, before[SeatId::One.index()]);
    assert_eq!(words(&game), 1);
}

#[test]
fn rejected_or_single_candidate_requests_do_not_draw_but_unsafe_choice_does_not_retry() {
    let mut none = state(PlayerCount::Four, [4; 32], 1);
    none.commit_teleport([false; 4], Tick(1));
    none.actors[0].alive = false;
    none.commit_teleport([true, false, false, false], Tick(2));
    assert_eq!(words(&none), 0);

    let mut single = state(PlayerCount::Four, [5; 32], 1);
    single.actors[2].alive = false;
    single.actors[3].alive = false;
    single.commit_teleport([true, false, false, false], Tick(1));
    assert_eq!(words(&single), 0);

    let mut control = state(PlayerCount::Four, [6; 32], 1);
    let before: Vec<_> = control.actors.iter().map(|actor| actor.position).collect();
    control.commit_teleport([true, false, false, false], Tick(1));
    let chosen = selected_target(&before, &control, SeatId::One);
    let mut unsafe_game = state(PlayerCount::Four, [6; 32], 1);
    unsafe_game.world.hazards.insert(CELLS[chosen.index()], Tick(10));
    let unchanged = unsafe_game.actors.iter().map(|actor| actor.position).collect::<Vec<_>>();
    unsafe_game.commit_teleport([true, false, false, false], Tick(1));
    assert_eq!(unsafe_game.actors.iter().map(|actor| actor.position).collect::<Vec<_>>(), unchanged);
    assert_eq!(unsafe_game.last_teleport_tick, None);
    assert_eq!(words(&unsafe_game), 1);
}

#[test]
fn late_step_error_rolls_back_seeded_stream_with_the_whole_state() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::TELEPORTER.bits() | EnabledItems::SHIELD.bits(),
    )
    .unwrap();
    let mut game = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena(PlayerCount::Three)),
        actors(PlayerCount::Three),
        Tick(u64::MAX - EffectKind::Shield.duration_ticks()),
        RoundAssignment::first(),
        SimulationSeed::new([9; 32]),
    )
    .unwrap();
    game.world
        .pickups
        .exposed
        .insert(CELLS[0], Pickup::new(PickupKind::Teleporter));
    game.world
        .pickups
        .exposed
        .insert(CELLS[1], Pickup::new(PickupKind::Shield));
    let before = game.clone();

    assert_eq!(
        game.step(&[]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(game, before);
    assert_eq!(words(&game), 0);
}
