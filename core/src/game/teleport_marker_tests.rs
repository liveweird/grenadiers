use super::{ActorInit, State};
use crate::game::{
    ArenaGenerationConfig, Direction, EnabledItems, EntityId, Pickup, PickupKind, PlayerCount,
    Position, RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, Tick, World,
    fixed_arena, generate_multiplayer_arena,
};
use crate::room::{BoardSize, RoomSettings};

fn game(count: PlayerCount, legacy: bool) -> State {
    let arena = if count == PlayerCount::Two {
        fixed_arena()
    } else {
        generate_multiplayer_arena(
            [0; 32],
            EnabledItems::EMPTY,
            ArenaGenerationConfig {
                player_count: count,
                board: BoardSize::FifteenByThirteen,
                assignment: RoundAssignment::first(),
            },
        )
        .unwrap()
        .arena()
        .clone()
    };
    let actors: Vec<_> = count
        .seats()
        .iter()
        .map(|&seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(arena.spawns()[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect();
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER);
    if legacy {
        State::new(
            RulesConfig::expanded_v5(settings),
            World::new(arena),
            actors,
        )
        .unwrap()
    } else {
        State::new_at_tick_with_assignment_and_seed(
            RulesConfig::multiplayer_v6(settings),
            World::new(arena),
            actors,
            Tick(0),
            RoundAssignment::first(),
            SimulationSeed::new([7; 32]),
        )
        .unwrap()
    }
}

fn collect(state: &mut State, seat: SeatId) {
    let cell = state.actor(seat).unwrap().position().cell().unwrap();
    state
        .world
        .pickups
        .exposed
        .insert(cell, Pickup::new(PickupKind::Teleporter));
    state.step(&[]).unwrap();
}

#[test]
fn teleport_markers_change_only_for_the_selected_pair_and_preserve_older_pairs() {
    for count in [PlayerCount::Three, PlayerCount::Four] {
        let mut state = game(count, false);
        collect(&mut state, SeatId::One);
        assert_eq!(state.last_teleport_tick(), Some(Tick(1)));
        assert_eq!(
            state
                .actors
                .iter()
                .filter(|actor| actor.last_teleport_tick() == Some(Tick(1)))
                .count(),
            2
        );
        let uninvolved = state
            .actors
            .iter()
            .find(|actor| actor.last_teleport_tick().is_none())
            .unwrap()
            .seat();
        let before: Vec<_> = state
            .actors
            .iter()
            .map(|actor| (actor.position(), actor.last_teleport_tick()))
            .collect();
        collect(&mut state, uninvolved);
        assert_eq!(state.last_teleport_tick(), Some(Tick(2)));
        let mut changed = 0;
        for (actor, (position, marker)) in state.actors.iter().zip(before) {
            if actor.position() != position {
                changed += 1;
                assert_eq!(actor.last_teleport_tick(), Some(Tick(2)));
            } else {
                assert_eq!(actor.last_teleport_tick(), marker);
            }
        }
        assert_eq!(changed, 2);
    }
}

#[test]
fn a_new_round_resets_actor_and_global_teleport_history() {
    let mut first = game(PlayerCount::Four, false);
    collect(&mut first, SeatId::One);
    let fresh = game(PlayerCount::Four, false);
    assert_eq!(fresh.last_teleport_tick(), None);
    assert!(
        fresh
            .actors
            .iter()
            .all(|actor| actor.last_teleport_tick().is_none())
    );
}

#[test]
fn archived_duel_global_marker_still_describes_both_participants() {
    let mut state = game(PlayerCount::Two, true);
    collect(&mut state, SeatId::One);
    assert_eq!(state.last_teleport_tick(), Some(Tick(1)));
    assert!(
        state
            .actors
            .iter()
            .all(|actor| actor.last_teleport_tick() == state.last_teleport_tick())
    );
    assert!(state.randomness.is_none());
}
