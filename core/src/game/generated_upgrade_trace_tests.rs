use super::{
    ActorInit, CellId, Direction, EntityId, InputAction, PickupKind, Position, RulesConfig, SeatId,
    SeatInput, SpeedLevel, State, World, generate_arena,
};
fn step(state: &mut State, inputs: &[SeatInput]) {
    state.step(inputs).unwrap();
    assert!(state.result().is_none(), "route must stay alive");
}
fn input(action: InputAction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action,
    }
}
fn move_cells(state: &mut State, direction: Direction, cells: usize) {
    step(
        state,
        &[
            input(InputAction::Clear),
            input(InputAction::Press(direction)),
        ],
    );
    for _ in 1..cells * 20 {
        step(state, &[]);
    }
    step(state, &[input(InputAction::Clear)]);
}
fn plant(state: &mut State) -> EntityId {
    step(
        state,
        &[input(InputAction::Clear), input(InputAction::PlaceBomb)],
    );
    *state.world().bombs().keys().next_back().unwrap()
}
fn wait_clear(state: &mut State) {
    for _ in 0..240 {
        if state.world().bombs().is_empty() && state.world().hazards().is_empty() {
            return;
        }
        step(state, &[]);
    }
    panic!("hazards failed to clear");
}
#[test]
fn public_fixture_reveals_collects_and_preserves_captured_bomb_ranges() {
    let text = include_str!("../../tests/fixtures/arena-v1-pickups.seed");
    let mut seed = [0_u8; 32];
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    let generated = generate_arena(seed);
    assert_eq!(generated.record().version(), 1);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::BombCapacity)
    );
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(1, 3)),
        Some(&PickupKind::BlastRange)
    );
    let spawns = generated.arena().spawns();
    let (world, _record) = World::from_generated(generated).unwrap();
    let actors = SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let mut state = State::new(RulesConfig::classic_v2(), world, actors).unwrap();
    move_cells(&mut state, Direction::East, 1);
    plant(&mut state);
    move_cells(&mut state, Direction::West, 1);
    move_cells(&mut state, Direction::South, 1);
    wait_clear(&mut state);
    move_cells(&mut state, Direction::North, 1);
    move_cells(&mut state, Direction::East, 2);
    assert_eq!(state.actors()[0].bomb_capacity(), 2);
    move_cells(&mut state, Direction::West, 2);
    move_cells(&mut state, Direction::South, 1);
    plant(&mut state);
    move_cells(&mut state, Direction::North, 1);
    move_cells(&mut state, Direction::East, 1);
    wait_clear(&mut state);
    assert!(
        state
            .world()
            .exposed_pickups()
            .contains_key(&CellId::new(1, 3))
    );
    let old = plant(&mut state);
    move_cells(&mut state, Direction::West, 1);
    move_cells(&mut state, Direction::South, 2);
    assert_eq!(state.actors()[0].bomb_range(), 3);
    let new = plant(&mut state);
    assert_ne!(old, new);
    assert_eq!(state.world().bombs().get(&old).unwrap().range(), 2);
    assert_eq!(state.world().bombs().get(&new).unwrap().range(), 3);
    assert_eq!(state.live_bomb_count(SeatId::One), 2);
    assert_eq!(state.tick().0, 631);
}
