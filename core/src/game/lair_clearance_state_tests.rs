use crate::game::{
    ActorInit, ArenaConstructionProfile, ArenaGenerationConfig, Direction, EnabledItems, EntityId,
    MobNavigationPolicy, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId,
    SimulationSeed, SpeedLevel, SpikePitPhase, State, StormCloudConfig, Tick, World,
    lair_layout_v1, position_contacts_spike_pit,
};
use crate::room::{
    BoardSize, BombCapacityDropMultiplier, RespawnLairConfig, RespawnLairSettings, RoomSettings,
    SpikePitConfig, SpikePitSettings,
};

#[test]
fn combined_construction_spawns_mobs_with_progress_and_safe_initial_pits() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let assignment = RoundAssignment::first();
        let profile = ArenaConstructionProfile::WeightedLairClearanceV7;
        let generated = profile
            .generate(
                [41; 32],
                EnabledItems::EMPTY,
                ArenaGenerationConfig {
                    player_count: PlayerCount::Two,
                    board,
                    assignment,
                },
                BombCapacityDropMultiplier::DEFAULT,
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
        .with_spike_pit_layout_policy(profile.spike_pit_layout_policy());
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
        assert!(births.iter().all(|mob| {
            game.spike_pits().unwrap().iter().all(|pit| {
                pit.phase == SpikePitPhase::Idle
                    && !position_contacts_spike_pit(mob.position, pit.cell)
            })
        }));
        for _ in 0..12 {
            game.step(&[]).unwrap();
        }
        assert!(
            game.mobs()
                .iter()
                .zip(births)
                .all(|(after, before)| after.position != before.position)
        );
    }
}

#[test]
fn construction_profiles_bind_generator_and_pit_versions() {
    for (profile, generator, pits) in [
        (ArenaConstructionProfile::LairClearanceV5, 5, 2),
        (ArenaConstructionProfile::WeightedV6, 6, 1),
        (ArenaConstructionProfile::WeightedLairClearanceV7, 7, 2),
    ] {
        assert_eq!(
            profile.generator_version(BoardSize::ElevenByNine),
            generator
        );
        assert_eq!(profile.spike_pit_layout_policy().version(), pits);
    }
    assert_eq!(
        ArenaConstructionProfile::ReflectedV3V4.generator_version(BoardSize::FifteenByThirteen),
        3
    );
    assert_eq!(
        ArenaConstructionProfile::ReflectedV3V4.generator_version(BoardSize::ElevenByNine),
        4
    );
}
