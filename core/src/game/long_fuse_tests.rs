use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    CellId, Direction, EffectKind, EnabledItems, EntityId, PickupKind, Position, RulesConfig,
    SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;
use crate::room::settings::MAX_TURN_ASSIST_UNITS;

const TARGET: CellId = CellId::new(5, 5);

fn state(enabled: EnabledItems, pickup: bool, fuse: u64) -> State {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        if pickup {
            BTreeMap::from([(TARGET, PickupKind::LongFuse)])
        } else {
            BTreeMap::new()
        },
    )
    .unwrap();
    let settings = RoomSettings::new(SpeedLevel::Base, MAX_TURN_ASSIST_UNITS, 3, fuse)
        .unwrap()
        .with_enabled_items(enabled);
    State::new(
        RulesConfig::expanded_v5(settings),
        world,
        [
            ActorInit {
                id: EntityId(10),
                seat: SeatId::One,
                position: Position::center(TARGET),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(20),
                seat: SeatId::Two,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap()
}

fn place() -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }
}

#[test]
fn collection_applies_after_placement_and_expiry_applies_before_placement() {
    let mut game = state(EnabledItems::LONG_FUSE, true, 120);
    game.step(&[place()]).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(121))
    );
    assert!(
        game.actors[0]
            .effects
            .active(EffectKind::LongFuse, Tick(600))
    );
    game.world.bombs.clear();
    game.actors[0].bomb_grace.clear();
    game.tick = Tick(600);
    game.step(&[place()]).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(721))
    );
}

#[test]
fn long_and_short_fuses_cancel_then_expire_independently_without_retiming_bombs() {
    let mut game = state(EnabledItems::ALL, false, 180);
    game.actors[0].capacity = 3;
    game.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(0))
        .unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(100))
        .unwrap();
    game.step(&[place()]).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(181))
    );
    game.world.bombs.clear();
    game.actors[0].bomb_grace.clear();
    game.tick = Tick(600);
    game.step(&[place()]).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(691))
    );
    game.tick = Tick(700);
    assert_eq!(
        game.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(691))
    );

    let mut reverse = state(EnabledItems::ALL, false, 180);
    reverse.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(0))
        .unwrap();
    reverse.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(100))
        .unwrap();
    reverse.tick = Tick(600);
    reverse.step(&[place()]).unwrap();
    assert_eq!(
        reverse.world.bombs.values().next().unwrap().detonates_at(),
        Some(Tick(961))
    );
}

#[test]
fn repeated_long_fuse_refreshes_exactly_and_both_collection_orders_cancel() {
    for (first, second) in [
        (EffectKind::ShortFuse, EffectKind::LongFuse),
        (EffectKind::LongFuse, EffectKind::ShortFuse),
    ] {
        let mut game = state(EnabledItems::ALL, false, 180);
        game.actors[0].effects.apply(first, Tick(0)).unwrap();
        game.actors[0].effects.apply(second, Tick(20)).unwrap();
        assert_eq!(
            game.actors[0].effects.bomb_fuse_ticks(180, Tick(20)),
            Some(180)
        );
        if second == EffectKind::LongFuse {
            game.actors[0]
                .effects
                .apply(EffectKind::LongFuse, Tick(30))
                .unwrap();
            assert!(
                game.actors[0]
                    .effects
                    .active(EffectKind::LongFuse, Tick(629))
            );
            assert!(
                !game.actors[0]
                    .effects
                    .active(EffectKind::LongFuse, Tick(630))
            );
        }
    }
}

#[path = "reverse_controls_tests.rs"]
mod reverse_controls_tests;
