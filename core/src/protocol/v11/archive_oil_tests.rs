use crate::game::{
    self, ActorInit, EnabledItems, EntityId as GameEntityId, Position as GamePosition, RulesConfig,
    SeatId, SpeedLevel, State, World,
};
use crate::protocol::{Cell, Counter};
use crate::room::RoomSettings;

use super::oil_tests::room;
use super::*;
use crate::protocol::v8::projection_facts::{OilFacts, OilPhase};

type ArchiveBinder = fn(
    &RoomView,
    Counter,
    &ExpandedCapture,
    Option<game::GenerationRecord>,
) -> Result<BoundPublication, ProjectionError>;

fn neutral_capture() -> ExpandedCapture {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT),
        World::new(arena),
        SeatId::DUEL.map(|seat| ActorInit {
            id: GameEntityId(seat.index() as u32 + 1),
            seat,
            position: GamePosition::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap();
    capture_expanded(&state, &[]).unwrap()
}

fn v9_items() -> ItemSet {
    ItemSet::new([
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::Stopwatch,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
        ItemId::ReverseControls,
    ])
    .unwrap()
}

fn archive_rejects_each_private_oil_fact(supported: ItemSet, seed_byte: u8, bind: ArchiveBinder) {
    let mut capture = neutral_capture();
    capture.world.0.available_items = supported;
    let mut archived_room = room(ItemSet::empty(), capture.world.0.physical_tick);
    archived_room.available_items = supported;
    let generation = game::generate_expanded_arena([seed_byte; 32], EnabledItems::EMPTY).record();

    assert!(capture.world.0.oil.is_empty());
    assert!(capture.world.0.unlit_oil_grace.iter().all(Vec::is_empty));
    assert!(
        capture
            .world
            .0
            .actors
            .iter()
            .all(|actor| actor.inventory.equipment.is_none())
    );
    assert!(capture.world.0.last_teleport_tick.is_none());
    assert!(
        capture
            .world
            .0
            .last_successful_punch_ticks
            .iter()
            .all(Option::is_none)
    );
    assert!(
        capture
            .world
            .0
            .last_successful_shove_ticks
            .iter()
            .all(Option::is_none)
    );
    assert!(
        bind(
            &archived_room,
            archived_room.round_id,
            &capture,
            Some(generation)
        )
        .is_ok()
    );

    for (cell, phase) in [
        (Cell::new(5, 1).unwrap(), OilPhase::Concealed),
        (Cell::new(3, 1).unwrap(), OilPhase::Pending),
        (Cell::new(4, 1).unwrap(), OilPhase::Unlit),
        (Cell::new(3, 3).unwrap(), OilPhase::Burning),
    ] {
        let mut private = capture.clone();
        private.world.0.oil = vec![OilFacts { cell, phase }];
        assert_eq!(
            bind(
                &archived_room,
                archived_room.round_id,
                &private,
                Some(generation)
            ),
            Err(ProjectionError::InvalidExpandedBinding),
            "archive accepted private Oil phase {phase:?}"
        );
    }

    let mut no_longer_holds_wall_pass = capture;
    let occupied = no_longer_holds_wall_pass.world.0.actors[0].position;
    let occupied = Cell::new(
        u8::try_from(occupied.x() / 1_024).unwrap(),
        u8::try_from(occupied.y() / 1_024).unwrap(),
    )
    .unwrap();
    no_longer_holds_wall_pass.world.0.oil = vec![OilFacts {
        cell: occupied,
        phase: OilPhase::Unlit,
    }];
    no_longer_holds_wall_pass.world.0.unlit_oil_grace[0] = vec![occupied];
    assert!(
        no_longer_holds_wall_pass.world.0.actors[0]
            .inventory
            .equipment
            .is_none()
    );
    assert_eq!(
        bind(
            &archived_room,
            archived_room.round_id,
            &no_longer_holds_wall_pass,
            Some(generation)
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn v8_and_v9_reject_each_private_oil_phase_and_typed_grace_after_neutral_control() {
    archive_rejects_each_private_oil_fact(
        crate::protocol::v8::projection_binding::expanded_items(),
        8,
        crate::protocol::v8::bind_expanded,
    );
    archive_rejects_each_private_oil_fact(v9_items(), 9, crate::protocol::v9::bind_expanded);
}
