use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, Direction, EnabledItems, EntityId as DomainEntityId, Position, RulesConfig,
    SeatId, SpeedLevel, State, World,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat, Tick};
use crate::room::RoomSettings;

use super::{
    ConnectionStatus, GameSettings, ItemId, ItemSet, MapId, RoomPhase, RoomView, RoundCount,
    RulesVersion, Scores, SeatStatus, Wins, capture_expanded, expanded_settings,
};

const RESERVED_ITEMS: [ItemId; 22] = [
    ItemId::Kick,
    ItemId::Punch,
    ItemId::Bracelet,
    ItemId::WallPass,
    ItemId::Heart,
    ItemId::RemoteBomb,
    ItemId::PiercingBomb,
    ItemId::PowerBomb,
    ItemId::Landmine,
    ItemId::Shield,
    ItemId::Stopwatch,
    ItemId::InvisibilityCloak,
    ItemId::GoldenFlame,
    ItemId::ShortFuse,
    ItemId::Diarrhea,
    ItemId::SlowFoot,
    ItemId::ShortFlame,
    ItemId::LongFuse,
    ItemId::ReverseControls,
    ItemId::InvisibleBombs,
    ItemId::OilBarrel,
    ItemId::Teleporter,
];

#[test]
fn archived_v8_settings_pin_the_cloak_catalog_across_every_reserved_item() {
    let supported = super::projection_binding::expanded_items();
    assert_eq!(supported.iter().count(), 14);
    assert_eq!(1_usize << supported.iter().count(), 16_384);
    for item in RESERVED_ITEMS {
        let items = ItemSet::new([item]).unwrap();
        let settings = GameSettings::new(0, 256, 3, 180, items).unwrap();
        assert_eq!(
            expanded_settings(settings).is_ok(),
            supported.contains(item),
            "{item:?}"
        );
    }
    let settings = GameSettings::new(0, 256, 3, 180, supported).unwrap();
    assert_eq!(
        expanded_settings(settings).unwrap().enabled_items().bits(),
        523_248
    );
}

#[test]
fn internal_binding_uses_the_selected_catalog_for_room_and_captured_world() {
    let supported = ItemSet::new([
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::InvisibilityCloak,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
    ])
    .unwrap();
    assert!(!supported.contains(ItemId::ReverseControls));

    let generated = game::generate_expanded_arena([7; 32], EnabledItems::HEART);
    let (arena, concealed, record) = generated.into_parts();
    let spawns = arena.spawns().to_vec();
    let state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART)),
        World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap(),
        SeatId::DUEL.map(|seat| ActorInit {
            id: DomainEntityId(seat.index() as u32 + 1),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap();
    let mut capture = capture_expanded(&state, &[]).unwrap();
    capture.world.0.available_items = supported;

    let heart = ItemSet::new([ItemId::Heart]).unwrap();
    let room = running_room(heart, supported);
    assert!(
        super::projection_binding::bind_expanded_for_items(
            &room,
            room.round_id,
            &capture,
            Some(record),
            supported,
        )
        .is_ok()
    );
}

fn running_room(enabled: ItemSet, available: ItemSet) -> RoomView {
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, enabled).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: available,
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(0).unwrap(),
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: true,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Connected,
        }),
        phase: RoomPhase::Running,
    }
}
