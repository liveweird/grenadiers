use super::RulesConfig;
use crate::game::{EnabledItems, MobNavigationPolicy, SpeedLevel};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

#[test]
fn classic_v1_exposes_the_fixed_geometry_and_tick_parameters() {
    let rules = RulesConfig::classic_v1();

    assert_eq!(rules.version(), 1);
    assert_eq!(rules.tick_rate(), 60);
    assert_eq!(rules.units_per_cell(), 1_024);
    assert_eq!(rules.half_footprint(), 320);
    assert_eq!(rules.turn_assist(), 128);
    assert_eq!(rules.bomb_capacity(), 1);
    assert_eq!(rules.bomb_range(), 2);
    assert!(!rules.pickup_collection());
    assert_eq!(rules.bomb_capacity_cap(), 8);
    assert_eq!(rules.bomb_range_cap(), 8);
    assert_eq!(rules.bomb_fuse_ticks(), 180);
    assert_eq!(rules.flame_ticks(), 30);
    assert_eq!(rules.round_ticks(), 10_800);
}

#[test]
fn expanded_profile_binds_enabled_items_at_version_five() {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART);
    let classic = RulesConfig::configurable_v4(settings);
    let expanded = RulesConfig::expanded_v5(settings);

    assert_eq!(classic.enabled_items(), EnabledItems::EMPTY);
    assert_eq!(expanded.version(), 5);
    assert_eq!(expanded.enabled_items(), EnabledItems::HEART);
}

#[test]
fn environment_v9_uses_fixed_one_respawn_lairs() {
    let rules = RulesConfig::environment_v9(
        RoomSettings::DEFAULT,
        super::StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
    );

    assert_eq!(rules.version(), 9);
    let crate::room::LairConfig::Enabled(settings) = rules.lair_config() else {
        panic!("rules9 lairs should be enabled");
    };
    assert_eq!(settings.spawn_ticks(), 300);
    assert_eq!(settings.mobs_per_lair(), 1);
    assert_eq!(rules.mob_navigation_policy(), MobNavigationPolicy::LegacyV1);
}

#[test]
fn environment_v9_can_select_escape_navigation_explicitly() {
    let rules = RulesConfig::environment_v9_with_mob_navigation(
        RoomSettings::DEFAULT,
        super::StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        MobNavigationPolicy::EscapeV2,
    );

    assert_eq!(MobNavigationPolicy::LegacyV1.version(), 1);
    assert_eq!(MobNavigationPolicy::EscapeV2.version(), 2);
    assert_eq!(rules.mob_navigation_policy(), MobNavigationPolicy::EscapeV2);
}

#[test]
fn environment_v9_can_select_timed_escape_without_changing_constructor_defaults() {
    let legacy = RulesConfig::environment_v9(
        RoomSettings::DEFAULT,
        super::StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
    );
    let timed = RulesConfig::environment_v9_with_mob_navigation(
        RoomSettings::DEFAULT,
        super::StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
        MobNavigationPolicy::TimedEscapeV3,
    );

    assert_eq!(MobNavigationPolicy::TimedEscapeV3.version(), 3);
    assert_eq!(
        legacy.mob_navigation_policy(),
        MobNavigationPolicy::LegacyV1
    );
    assert_eq!(
        timed.mob_navigation_policy(),
        MobNavigationPolicy::TimedEscapeV3
    );
    assert_eq!(timed.version(), legacy.version());
    assert_eq!(timed.lair_config(), legacy.lair_config());
}

#[test]
fn environment_v10_is_the_only_profile_with_opt_in_spike_pits() {
    let settings = RoomSettings::DEFAULT;
    let lairs = RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT);
    let ninth = RulesConfig::environment_v9(settings, super::StormCloudConfig::Disabled, lairs);
    let tenth = RulesConfig::environment_v10_with_mob_navigation(
        settings,
        super::StormCloudConfig::Disabled,
        lairs,
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        MobNavigationPolicy::EscapeV2,
    );

    assert_eq!(ninth.spike_pit_config(), SpikePitConfig::Disabled);
    assert_eq!(tenth.version(), 10);
    assert_eq!(
        tenth.spike_pit_config(),
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT)
    );
    assert_eq!(tenth.mob_navigation_policy(), MobNavigationPolicy::EscapeV2);
    assert_eq!(tenth.lair_config(), ninth.lair_config());
}

#[test]
fn classic_v2_only_enables_domain_pickup_collection() {
    let first = RulesConfig::classic_v1();
    let second = RulesConfig::classic_v2();

    assert_eq!(second.version(), 2);
    assert!(second.pickup_collection());
    assert_eq!(second.bomb_capacity(), first.bomb_capacity());
    assert_eq!(second.bomb_range(), first.bomb_range());
    assert_eq!(second.bomb_fuse_ticks(), first.bomb_fuse_ticks());
    assert_eq!(second.flame_ticks(), first.flame_ticks());
    assert_eq!(second.round_ticks(), first.round_ticks());
}

#[test]
fn classic_v3_only_widens_the_turn_assist_window() {
    let second = RulesConfig::classic_v2();
    let third = RulesConfig::classic_v3();

    assert_eq!(third.version(), 3);
    assert_eq!(third.turn_assist(), 256);
    assert_eq!(second.turn_assist(), 128);
    assert_eq!(third.tick_rate(), second.tick_rate());
    assert_eq!(third.units_per_cell(), second.units_per_cell());
    assert_eq!(third.half_footprint(), second.half_footprint());
    assert_eq!(third.pickup_collection(), second.pickup_collection());
    assert_eq!(third.bomb_capacity(), second.bomb_capacity());
    assert_eq!(third.bomb_range(), second.bomb_range());
    assert_eq!(third.bomb_fuse_ticks(), second.bomb_fuse_ticks());
    assert_eq!(third.flame_ticks(), second.flame_ticks());
    assert_eq!(third.round_ticks(), second.round_ticks());
}

#[test]
fn configurable_v4_applies_only_simulation_owned_room_settings() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 9, 60).unwrap();
    let third = RulesConfig::classic_v3();
    let fourth = RulesConfig::configurable_v4(settings);

    assert_eq!(fourth.version(), 4);
    assert_eq!(fourth.turn_assist(), 64);
    assert_eq!(fourth.bomb_fuse_ticks(), 60);
    assert_eq!(fourth.tick_rate(), third.tick_rate());
    assert_eq!(fourth.units_per_cell(), third.units_per_cell());
    assert_eq!(fourth.half_footprint(), third.half_footprint());
    assert_eq!(fourth.pickup_collection(), third.pickup_collection());
    assert_eq!(fourth.bomb_capacity(), third.bomb_capacity());
    assert_eq!(fourth.bomb_range(), third.bomb_range());
    assert_eq!(fourth.flame_ticks(), third.flame_ticks());
    assert_eq!(fourth.round_ticks(), third.round_ticks());

    assert_eq!(settings.starting_speed(), SpeedLevel::Four);
    assert_eq!(settings.wins_to_finish(), 9);
}
