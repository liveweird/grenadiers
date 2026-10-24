use crate::game::geometry::MovementTrace;
use crate::game::input::Direction;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::{EffectKind, EnabledItems, PickupKind, StepEvent};

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
    IllegalBombCell(CellId),
    OccupiedBombCell(CellId),
    UnknownBombOwner(EntityId),
    BombIdMismatch { key: EntityId, id: EntityId },
    UnsupportedBombTrigger(EntityId),
    UnsupportedBombBlast(EntityId),
    DuplicateRemoteOwner(EntityId),
    UnsupportedBombKickHistory(EntityId),
    UnsupportedBombPunchHistory(EntityId),
    UnsupportedEnabledItems(EnabledItems),
    UnsupportedPickup { cell: CellId, kind: PickupKind },
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
    UnsupportedEquipmentAction,
    EffectExpiryOverflow(EffectKind),
    BombPauseOverflow(EntityId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepReport {
    pub tick: Tick,
    pub traces: [MovementTrace; 2],
    pub events: Vec<StepEvent>,
}
