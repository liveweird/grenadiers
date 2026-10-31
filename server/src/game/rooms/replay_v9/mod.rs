//! Private format9 envelope: checked accepted-input body plus explicit mob AI identity.
//! Format8's standalone reader always selects the archived navigation policy.
mod recording;
#[cfg(test)]
mod tests;

use grenadiers_core::game::MobNavigationPolicy;

use super::{replay_v4::runner::ReplayError, replay_v8, roster_runtime::RosterRuntimeInspection};

pub(in crate::game::rooms) use recording::RecordingRosterRuntime;
pub(in crate::game::rooms) use replay_v8::{CpuProvenance, ReplayAction};

#[derive(Clone, Debug)]
pub(in crate::game::rooms) struct Journal {
    format_version: u16,
    mob_navigation_version: u16,
    body: replay_v8::Journal,
}

impl Journal {
    fn new(body: replay_v8::Journal) -> Self {
        Self {
            format_version: 9,
            mob_navigation_version: 2,
            body,
        }
    }
}

pub(in crate::game::rooms) fn replay(
    journal: &Journal,
) -> Result<RosterRuntimeInspection, ReplayError> {
    if journal.format_version != 9
        || journal.mob_navigation_version != MobNavigationPolicy::EscapeV2.version()
        || journal.body.cpu_policy_version() != 23
    {
        return Err(ReplayError::InvalidHeader);
    }
    // Reuse operation validation, normalization, seed derivation and execution.
    // The body is not interpreted through the archived format8 entry point.
    replay_v8::replay_with_mob_navigation(&journal.body, MobNavigationPolicy::EscapeV2)
}

pub(in crate::game::rooms) fn verify_exact(
    journal: &Journal,
    expected: &RosterRuntimeInspection,
) -> Result<(), String> {
    let actual = replay(journal).map_err(|error| format!("replay failed: {error:?}"))?;
    (actual == *expected)
        .then_some(())
        .ok_or_else(|| "replay9 state mismatch".to_owned())
}
