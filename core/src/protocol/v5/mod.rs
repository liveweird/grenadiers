mod client;
mod projection;
mod projection_validation;
mod scalars;
mod server;
mod state_binding;

#[cfg(test)]
mod projection_test_support;

use std::fmt;

pub use client::GameClientMessage;
pub use projection::{RoomMetadata, RoomProjectionError, project_room_view};
pub use projection_validation::validate_room_view;
pub use scalars::{GameplayVersion, ResumeToken, RoundCount, Wins};
pub use server::{
    CompletedRound, ConnectionStatus, CountdownKind, GameServerMessage, MatchInterruptReason,
    MatchResult, MatchWinnerReason, RoomPhase, RoomView, Scores, SeatStatus, V5ErrorCode,
};
pub use state_binding::{validate_admission, validate_state_binding};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v5 client message")
    }
}

impl std::error::Error for ParseClientMessageError {}

pub fn parse_client_message(input: &str) -> Result<GameClientMessage, ParseClientMessageError> {
    if input.len() > super::MAX_CLIENT_MESSAGE_BYTES {
        return Err(ParseClientMessageError::InvalidMessage);
    }
    serde_json::from_str(input).map_err(|_| ParseClientMessageError::InvalidMessage)
}

pub fn encode_server_message(
    message: &GameServerMessage,
) -> Result<String, super::EncodeServerMessageError> {
    let encoded = serde_json::to_string(message)
        .map_err(|_| super::EncodeServerMessageError::EncodingFailed)?;
    super::enforce_server_message_size(encoded)
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod client_tests;
#[cfg(test)]
#[path = "max_frame_tests.rs"]
mod max_frame_tests;
#[cfg(test)]
#[path = "physical_parity_tests.rs"]
mod physical_parity_tests;
#[cfg(test)]
#[path = "projection_tests.rs"]
mod projection_tests;
#[cfg(test)]
#[path = "projection_validation_tests.rs"]
mod projection_validation_tests;
#[cfg(test)]
#[path = "server_tests.rs"]
mod server_tests;
#[cfg(test)]
#[path = "state_binding_tests.rs"]
mod state_binding_tests;
