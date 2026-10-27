mod record;
#[cfg(test)]
mod runner;
#[cfg(test)]
mod validation;

#[cfg(test)]
pub(in crate::game::rooms) use record::Header;
#[cfg(test)]
pub(in crate::game::rooms) use record::M5_LIFECYCLE_PROFILE;
#[cfg(test)]
use record::ReplayStatus;
pub(in crate::game::rooms) use record::{Journal, Operation, ReplayAction};
#[cfg(test)]
use runner::{ReplayError, replay};

#[cfg(test)]
pub(in crate::game::rooms) fn verify_exact(
    journal: &Journal,
    expected: &super::match_runtime::RuntimeInspection,
) -> Result<(), String> {
    let actual = replay(journal).map_err(|error| format!("replay failed: {error:?}"))?;
    (actual == *expected)
        .then_some(())
        .ok_or_else(|| format!("replay mismatch: actual={actual:?} expected={expected:?}"))
}

#[cfg(test)]
mod capture_tests;
#[cfg(test)]
mod cpu_contract_tests;
#[cfg(test)]
mod item_mask_plan;
#[cfg(test)]
mod optional_recording_tests;
#[cfg(test)]
mod recording_measurement_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod shield_tests;
#[cfg(test)]
mod stopwatch_tests;

#[cfg(test)]
mod golden_flame_tests;

#[cfg(test)]
mod short_fuse_tests;

#[cfg(test)]
mod short_flame_tests;

#[cfg(test)]
mod long_fuse_tests;

#[cfg(test)]
mod landmine_tests;

#[cfg(test)]
mod invisibility_cloak_tests;

#[cfg(test)]
mod invisible_bombs_tests;

#[cfg(test)]
mod kick_tests;

#[cfg(test)]
mod punch_tests;

#[cfg(test)]
mod punch_resume_tests;

#[cfg(test)]
mod bracelet_tests;

#[cfg(test)]
mod reverse_controls_tests;

#[cfg(test)]
mod diarrhea_tests;

#[cfg(test)]
mod slow_foot_tests;

#[cfg(test)]
mod piercing_bomb_tests;

#[cfg(test)]
mod power_bomb_tests;

#[cfg(test)]
mod remote_bomb_tests;

#[cfg(test)]
mod teleporter_tests;

#[cfg(test)]
mod wall_pass_tests;

#[cfg(test)]
mod oil_tests;
