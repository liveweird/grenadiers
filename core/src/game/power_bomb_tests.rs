use std::collections::BTreeMap;

use crate::game::{
    ActorInit, BlastKind, BombTrigger, Direction, EffectKind, EnabledItems, EntityId, InputAction,
    PickupKind, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State,
    StepError, Tick, World, fixed_arena, generate_expanded_arena,
};
use crate::room::RoomSettings;

fn state(enabled: EnabledItems, pickup: Option<PickupKind>, tick: Tick) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            pickup
                .map(|kind| BTreeMap::from([(spawns[0], kind)]))
                .unwrap_or_default(),
        )
        .unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        tick,
    )
    .unwrap()
}

fn plant() -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }]
}

fn arm(game: &mut State, seat: SeatId, weapon: SpecialWeapon) {
    game.actors[seat.index()].inventory.replace_weapon(weapon);
}

#[test]
fn pickup_replaces_weapon_and_only_successful_manual_placement_consumes_power() {
    let both = EnabledItems::from_bits(
        EnabledItems::PIERCING_BOMB.bits() | EnabledItems::POWER_BOMB.bits(),
    )
    .unwrap();
    let mut game = state(both, Some(PickupKind::PowerBomb), Tick(0));
    arm(&mut game, SeatId::One, SpecialWeapon::Piercing);
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );

    game.world.pickups.exposed.insert(
        game.actors[0].position.cell().unwrap(),
        crate::game::Pickup::new(PickupKind::PowerBomb),
    );
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );

    game.step(&plant()).unwrap();
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert_eq!(
        game.world.bombs.values().next().unwrap().blast(),
        BlastKind::Power
    );

    game.world.bombs.clear();
    game.step(&plant()).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().blast(),
        BlastKind::Standard
    );
}

#[test]
fn placement_precedes_collection_and_failed_attempts_retain_power() {
    let mut game = state(
        EnabledItems::POWER_BOMB,
        Some(PickupKind::PowerBomb),
        Tick(0),
    );
    arm(&mut game, SeatId::One, SpecialWeapon::Power);
    game.step(&plant()).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().blast(),
        BlastKind::Power
    );
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );

    game.step(&plant()).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
    assert_eq!(game.world.bombs.len(), 1);

    game.world.bombs.clear();
    game.actors[0].capacity = 0;
    game.step(&plant()).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
    assert!(game.world.bombs.is_empty());
}

#[test]
fn automatic_bombs_are_standard_and_never_consume_power() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::POWER_BOMB.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut game = state(enabled, None, Tick(0));
    arm(&mut game, SeatId::One, SpecialWeapon::Power);
    game.actors[0]
        .effects
        .apply(EffectKind::Diarrhea, Tick(0))
        .unwrap();
    game.step(&[]).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().blast(),
        BlastKind::Standard
    );
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
}

#[test]
fn manual_and_automatic_same_cell_conflict_has_no_fallback_or_consumption() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::POWER_BOMB.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut game = state(enabled, None, Tick(0));
    game.actors[1].position = game.actors[0].position;
    arm(&mut game, SeatId::One, SpecialWeapon::Power);
    game.actors[1]
        .effects
        .apply(EffectKind::Diarrhea, Tick(0))
        .unwrap();
    game.step(&plant()).unwrap();
    assert!(game.world.bombs.is_empty());
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
}

#[test]
fn power_captures_range_fuse_and_kind_across_later_inventory_changes() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::POWER_BOMB.bits()
            | EnabledItems::PIERCING_BOMB.bits()
            | EnabledItems::GOLDEN_FLAME.bits()
            | EnabledItems::SHORT_FUSE.bits(),
    )
    .unwrap();
    let mut game = state(enabled, None, Tick(0));
    arm(&mut game, SeatId::One, SpecialWeapon::Power);
    game.actors[0]
        .effects
        .apply(EffectKind::GoldenFlame, Tick(0))
        .unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(0))
        .unwrap();
    game.step(&plant()).unwrap();
    let bomb = *game.world.bombs.values().next().unwrap();
    assert_eq!((bomb.blast(), bomb.range()), (BlastKind::Power, 8));
    assert_eq!(
        bomb.trigger(),
        BombTrigger::Timed {
            detonates_at: Tick(91)
        }
    );

    arm(&mut game, SeatId::One, SpecialWeapon::Piercing);
    game.actors[0].effects.clear();
    assert_eq!(game.world.bombs.get(&bomb.id()), Some(&bomb));
}

#[test]
fn short_flame_and_long_fuse_shape_power_without_mutating_it_later() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::POWER_BOMB.bits()
            | EnabledItems::SHORT_FLAME.bits()
            | EnabledItems::LONG_FUSE.bits(),
    )
    .unwrap();
    let mut game = state(enabled, None, Tick(0));
    arm(&mut game, SeatId::One, SpecialWeapon::Power);
    game.actors[0]
        .effects
        .apply(EffectKind::ShortFlame, Tick(0))
        .unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(0))
        .unwrap();
    game.step(&plant()).unwrap();
    let bomb = game.world.bombs.values().next().unwrap();
    assert_eq!((bomb.blast(), bomb.range()), (BlastKind::Power, 1));
    assert_eq!(
        bomb.trigger(),
        BombTrigger::Timed {
            detonates_at: Tick(361)
        }
    );
}

#[test]
fn overflow_after_power_placement_rolls_back_every_mutation() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::POWER_BOMB.bits() | EnabledItems::SHIELD.bits())
            .unwrap();
    let mut game = state(enabled, Some(PickupKind::Shield), Tick(u64::MAX - 300));
    arm(&mut game, SeatId::One, SpecialWeapon::Power);
    let before = game.clone();
    assert_eq!(
        game.step(&plant()),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(game, before);
}

#[test]
fn shared_cell_id_and_deadline_failures_retain_power_charges() {
    let mut shared = state(EnabledItems::POWER_BOMB, None, Tick(0));
    shared.actors[1].position = shared.actors[0].position;
    arm(&mut shared, SeatId::One, SpecialWeapon::Power);
    arm(&mut shared, SeatId::Two, SpecialWeapon::Power);
    shared
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    assert!(shared.world.bombs.is_empty());
    assert!(
        shared
            .actors
            .iter()
            .all(|actor| { actor.inventory.weapon() == Some(SpecialWeapon::Power) })
    );

    let mut exhausted = state(EnabledItems::POWER_BOMB, None, Tick(0));
    arm(&mut exhausted, SeatId::One, SpecialWeapon::Power);
    exhausted.next_entity_id = u64::from(u32::MAX) + 1;
    exhausted.step(&plant()).unwrap();
    assert!(exhausted.world.bombs.is_empty());
    assert_eq!(
        exhausted.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );

    let mut deadline = state(EnabledItems::POWER_BOMB, None, Tick(u64::MAX - 1));
    arm(&mut deadline, SeatId::One, SpecialWeapon::Power);
    deadline.step(&plant()).unwrap();
    assert!(deadline.world.bombs.is_empty());
    assert_eq!(
        deadline.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
}

#[test]
fn canceled_and_duplicate_requests_get_one_attempt_and_retain_on_failure() {
    let mut canceled = state(EnabledItems::POWER_BOMB, None, Tick(0));
    arm(&mut canceled, SeatId::One, SpecialWeapon::Power);
    canceled
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
        ])
        .unwrap();
    assert!(canceled.world.bombs.is_empty());
    assert_eq!(
        canceled.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );

    canceled.actors[0].capacity = 1;
    canceled
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    assert_eq!(canceled.world.bombs.len(), 1);
    assert_eq!(canceled.actors[0].inventory.weapon(), None);
}

#[test]
fn seed_31_survives_the_ordinary_reveal_collect_plant_and_escape_route() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::POWER_BOMB);
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    let mut game = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::POWER_BOMB),
        ),
        World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap();

    game.actors[0].position = Position::center(crate::game::CellId::new(2, 1));
    game.step(&plant()).unwrap();
    game.actors[0].position = Position::center(crate::game::CellId::new(1, 2));
    while game.world.bombs.len() == 1 {
        game.step(&[]).unwrap();
    }
    while game
        .world
        .hazards
        .contains_key(&crate::game::CellId::new(2, 1))
    {
        game.step(&[]).unwrap();
    }
    game.actors[0].position = Position::center(crate::game::CellId::new(3, 1));
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
    game.actors[0].position = Position::center(crate::game::CellId::new(1, 2));
    game.step(&plant()).unwrap();
    let power = game.world.bombs.values().next().unwrap();
    assert_eq!(
        (power.cell(), power.range(), power.blast()),
        (crate::game::CellId::new(1, 2), 2, BlastKind::Power)
    );
    game.actors[0].position = Position::center(crate::game::CellId::new(3, 1));
    while game.world.bombs.len() == 1 {
        game.step(&[]).unwrap();
    }
    assert!(game.actors[0].alive());
    assert!(
        game.world
            .hazards
            .contains_key(&crate::game::CellId::new(3, 2))
    );
    assert!(
        !game
            .world
            .hazards
            .contains_key(&crate::game::CellId::new(2, 2))
    );
}
