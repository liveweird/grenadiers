use grenadiers_core::{
    game::{GenerationRecord, SeatId},
    protocol::{Counter, v14, v15, v16, v17, v18},
};

pub(super) enum PreparedProjection {
    Legacy {
        room: v14::RoomView,
        state_epoch: Option<Counter>,
        capture: Option<v14::ExpandedCapture>,
        generation: Option<GenerationRecord>,
    },
    Environment {
        room: v15::RoomView,
        state_epoch: Option<Counter>,
        capture: Option<v15::ExpandedCapture>,
        generation: Option<GenerationRecord>,
    },
    Lairs {
        room: v16::RoomView,
        state_epoch: Option<Counter>,
        capture: Option<v16::ExpandedCapture>,
        generation: Option<GenerationRecord>,
    },
    Respawn {
        room: v17::RoomView,
        state_epoch: Option<Counter>,
        capture: Option<v17::ExpandedCapture>,
        generation: Option<GenerationRecord>,
    },
    Spike {
        room: v18::RoomView,
        state_epoch: Option<Counter>,
        capture: Option<v18::ExpandedCapture>,
        generation: Option<GenerationRecord>,
    },
}

impl PreparedProjection {
    pub(super) fn encode(
        &self,
        seat: SeatId,
        last_action_id: Option<Counter>,
        sequence: Counter,
        resume_token: v14::ResumeToken,
    ) -> Result<String, v14::ProjectionError> {
        match self {
            Self::Legacy {
                room,
                state_epoch,
                capture,
                generation,
            } => v14::prepare_admission(
                v14::AdmissionProjection {
                    room,
                    state_epoch: *state_epoch,
                    capture: capture.as_ref(),
                    generation: *generation,
                },
                v14::PersonalAdmission {
                    recipient: super::types::wire_seat(seat),
                    last_action_id,
                    sequence,
                    resume_token,
                },
            ),
            Self::Environment {
                room,
                state_epoch,
                capture,
                generation,
            } => v15::prepare_admission(
                v15::AdmissionProjection {
                    room,
                    state_epoch: *state_epoch,
                    capture: capture.as_ref(),
                    generation: *generation,
                },
                v15::PersonalAdmission {
                    recipient: super::types::wire_seat(seat),
                    last_action_id,
                    sequence,
                    resume_token,
                },
            ),
            Self::Lairs {
                room,
                state_epoch,
                capture,
                generation,
            } => v16::prepare_admission(
                v16::AdmissionProjection {
                    room,
                    state_epoch: *state_epoch,
                    capture: capture.as_ref(),
                    generation: *generation,
                },
                v16::PersonalAdmission {
                    recipient: super::types::wire_seat_v16(seat),
                    last_action_id,
                    sequence,
                    resume_token,
                },
            ),
            Self::Respawn {
                room,
                state_epoch,
                capture,
                generation,
            } => v17::prepare_admission(
                v17::AdmissionProjection {
                    room,
                    state_epoch: *state_epoch,
                    capture: capture.as_ref(),
                    generation: *generation,
                },
                v17::PersonalAdmission {
                    recipient: super::types::wire_seat_v17(seat),
                    last_action_id,
                    sequence,
                    resume_token,
                },
            ),
            Self::Spike {
                room,
                state_epoch,
                capture,
                generation,
            } => v18::prepare_admission(
                v18::AdmissionProjection {
                    room,
                    state_epoch: *state_epoch,
                    capture: capture.as_ref(),
                    generation: *generation,
                },
                v18::PersonalAdmission {
                    recipient: super::types::wire_seat_v18(seat),
                    last_action_id,
                    sequence,
                    resume_token,
                },
            ),
        }
    }
}
