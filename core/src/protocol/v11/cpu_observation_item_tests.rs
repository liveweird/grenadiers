use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::WorldId;
use crate::game::{self, PickupKind};
use crate::protocol::Seat;
use crate::room::RoomSettings;

use super::ItemId;
use super::cpu_observation_tests::{enabled, observe, state};

const PUBLIC_PICKUPS: [PickupKind; 24] = [
    PickupKind::BombCapacity,
    PickupKind::BlastRange,
    PickupKind::MovementSpeed,
    PickupKind::Kick,
    PickupKind::Punch,
    PickupKind::Bracelet,
    PickupKind::WallPass,
    PickupKind::Heart,
    PickupKind::RemoteBomb,
    PickupKind::PiercingBomb,
    PickupKind::PowerBomb,
    PickupKind::Landmine,
    PickupKind::Shield,
    PickupKind::Stopwatch,
    PickupKind::InvisibilityCloak,
    PickupKind::GoldenFlame,
    PickupKind::ShortFuse,
    PickupKind::Diarrhea,
    PickupKind::SlowFoot,
    PickupKind::ShortFlame,
    PickupKind::LongFuse,
    PickupKind::ReverseControls,
    PickupKind::InvisibleBombs,
    PickupKind::Teleporter,
];

const ALL_ITEMS: [ItemId; 22] = [
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
fn every_public_pickup_category_survives_the_recipient_adapter() {
    let enabled = enabled(&ALL_ITEMS);
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let arena = game::test_support::open_border_arena(BTreeSet::new());
    let exposed = PUBLIC_PICKUPS
        .iter()
        .copied()
        .enumerate()
        .map(|(index, kind)| {
            let column = u16::try_from(index % 12).unwrap() + 1;
            let row = u16::try_from(index / 12).unwrap() + 3;
            (game::CellId::new(column, row), kind)
        })
        .collect::<BTreeMap<_, _>>();
    let world = game::World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed.clone())
        .unwrap();
    let state = state(settings, world);
    let observation = observe(&state, settings, Seat::Two);
    let observed = observation
        .parts()
        .pickups
        .iter()
        .map(|pickup| (pickup.cell, pickup.kind))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(observed, exposed);
    assert_eq!(
        observation.parts().world,
        WorldId {
            room_instance: 77,
            physical_round: 9,
        }
    );
}
