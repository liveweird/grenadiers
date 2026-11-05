use grenadiers_core::game::SeatId;

use crate::game::rooms::replay_v37::{RecordingRosterRuntime, ReplayAction};

pub(super) fn start_round(recording: &mut RecordingRosterRuntime) -> Result<(), String> {
    recording
        .admit(SeatId::One, false)
        .map_err(|error| format!("admit diagnostic human: {error:?}"))?;
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .map_err(|error| format!("ready diagnostic human: {error:?}"))?;
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recording
            .settle([None; 4])
            .map_err(|error| format!("diagnostic countdown: {error:?}"))?;
    }
    recording
        .publish_flush()
        .map_err(|error| format!("initial diagnostic flush: {error:?}"))
}
