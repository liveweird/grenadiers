use grenadiers_core::room::{BoardSize, RespawnLairTuning, SeatControl};

use super::record::*;

pub(super) fn validate_header(header: &ReplayHeader) -> Result<(), ()> {
    let base = header.configuration.base();
    let expected_generator = if base.board() == BoardSize::FifteenByThirteen {
        grenadiers_core::game::MULTIPLAYER_GENERATOR_VERSION
    } else {
        grenadiers_core::game::VARIABLE_GENERATOR_VERSION
    };
    if header.format_version != FORMAT_VERSION
        || header.lifecycle_profile != LIFECYCLE_PROFILE
        || header.lifecycle_profile != grenadiers_core::room::ROSTER_LIFECYCLE_PROFILE
        || header.rules_version != RULES_VERSION
        || !matches!(
            header.cpu_policy_version,
            ARCHIVED_CPU_POLICY_VERSION
                | PICKUP_CPU_POLICY_VERSION
                | INTENT_CPU_POLICY_VERSION
                | STRATEGY_CPU_POLICY_VERSION
                | CPU_POLICY_VERSION
        )
        || header.generator_version != expected_generator
        || header.simulation_randomness_version != SIMULATION_RANDOMNESS_VERSION
        || header.simulation_seed_derivation_version != SIMULATION_SEED_DERIVATION_VERSION
        || header.environment_randomness_version != ENVIRONMENT_RANDOMNESS_VERSION
        || header.environment_randomness_version
            != grenadiers_core::game::STORM_CLOUD_RANDOMNESS_VERSION
        || header.lair_layout_version != LAIR_LAYOUT_VERSION
        || header.lair_layout_version != grenadiers_core::game::LAIR_LAYOUT_VERSION
        || header.initial_settings_revision != 1
        || !(2..=4).contains(&base.player_count().get())
        || RespawnLairTuning::try_from(header.configuration.lairs()).is_err()
    {
        return Err(());
    }
    validate_configuration(header)
}

fn validate_configuration(header: &ReplayHeader) -> Result<(), ()> {
    let base = header.configuration.base();
    let expected_difficulty = base.cpu_difficulty();
    let mut provenance = header.cpus.iter();
    for &seat in base.player_count().seats() {
        match base.controller(seat) {
            Some(SeatControl::Human) => {
                if header.cpus.iter().any(|entry| entry.seat == seat) {
                    return Err(());
                }
            }
            Some(SeatControl::Cpu) => {
                let entry = provenance.next().ok_or(())?;
                if entry.seat != seat
                    || Some(entry.difficulty) != expected_difficulty
                    || entry.policy_version != header.cpu_policy_version
                    || header.initially_occupied_humans[seat.index()]
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
    for (index, occupied) in header.initially_occupied_humans.iter().copied().enumerate() {
        if occupied
            && base.controller(grenadiers_core::game::SeatId::ALL[index])
                != Some(SeatControl::Human)
        {
            return Err(());
        }
    }
    Ok(())
}

pub(super) fn validate_operation(header: &ReplayHeader, operation: Operation) -> Result<(), ()> {
    let Operation::Action {
        seat,
        epoch,
        source,
        action,
    } = operation
    else {
        return Ok(());
    };
    if epoch == 0 {
        return Err(());
    }
    let expected = match header.configuration.base().controller(seat) {
        Some(SeatControl::Human) => ActionSource::Human,
        Some(SeatControl::Cpu) => ActionSource::Cpu,
        None => return Err(()),
    };
    if source != expected
        || source == ActionSource::Cpu
            && !matches!(
                action,
                ReplayAction::Movement(_) | ReplayAction::Bomb(_) | ReplayAction::Equipment(_)
            )
        || matches!(action, ReplayAction::Bomb(0) | ReplayAction::Equipment(0))
        || matches!(action, ReplayAction::Bomb(id) | ReplayAction::Equipment(id) if id > grenadiers_core::protocol::MAX_COUNTER)
        || matches!(
            action,
            ReplayAction::Ready {
                settings_revision: 0,
                ..
            } | ReplayAction::UpdateSettings {
                settings_revision: 0,
                ..
            }
        )
    {
        return Err(());
    }
    Ok(())
}
