use super::geometry::MovementTrace;
use super::input::Direction;
use super::types::{EntityId, Position, SeatId, SpeedLevel, Tick};
use super::{EffectKind, StepEvent};

pub const MAX_INPUTS_PER_TICK: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorInit {
    pub id: EntityId,
    pub seat: SeatId,
    pub position: Position,
    pub heading: Direction,
    pub speed: SpeedLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputAction {
    Press(Direction),
    Release(Direction),
    Clear,
    PlaceBomb,
    UseEquipment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeatInput {
    pub seat: SeatId,
    pub action: InputAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    DuplicateActorId(EntityId),
    DuplicateSeat(SeatId),
    IllegalActorPosition(EntityId),
    DuplicateEntityId(EntityId),
    IllegalBombCell(super::CellId),
    OccupiedBombCell(super::CellId),
    UnknownBombOwner(EntityId),
    BombIdMismatch { key: EntityId, id: EntityId },
    UnsupportedBombTrigger(EntityId),
    UnsupportedBombBlast(EntityId),
    DuplicateRemoteOwner(EntityId),
    UnsupportedBombKickHistory(EntityId),
    UnsupportedBombPunchHistory(EntityId),
    UnsupportedEnabledItems(super::EnabledItems),
    UnsupportedPickup {
        cell: super::CellId,
        kind: super::PickupKind,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    TooManyInputs { count: usize, maximum: usize },
    TickOverflow,
    UnsupportedBombTrigger(EntityId),
    UnsupportedBombBlast(EntityId),
    DuplicateRemoteOwner(EntityId),
    UnsupportedBombKickHistory(EntityId),
    UnsupportedBombPunchHistory(EntityId),
    EffectExpiryOverflow(EffectKind),
    BombPauseOverflow(EntityId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepReport {
    pub tick: Tick,
    pub traces: [MovementTrace; 2],
    pub events: Vec<StepEvent>,
}
