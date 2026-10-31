use grenadiers_core::{
    game::{CellId, PickupKind, SeatId},
    protocol::{Cell, RoomId},
};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CpuDebugResponse {
    pub(crate) protocol_version: u8,
    pub(crate) room_id: RoomId,
    pub(crate) round_id: u64,
    pub(crate) tick: u64,
    pub(crate) revision: u32,
    pub(crate) you_seat: DebugSeat,
    pub(crate) cpus: Vec<CpuDebugRow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DebugSeat {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum CpuDebugRow {
    Unavailable {
        seat: DebugSeat,
    },
    Available {
        seat: DebugSeat,
        observed_tick: u64,
        objective: DebugObjective,
        target: Option<DebugTarget>,
        approach: Option<Cell>,
        phase: DebugPhase,
        switch_reason: DebugSwitchReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DebugObjective {
    Collect,
    Farm,
    Attack,
    Explore,
    Survive,
    Wait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum DebugTarget {
    Pickup { cell: Cell, kind: DebugPickupKind },
    Block { cell: Cell },
    Opponent { seat: DebugSeat },
    Tile { cell: Cell },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DebugPhase {
    Approach,
    Plant,
    Escape,
    Reengage,
    Evade,
    Waiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DebugSwitchReason {
    Initial,
    Completed,
    TargetInvalid,
    VisibilityLost,
    BetterOpportunity,
    Stalled,
    NoTargets,
    SafetyInterrupt,
    Resumed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DebugPickupKind {
    BombCapacity,
    BlastRange,
    MovementSpeed,
    Kick,
    Punch,
    Bracelet,
    WallPass,
    Heart,
    RemoteBomb,
    PiercingBomb,
    PowerBomb,
    Landmine,
    Shield,
    Stopwatch,
    InvisibilityCloak,
    GoldenFlame,
    ShortFuse,
    Diarrhea,
    SlowFoot,
    ShortFlame,
    LongFuse,
    ReverseControls,
    InvisibleBombs,
    OilBarrel,
    Teleporter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CpuDebugQueryError {
    Unauthorized,
    Unavailable,
}

pub(crate) fn debug_seat(value: SeatId) -> DebugSeat {
    match value {
        SeatId::One => DebugSeat::One,
        SeatId::Two => DebugSeat::Two,
        SeatId::Three => DebugSeat::Three,
        SeatId::Four => DebugSeat::Four,
    }
}

pub(crate) fn debug_cell(value: CellId) -> Option<Cell> {
    Cell::new(
        u8::try_from(value.column).ok()?,
        u8::try_from(value.row).ok()?,
    )
    .ok()
}

pub(crate) fn debug_pickup(value: PickupKind) -> DebugPickupKind {
    match value {
        PickupKind::BombCapacity => DebugPickupKind::BombCapacity,
        PickupKind::BlastRange => DebugPickupKind::BlastRange,
        PickupKind::MovementSpeed => DebugPickupKind::MovementSpeed,
        PickupKind::Kick => DebugPickupKind::Kick,
        PickupKind::Punch => DebugPickupKind::Punch,
        PickupKind::Bracelet => DebugPickupKind::Bracelet,
        PickupKind::WallPass => DebugPickupKind::WallPass,
        PickupKind::Heart => DebugPickupKind::Heart,
        PickupKind::RemoteBomb => DebugPickupKind::RemoteBomb,
        PickupKind::PiercingBomb => DebugPickupKind::PiercingBomb,
        PickupKind::PowerBomb => DebugPickupKind::PowerBomb,
        PickupKind::Landmine => DebugPickupKind::Landmine,
        PickupKind::Shield => DebugPickupKind::Shield,
        PickupKind::Stopwatch => DebugPickupKind::Stopwatch,
        PickupKind::InvisibilityCloak => DebugPickupKind::InvisibilityCloak,
        PickupKind::GoldenFlame => DebugPickupKind::GoldenFlame,
        PickupKind::ShortFuse => DebugPickupKind::ShortFuse,
        PickupKind::Diarrhea => DebugPickupKind::Diarrhea,
        PickupKind::SlowFoot => DebugPickupKind::SlowFoot,
        PickupKind::ShortFlame => DebugPickupKind::ShortFlame,
        PickupKind::LongFuse => DebugPickupKind::LongFuse,
        PickupKind::ReverseControls => DebugPickupKind::ReverseControls,
        PickupKind::InvisibleBombs => DebugPickupKind::InvisibleBombs,
        PickupKind::OilBarrel => DebugPickupKind::OilBarrel,
        PickupKind::Teleporter => DebugPickupKind::Teleporter,
    }
}
