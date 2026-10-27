mod error;
mod frames;
mod publication;
mod runtime;

pub(super) use error::MatchRuntimeError;
pub(crate) use frames::SeatFrames;
pub(super) use runtime::MatchRuntime;
pub(super) use runtime::admission::PreparedRuntimeAdmission;
pub(super) use runtime::admission::ResumeKind;
#[cfg(test)]
pub(super) use runtime::replay::RuntimeInspection;
pub(crate) use runtime::{MatchAction, MatchNotice};

#[cfg(test)]
#[path = "heart_tests.rs"]
mod heart_tests;
#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;
#[cfg(test)]
#[path = "invisibility_cloak_tests.rs"]
mod invisibility_cloak_tests;
#[cfg(test)]
#[path = "invisible_bombs_tests.rs"]
mod invisible_bombs_tests;
#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "performance_tests.rs"]
mod performance_tests;
#[cfg(test)]
#[path = "publication_tests.rs"]
mod publication_tests;
#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
#[cfg(test)]
pub(in crate::game::rooms) use test_support::{
    absorb_generated_heart, collect_generated_heart, collect_generated_pickup,
    expanded_heart_runtime, move_cells, plant, start_expanded_heart_round, start_optional_runtime,
    start_optional_runtime_with_fuse, start_shield_runtime, wait_clear,
};

#[cfg(test)]
macro_rules! prepare_expanded_admission {
    ($room:expr, $epoch:expr, $capture:expr, $generation:expr, $recipient:expr,
     $last_action:expr, $sequence:expr, $token:expr $(,)?) => {
        grenadiers_core::protocol::v13::prepare_admission(
            grenadiers_core::protocol::v13::AdmissionProjection {
                room: $room,
                state_epoch: $epoch,
                capture: $capture,
                generation: $generation,
            },
            grenadiers_core::protocol::v13::PersonalAdmission {
                recipient: $recipient,
                last_action_id: $last_action,
                sequence: $sequence,
                resume_token: $token,
            },
        )
    };
}
#[cfg(test)]
pub(in crate::game::rooms) use prepare_expanded_admission;
