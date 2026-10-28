use std::collections::BTreeMap;

use super::*;
use crate::game::{
    ActorInit, CellId, Direction, EnabledItems, EntityId, OilState, Position, RulesConfig, SeatId,
    SpeedLevel, State, World, fixed_arena,
};

fn oil_state() -> State {
    let oil = BTreeMap::from([
        (CellId::new(5, 1), OilState::Concealed),
        (CellId::new(3, 1), OilState::Pending),
        (CellId::new(10, 1), OilState::Unlit),
        (CellId::new(3, 3), OilState::Burning),
    ]);
    let world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        oil,
    )
    .unwrap();
    let actors = [
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
    ];
    State::new(
        RulesConfig::expanded_v5(
            crate::room::RoomSettings::DEFAULT.with_enabled_items(EnabledItems::OIL_BARREL),
        ),
        world,
        actors,
    )
    .unwrap()
}

#[test]
fn actual_oil_phases_capture_and_project_only_unlit_and_burning() {
    let state = oil_state();
    let capture = capture_expanded(&state, &[]).unwrap();
    assert_eq!(capture.world.oil.len(), 4);
    assert!(super::projection_oil::validate(&capture.world).is_ok());
    let visible: Vec<_> = capture
        .world
        .oil
        .iter()
        .filter_map(|oil| match oil.phase {
            super::projection_facts::OilPhase::Unlit => Some(OilBarrelState::Unlit),
            super::projection_facts::OilPhase::Burning => Some(OilBarrelState::Burning),
            super::projection_facts::OilPhase::Concealed
            | super::projection_facts::OilPhase::Pending => None,
        })
        .collect();
    assert_eq!(
        visible,
        vec![OilBarrelState::Unlit, OilBarrelState::Burning]
    );
}

#[test]
fn oil_phase_and_per_actor_grace_corruption_is_rejected() {
    let capture = capture_expanded(&oil_state(), &[]).unwrap();
    let mut concealed_on_floor = capture.world.clone();
    concealed_on_floor.oil[0].cell = crate::protocol::Cell::new(7, 7).unwrap();
    assert_eq!(
        super::projection_oil::validate(&concealed_on_floor),
        Err(ProjectionError::InvalidItems)
    );

    let unlit = capture
        .world
        .oil
        .iter()
        .find(|oil| oil.phase == super::projection_facts::OilPhase::Unlit)
        .unwrap()
        .cell;
    let mut grace = capture.world.clone();
    grace.actors[0].view.position = crate::protocol::Position::new(
        u16::from(unlit.column()) * 1_024 + 512,
        u16::from(unlit.row()) * 1_024 + 512,
    )
    .unwrap();
    grace.actors[0].unlit_oil_grace = vec![unlit];
    assert!(super::projection_oil::validate(&grace).is_ok());

    let mut missing = grace.clone();
    missing.actors[0].unlit_oil_grace.clear();
    assert_eq!(
        super::projection_oil::validate(&missing),
        Err(ProjectionError::InvalidActors)
    );

    let mut holder = grace;
    holder.actors[0].view.inventory.equipment = Some(EquipmentId::WallPass);
    assert_eq!(
        super::projection_oil::validate(&holder),
        Err(ProjectionError::InvalidActors)
    );

    let mut disabled_wall_pass = capture.world;
    let block = disabled_wall_pass.arena.blocks[0];
    disabled_wall_pass.actors[1].view.position = crate::protocol::Position::new(
        u16::from(block.column()) * 1_024 + 512,
        u16::from(block.row()) * 1_024 + 512,
    )
    .unwrap();
    disabled_wall_pass.actors[1].soft_wall_grace = vec![block];
    assert_eq!(
        super::projection_oil::validate(&disabled_wall_pass),
        Err(ProjectionError::InvalidActors)
    );
}
