use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, Position, RulesConfig,
    SeatId, SpeedLevel, StepEventKind, Tick, World,
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

fn state(one: CellId, two: CellId, enabled: EnabledItems) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena()),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(one),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(two),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn equipment_padding_cannot_defeat_whole_update_shared_cell_placement_fairness() {
    let cell = CellId::new(5, 5);
    let mut state = state(cell, cell, EnabledItems::PUNCH);
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);

    state
        .step(&[
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::Two, InputAction::PlaceBomb),
            input(SeatId::One, InputAction::PlaceBomb),
        ])
        .unwrap();

    assert!(state.world.bombs.is_empty());
}

#[test]
fn an_earlier_wave_placement_can_be_punched_by_a_later_wave() {
    let mut state = state(
        CellId::new(1, 1),
        CellId::new(2, 1),
        EnabledItems::PUNCH,
    );
    state.replace_actor_equipment(SeatId::Two, Equipment::Punch);

    state
        .step(&[
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::Two, InputAction::UseEquipment),
            input(SeatId::Two, InputAction::UseEquipment),
        ])
        .unwrap();

    let bomb = state.world.bombs.values().next().unwrap();
    assert_eq!(bomb.cell(), CellId::new(11, 1));
    assert!(bomb.has_been_punched());
}

#[test]
fn later_wave_cannot_rescue_a_bomb_punched_onto_existing_fire() {
    let mut state = state(
        CellId::new(1, 5),
        CellId::new(4, 5),
        EnabledItems::PUNCH,
    );
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.replace_actor_equipment(SeatId::Two, Equipment::Punch);
    state.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(2, 5),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap(),
    );
    state.world.hazards.insert(CellId::new(5, 5), Tick(10));

    let report = state
        .step(&[
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::Two, InputAction::UseEquipment),
            input(SeatId::Two, InputAction::UseEquipment),
        ])
        .unwrap();

    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
        .unwrap();
    assert!(matches!(
        event.kind,
        StepEventKind::BombDetonated(view) if view.cell == CellId::new(5, 5)
    ));
    assert!(event.bomb_has_been_punched);
}

#[test]
fn only_first_bomb_command_has_gameplay_effect_but_failed_equipment_can_retry() {
    let mut state = state(
        CellId::new(1, 1),
        CellId::new(13, 11),
        EnabledItems::PUNCH,
    );
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(2, 1),
            EntityId(2),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap(),
    );

    state
        .step(&[
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::One, InputAction::UseEquipment),
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::One, InputAction::PlaceBomb),
        ])
        .unwrap();

    assert_eq!(state.live_bomb_count(SeatId::One), 1);
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(5, 1));
}
