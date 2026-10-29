use std::sync::Arc;

use grenadiers_core::{
    game::SeatId,
    protocol::{RoomId, Seat, v14},
};
use tokio::sync::{oneshot, watch};

use super::super::{
    ClosingReason, RoomError,
    match_owner::{MatchAdmission, MatchOwnerNotice},
    roster_owner::{RosterAdmission, RosterOwnerNotice},
    roster_runtime::RosterNotice,
};

pub(crate) enum LiveAdmission {
    Legacy(MatchAdmission),
    Roster(RosterAdmission),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LiveDelivery {
    Publication(Arc<str>),
    Closing {
        last: Arc<str>,
        reason: ClosingReason,
    },
}

pub(crate) enum LiveMemberEvent {
    Notice(Result<(), watch::error::RecvError>),
    Replaced,
}

impl LiveAdmission {
    pub(crate) fn room_id(&self) -> &RoomId {
        match self {
            Self::Legacy(value) => &value.room_id,
            Self::Roster(value) => &value.room_id,
        }
    }

    pub(crate) fn seat(&self) -> SeatId {
        match self {
            Self::Legacy(value) => match value.seat {
                Seat::One => SeatId::One,
                Seat::Two => SeatId::Two,
            },
            Self::Roster(value) => value.seat,
        }
    }

    pub(crate) fn personal_frame(&self) -> &Arc<str> {
        match self {
            Self::Legacy(value) => &value.personal_frame,
            Self::Roster(value) => &value.personal_frame,
        }
    }

    pub(crate) fn disarm(&mut self) {
        match self {
            Self::Legacy(value) => value.guard.disarm(),
            Self::Roster(value) => value.guard.disarm(),
        }
    }

    pub(crate) fn acknowledge_notices(&mut self) {
        match self {
            Self::Legacy(value) => {
                value.notices.borrow_and_update();
            }
            Self::Roster(value) => {
                value.notices.borrow_and_update();
            }
        }
    }

    pub(crate) fn latest_delivery(&mut self) -> Result<LiveDelivery, RoomError> {
        let seat = self.seat();
        match self {
            Self::Legacy(value) => match value.notices.borrow_and_update().clone() {
                MatchOwnerNotice::Publication(notice) => {
                    Ok(LiveDelivery::Publication(legacy_frame(&notice, seat)?))
                }
                MatchOwnerNotice::Closing { last, reason } => Ok(LiveDelivery::Closing {
                    last: legacy_frame(&last, seat)?,
                    reason,
                }),
            },
            Self::Roster(value) => match value.notices.borrow_and_update().clone() {
                RosterOwnerNotice::Publication(notice) => {
                    Ok(LiveDelivery::Publication(roster_frame(&notice, seat)?))
                }
                RosterOwnerNotice::Closing { last, reason } => Ok(LiveDelivery::Closing {
                    last: roster_frame(&last, seat)?,
                    reason,
                }),
            },
        }
    }

    pub(crate) async fn next_event(&mut self, replacement_closed: &mut bool) -> LiveMemberEvent {
        match self {
            Self::Legacy(value) => {
                next_event(&mut value.notices, &mut value.replaced, replacement_closed).await
            }
            Self::Roster(value) => {
                next_event(&mut value.notices, &mut value.replaced, replacement_closed).await
            }
        }
    }
}

fn legacy_frame(
    notice: &super::super::match_runtime::MatchNotice,
    seat: SeatId,
) -> Result<Arc<str>, RoomError> {
    if !SeatId::DUEL.contains(&seat) {
        return Err(RoomError::CapacityReached);
    }
    notice.frame(seat).ok_or(RoomError::CapacityReached)
}

fn roster_frame(notice: &RosterNotice, seat: SeatId) -> Result<Arc<str>, RoomError> {
    let room = notice.legacy_room().ok_or(RoomError::CapacityReached)?;
    if !room.configuration.domain().player_count().contains(seat) {
        return Err(RoomError::CapacityReached);
    }
    if let Some(frame) = notice.frame(seat) {
        return Ok(frame);
    }
    v14::validate_room_view(room).map_err(|_| RoomError::CapacityReached)?;
    v14::encode_server_message(&v14::GameServerMessage::RoomState { room: room.clone() })
        .map(Arc::from)
        .map_err(|_| RoomError::CapacityReached)
}

async fn next_event<T>(
    notices: &mut watch::Receiver<T>,
    replaced: &mut oneshot::Receiver<()>,
    replacement_closed: &mut bool,
) -> LiveMemberEvent {
    if *replacement_closed {
        return LiveMemberEvent::Notice(notices.changed().await);
    }
    tokio::select! {
        biased;
        result = replaced => match result {
            Ok(()) => LiveMemberEvent::Replaced,
            Err(_) => {
                *replacement_closed = true;
                LiveMemberEvent::Notice(notices.changed().await)
            }
        },
        result = notices.changed() => LiveMemberEvent::Notice(result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn replacement_wins_when_notice_and_replacement_are_both_ready() {
        let (notices, mut receiver) = watch::channel(0_u8);
        let (replaced, mut replacement) = oneshot::channel();
        notices.send_replace(1);
        replaced.send(()).unwrap();
        let mut replacement_closed = false;
        assert!(matches!(
            next_event(&mut receiver, &mut replacement, &mut replacement_closed).await,
            LiveMemberEvent::Replaced
        ));
    }

    #[tokio::test]
    async fn closed_replacement_channel_falls_through_to_notice_stream() {
        let (notices, mut receiver) = watch::channel(0_u8);
        let (replaced, mut replacement) = oneshot::channel::<()>();
        drop(replaced);
        notices.send_replace(1);
        let mut replacement_closed = false;
        assert!(matches!(
            next_event(&mut receiver, &mut replacement, &mut replacement_closed).await,
            LiveMemberEvent::Notice(Ok(()))
        ));
        assert!(replacement_closed);
    }
}
