use super::*;

#[test]
fn placement_precedes_range_collection_and_later_bombs_use_the_upgrade() {
    let mut state = v2_state(
        world_with_pickups([(TARGET, PickupKind::BlastRange)]),
        Position::center(TARGET),
        safe_two(),
    );
    let place = SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    };
    state.actors[0].capacity = 3;
    add_test_bomb(&mut state, EntityId(30), CellId::new(8, 5), EntityId(10)).unwrap();
    state.step(&[place]).unwrap();
    let first = *state
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.cell() == TARGET)
        .unwrap();
    assert_eq!(first.cell(), TARGET);
    assert_eq!(first.range(), 2);
    assert_eq!(state.world().bombs().get(&EntityId(30)).unwrap().range(), 2);
    assert_eq!(state.actors()[0].bomb_range(), 3);
    assert!(state.world().exposed_pickups().is_empty());
    for update in 0..20 {
        let inputs = if update == 0 {
            vec![SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            }]
        } else {
            Vec::new()
        };
        state.step(&inputs).unwrap();
    }
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            place,
        ])
        .unwrap();
    let bombs = state.world().bombs();
    assert_eq!(bombs.len(), 3);
    assert_eq!(bombs.get(&EntityId(30)).unwrap().range(), 2);
    assert_eq!(bombs.get(&first.id()).unwrap().range(), 2);
    assert_eq!(
        bombs
            .values()
            .find(|bomb| bomb.cell() == CellId::new(6, 5))
            .unwrap()
            .range(),
        3
    );
}

#[test]
fn capacity_upgrade_is_available_only_to_the_following_placement_phase() {
    let mut state = v2_state(
        world_with_pickups([(TARGET, PickupKind::BombCapacity)]),
        Position::center(TARGET),
        safe_two(),
    );
    add_test_bomb(&mut state, EntityId(30), CellId::new(8, 5), EntityId(10)).unwrap();
    let place = SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    };
    state.step(&[place]).unwrap();
    assert_eq!(state.world().bombs().len(), 1);
    assert_eq!(state.actors()[0].bomb_capacity(), 2);
    state.step(&[place]).unwrap();
    assert_eq!(state.world().bombs().len(), 2);
}

#[test]
fn speed_upgrade_preserves_residual_and_changes_movement_on_the_next_tick() {
    let start = Position::center(TARGET);
    let mut state = v2_state(
        world_with_pickups([(TARGET, PickupKind::MovementSpeed)]),
        start,
        safe_two(),
    );
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.actors()[0].position().x - start.x, 51);
    assert_eq!(state.actors()[0].speed(), SpeedLevel::One);
    assert_eq!(state.actors[0].residual, 12);
    let before = state.actors()[0].position().x;
    state.step(&[]).unwrap();
    assert_eq!(state.actors()[0].position().x - before, 59);
    assert_eq!(state.actors[0].residual, 56);
}
