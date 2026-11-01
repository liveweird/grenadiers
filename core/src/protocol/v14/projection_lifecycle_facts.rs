use super::{CompletedRound, CountdownKind, MatchResult, RoomPhase, RoomView, Score, SeatStatus};
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
    pub seats: Vec<LifecycleSeat>,
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
            seats: room.seats.iter().map(LifecycleSeat::v14).collect(),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LifecycleConnection {
    Vacant,
    Connected,
    Disconnected { expires_at: Tick },
    Expired,
    ServerControlled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LifecycleSeat {
    pub seat: super::Seat,
    pub occupied: bool,
    pub ready: bool,
    pub rematch_ready: bool,
    pub connection: LifecycleConnection,
    pub is_cpu: bool,
}

impl LifecycleSeat {
    pub(crate) const fn v14(value: &SeatStatus) -> Self {
        Self {
            seat: value.seat,
            occupied: value.occupied,
            ready: value.ready,
            rematch_ready: value.rematch_ready,
            connection: match value.connection {
                super::ConnectionStatus::Vacant => LifecycleConnection::Vacant,
                super::ConnectionStatus::Connected => LifecycleConnection::Connected,
                super::ConnectionStatus::Disconnected { expires_at } => {
                    LifecycleConnection::Disconnected { expires_at }
                }
                super::ConnectionStatus::Expired => LifecycleConnection::Expired,
                super::ConnectionStatus::ServerControlled => LifecycleConnection::ServerControlled,
            },
            is_cpu: matches!(value.controller, super::Controller::Cpu { .. }),
        }
    }
}

pub(crate) const fn connected(value: LifecycleConnection) -> bool {
    matches!(
        value,
        LifecycleConnection::Connected | LifecycleConnection::ServerControlled
    )
}
