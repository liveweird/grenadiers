use crate::game;
use crate::protocol::{Cell, Direction, Position, Seat, Tick, WireRoundResult};

use super::{EffectKind, EquipmentId, ItemSet, ProjectionError, WeaponId};

pub(super) use crate::protocol::item_foundation::projection_items::wire_pickup;

pub(super) fn wire_tick(value: game::Tick) -> Result<Tick, ProjectionError> {
    Tick::new(value.0).map_err(|_| ProjectionError::InvalidClock)
}
pub(super) fn wire_cell(value: game::CellId) -> Result<Cell, ProjectionError> {
    Cell::new(
        u8::try_from(value.column).map_err(|_| ProjectionError::InvalidArena)?,
        u8::try_from(value.row).map_err(|_| ProjectionError::InvalidArena)?,
    )
    .map_err(|_| ProjectionError::InvalidArena)
}
pub(super) fn wire_position(value: game::Position) -> Result<Position, ProjectionError> {
    Position::new(
        u16::try_from(value.x).map_err(|_| ProjectionError::InvalidActors)?,
        u16::try_from(value.y).map_err(|_| ProjectionError::InvalidActors)?,
    )
    .map_err(|_| ProjectionError::InvalidActors)
}
pub(super) fn wire_seat(value: game::SeatId) -> Seat {
    match value {
        game::SeatId::One => Seat::One,
        game::SeatId::Two => Seat::Two,
    }
}
pub(super) fn wire_direction(value: game::Direction) -> Direction {
    match value {
        game::Direction::North => Direction::North,
        game::Direction::East => Direction::East,
        game::Direction::South => Direction::South,
        game::Direction::West => Direction::West,
    }
}
pub(super) fn wire_speed(value: game::SpeedLevel) -> u8 {
    match value {
        game::SpeedLevel::Base => 0,
        game::SpeedLevel::One => 1,
        game::SpeedLevel::Two => 2,
        game::SpeedLevel::Three => 3,
        game::SpeedLevel::Four => 4,
    }
}
pub(super) fn wire_result(value: Option<game::RoundResult>) -> Option<WireRoundResult> {
    value.map(|result| match result {
        game::RoundResult::Winner(seat) => WireRoundResult::Winner {
            seat: wire_seat(seat),
        },
        game::RoundResult::Draw(game::DrawReason::MutualElimination) => WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::MutualElimination,
        },
        game::RoundResult::Draw(game::DrawReason::Timeout) => WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::Timeout,
        },
    })
}
pub(super) fn item_set(enabled: game::EnabledItems) -> Result<ItemSet, ProjectionError> {
    let mask = super::ItemMask::new(enabled.bits()).map_err(|_| ProjectionError::InvalidItems)?;
    Ok(ItemSet::from_mask(mask))
}
pub(super) fn wire_equipment(value: game::Equipment) -> EquipmentId {
    match value {
        game::Equipment::Kick => EquipmentId::Kick,
        game::Equipment::Punch => EquipmentId::Punch,
        game::Equipment::Bracelet => EquipmentId::Bracelet,
        game::Equipment::WallPass => EquipmentId::WallPass,
        game::Equipment::Heart => EquipmentId::Heart,
    }
}
pub(super) fn wire_weapon(value: game::SpecialWeapon) -> WeaponId {
    match value {
        game::SpecialWeapon::Remote => WeaponId::RemoteBomb,
        game::SpecialWeapon::Piercing => WeaponId::PiercingBomb,
        game::SpecialWeapon::Power => WeaponId::PowerBomb,
        game::SpecialWeapon::Landmine => WeaponId::Landmine,
    }
}
pub(super) fn wire_effect(value: game::EffectKind) -> EffectKind {
    match value {
        game::EffectKind::Shield => EffectKind::Shield,
        game::EffectKind::Stopwatch => EffectKind::Stopwatch,
        game::EffectKind::InvisibilityCloak => EffectKind::InvisibilityCloak,
        game::EffectKind::GoldenFlame => EffectKind::GoldenFlame,
        game::EffectKind::ShortFuse => EffectKind::ShortFuse,
        game::EffectKind::Diarrhea => EffectKind::Diarrhea,
        game::EffectKind::SlowFoot => EffectKind::SlowFoot,
        game::EffectKind::ShortFlame => EffectKind::ShortFlame,
        game::EffectKind::LongFuse => EffectKind::LongFuse,
        game::EffectKind::ReverseControls => EffectKind::ReverseControls,
        game::EffectKind::InvisibleBombs => EffectKind::InvisibleBombs,
    }
}
