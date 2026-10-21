use std::collections::{BTreeMap, BTreeSet};

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::{BlastKind, Bomb, BombMotion, BombTrigger};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, PickupKind, Position, RulesConfig,
    SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

fn arena(blocks: BTreeSet<CellId>) -> Arena {
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
        blocks,
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn enabled(extra: EnabledItems) -> EnabledItems {
    EnabledItems::from_bits(EnabledItems::PUNCH.bits() | extra.bits()).unwrap()
}

fn state_at(one: Position, two: Position, tick: Tick, extra: EnabledItems) -> State {
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(enabled(extra)),
        ),
        World::new(arena(BTreeSet::new())),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: one,
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: two,
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
        tick,
    )
    .unwrap()
}

fn action(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn timed(id: u32, cell: CellId, owner: u32, placed_at: Tick, detonates_at: Tick) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        EntityId(owner),
        3,
        placed_at,
        BombTrigger::Timed { detonates_at },
    )
    .unwrap()
}

fn equip_punch(state: &mut State, seat: SeatId) {
    state.replace_actor_equipment(seat, Equipment::Punch);
}

#[test]
fn punch_moves_three_cells_preserving_identity_timing_blast_and_kick_history() {
    let source = CellId::new(5, 5);
    let mut state = state_at(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
        Tick(20),
        EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::POWER_BOMB.bits())
            .unwrap(),
    );
    let mut bomb = Bomb::new_with_blast(
        EntityId(9),
        source,
        EntityId(2),
        7,
        Tick(3),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
        BlastKind::Power,
    )
    .unwrap();
    bomb.restore_kick_history(
        Some(BombMotion {
            direction: Direction::North,
            countdown: 4,
        }),
        true,
    );
    state.world.bombs.insert(bomb.id(), bomb);
    state.actors[0].bomb_grace.insert(bomb.id());
    state.actors[1].bomb_grace.insert(bomb.id());
    equip_punch(&mut state, SeatId::One);

    state
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();

    let moved = state.world.bombs[&EntityId(9)];
    assert_eq!(moved.cell(), CellId::new(8, 5));
    assert_eq!(moved.id(), EntityId(9));
    assert_eq!(moved.owner(), EntityId(2));
    assert_eq!(moved.range(), 7);
    assert_eq!(moved.placed_at(), Tick(3));
    assert_eq!(moved.detonates_at(), Some(Tick(100)));
    assert_eq!(moved.blast(), BlastKind::Power);
    assert_eq!(moved.motion(), None);
    assert!(moved.has_been_kicked());
    assert!(moved.has_been_punched());
    assert!(state.actors.iter().all(|actor| !actor.bomb_grace.contains(&EntityId(9))));
    assert_eq!(state.actors[0].last_successful_punch_tick(), Some(Tick(21)));
}

#[test]
fn cooldown_blocks_through_eleven_elapsed_ticks_and_failure_does_not_start_it() {
    let mut state = state_at(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
        Tick(0),
        EnabledItems::EMPTY,
    );
    equip_punch(&mut state, SeatId::One);
    state
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert_eq!(state.actors[0].last_successful_punch_tick(), None);

    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, CellId::new(5, 5), 2, Tick(0), Tick(100)));
    state
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
    state.actors[0].heading = Direction::West;
    state.actors[0].position = Position::center(CellId::new(9, 5));
    for _ in 0..11 {
        state
            .step(&[action(SeatId::One, InputAction::UseEquipment)])
            .unwrap();
    }
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
    state
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 5));
}

#[test]
fn search_wraps_by_three_skips_blockers_and_accepts_tangent_actor() {
    let source = CellId::new(12, 5);
    let mut state = state_at(
        Position::center(CellId::new(11, 5)),
        Position::center(CellId::new(13, 8)),
        Tick(0),
        EnabledItems::EMPTY,
    );
    state.world.arena = arena(BTreeSet::from([CellId::new(2, 5)]));
    state
        .world
        .bombs
        .insert(EntityId(9), timed(9, source, 2, Tick(0), Tick(100)));
    state.world.bombs.insert(
        EntityId(10),
        timed(10, CellId::new(5, 5), 2, Tick(0), Tick(100)),
    );
    let tangent = Position::center(CellId::new(8, 5));
    state.actors[1].position = Position {
        x: tangent.x + crate::game::types::UNITS_PER_CELL / 2
            + crate::game::types::HALF_FOOTPRINT,
        y: tangent.y,
    };
    equip_punch(&mut state, SeatId::One);

    state
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
}

#[test]
fn same_bomb_uses_round_priority_while_distinct_same_destination_rejects_both() {
    let mut same = state_at(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(6, 5)),
        Tick(0),
        EnabledItems::EMPTY,
    );
    same.actors[0].heading = Direction::East;
    same.actors[1].heading = Direction::West;
    same.world
        .bombs
        .insert(EntityId(9), timed(9, CellId::new(5, 5), 1, Tick(0), Tick(100)));
    equip_punch(&mut same, SeatId::One);
    equip_punch(&mut same, SeatId::Two);
    same.step(&[
        action(SeatId::One, InputAction::UseEquipment),
        action(SeatId::Two, InputAction::UseEquipment),
    ])
    .unwrap();
    assert_eq!(same.world.bombs[&EntityId(9)].cell(), CellId::new(8, 5));
    assert_eq!(same.actors[0].last_successful_punch_tick(), Some(Tick(1)));
    assert_eq!(same.actors[1].last_successful_punch_tick(), None);

    let mut distinct = state_at(
        Position::center(CellId::new(1, 5)),
        Position::center(CellId::new(9, 5)),
        Tick(0),
        EnabledItems::EMPTY,
    );
    distinct.actors[0].heading = Direction::East;
    distinct.actors[1].heading = Direction::West;
    distinct.world.bombs.insert(
        EntityId(9),
        timed(9, CellId::new(2, 5), 1, Tick(0), Tick(100)),
    );
    distinct.world.bombs.insert(
        EntityId(10),
        timed(10, CellId::new(8, 5), 2, Tick(0), Tick(100)),
    );
    equip_punch(&mut distinct, SeatId::One);
    equip_punch(&mut distinct, SeatId::Two);
    distinct
        .step(&[
            action(SeatId::One, InputAction::UseEquipment),
            action(SeatId::Two, InputAction::UseEquipment),
        ])
        .unwrap();
    assert_eq!(distinct.world.bombs[&EntityId(9)].cell(), CellId::new(2, 5));
    assert_eq!(distinct.world.bombs[&EntityId(10)].cell(), CellId::new(8, 5));
}

#[test]
fn remote_reservation_follows_wave_chronology() {
    let items = EnabledItems::REMOTE_BOMB;
    let mut reserved = state_at(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
        Tick(0),
        items,
    );
    reserved.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(5, 5),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Remote,
        )
        .unwrap(),
    );
    equip_punch(&mut reserved, SeatId::One);
    let report = reserved
        .step(&[
            action(SeatId::One, InputAction::PlaceBomb),
            action(SeatId::One, InputAction::UseEquipment),
        ])
        .unwrap();
    assert!(!reserved.world.bombs.contains_key(&EntityId(9)));
    assert!(report.events.iter().any(|event| !event.bomb_has_been_punched));

    let mut moved = state_at(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
        Tick(0),
        items,
    );
    moved.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(5, 5),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Remote,
        )
        .unwrap(),
    );
    equip_punch(&mut moved, SeatId::One);
    let report = moved
        .step(&[
            action(SeatId::One, InputAction::UseEquipment),
            action(SeatId::One, InputAction::PlaceBomb),
        ])
        .unwrap();
    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, crate::game::StepEventKind::BombDetonated(_)))
        .unwrap();
    assert!(event.bomb_has_been_punched);
}

#[test]
fn active_source_fire_due_timing_contact_and_actor_overlap_all_reject() {
    for trigger in [
        BombTrigger::Timed {
            detonates_at: Tick(1),
        },
        BombTrigger::Contact,
    ] {
        let extra = if matches!(trigger, BombTrigger::Contact) {
            EnabledItems::LANDMINE
        } else {
            EnabledItems::EMPTY
        };
        let mut state = state_at(
            Position::center(CellId::new(4, 5)),
            Position::center(CellId::new(13, 11)),
            Tick(0),
            extra,
        );
        state.world.bombs.insert(
            EntityId(9),
            Bomb::new(EntityId(9), CellId::new(5, 5), EntityId(2), 2, Tick(0), trigger)
                .unwrap(),
        );
        equip_punch(&mut state, SeatId::One);
        state
            .step(&[action(SeatId::One, InputAction::UseEquipment)])
            .unwrap();
        assert!(
            !state.world.bombs.contains_key(&EntityId(9))
                || state.world.bombs[&EntityId(9)].cell() == CellId::new(5, 5)
        );
    }

    let mut burning = state_at(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(13, 11)),
        Tick(0),
        EnabledItems::EMPTY,
    );
    burning.world.bombs.insert(
        EntityId(9),
        timed(9, CellId::new(5, 5), 2, Tick(0), Tick(100)),
    );
    burning.world.hazards.insert(CellId::new(5, 5), Tick(10));
    equip_punch(&mut burning, SeatId::One);
    burning
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert!(!burning.world.bombs.contains_key(&EntityId(9)));
}

#[test]
fn punch_pickup_replaces_equipment_after_actions() {
    let cell = CellId::new(4, 5);
    let mut state = state_at(
        Position::center(cell),
        Position::center(CellId::new(13, 11)),
        Tick(0),
        EnabledItems::KICK,
    );
    state.world.pickups.exposed = BTreeMap::from([(
        cell,
        crate::game::Pickup::new(PickupKind::Punch),
    )]);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state.world.bombs.insert(
        EntityId(9),
        timed(9, CellId::new(5, 5), 2, Tick(0), Tick(100)),
    );

    state
        .step(&[action(SeatId::One, InputAction::UseEquipment)])
        .unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 5));
    assert_eq!(state.actors[0].inventory().equipment(), Some(Equipment::Punch));
}

