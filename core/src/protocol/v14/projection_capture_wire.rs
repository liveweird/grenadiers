use crate::game;
use crate::protocol::{Cell, Direction, EntityId, Position, Tick};

use super::{
    DamagePreventionSource, EffectKind, EquipmentId, EventKind, PickupKind, ProjectionError,
    RoundResult, Seat, WeaponId,
};

pub(crate) fn tick(value: game::Tick) -> Result<Tick, ProjectionError> {
    Tick::new(value.0).map_err(|_| ProjectionError::InvalidClock)
}
pub(crate) fn cell(value: game::CellId) -> Result<Cell, ProjectionError> {
    Cell::new(
        u8::try_from(value.column).map_err(|_| ProjectionError::InvalidArena)?,
        u8::try_from(value.row).map_err(|_| ProjectionError::InvalidArena)?,
    )
    .map_err(|_| ProjectionError::InvalidArena)
}
pub(crate) fn position(value: game::Position) -> Result<Position, ProjectionError> {
    Position::new(
        u16::try_from(value.x).map_err(|_| ProjectionError::InvalidActors)?,
        u16::try_from(value.y).map_err(|_| ProjectionError::InvalidActors)?,
    )
    .map_err(|_| ProjectionError::InvalidActors)
}
pub(crate) const fn seat(value: game::SeatId) -> Seat {
    match value {
        game::SeatId::One => Seat::One,
        game::SeatId::Two => Seat::Two,
        game::SeatId::Three => Seat::Three,
        game::SeatId::Four => Seat::Four,
    }
}
pub(crate) const fn domain_seat(value: Seat) -> game::SeatId {
    match value {
        Seat::One => game::SeatId::One,
        Seat::Two => game::SeatId::Two,
        Seat::Three => game::SeatId::Three,
        Seat::Four => game::SeatId::Four,
    }
}
pub(crate) const fn direction(value: game::Direction) -> Direction {
    match value {
        game::Direction::North => Direction::North,
        game::Direction::East => Direction::East,
        game::Direction::South => Direction::South,
        game::Direction::West => Direction::West,
    }
}
pub(crate) const fn speed(value: game::SpeedLevel) -> u8 {
    match value {
        game::SpeedLevel::Base => 0,
        game::SpeedLevel::One => 1,
        game::SpeedLevel::Two => 2,
        game::SpeedLevel::Three => 3,
        game::SpeedLevel::Four => 4,
    }
}
pub(crate) const fn equipment(value: game::Equipment) -> EquipmentId {
    match value {
        game::Equipment::Kick => EquipmentId::Kick,
        game::Equipment::Punch => EquipmentId::Punch,
        game::Equipment::Bracelet => EquipmentId::Bracelet,
        game::Equipment::WallPass => EquipmentId::WallPass,
        game::Equipment::Heart => EquipmentId::Heart,
    }
}
pub(crate) const fn weapon(value: game::SpecialWeapon) -> WeaponId {
    match value {
        game::SpecialWeapon::Remote => WeaponId::RemoteBomb,
        game::SpecialWeapon::Piercing => WeaponId::PiercingBomb,
        game::SpecialWeapon::Power => WeaponId::PowerBomb,
        game::SpecialWeapon::Landmine => WeaponId::Landmine,
    }
}
pub(crate) const fn effect(value: game::EffectKind) -> EffectKind {
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
pub(crate) fn result(value: game::RoundResult) -> RoundResult {
    match value {
        game::RoundResult::Winner(winner) => RoundResult::Winner { seat: seat(winner) },
        game::RoundResult::Draw(game::DrawReason::MutualElimination) => RoundResult::Draw {
            reason: super::DrawReason::MutualElimination,
        },
        game::RoundResult::Draw(game::DrawReason::Timeout) => RoundResult::Draw {
            reason: super::DrawReason::Timeout,
        },
    }
}
pub(crate) fn event_kind(
    value: &game::StepEventKind,
    at: game::Tick,
    paused: bool,
) -> Result<EventKind, ProjectionError> {
    match value {
        game::StepEventKind::BombPlaced(bomb) => Ok(EventKind::BombPlaced {
            bomb: bomb_view(*bomb, at, paused)?,
        }),
        game::StepEventKind::BombDetonated(bomb) => Ok(EventKind::BombDetonated {
            bomb: bomb_view(*bomb, at, paused)?,
        }),
        game::StepEventKind::ActorEliminated { id, seat: owner } => {
            Ok(EventKind::ActorEliminated {
                id: EntityId::new(id.0),
                seat: seat(*owner),
            })
        }
        game::StepEventKind::PickupCollected {
            id,
            seat: owner,
            cell: at,
            kind,
        } => Ok(EventKind::PickupCollected {
            id: EntityId::new(id.0),
            seat: seat(*owner),
            cell: cell(*at)?,
            kind: pickup(*kind),
        }),
        game::StepEventKind::RoundEnded(value) => Ok(EventKind::RoundEnded {
            result: result(*value),
        }),
        game::StepEventKind::DamagePrevented {
            id,
            seat: owner,
            source,
        } => Ok(EventKind::DamagePrevented {
            id: EntityId::new(id.0),
            seat: seat(*owner),
            source: match source {
                game::DamagePreventionSource::Heart => DamagePreventionSource::Heart,
                game::DamagePreventionSource::Shield => DamagePreventionSource::Shield,
            },
        }),
    }
}
pub(crate) fn pickup(value: game::PickupKind) -> PickupKind {
    crate::protocol::item_foundation::projection_items::wire_pickup(value)
}

pub(crate) fn bomb_view(
    value: game::BombView,
    at: game::Tick,
    paused: bool,
) -> Result<super::BombView, ProjectionError> {
    let trigger = match value.trigger {
        game::BombTrigger::Timed { detonates_at } => super::BombTrigger::timed(
            u16::try_from(
                detonates_at
                    .0
                    .checked_sub(at.0)
                    .ok_or(ProjectionError::InvalidBombs)?,
            )
            .map_err(|_| ProjectionError::InvalidBombs)?,
            paused,
        )
        .map_err(|_| ProjectionError::InvalidBombs)?,
        game::BombTrigger::Remote => super::BombTrigger::remote(),
        game::BombTrigger::Contact => super::BombTrigger::contact(),
    };
    Ok(super::BombView {
        id: EntityId::new(value.id.0),
        cell: cell(value.cell)?,
        owner: EntityId::new(value.owner.0),
        range: u8::try_from(value.range).map_err(|_| ProjectionError::InvalidBombs)?,
        placed_at: tick(value.placed_at)?,
        trigger,
        blast: match value.blast {
            game::BlastKind::Standard => super::BlastKind::Standard,
            game::BlastKind::Piercing => super::BlastKind::Piercing,
            game::BlastKind::Power => super::BlastKind::Power,
        },
    })
}
