use super::{
    CompletedRound, ConnectionStatus, Controller, CountdownKind, MatchResult, RoomPhase, RoomView,
    Score, SeatStatus,
};
use crate::protocol::Tick;

#[derive(Clone, Copy)]
pub(crate) enum LifecyclePhase {
    Waiting,
    CountdownInitial {
        ends_at: Tick,
    },
    CountdownInterRound {
        ends_at: Tick,
    },
    Running,
    Result {
        hold_until: Tick,
    },
    MatchResult {
        ended_at: Tick,
        result: MatchResult,
        rematch_at: Tick,
    },
    Interrupted {
        ended_at: Tick,
    },
}

pub(crate) struct LifecycleRoom<'a> {
    pub tick: Tick,
    pub round_id: u32,
    pub round_number: u32,
    pub wins_to_finish: u8,
    pub scores: &'a [Score],
    pub last_round: Option<CompletedRound>,
    pub seats: &'a [SeatStatus],
    pub phase: LifecyclePhase,
}

impl<'a> LifecycleRoom<'a> {
    pub(crate) fn v14(room: &'a RoomView) -> Self {
        Self {
            tick: room.tick,
            round_id: room.round_id.get(),
            round_number: room.round_number.get(),
            wins_to_finish: room.settings.wins_to_finish(),
            scores: &room.scores,
            last_round: room.last_round,
            seats: &room.seats,
            phase: match room.phase {
                RoomPhase::Waiting => LifecyclePhase::Waiting,
                RoomPhase::Countdown {
                    kind: CountdownKind::Initial,
                    ends_at,
                } => LifecyclePhase::CountdownInitial { ends_at },
                RoomPhase::Countdown {
                    kind: CountdownKind::InterRound,
                    ends_at,
                } => LifecyclePhase::CountdownInterRound { ends_at },
                RoomPhase::Running => LifecyclePhase::Running,
                RoomPhase::Result { hold_until } => LifecyclePhase::Result { hold_until },
                RoomPhase::MatchResult {
                    ended_at,
                    result,
                    rematch_at,
                } => LifecyclePhase::MatchResult {
                    ended_at,
                    result,
                    rematch_at,
                },
                RoomPhase::Interrupted { ended_at, .. } => LifecyclePhase::Interrupted { ended_at },
            },
        }
    }
}

pub(crate) const fn connected(value: ConnectionStatus) -> bool {
    matches!(
        value,
        ConnectionStatus::Connected | ConnectionStatus::ServerControlled
    )
}

pub(crate) const fn cpu(value: Controller) -> bool {
    matches!(value, Controller::Cpu { .. })
}
