use std::collections::BTreeSet;
use std::num::NonZeroU32;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, Position,
    RoundAssignment, RulesConfig, SeatId, SpeedLevel, StepEventKind, Tick, World,
};
use crate::room::RoomSettings;

fn arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
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

fn state(one: CellId, two: CellId, round: u32) -> State {
    State::new_at_tick_with_assignment(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL)),
        World::new(arena()),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(one),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(two),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(0),
        RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn timed(id: u32, cell: CellId, owner: EntityId) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        owner,
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap()
}

#[test]
fn other_seat_remote_trigger_reservation_beats_same_wave_punch_in_both_round_parities() {
    for round in [1, 2] {
        for reversed_inputs in [false, true] {
            let mut state = state(CellId::new(1, 1), CellId::new(6, 5), round);
            let remote = Bomb::new(
                EntityId(9),
                CellId::new(5, 5),
                EntityId(1),
                2,
                Tick(0),
                BombTrigger::Remote,
            )
            .unwrap();
            state.world.bombs.insert(remote.id(), remote);
            state.replace_actor_equipment(SeatId::Two, Equipment::Punch);
            let mut inputs = [
                input(SeatId::One, InputAction::PlaceBomb),
                input(SeatId::Two, InputAction::UseEquipment),
            ];
            if reversed_inputs {
                inputs.reverse();
            }

            let report = state.step(&inputs).unwrap();

            assert!(!state.world.bombs.contains_key(&EntityId(9)));
            let event = report
                .events
                .iter()
                .find(|event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(9)))
                .unwrap();
            assert!(matches!(
                event.kind,
                StepEventKind::BombDetonated(view) if view.cell == CellId::new(5, 5)
            ));
            assert!(!event.bomb_has_been_punched);
            assert_eq!(state.actors[1].last_successful_punch_tick(), None);
        }
    }
}

#[test]
fn unfrozen_remote_first_bomb_suppresses_diarrhea_after_triggering() {
    let mut state = state(CellId::new(1, 1), CellId::new(13, 11), 1);
    let remote = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    state.world.bombs.insert(remote.id(), remote);
    state.apply_test_effect(SeatId::One, EffectKind::Diarrhea, Tick(0));

    let report = state
        .step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();

    assert!(state.world.bombs.is_empty());
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(event.kind, StepEventKind::BombPlaced(_)))
            .count(),
        0
    );
}

#[test]
fn failed_first_bomb_suppresses_diarrhea_and_later_bomb_does_not_retry_after_punch() {
    let mut state = state(CellId::new(5, 5), CellId::new(6, 5), 1);
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, CellId::new(5, 5), EntityId(2)));
    state.replace_actor_equipment(SeatId::Two, Equipment::Punch);
    state.apply_test_effect(SeatId::One, EffectKind::Diarrhea, Tick(0));

    let report = state
        .step(&[
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::Two, InputAction::PlaceBomb),
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::Two, InputAction::UseEquipment),
        ])
        .unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(2, 5));
    assert!(
        state
            .world
            .bombs
            .values()
            .all(|bomb| bomb.cell() != CellId::new(5, 5))
    );
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(event.kind, StepEventKind::BombPlaced(view) if view.owner == EntityId(1)))
            .count(),
        0
    );
}

#[test]
fn earlier_punch_clears_occupancy_for_a_later_first_placement() {
    let mut state = state(CellId::new(5, 5), CellId::new(6, 5), 1);
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, CellId::new(5, 5), EntityId(2)));
    state.replace_actor_equipment(SeatId::Two, Equipment::Punch);

    state
        .step(&[
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::Two, InputAction::UseEquipment),
            input(SeatId::One, InputAction::PlaceBomb),
        ])
        .unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(2, 5));
    assert!(
        state
            .world
            .bombs
            .values()
            .any(|bomb| bomb.owner() == EntityId(1) && bomb.cell() == CellId::new(5, 5))
    );
}

#[test]
fn clear_cancels_mixed_equipment_and_bomb_actions_before_following_actions() {
    let mut cleared = state(CellId::new(4, 5), CellId::new(13, 11), 1);
    cleared
        .world
        .bombs
        .insert(EntityId(9), timed(9, CellId::new(5, 5), EntityId(2)));
    cleared.replace_actor_equipment(SeatId::One, Equipment::Punch);
    cleared
        .step(&[
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::One, InputAction::Clear),
        ])
        .unwrap();
    assert_eq!(cleared.world.bombs[&EntityId(9)].cell(), CellId::new(5, 5));
    assert_eq!(cleared.live_bomb_count(SeatId::One), 0);

    let mut retained = state(CellId::new(4, 5), CellId::new(13, 11), 1);
    retained
        .world
        .bombs
        .insert(EntityId(9), timed(9, CellId::new(5, 5), EntityId(2)));
    retained.replace_actor_equipment(SeatId::One, Equipment::Punch);
    retained
        .step(&[
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::One, InputAction::Clear),
            input(SeatId::One, InputAction::UseEquipment),
        ])
        .unwrap();
    assert_eq!(retained.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
    assert_eq!(retained.live_bomb_count(SeatId::One), 0);
}
