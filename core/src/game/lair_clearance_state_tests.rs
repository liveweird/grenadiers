use crate::game::{
    ActorInit, ArenaConstructionProfile, ArenaGenerationConfig, Direction, EnabledItems, EntityId,
    MobNavigationPolicy, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId,
    SimulationSeed, SpeedLevel, SpikePitLayoutPolicy, SpikePitPhase, State, StormCloudConfig, Tick,
    World, generate_profiled_multiplayer_arena, lair_layout_v1, position_contacts_spike_pit,
};
use crate::room::{
    BoardSize, RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig,
    SpikePitSettings,
};

#[test]
fn generated_clearance_spawns_both_mobs_with_progress_and_no_exit_pit_contact() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let assignment = RoundAssignment::first();
        let generated = generate_profiled_multiplayer_arena(
            [41; 32],
            EnabledItems::EMPTY,
            ArenaGenerationConfig {
                player_count: PlayerCount::Two,
                board,
                assignment,
            },
            ArenaConstructionProfile::LairClearanceV5,
            true,
        )
        .unwrap();
        let (world, _) = World::from_generated(generated).unwrap();
        let actors = SeatId::DUEL.map(|seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(world.arena().spawns()[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        });
        let lair_settings = RespawnLairSettings::DEFAULT;
        let rules = RulesConfig::environment_v10_with_mob_navigation(
            RoomSettings::DEFAULT,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Enabled(lair_settings),
            SpikePitConfig::Enabled(SpikePitSettings::new(6, 60).unwrap()),
            MobNavigationPolicy::TimedEscapeV3,
        )
        .with_spike_pit_layout_policy(SpikePitLayoutPolicy::LairClearanceV2);
        let mut game = State::new_at_tick_with_assignment_and_seed(
            rules,
            world,
            actors,
            Tick(0),
            assignment,
            SimulationSeed::new([73; 32]),
        )
        .unwrap();
        game.tick = Tick(lair_settings.spawn_ticks() - 1);
        game.step(&[]).unwrap();
        let births = game.mobs();
        assert_eq!(births.len(), 2);
        assert!(births.iter().zip(lair_layout_v1(board)).all(|(mob, lair)| {
            let cell = mob.position.cell().unwrap();
            cell.column.abs_diff(lair.column) + cell.row.abs_diff(lair.row) == 1
        }));
        let birth_pits = game.spike_pits().unwrap();
        assert!(births.iter().all(|mob| {
            birth_pits
                .iter()
                .all(|pit| !position_contacts_spike_pit(mob.position, pit.cell))
        }));

        for _ in 0..12 {
            game.step(&[]).unwrap();
            let pits = game.spike_pits().unwrap();
            assert!(pits.iter().all(|pit| pit.phase == SpikePitPhase::Idle));
            assert!(game.mobs().iter().all(|mob| {
                pits.iter()
                    .all(|pit| !position_contacts_spike_pit(mob.position, pit.cell))
            }));
        }
        let progressed = game.mobs();
        assert!(
            progressed
                .iter()
                .zip(births)
                .all(|(after, before)| after.position != before.position)
        );
    }
}

#[test]
fn layout_policy_versions_and_new_timing_defaults_are_explicit() {
    assert_eq!(SpikePitLayoutPolicy::LegacyV1.version(), 1);
    assert_eq!(SpikePitLayoutPolicy::LairClearanceV2.version(), 2);
    assert_eq!(SpikePitSettings::DEFAULT.warning_ticks(), 60);
    assert_eq!(SpikePitSettings::DEFAULT.raised_ticks(), 300);

    let legacy = RulesConfig::environment_v10(
        RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
    );
    let clearance = legacy.with_spike_pit_layout_policy(SpikePitLayoutPolicy::LairClearanceV2);
    assert_eq!(
        legacy.spike_pit_layout_policy(),
        SpikePitLayoutPolicy::LegacyV1
    );
    assert_eq!(
        clearance.spike_pit_layout_policy(),
        SpikePitLayoutPolicy::LairClearanceV2
    );
    assert_eq!(clearance.version(), legacy.version());
}
