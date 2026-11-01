use grenadiers_core::room::SeatControl;

use super::record::{
    CPU_POLICY_VERSION, LIFECYCLE_PROFILE, RULES_VERSION, ReplayV4Header,
    SIMULATION_RANDOMNESS_VERSION, SIMULATION_SEED_DERIVATION_VERSION,
};

pub(super) fn validate_header(header: &ReplayV4Header) -> Result<(), ()> {
    validate_header_for_profile(super::profile::ReplayProfile::V4, header)
}

pub(super) fn validate_header_v5(header: &ReplayV4Header) -> Result<(), ()> {
    validate_header_for_profile(super::profile::ReplayProfile::V5, header)
}

pub(super) fn validate_header_for_profile(
    profile: super::profile::ReplayProfile,
    header: &ReplayV4Header,
) -> Result<(), ()> {
    if header.format_version != profile.format()
        || header.lifecycle_profile != LIFECYCLE_PROFILE
        || header.lifecycle_profile != grenadiers_core::room::ROSTER_LIFECYCLE_PROFILE
        || header.rules_version != RULES_VERSION
        || header.generator_version != profile.generator()
        || header.simulation_randomness_version != SIMULATION_RANDOMNESS_VERSION
        || header.simulation_seed_derivation_version != SIMULATION_SEED_DERIVATION_VERSION
        || header.initial_settings_revision != 1
        || !profile.accepts_board(header.configuration.board())
    {
        return Err(());
    }

    validate_configuration(
        profile,
        header.configuration,
        header.initially_occupied_humans,
        &header.cpus,
    )
}

pub(super) fn validate_configuration(
    profile: super::profile::ReplayProfile,
    configuration: grenadiers_core::room::RoomConfiguration,
    initially_occupied_humans: [bool; 4],
    cpus: &[super::record::CpuProvenance],
) -> Result<(), ()> {
    if !profile.accepts_board(configuration.board()) {
        return Err(());
    }
    let expected_difficulty = configuration.cpu_difficulty();
    validate_archived_difficulty(expected_difficulty)?;
    let mut provenance = cpus.iter();
    for &seat in configuration.player_count().seats() {
        match configuration.controller(seat) {
            Some(SeatControl::Human) => {
                if cpus.iter().any(|entry| entry.seat == seat) {
                    return Err(());
                }
            }
            Some(SeatControl::Cpu) => {
                let entry = provenance.next().ok_or(())?;
                if entry.seat != seat
                    || Some(entry.difficulty) != expected_difficulty
                    || entry.policy_version != CPU_POLICY_VERSION
                    || initially_occupied_humans[seat.index()]
                {
                    return Err(());
                }
            }
            None => return Err(()),
        }
    }
    if provenance.next().is_some() {
        return Err(());
    }
    for (index, occupied) in initially_occupied_humans.iter().copied().enumerate() {
        if occupied
            && configuration.controller(grenadiers_core::game::SeatId::ALL[index])
                != Some(SeatControl::Human)
        {
            return Err(());
        }
    }
    Ok(())
}

pub(in crate::game::rooms) fn validate_archived_difficulty(
    difficulty: Option<grenadiers_core::room::CpuDifficulty>,
) -> Result<(), ()> {
    if difficulty == Some(grenadiers_core::room::CpuDifficulty::Hard) {
        Err(())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod hard_tests {
    use super::validate_archived_difficulty;
    use grenadiers_core::room::CpuDifficulty;

    #[test]
    fn archived_replays_reject_hard_without_a_standard_fallback() {
        assert!(validate_archived_difficulty(Some(CpuDifficulty::Hard)).is_err());
        assert!(validate_archived_difficulty(Some(CpuDifficulty::Easy)).is_ok());
        assert!(validate_archived_difficulty(Some(CpuDifficulty::Standard)).is_ok());
        assert!(validate_archived_difficulty(None).is_ok());
    }
}
