use crate::game::geometry::{MovementSegment, MovementTrace};
use crate::game::input::Direction;
use crate::game::types::{CellId, EntityId, PlayerCount, Position, SeatId, SpeedLevel, Tick};
use crate::game::{EffectKind, EnabledItems, PickupKind, StepEvent};
use crate::room::BoardSize;

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
    MissingSimulationRandomness,
    InsufficientParticipants(usize),
    InvalidPlayerCount(usize),
    UnsupportedPlayerCount { version: u16, count: PlayerCount },
    UnsupportedBoardSize { version: u16, board: BoardSize },
    NonCanonicalSeats,
    SpawnCountMismatch { actors: usize, spawns: usize },
    DuplicateActorId(EntityId),
    DuplicateSeat(SeatId),
    IllegalActorPosition(EntityId),
    ConflictingActorPositions { first: SeatId, second: SeatId },
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
    UnsupportedShoveHistory(EntityId),
    UnsupportedEnabledItems(EnabledItems),
    UnsupportedPickup { cell: CellId, kind: PickupKind },
    UnsupportedOilBarrels,
    InvalidOilBarrel(CellId),
    InvalidLairCell(CellId),
    LairTimerOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    TooManyInputs { count: usize, maximum: usize },
    SeatNotParticipating(SeatId),
    TickOverflow,
    UnsupportedBombTrigger(EntityId),
    UnsupportedBombBlast(EntityId),
    DuplicateRemoteOwner(EntityId),
    UnsupportedBombKickHistory(EntityId),
    UnsupportedBombPunchHistory(EntityId),
    UnsupportedShoveHistory(EntityId),
    UnsupportedEquipmentAction,
    EffectExpiryOverflow(EffectKind),
    BombPauseOverflow(EntityId),
    StormSlowExpiryOverflow,
    LairTimerOverflow,
    EntityIdOverflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepReport {
    pub tick: Tick,
    pub traces: Vec<MovementTrace>,
    pub events: Vec<StepEvent>,
    pub(crate) forced_movements: Vec<ForcedMovement>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForcedMovement {
    pub(crate) pusher: SeatId,
    pub(crate) seat: SeatId,
    pub(crate) segment: MovementSegment,
}
