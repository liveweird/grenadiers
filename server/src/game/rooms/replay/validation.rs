use grenadiers_core::game::{EXPANDED_GENERATOR_VERSION, EnabledItems, GENERATOR_VERSION, SeatId};

use super::{
    record::{
        CLASSIC_V2_RULES, CLASSIC_V3_RULES, CONFIGURABLE_V4_RULES, EXPANDED_FORMAT_VERSION,
        EXPANDED_V5_RULES, FORMAT_VERSION, GENERATED_V1, GENERATED_V2, Journal,
        M5_LIFECYCLE_PROFILE, M6_LIFECYCLE_PROFILE, ReplayStatus, SOLO_FORMAT_VERSION,
    },
    runner::ReplayError,
};

pub(super) fn validate_header(journal: &Journal) -> Result<(), ReplayError> {
    match journal.status {
        ReplayStatus::Complete => {}
        ReplayStatus::Truncated { .. } => return Err(ReplayError::Truncated),
        ReplayStatus::Incomplete { .. } => return Err(ReplayError::Incomplete),
        ReplayStatus::UnsupportedProfile => return Err(ReplayError::UnsupportedProfile),
    }
    let header = journal.header;
    if !matches!(
        header.format_version,
        FORMAT_VERSION | EXPANDED_FORMAT_VERSION | SOLO_FORMAT_VERSION
    ) {
        return Err(ReplayError::UnsupportedFormat);
    }
    if !matches!(
        header.lifecycle_profile,
        M5_LIFECYCLE_PROFILE | M6_LIFECYCLE_PROFILE
    ) {
        return Err(ReplayError::UnsupportedLifecycle);
    }
    let valid_rules = match header.format_version {
        FORMAT_VERSION => matches!(
            header.rules_version,
            CLASSIC_V2_RULES | CLASSIC_V3_RULES | CONFIGURABLE_V4_RULES
        ),
        EXPANDED_FORMAT_VERSION | SOLO_FORMAT_VERSION => header.rules_version == EXPANDED_V5_RULES,
        _ => false,
    };
    if !valid_rules {
        return Err(ReplayError::UnsupportedRules);
    }
    if header.lifecycle_profile == M5_LIFECYCLE_PROFILE && header.rules_version != CLASSIC_V2_RULES
    {
        return Err(ReplayError::UnsupportedLifecycle);
    }
    let expected_generator = if header.format_version == FORMAT_VERSION {
        GENERATED_V1
    } else {
        GENERATED_V2
    };
    if header.generator_version != expected_generator
        || expected_generator == GENERATED_V1 && expected_generator != GENERATOR_VERSION
        || expected_generator == GENERATED_V2 && expected_generator != EXPANDED_GENERATOR_VERSION
    {
        return Err(ReplayError::UnsupportedGenerator);
    }
    if header.initial_tick.0 != 0 {
        return Err(ReplayError::UnsupportedInitialTick);
    }
    validate_provenance(header)?;
    let invalid_configuration = if matches!(
        header.rules_version,
        CONFIGURABLE_V4_RULES | EXPANDED_V5_RULES
    ) {
        header.initial_settings_editor != Some(SeatId::One)
            || header.rules_version == CONFIGURABLE_V4_RULES
                && header.initial_settings.enabled_items() != EnabledItems::EMPTY
    } else {
        header.initial_settings != legacy_default_settings(header.rules_version)?
            || header.initial_settings_editor.is_some()
    };
    if header.initial_settings_revision != 1 || invalid_configuration {
        return Err(ReplayError::UnsupportedRules);
    }
    Ok(())
}

fn validate_provenance(header: super::record::Header) -> Result<(), ReplayError> {
    let valid = match header.format_version {
        FORMAT_VERSION | EXPANDED_FORMAT_VERSION => {
            header.mode == grenadiers_core::room::RoomMode::HumanDuel && header.solo.is_none()
        }
        SOLO_FORMAT_VERSION => header.solo.is_some_and(|solo| {
            header.mode == grenadiers_core::room::RoomMode::Solo(solo.profile)
                && matches!(
                    solo.profile,
                    grenadiers_core::room::CpuProfile::INITIAL
                        | grenadiers_core::room::CpuProfile::PULSE_V2
                        | grenadiers_core::room::CpuProfile::ROUTE_V3
                        | grenadiers_core::room::CpuProfile::PLANT_V4
                        | grenadiers_core::room::CpuProfile::GOALS_V5
                        | grenadiers_core::room::CpuProfile::WEAPONS_V6
                        | grenadiers_core::room::CpuProfile::WALL_PASS_V7
                        | grenadiers_core::room::CpuProfile::RESPONSIVE_V8
                        | grenadiers_core::room::CpuProfile::ORIGIN_V9
                        | grenadiers_core::room::CpuProfile::GUIDED_V10
                        | grenadiers_core::room::CpuProfile::CLEAR_GAP_V11
                        | grenadiers_core::room::CpuProfile::CURRENT
                )
                && solo.seat == SeatId::Two
                && solo.policy_version == solo.profile.policy.version()
                && header.initial_seats == [false, true]
        }),
        _ => false,
    };
    valid.then_some(()).ok_or(ReplayError::UnsupportedFormat)
}

fn legacy_default_settings(rules: u16) -> Result<grenadiers_core::room::RoomSettings, ReplayError> {
    let version = grenadiers_core::protocol::RulesVersion::try_from(rules)
        .map_err(|_| ReplayError::UnsupportedRules)?;
    Ok(grenadiers_core::protocol::v6::GameSettings::for_rules(version).domain())
}
