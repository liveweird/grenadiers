use std::collections::BTreeSet;
use std::num::NonZeroU32;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, Position,
    RoundAssignment, RulesConfig, SeatId, SpeedLevel, Tick, World,
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

fn bomb(id: u32, cell: CellId) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap()
}

fn use_punch(seat: SeatId) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::UseEquipment,
    }
}

#[test]
fn same_bomb_claims_use_each_round_priority_independent_of_input_order_and_id() {
    for (round, winner, landing) in [
        (1, SeatId::One, CellId::new(8, 5)),
        (2, SeatId::Two, CellId::new(2, 5)),
    ] {
        for reversed_inputs in [false, true] {
            for id in [9, 10] {
                let mut state = state(CellId::new(4, 5), CellId::new(6, 5), round);
                state
                    .world
                    .bombs
                    .insert(EntityId(id), bomb(id, CellId::new(5, 5)));
                state.replace_actor_equipment(SeatId::One, Equipment::Punch);
                state.replace_actor_equipment(SeatId::Two, Equipment::Punch);
                let mut inputs = [use_punch(SeatId::One), use_punch(SeatId::Two)];
                if reversed_inputs {
                    inputs.reverse();
                }

                state.step(&inputs).unwrap();

                assert_eq!(state.world.bombs[&EntityId(id)].cell(), landing);
                for seat in SeatId::DUEL {
                    assert_eq!(
                        state.actors[seat.index()].last_successful_punch_tick(),
                        (seat == winner).then_some(Tick(1))
                    );
                }
            }
        }
    }
}

#[test]
fn distinct_same_destination_claims_reject_both_for_input_and_bomb_id_permutations() {
    for reversed_inputs in [false, true] {
        for ids in [[9, 10], [10, 9]] {
            let mut state = state(CellId::new(1, 5), CellId::new(9, 5), 1);
            state
                .world
                .bombs
                .insert(EntityId(ids[0]), bomb(ids[0], CellId::new(2, 5)));
            state
                .world
                .bombs
                .insert(EntityId(ids[1]), bomb(ids[1], CellId::new(8, 5)));
            state.replace_actor_equipment(SeatId::One, Equipment::Punch);
            state.replace_actor_equipment(SeatId::Two, Equipment::Punch);
            let mut inputs = [use_punch(SeatId::One), use_punch(SeatId::Two)];
            if reversed_inputs {
                inputs.reverse();
            }

            state.step(&inputs).unwrap();

            assert_eq!(
                state.world.bombs[&EntityId(ids[0])].cell(),
                CellId::new(2, 5)
            );
            assert_eq!(
                state.world.bombs[&EntityId(ids[1])].cell(),
                CellId::new(8, 5)
            );
            assert!(
                state
                    .actors
                    .iter()
                    .all(|actor| actor.last_successful_punch_tick().is_none())
            );
        }
    }
}

#[test]
fn a_bomb_outside_the_authoritative_heading_is_not_a_target() {
    let mut state = state(CellId::new(4, 5), CellId::new(13, 11), 1);
    state
        .world
        .bombs
        .insert(EntityId(9), bomb(9, CellId::new(4, 4)));
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);

    state.step(&[use_punch(SeatId::One)]).unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(4, 4));
    assert_eq!(state.actors[0].last_successful_punch_tick(), None);
}

#[test]
fn living_landing_blockers_apply_through_status_and_equipment_variants() {
    for variant in ["frozen", "hidden", "shield", "heart", "wall-pass"] {
        let mut state = state(CellId::new(4, 5), CellId::new(8, 5), 1);
        state
            .world
            .bombs
            .insert(EntityId(9), bomb(9, CellId::new(5, 5)));
        state.replace_actor_equipment(SeatId::One, Equipment::Punch);
        match variant {
            "frozen" => state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0)),
            "hidden" => {
                state.apply_test_effect(SeatId::Two, EffectKind::InvisibilityCloak, Tick(0));
            }
            "shield" => state.apply_test_effect(SeatId::Two, EffectKind::Shield, Tick(0)),
            "heart" => state.replace_actor_equipment(SeatId::Two, Equipment::Heart),
            "wall-pass" => state.replace_actor_equipment(SeatId::Two, Equipment::WallPass),
            _ => unreachable!(),
        }

        state.step(&[use_punch(SeatId::One)]).unwrap();

        assert_eq!(
            state.world.bombs[&EntityId(9)].cell(),
            CellId::new(11, 5),
            "{variant} actor must block the first landing"
        );
        assert_eq!(state.actors[0].last_successful_punch_tick(), Some(Tick(1)));
    }
}
