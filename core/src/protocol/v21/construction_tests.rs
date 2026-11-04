use super::*;
use crate::{
    game::{
        ActorInit, ArenaConstructionProfile, ArenaGenerationConfig, Direction, EnabledItems,
        EntityId, PlayerCount, Position, PunchActivationPolicy, RoundAssignment, RulesConfig,
        SeatId, SpeedLevel, State, StormCloudConfig, World,
    },
    protocol::Counter,
    room::{
        BoardSize as DomainBoard, BombCapacityDropMultiplier, RespawnLairConfig,
        RoomSettings as DomainSettings, SpikePitConfig,
    },
};

fn scene(
    construction: ArenaConstructionProfile,
    multiplier: u64,
) -> (RoomView, ExpandedCapture, crate::game::GenerationRecord) {
    let multiplier = BombCapacityDropMultiplier::new(multiplier).unwrap();
    let generated = construction
        .generate(
            [21; 32],
            EnabledItems::EMPTY,
            ArenaGenerationConfig {
                player_count: PlayerCount::Two,
                board: DomainBoard::FifteenByThirteen,
                assignment: RoundAssignment::first(),
            },
            multiplier,
            false,
        )
        .unwrap();
    let (world, generation) = World::from_generated(generated).unwrap();
    let spawns = world.arena().spawns().to_vec();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(21 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let settings = DomainSettings::DEFAULT.with_bomb_capacity_drop_multiplier(multiplier);
    let rules = RulesConfig::environment_v10(
        settings,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        SpikePitConfig::Disabled,
    )
    .with_punch_activation_policy(PunchActivationPolicy::ContactV2)
    .with_spike_pit_layout_policy(construction.spike_pit_layout_policy());
    let state = State::new(rules, world, actors).unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();
    let legacy = crate::protocol::v19::server_conformance_tests::room(
        state.tick().0,
        RoomPhase::Running,
        false,
    );
    let settings = RoomSettings::new(
        legacy.settings.base,
        legacy.settings.environment,
        multiplier,
    );
    (RoomView::from_v20(legacy, settings), capture, generation)
}

#[test]
fn weighted_construction_bindings_reject_crossed_generator_pit_and_setting_identities() {
    let profiles = [
        ArenaConstructionProfile::WeightedV6,
        ArenaConstructionProfile::WeightedLairClearanceV7,
    ];
    for multiplier in [1, 5, 10] {
        for profile in profiles {
            let (room, capture, generation) = scene(profile, multiplier);
            let epoch = Counter::new(1).unwrap();
            bind_expanded_with_construction(&room, epoch, &capture, generation, profile).unwrap();
            for other in profiles
                .into_iter()
                .filter(|candidate| *candidate != profile)
            {
                assert!(
                    bind_expanded_with_construction(&room, epoch, &capture, generation, other)
                        .is_err()
                );
                let (_, crossed_capture, _) = scene(other, multiplier);
                assert!(
                    bind_expanded_with_construction(
                        &room,
                        epoch,
                        &crossed_capture,
                        generation,
                        profile
                    )
                    .is_err()
                );
            }
            let wrong_multiplier = if multiplier == 5 { 10 } else { 5 };
            let (wrong_room, _, _) = scene(profile, wrong_multiplier);
            assert!(
                bind_expanded_with_construction(&wrong_room, epoch, &capture, generation, profile)
                    .is_err()
            );
            assert_eq!(
                bind_expanded(&room, epoch, &capture, generation).is_ok(),
                profile == ArenaConstructionProfile::WeightedV6
            );
        }
    }
}

#[test]
fn clearance_admission_uses_the_expected_profile_and_preserves_recipient_privacy() {
    let profile = ArenaConstructionProfile::WeightedLairClearanceV7;
    let (room, capture, generation) = scene(profile, 5);
    let projection = || AdmissionProjection {
        room: &room,
        state_epoch: Some(Counter::new(1).unwrap()),
        capture: Some(&capture),
        generation: Some(generation),
    };
    let personal = |recipient| PersonalAdmission {
        recipient,
        last_action_id: None,
        sequence: Counter::new(2).unwrap(),
        resume_token: ResumeToken::new(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap(),
    };
    let encoded =
        prepare_admission_with_construction(projection(), personal(Seat::One), profile).unwrap();
    let message: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(message["room"]["settings"]["bombCapacityDropMultiplier"], 5);
    assert_eq!(message["youSeat"], "one");
    assert!(prepare_admission(projection(), personal(Seat::One)).is_err());
    assert!(
        prepare_admission_with_construction(projection(), personal(Seat::Two), profile).is_err()
    );
    for unsupported in [
        ArenaConstructionProfile::ReflectedV3V4,
        ArenaConstructionProfile::LairClearanceV5,
    ] {
        assert!(
            prepare_admission_with_construction(projection(), personal(Seat::One), unsupported)
                .is_err()
        );
    }
}
