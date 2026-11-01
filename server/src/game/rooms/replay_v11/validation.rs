use grenadiers_core::room::{BoardSize, ROSTER_LIFECYCLE_PROFILE, RespawnLairTuning, SeatControl};

use super::{Journal, ReplayHeader, replay_v8};

pub(super) fn validate(journal: &Journal) -> Result<(), ()> {
    validate_header(&journal.header, &journal.body)?;
    for record in &journal.body.records {
        validate_operation(&journal.header, record.operation)?;
    }
    Ok(())
}

pub(super) fn validate_header(header: &ReplayHeader, body: &replay_v8::Journal) -> Result<(), ()> {
    let base = header.configuration.base().base();
    let expected_generator = if base.board() == BoardSize::FifteenByThirteen {
        grenadiers_core::game::MULTIPLAYER_GENERATOR_VERSION
    } else {
        grenadiers_core::game::VARIABLE_GENERATOR_VERSION
    };
    if header.format_version != super::FORMAT_VERSION
        || header.lifecycle_profile != ROSTER_LIFECYCLE_PROFILE
        || header.rules_version != super::RULES_VERSION
        || header.cpu_policy_version != super::CPU_POLICY_VERSION
        || header.generator_version != expected_generator
        || header.simulation_randomness_version != replay_v8::record::SIMULATION_RANDOMNESS_VERSION
        || header.simulation_seed_derivation_version
            != replay_v8::record::SIMULATION_SEED_DERIVATION_VERSION
        || header.environment_randomness_version
            != grenadiers_core::game::STORM_CLOUD_RANDOMNESS_VERSION
        || header.lair_layout_version != grenadiers_core::game::LAIR_LAYOUT_VERSION
        || header.spike_pit_layout_version != grenadiers_core::game::SPIKE_PIT_LAYOUT_VERSION
        || header.mob_navigation_version
            != grenadiers_core::game::MobNavigationPolicy::TimedEscapeV3.version()
        || header.initial_settings_revision != 1
        || header
            .initially_occupied_humans
            .iter()
            .any(|&occupied| occupied)
        || !(2..=4).contains(&base.player_count().get())
        || RespawnLairTuning::try_from(header.configuration.base().lairs()).is_err()
        || body.header.configuration != *header.configuration.base()
        || body.header.initial_settings != header.initial_settings
        || body.header.initial_settings_revision != header.initial_settings_revision
        || body.header.initially_occupied_humans != header.initially_occupied_humans
        || body.header.cpu_policy_version != super::CPU_POLICY_VERSION
        || body.header.cpus.as_ref() != header.cpus.as_ref()
    {
        return Err(());
    }
    validate_configuration(header)
}

fn validate_configuration(header: &ReplayHeader) -> Result<(), ()> {
    let base = header.configuration.base().base();
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
                    || entry.policy_version != super::CPU_POLICY_VERSION
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
    Ok(())
}

fn validate_operation(
    header: &ReplayHeader,
    operation: replay_v8::record::Operation,
) -> Result<(), ()> {
    use replay_v8::record::{ActionSource, Operation, ReplayAction};
    let Operation::Action {
        seat,
        epoch,
        source,
        action,
    } = operation
    else {
        return Ok(());
    };
    let expected = match header.configuration.base().base().controller(seat) {
        Some(SeatControl::Human) => ActionSource::Human,
        Some(SeatControl::Cpu) => ActionSource::Cpu,
        None => return Err(()),
    };
    if epoch == 0
        || source != expected
        || source == ActionSource::Cpu
            && !matches!(
                action,
                ReplayAction::Movement(_) | ReplayAction::Bomb(_) | ReplayAction::Equipment(_)
            )
        || matches!(action, ReplayAction::UpdateSettings { .. })
        || matches!(action, ReplayAction::Bomb(0) | ReplayAction::Equipment(0))
        || matches!(action, ReplayAction::Bomb(id) | ReplayAction::Equipment(id) if id > grenadiers_core::protocol::MAX_COUNTER)
        || matches!(
            action,
            ReplayAction::Ready {
                settings_revision: 0,
                ..
            } | ReplayAction::UpdateSpikeSettings {
                settings_revision: 0,
                ..
            }
        )
    {
        return Err(());
    }
    Ok(())
}
