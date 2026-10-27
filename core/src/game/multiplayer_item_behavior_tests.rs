use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::effects::GOLDEN_FLAME_RANGE;
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::{
    BlastKind, CellId, Direction, EnabledItems, EntityId, Equipment, OilState, Pickup, PickupKind,
    PlayerCount, Position, RulesConfig, SeatId, SpecialWeapon, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

const CORNERS: [CellId; 4] = [
    CellId::new(1, 1),
    CellId::new(13, 1),
    CellId::new(1, 11),
    CellId::new(13, 11),
];

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, CORNERS).unwrap()
}

fn actors() -> Vec<ActorInit> {
    PlayerCount::Four
        .seats()
        .iter()
        .map(|&seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(CORNERS[seat.index()]),
            heading: if matches!(seat, SeatId::One | SeatId::Three) {
                Direction::East
            } else {
                Direction::West
            },
            speed: SpeedLevel::Base,
        })
        .collect()
}

fn enabled(bits: &[EnabledItems]) -> EnabledItems {
    EnabledItems::from_bits(bits.iter().fold(0, |mask, item| mask | item.bits())).unwrap()
}

fn rules(items: EnabledItems) -> RulesConfig {
    RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(items))
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn seat_three_collects_wall_pass_and_crosses_a_live_soft_block() {
    let start = CellId::new(5, 5);
    let block = CellId::new(6, 5);
    let world = World::with_pickups(
        arena(BTreeSet::from([block])),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(start, PickupKind::WallPass)]),
    )
    .unwrap();
    let mut actor_inits = actors();
    actor_inits[SeatId::Three.index()].position = Position::center(start);
    let mut game = State::new(rules(EnabledItems::WALL_PASS), world, actor_inits).unwrap();

    game.step(&[]).unwrap();
    assert_eq!(
        game.actor(SeatId::Three).unwrap().inventory().equipment(),
        Some(Equipment::WallPass)
    );
    game.step(&[input(SeatId::Three, InputAction::Press(Direction::East))])
        .unwrap();
    for _ in 1..20 {
        game.step(&[]).unwrap();
    }

    assert_eq!(
        game.actor(SeatId::Three).unwrap().position(),
        Position::center(block)
    );
    assert!(game.world().arena().blocks().contains(&block));
}

#[test]
fn higher_seat_weapon_pickups_capture_piercing_and_power_blast_metadata() {
    let items = enabled(&[EnabledItems::PIERCING_BOMB, EnabledItems::POWER_BOMB]);
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (CORNERS[2], PickupKind::PiercingBomb),
            (CORNERS[3], PickupKind::PowerBomb),
        ]),
    )
    .unwrap();
    let mut game = State::new(rules(items), world, actors()).unwrap();

    game.step(&[]).unwrap();
    assert_eq!(
        game.actor(SeatId::Three).unwrap().inventory().weapon(),
        Some(SpecialWeapon::Piercing)
    );
    assert_eq!(
        game.actor(SeatId::Four).unwrap().inventory().weapon(),
        Some(SpecialWeapon::Power)
    );
    game.step(&[
        input(SeatId::Three, InputAction::PlaceBomb),
        input(SeatId::Four, InputAction::PlaceBomb),
    ])
    .unwrap();

    let blast_for = |owner| {
        game.world()
            .bombs()
            .values()
            .find(|bomb| bomb.owner() == owner)
            .unwrap()
            .blast()
    };
    assert_eq!(blast_for(EntityId(12)), BlastKind::Piercing);
    assert_eq!(blast_for(EntityId(13)), BlastKind::Power);
    assert_eq!(
        game.actor(SeatId::Three).unwrap().inventory().weapon(),
        None
    );
    assert_eq!(game.actor(SeatId::Four).unwrap().inventory().weapon(), None);
}

#[test]
fn seat_four_contacts_solid_oil_and_an_adjacent_blast_ignites_it() {
    let start = CellId::new(5, 5);
    let oil = CellId::new(6, 5);
    let world = World::with_pickups_and_oil(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Unlit)]),
    )
    .unwrap();
    let mut actor_inits = actors();
    actor_inits[SeatId::Four.index()].position = Position::center(start);
    let mut game = State::new(rules(EnabledItems::OIL_BARREL), world, actor_inits).unwrap();

    game.step(&[input(SeatId::Four, InputAction::Press(Direction::East))])
        .unwrap();
    for _ in 1..20 {
        game.step(&[]).unwrap();
    }
    assert!(game.actor(SeatId::Four).unwrap().position().x < Position::center(oil).x);
    assert_eq!(game.world.oil.state(oil), Some(OilState::Unlit));

    let at = game.tick();
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(90),
        start,
        EntityId(10),
        1,
        at,
        Tick(at.0 + 1),
    )
    .unwrap();
    game.step(&[]).unwrap();
    assert_eq!(game.world.oil.state(oil), Some(OilState::Burning));
}

#[test]
fn higher_seat_effect_pickups_change_motion_fuse_and_range() {
    let items = enabled(&[
        EnabledItems::REVERSE_CONTROLS,
        EnabledItems::SLOW_FOOT,
        EnabledItems::SHORT_FUSE,
        EnabledItems::GOLDEN_FLAME,
    ]);
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (CORNERS[2], PickupKind::ReverseControls),
            (CORNERS[3], PickupKind::SlowFoot),
        ]),
    )
    .unwrap();
    let mut game = State::new(rules(items), world, actors()).unwrap();

    game.step(&[
        input(SeatId::Three, InputAction::Press(Direction::East)),
        input(SeatId::Four, InputAction::Press(Direction::West)),
    ])
    .unwrap();
    let three_before = game.actor(SeatId::Three).unwrap().position().x;
    let four_before = game.actor(SeatId::Four).unwrap().position().x;
    game.step(&[]).unwrap();
    assert!(game.actor(SeatId::Three).unwrap().position().x < three_before);
    assert_eq!(
        four_before - game.actor(SeatId::Four).unwrap().position().x,
        25
    );

    let three_cell = game
        .actor(SeatId::Three)
        .unwrap()
        .position()
        .cell()
        .unwrap();
    let four_cell = game.actor(SeatId::Four).unwrap().position().cell().unwrap();
    game.world
        .pickups
        .exposed
        .insert(three_cell, Pickup::new(PickupKind::ShortFuse));
    game.world
        .pickups
        .exposed
        .insert(four_cell, Pickup::new(PickupKind::GoldenFlame));
    game.step(&[
        input(SeatId::Three, InputAction::Clear),
        input(SeatId::Four, InputAction::Clear),
    ])
    .unwrap();
    let placement_tick = Tick(game.tick().0 + 1);
    game.step(&[
        input(SeatId::Three, InputAction::PlaceBomb),
        input(SeatId::Four, InputAction::PlaceBomb),
    ])
    .unwrap();

    let bomb_for = |owner| {
        game.world()
            .bombs()
            .values()
            .find(|bomb| bomb.owner() == owner)
            .copied()
            .unwrap()
    };
    let short = bomb_for(EntityId(12));
    let golden = bomb_for(EntityId(13));
    assert_eq!(short.detonates_at(), Some(Tick(placement_tick.0 + 90)));
    assert_eq!(golden.range(), GOLDEN_FLAME_RANGE);
}
