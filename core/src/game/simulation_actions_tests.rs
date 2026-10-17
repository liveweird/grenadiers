use super::ManualAction;
use crate::game::types::{EntityId, Position};
use crate::game::{
    ActorInit, Direction, InputAction, RulesConfig, SeatId, SeatInput, SpeedLevel, State, World,
    fixed_arena,
};

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
