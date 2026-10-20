use std::collections::BTreeMap;

use crate::game::{
    ActorInit, BlastKind, BombTrigger, Direction, EffectKind, EnabledItems, EntityId, InputAction,
    PickupKind, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State,
    StepError, Tick, World, fixed_arena,
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

fn arm(game: &mut State, seat: SeatId) {
    game.actors[seat.index()]
        .inventory
        .replace_weapon(SpecialWeapon::Piercing);
}

#[test]
fn pickup_charge_is_consumed_only_by_successful_manual_placement() {
    let mut game = state(
        EnabledItems::PIERCING_BOMB,
        Some(PickupKind::PiercingBomb),
        Tick(0),
    );
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
    game.step(&plant()).unwrap();
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert_eq!(
        game.world.bombs.values().next().unwrap().blast(),
        BlastKind::Piercing
    );

    game.world.bombs.clear();
    game.step(&plant()).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().blast(),
        BlastKind::Standard
    );
}

#[test]
fn failed_manual_placement_retains_charge_and_collection_follows_placement() {
    let mut blocked = state(
        EnabledItems::PIERCING_BOMB,
        Some(PickupKind::PiercingBomb),
        Tick(0),
    );
    blocked.step(&[]).unwrap();
    blocked.step(&plant()).unwrap();
    assert_eq!(blocked.actors[0].inventory.weapon(), None);
    blocked.world.pickups.exposed.insert(
        blocked.actors[0].position.cell().unwrap(),
        crate::game::Pickup::new(PickupKind::PiercingBomb),
    );
    blocked.step(&plant()).unwrap();
    assert_eq!(
        blocked.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
    assert_eq!(blocked.world.bombs.len(), 1);

    blocked.step(&plant()).unwrap();
    assert_eq!(
        blocked.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
    assert_eq!(blocked.world.bombs.len(), 1);

    blocked.world.bombs.clear();
    blocked.step(&plant()).unwrap();
    assert_eq!(
        blocked.world.bombs.values().next().unwrap().blast(),
        BlastKind::Piercing
    );
}

#[test]
fn diarrhea_auto_bombs_never_consume_or_capture_piercing() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::PIERCING_BOMB.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut game = state(enabled, None, Tick(0));
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Piercing);
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
        Some(SpecialWeapon::Piercing)
    );
}

#[test]
fn manual_piercing_and_automatic_diarrhea_conflict_retain_the_charge() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::PIERCING_BOMB.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut game = state(enabled, None, Tick(0));
    game.actors[1].position = game.actors[0].position;
    arm(&mut game, SeatId::One);
    game.actors[1]
        .effects
        .apply(EffectKind::Diarrhea, Tick(0))
        .unwrap();

    game.step(&plant()).unwrap();

    assert!(game.world.bombs.is_empty());
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
}

#[test]
fn piercing_bombs_capture_the_same_fuse_and_range_modifiers_as_ordinary_bombs() {
    let first_items = EnabledItems::from_bits(
        EnabledItems::PIERCING_BOMB.bits()
            | EnabledItems::GOLDEN_FLAME.bits()
            | EnabledItems::SHORT_FUSE.bits(),
    )
    .unwrap();
    let mut first = state(first_items, None, Tick(0));
    arm(&mut first, SeatId::One);
    first.actors[0]
        .effects
        .apply(EffectKind::GoldenFlame, Tick(0))
        .unwrap();
    first.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(0))
        .unwrap();
    first.step(&plant()).unwrap();
    let bomb = first.world.bombs.values().next().unwrap();
    assert_eq!((bomb.blast(), bomb.range()), (BlastKind::Piercing, 8));
    assert_eq!(
        bomb.trigger(),
        BombTrigger::Timed {
            detonates_at: Tick(91)
        }
    );

    let second_items = EnabledItems::from_bits(
        EnabledItems::PIERCING_BOMB.bits()
            | EnabledItems::SHORT_FLAME.bits()
            | EnabledItems::LONG_FUSE.bits(),
    )
    .unwrap();
    let mut second = state(second_items, None, Tick(0));
    arm(&mut second, SeatId::One);
    second.actors[0]
        .effects
        .apply(EffectKind::ShortFlame, Tick(0))
        .unwrap();
    second.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(0))
        .unwrap();
    second.step(&plant()).unwrap();
    let bomb = second.world.bombs.values().next().unwrap();
    assert_eq!((bomb.blast(), bomb.range()), (BlastKind::Piercing, 1));
    assert_eq!(
        bomb.trigger(),
        BombTrigger::Timed {
            detonates_at: Tick(361)
        }
    );
}

#[test]
fn overflow_after_piercing_placement_rolls_back_charge_bomb_and_pickup() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::PIERCING_BOMB.bits() | EnabledItems::SHIELD.bits())
            .unwrap();
    let mut game = state(enabled, Some(PickupKind::Shield), Tick(u64::MAX - 300));
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Piercing);
    let before = game.clone();
    assert_eq!(
        game.step(&plant()),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(game, before);
}

#[test]
fn shared_cell_conflict_and_id_exhaustion_retain_every_manual_charge() {
    let mut shared = state(EnabledItems::PIERCING_BOMB, None, Tick(0));
    shared.actors[1].position = shared.actors[0].position;
    arm(&mut shared, SeatId::One);
    arm(&mut shared, SeatId::Two);
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
            .all(|actor| { actor.inventory.weapon() == Some(SpecialWeapon::Piercing) })
    );

    let mut exhausted = state(EnabledItems::PIERCING_BOMB, None, Tick(0));
    arm(&mut exhausted, SeatId::One);
    exhausted.next_entity_id = u64::from(u32::MAX) + 1;
    exhausted.world.bombs.insert(
        EntityId(u32::MAX),
        crate::game::Bomb::new(
            EntityId(u32::MAX),
            crate::game::CellId::new(10, 6),
            exhausted.actors[1].id,
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(180),
            },
        )
        .unwrap(),
    );
    exhausted.step(&plant()).unwrap();
    assert_eq!(
        exhausted.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Piercing)
    );
    assert_eq!(exhausted.world.bombs.len(), 1);
}
