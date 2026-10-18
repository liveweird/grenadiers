use crate::game::{EventAudience, RulesConfig, SpeedLevel};
use crate::protocol::{ProjectedRound, RulesVersion, Tick, WireRoundResult};

use super::{
    ArenaView, BombView, EventKind, FlameView, GameSettings, ItemSet, PickupView, RoomView,
    VisibleActorView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldFacts {
    pub(crate) rules_version: RulesVersion,
    pub(crate) physical_tick: Tick,
    pub(crate) running_updates: u16,
    pub(crate) remaining_ticks: u16,
    pub(crate) result: Option<WireRoundResult>,
    pub(crate) arena: ArenaView,
    pub(crate) actors: [VisibleActorView; 2],
    pub(crate) bombs: Vec<BombView>,
    pub(crate) flames: Vec<FlameView>,
    pub(crate) pickups: Vec<PickupView>,
    pub(crate) available_items: ItemSet,
    pub(crate) enabled_items: ItemSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EventFacts {
    pub(crate) tick: Tick,
    pub(crate) audience: EventAudience,
    pub(crate) kind: EventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReportFacts {
    pub(crate) tick: Tick,
    pub(crate) events: Vec<EventFacts>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedWorld(pub(super) WorldFacts);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedReports(pub(super) Vec<ReportFacts>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassicCapture {
    pub(super) rules: RulesConfig,
    pub(super) world: ValidatedWorld,
    pub(super) reports: ValidatedReports,
    pub(super) legacy: ProjectedRound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundPublication {
    pub(super) room: RoomView,
    pub(super) world: ValidatedWorld,
    pub(super) reports: ValidatedReports,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidRules,
    InvalidClock,
    InvalidArena,
    InvalidActors,
    InvalidBombs,
    InvalidFlames,
    InvalidPickups,
    InvalidResult,
    InvalidEffects,
    InvalidItems,
    InvalidReports,
    InvalidEvent,
    InvalidClassicBinding,
    EncodingFailed,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProjectionError {}

pub(super) fn rules_settings(rules: RulesConfig) -> Result<GameSettings, ProjectionError> {
    let (speed, assist, wins, fuse) = if let Some(settings) = rules.room_settings() {
        (
            match settings.starting_speed() {
                SpeedLevel::Base => 0,
                SpeedLevel::One => 1,
                SpeedLevel::Two => 2,
                SpeedLevel::Three => 3,
                SpeedLevel::Four => 4,
            },
            settings.turn_assist_units(),
            settings.wins_to_finish(),
            u16::try_from(settings.bomb_fuse_ticks()).map_err(|_| ProjectionError::InvalidRules)?,
        )
    } else {
        (
            0,
            rules.turn_assist(),
            3,
            u16::try_from(rules.bomb_fuse_ticks()).map_err(|_| ProjectionError::InvalidRules)?,
        )
    };
    GameSettings::new(speed, assist, wins, fuse, ItemSet::empty())
        .map_err(|_| ProjectionError::InvalidRules)
}
