use super::*;

#[test]
fn defaults_match_the_first_configurable_profile() {
    let settings = RoomSettings::default();

    assert_eq!(settings.starting_speed(), SpeedLevel::Base);
    assert_eq!(settings.turn_assist_units(), 256);
    assert_eq!(settings.wins_to_finish(), 3);
    assert_eq!(settings.bomb_fuse_ticks(), 180);
    assert_eq!(settings.enabled_items(), EnabledItems::EMPTY);
    assert_eq!(settings, RoomSettings::DEFAULT);
    assert!(std::mem::size_of::<RoomSettings>() <= 16);
}

#[test]
fn every_boundary_and_discrete_step_is_accepted() {
    for speed in [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        for turn_assist in [64, 128, 192, 256] {
            for wins in [1, 9] {
                for fuse in (60..=300).step_by(30) {
                    assert_eq!(
                        RoomSettings::new(speed, turn_assist, wins, fuse),
                        Ok(RoomSettings {
                            starting_speed: speed,
                            turn_assist_units: turn_assist,
                            wins_to_finish: wins,
                            bomb_fuse_ticks: fuse as u16,
                            enabled_items: EnabledItems::EMPTY,
                        })
                    );
                }
            }
        }
    }
}

#[test]
fn enabled_items_are_added_without_weakening_validated_settings() {
    let settings = RoomSettings::new(SpeedLevel::Two, 128, 5, 240)
        .unwrap()
        .with_enabled_items(EnabledItems::HEART);

    assert_eq!(settings.enabled_items(), EnabledItems::HEART);
    assert_eq!(settings.bomb_fuse_ticks(), 240);
}

#[test]
fn out_of_range_and_off_step_values_are_rejected_by_field() {
    for value in [-1, 0, 1, 63, 65, 255, 257] {
        assert_eq!(
            RoomSettings::new(SpeedLevel::Base, value, 3, 180),
            Err(SettingsError::InvalidTurnAssistUnits(value))
        );
    }
    for value in [0, 10, u8::MAX] {
        assert_eq!(
            RoomSettings::new(SpeedLevel::Base, 256, value, 180),
            Err(SettingsError::InvalidWinsToFinish(value))
        );
    }
    for value in [0, 59, 61, 299, 301, u64::MAX] {
        assert_eq!(
            RoomSettings::new(SpeedLevel::Base, 256, 3, value),
            Err(SettingsError::InvalidBombFuseTicks(value))
        );
    }
}
