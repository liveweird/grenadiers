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
    assert_eq!(rules.bomb_fuse_ticks(), 180);
    assert_eq!(rules.flame_ticks(), 30);
}
