use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId,
    InputAction, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    StepError, StepEventKind, Tick, World, fixed_arena,
};

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn state(construction: Tick, next: Tick, effect_at: Tick) -> State {
    let mut world = World::with_pickups(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(1, 1), PickupKind::Shield)]),
    )
    .unwrap();
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(7, 5),
            EntityId(2),
            1,
            construction,
            BombTrigger::Timed {
                detonates_at: Tick(next.0 + 9),
            },
        )
        .unwrap(),
    );
    let enabled = EnabledItems::from_bits(
        EnabledItems::OIL_BARREL.bits()
            | EnabledItems::STOPWATCH.bits()
            | EnabledItems::SHIELD.bits(),
    )
    .unwrap();
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        world,
        actors(),
        construction,
    )
    .unwrap();
    state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, effect_at);
    state
}

fn place() -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }]
}

#[test]
fn late_pickup_overflow_rolls_back_real_allocator_and_timed_pause_mutations() {
    let next = Tick(u64::MAX - 299);
    let construction = Tick(next.0 - 1);
    let mut overflow = state(construction, next, Tick(next.0 - 179));
    let before = overflow.clone();

    assert_eq!(
        overflow.step(&place()),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(overflow, before);

    let mut control = state(Tick(0), Tick(1), Tick(0));
    let report = control.step(&place()).unwrap();
    let shifted = control.world.bombs.get(&EntityId(9)).unwrap();
    assert_eq!(shifted.detonates_at(), Some(Tick(11)));
    assert_eq!(shifted.timed_history().unwrap().paused_ticks, 1);
    let placed = report
        .events
        .iter()
        .find_map(|event| match event.kind {
            StepEventKind::BombPlaced(view) if view.owner == EntityId(1) => Some(view),
            _ => None,
        })
        .expect("positive control allocates the requested bomb");
    assert_eq!(placed.id, EntityId(10));
    assert_eq!(placed.placed_at, Tick(1));
    assert_eq!(
        placed.trigger,
        BombTrigger::Timed {
            detonates_at: Tick(181)
        }
    );
    assert_eq!(control.next_entity_id, 11);
    assert!(control.actors[0].effects.damage_immune(Tick(1)));
}
