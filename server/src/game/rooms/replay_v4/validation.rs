use grenadiers_core::{
    game::{
        MULTIPLAYER_GENERATOR_VERSION, SIMULATION_RANDOMNESS_VERSION,
        SIMULATION_SEED_DERIVATION_VERSION,
    },
    room::{BoardSize, SeatControl},
};

use super::record::{
    CPU_POLICY_VERSION, FORMAT_VERSION, GENERATOR_VERSION, LIFECYCLE_PROFILE, RULES_VERSION,
    ReplayV4Header,
};

pub(super) fn validate_header(header: &ReplayV4Header) -> Result<(), ()> {
    if header.format_version != FORMAT_VERSION
        || header.lifecycle_profile != LIFECYCLE_PROFILE
        || header.lifecycle_profile != grenadiers_core::room::ROSTER_LIFECYCLE_PROFILE
        || header.rules_version != RULES_VERSION
        || header.generator_version != GENERATOR_VERSION
        || header.generator_version != MULTIPLAYER_GENERATOR_VERSION
        || header.simulation_randomness_version != SIMULATION_RANDOMNESS_VERSION
        || header.simulation_seed_derivation_version != SIMULATION_SEED_DERIVATION_VERSION
        || header.initial_settings_revision != 1
        || header.configuration.board() != BoardSize::FifteenByThirteen
    {
        return Err(());
    }

    validate_configuration(
        header.configuration,
        header.initially_occupied_humans,
        &header.cpus,
    )
}

pub(super) fn validate_configuration(
    configuration: grenadiers_core::room::RoomConfiguration,
    initially_occupied_humans: [bool; 4],
    cpus: &[super::record::CpuProvenance],
) -> Result<(), ()> {
    if configuration.board() != BoardSize::FifteenByThirteen {
        return Err(());
    }
    let expected_difficulty = configuration.cpu_difficulty();
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
