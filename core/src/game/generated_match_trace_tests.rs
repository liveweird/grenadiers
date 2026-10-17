use super::{
    ActorInit, CellId, Direction, DrawReason, EntityId, InputAction, Position, RoundResult,
    RulesConfig, SeatId, SeatInput, SpeedLevel, State, World, generate_arena,
};

fn input(action: InputAction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action,
    }
}

#[test]
fn match_fixture_has_a_walkable_corridor_and_one_bomb_can_draw_both_players() {
    let text = include_str!("../../tests/fixtures/arena-v1-matches.seed");
    let mut seed = [0_u8; 32];
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    let generated = generate_arena(seed);
    assert!(!generated.record().fallback_used());
    let spawns = generated.arena().spawns();
    let (world, _) = World::from_generated(generated).unwrap();
    let actors = SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let mut state = State::new(RulesConfig::classic_v2(), world, actors).unwrap();
    for (direction, cells, target) in [
        (Direction::East, 4, CellId::new(5, 1)),
        (Direction::South, 4, CellId::new(5, 5)),
        (Direction::East, 2, CellId::new(7, 5)),
        (Direction::South, 2, CellId::new(7, 7)),
        (Direction::East, 2, CellId::new(9, 7)),
        (Direction::South, 2, CellId::new(9, 9)),
        (Direction::East, 2, CellId::new(11, 9)),
        (Direction::South, 2, CellId::new(11, 11)),
        (Direction::East, 2, CellId::new(13, 11)),
    ] {
        state
            .step(&[
                input(InputAction::Clear),
                input(InputAction::Press(direction)),
            ])
            .unwrap();
        for _ in 1..cells * 20 {
            state.step(&[]).unwrap();
        }
        state.step(&[input(InputAction::Clear)]).unwrap();
        assert_eq!(state.actors()[0].position(), Position::center(target));
        assert!(state.result().is_none());
    }
    assert_eq!(state.actors()[0].position(), state.actors()[1].position());
    state.step(&[input(InputAction::PlaceBomb)]).unwrap();
    assert_eq!(state.world().bombs().len(), 1);
    let detonation = state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap()
        .detonates_at()
        .unwrap();
    while state.tick().0 < detonation.0 {
        assert!(state.actors().iter().all(|actor| actor.alive()));
        state.step(&[]).unwrap();
    }
    assert_eq!(
        state.result(),
        Some(RoundResult::Draw(DrawReason::MutualElimination))
    );
    assert!(state.actors().iter().all(|actor| !actor.alive()));
}
