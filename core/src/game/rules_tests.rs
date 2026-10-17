use super::RulesConfig;

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
