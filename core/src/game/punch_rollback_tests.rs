use std::collections::{BTreeMap, BTreeSet};

use super::super::{ActorInit, InputAction, SeatInput, State, StepError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment,
    PickupKind, Position, RulesConfig, SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

fn arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

#[test]
fn late_pickup_overflow_rolls_back_punch_cell_history_cooldown_motion_and_grace() {
    let pickup_cell = CellId::new(1, 1);
    let mut world = World::with_pickups(
        arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup_cell, PickupKind::Shield)]),
    )
    .unwrap();
    let mut bomb = Bomb::new(
        EntityId(9),
        CellId::new(2, 1),
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    bomb.restore_kick_history(
        Some(crate::game::BombMotion {
            direction: Direction::North,
            countdown: 5,
        }),
        true,
    );
    world.bombs.insert(bomb.id(), bomb);
    let enabled = EnabledItems::from_bits(
        EnabledItems::PUNCH.bits()
            | EnabledItems::KICK.bits()
            | EnabledItems::REMOTE_BOMB.bits()
            | EnabledItems::SHIELD.bits(),
    )
    .unwrap();
    let rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled));
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(pickup_cell),
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
    ];
    let mut state = State::new_at_tick(rules, world, actors, Tick(u64::MAX - 300)).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.actors[0].bomb_grace.insert(EntityId(9));
    let before = state.clone();
    let input = [SeatInput {
        seat: SeatId::One,
        action: InputAction::UseEquipment,
    }];

    let mut control = before.clone();
    control.world.pickups.exposed.clear();
    control.step(&input).unwrap();
    let punched = control.world.bombs[&EntityId(9)];
    assert_eq!(punched.cell(), CellId::new(5, 1));
    assert_eq!(punched.motion(), None);
    assert!(punched.has_been_kicked());
    assert!(punched.has_been_punched());
    assert_eq!(
        control.actors[0].last_successful_punch_tick(),
        Some(Tick(u64::MAX - 299))
    );

    assert_eq!(
        state.step(&input),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(state, before);
}
