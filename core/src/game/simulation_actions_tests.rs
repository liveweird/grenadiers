use std::collections::BTreeMap;

use super::{ManualAction, PlacementOrigin};
use crate::game::types::{EntityId, Position};
use crate::game::{
    ActorInit, CellId, Direction, EnabledItems, InputAction, OilState, RulesConfig, SeatId,
    SeatInput, SpeedLevel, State, World, fixed_arena,
};
use crate::room::RoomSettings;

fn state() -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    State::new(
        RulesConfig::classic_v1(),
        World::new(arena),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn oil_state() -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let oil = CellId::new(3, 1);
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::OIL_BARREL),
        ),
        World::with_pickups_and_oil(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(oil, OilState::Unlit)]),
        )
        .unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn applying_inputs_retains_every_manual_action_in_per_seat_order() {
    let mut state = state();
    let actions = state.apply_inputs(&[
        input(SeatId::One, InputAction::PlaceBomb),
        input(SeatId::Two, InputAction::PlaceBomb),
        input(SeatId::One, InputAction::PlaceBomb),
    ]);

    assert_eq!(
        actions.for_seat(SeatId::One).as_slice(),
        &[ManualAction::PlaceBomb, ManualAction::PlaceBomb]
    );
    assert_eq!(
        actions.for_seat(SeatId::Two).as_slice(),
        &[ManualAction::PlaceBomb]
    );
}

#[test]
fn clear_discards_only_earlier_manual_actions_for_its_seat() {
    let mut state = state();
    let actions = state.apply_inputs(&[
        input(SeatId::One, InputAction::PlaceBomb),
        input(SeatId::Two, InputAction::PlaceBomb),
        input(SeatId::One, InputAction::Clear),
        input(SeatId::One, InputAction::PlaceBomb),
        input(SeatId::Two, InputAction::PlaceBomb),
        input(SeatId::Two, InputAction::Clear),
    ]);

    assert_eq!(
        actions.for_seat(SeatId::One).as_slice(),
        &[ManualAction::PlaceBomb]
    );
    assert!(actions.for_seat(SeatId::Two).as_slice().is_empty());
}

#[test]
fn shared_placement_veto_excludes_unlit_centers_but_includes_free_centers() {
    let mut state = oil_state();
    let requests = [Some(PlacementOrigin::Manual); 2];
    let oil = CellId::new(3, 1);

    state.actors[0].position = Position::center(oil);
    state.actors[1].position = Position::center(oil);
    assert_eq!(state.shared_placement_veto(requests), [false; 2]);

    let free = Position::center(CellId::new(2, 1));
    state.actors[0].position = free;
    state.actors[1].position = free;
    assert_eq!(state.shared_placement_veto(requests), [true; 2]);

    let partial_overlap = Position { x: 2_753, y: 1_536 };
    assert_eq!(partial_overlap.cell(), Some(CellId::new(2, 1)));
    assert!(crate::game::movement::position_overlaps_cell(
        partial_overlap,
        oil
    ));
    state.actors[0].position = partial_overlap;
    state.actors[1].position = partial_overlap;
    assert_eq!(state.shared_placement_veto(requests), [true; 2]);
}
