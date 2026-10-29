use super::State;
use super::storm_test_support::{actors, arena, rules, state, state_with};
use crate::game::{
    CellId, Direction, EffectiveSpeed, EnabledItems, InputAction, PlayerCount, Position,
    RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, StateError, StepError,
    StormCloudConfig, Tick, World,
};
use crate::room::{BoardSize, RoomSettings};

#[test]
fn rules_seven_accepts_every_board_and_player_count_only_with_a_seed() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            let world = World::new(arena(board, count));
            let init = actors(board, count);
            assert_eq!(
                State::new(
                    rules(EnabledItems::EMPTY, StormCloudConfig::Single),
                    world,
                    init
                ),
                Err(StateError::MissingSimulationRandomness)
            );
            let game = state_with(board, count, EnabledItems::EMPTY, [count.get() as u8; 32]);
            assert!(game.storm_cloud().is_some());
        }
    }
}

#[test]
fn disabled_cloud_preserves_seedless_construction_and_old_profiles() {
    let board = BoardSize::FifteenByThirteen;
    let count = PlayerCount::Two;
    let game = State::new(
        rules(EnabledItems::EMPTY, StormCloudConfig::Disabled),
        World::new(arena(board, count)),
        actors(board, count),
    )
    .unwrap();
    assert_eq!(game.storm_cloud(), None);
    assert!(
        State::new(
            RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
            World::new(arena(board, count)),
            actors(board, count),
        )
        .is_ok()
    );
}

#[test]
fn seeded_initial_pose_and_first_leg_are_stable() {
    let mut game = state([0; 32]);
    let initial = game.storm_cloud().unwrap();
    assert_eq!(initial.position, Position::center(CellId::new(7, 6)));
    assert_eq!(initial.direction, Direction::West);
    assert_eq!(initial.leg_remaining_ticks, 128);
    game.step(&[]).unwrap();
    let moved = game.storm_cloud().unwrap();
    assert_eq!(moved.position.x, initial.position.x - 8);
    assert_eq!(moved.position.y, initial.position.y);
    assert_eq!(moved.leg_remaining_ticks, 127);
}

#[test]
fn leg_turns_only_after_all_128_movements_and_clone_continuation_matches() {
    let mut game = state([4; 32]);
    for _ in 0..128 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.storm_cloud().unwrap().leg_remaining_ticks, 0);
    let mut clone = game.clone();
    game.step(&[]).unwrap();
    clone.step(&[]).unwrap();
    assert_eq!(game, clone);
    assert_eq!(game.storm_cloud().unwrap().leg_remaining_ticks, 127);
}

#[test]
fn contact_is_positive_area_and_applies_on_the_following_movement_tick() {
    let mut game = state([0; 32]);
    let cloud = game.storm_cloud().unwrap();
    game.actors[0].position = Position {
        x: cloud.position.x,
        y: cloud.position.y + 512 + 320,
    };
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].storm_slow_until(),
        None,
        "exact edge is not contact"
    );

    game.actors[0].position.y -= 1;
    game.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(Direction::East),
    }])
    .unwrap();
    assert_eq!(game.actors[0].storm_slow_until(), Some(Tick(182)));
    let before = game.actors[0].position.x;
    game.step(&[]).unwrap();
    assert!(game.actors[0].position.x - before <= 26);
}

#[test]
fn continuous_overlap_refreshes_without_stacking_and_expiry_is_half_open() {
    let mut game = state([0; 32]);
    game.actors[0].position = game.storm_cloud().unwrap().position;
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].storm_slow_until(), Some(Tick(181)));
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].storm_slow_until(), Some(Tick(182)));
    assert!(game.actors[0].storm_slow_active(Tick(181)));
    assert!(!game.actors[0].storm_slow_active(Tick(182)));
}

#[test]
fn storm_speed_composes_with_every_base_and_six_item_stacks() {
    for speed in [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        for stacks in 0..=6 {
            let effective = EffectiveSpeed::from_modifiers(speed, stacks, true);
            assert_eq!(effective.denominator(), 1 << (stacks + 1));
            assert_eq!(
                effective.units_per_second(),
                speed.units_per_second() >> (stacks + 1)
            );
        }
    }
}

#[test]
fn item_disabled_ignores_stacks_while_storm_slow_and_residual_zero_ticks_remain() {
    let mut plain = state([0; 32]);
    let mut disabled_stacks = plain.clone();
    let center = plain.storm_cloud().unwrap().position;
    plain.actors[0].position = center;
    disabled_stacks.actors[0].position = center;
    for _ in 0..6 {
        disabled_stacks.apply_test_effect(SeatId::One, crate::game::EffectKind::SlowFoot, Tick(0));
    }
    plain.step(&[]).unwrap();
    disabled_stacks.step(&[]).unwrap();
    let input = [SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(Direction::East),
    }];
    plain.step(&input).unwrap();
    disabled_stacks.step(&input).unwrap();
    assert_eq!(plain.actors[0].position, disabled_stacks.actors[0].position);

    let mut enabled = state_with(
        BoardSize::FifteenByThirteen,
        PlayerCount::Two,
        EnabledItems::SLOW_FOOT,
        [0; 32],
    );
    enabled.actors[0].position = enabled.storm_cloud().unwrap().position;
    for _ in 0..6 {
        enabled.apply_test_effect(SeatId::One, crate::game::EffectKind::SlowFoot, Tick(0));
    }
    enabled.step(&[]).unwrap();
    let before = enabled.actors[0].position;
    enabled.step(&input).unwrap();
    assert_eq!(enabled.actors[0].position, before);
}

#[test]
fn stopwatch_freezes_actor_but_not_cloud() {
    let mut game = state_with(
        BoardSize::FifteenByThirteen,
        PlayerCount::Two,
        EnabledItems::STOPWATCH,
        [0; 32],
    );
    game.apply_test_effect(SeatId::One, crate::game::EffectKind::Stopwatch, Tick(0));
    let actor = game.actors[1].position;
    let cloud = game.storm_cloud().unwrap().position;
    game.step(&[SeatInput {
        seat: SeatId::Two,
        action: InputAction::Press(Direction::West),
    }])
    .unwrap();
    assert_eq!(game.actors[1].position, actor);
    assert_ne!(game.storm_cloud().unwrap().position, cloud);
}

#[test]
fn shield_and_heart_do_not_prevent_storm_slow() {
    let mut game = state_with(
        BoardSize::FifteenByThirteen,
        PlayerCount::Two,
        EnabledItems::HEART_AND_SHIELD,
        [0; 32],
    );
    let cloud = game.storm_cloud().unwrap().position;
    game.actors[0].position = cloud;
    game.actors[1].position = cloud;
    game.apply_test_effect(SeatId::One, crate::game::EffectKind::Shield, Tick(0));
    game.replace_test_actor_equipment(SeatId::Two, crate::game::Equipment::Heart);
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].storm_slow_until(), Some(Tick(181)));
    assert_eq!(game.actors[1].storm_slow_until(), Some(Tick(181)));
}

#[test]
fn storm_overflow_precedes_later_pickup_overflow_and_leaves_state_unchanged() {
    let mut game = State::new_at_tick_with_assignment_and_seed(
        rules(EnabledItems::INVISIBILITY_CLOAK, StormCloudConfig::Single),
        World::new(arena(BoardSize::FifteenByThirteen, PlayerCount::Two)),
        actors(BoardSize::FifteenByThirteen, PlayerCount::Two),
        Tick(u64::MAX - 180),
        crate::game::RoundAssignment::first(),
        SimulationSeed::new([0; 32]),
    )
    .unwrap();
    let pickup = game.actors[0].position.cell().unwrap();
    game.insert_test_pickup(pickup, crate::game::PickupKind::InvisibilityCloak);
    let before = game.clone();
    assert_eq!(game.step(&[]), Err(StepError::StormSlowExpiryOverflow));
    assert_eq!(game, before);
}

#[test]
fn cloud_rng_is_redacted_and_teleporter_stream_is_unchanged() {
    let cloud = state_with(
        BoardSize::FifteenByThirteen,
        PlayerCount::Four,
        EnabledItems::TELEPORTER,
        [9; 32],
    );
    assert!(format!("{cloud:?}").contains("<redacted>"));
    assert!(!format!("{cloud:?}").contains("[9, 9"));

    let board = BoardSize::FifteenByThirteen;
    let count = PlayerCount::Four;
    let mut without = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::multiplayer_v6(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER),
        ),
        World::new(arena(board, count)),
        actors(board, count),
        Tick(0),
        crate::game::RoundAssignment::first(),
        SimulationSeed::new([9; 32]),
    )
    .unwrap();
    let mut with = cloud;
    without.commit_test_teleport([true, false, false, false], Tick(1));
    with.commit_test_teleport([true, false, false, false], Tick(1));
    let without_positions: Vec<_> = without
        .actors()
        .iter()
        .map(|actor| actor.position())
        .collect();
    let with_positions: Vec<_> = with.actors().iter().map(|actor| actor.position()).collect();
    assert_eq!(with_positions, without_positions);
}

#[test]
fn later_pickup_overflow_rolls_back_cloud_pose_and_rng_progress() {
    let board = BoardSize::FifteenByThirteen;
    let count = PlayerCount::Two;
    let mut game = State::new_at_tick_with_assignment_and_seed(
        rules(EnabledItems::INVISIBILITY_CLOAK, StormCloudConfig::Single),
        World::new(arena(board, count)),
        actors(board, count),
        Tick(u64::MAX - 600),
        crate::game::RoundAssignment::first(),
        SimulationSeed::new([0; 32]),
    )
    .unwrap();
    let cell = game.actors[0].position.cell().unwrap();
    game.insert_test_pickup(cell, crate::game::PickupKind::InvisibilityCloak);
    let before = game.clone();
    assert_eq!(
        game.step(&[]),
        Err(StepError::EffectExpiryOverflow(
            crate::game::EffectKind::InvisibilityCloak
        ))
    );
    assert_eq!(game, before);
}

#[test]
fn actor_segments_are_tested_individually_instead_of_as_a_corner_bounding_box() {
    let game = state([0; 32]);
    let cloud = game.storm_cloud().unwrap().position;
    let mut around = crate::game::MovementTrace::new(Position {
        x: cloud.x - 900,
        y: cloud.y - 900,
    });
    around.push(crate::game::MovementSegment::new(
        around.start(),
        Position {
            x: cloud.x + 900,
            y: cloud.y - 900,
        },
    ));
    around.push(crate::game::MovementSegment::new(
        Position {
            x: cloud.x + 900,
            y: cloud.y - 900,
        },
        Position {
            x: cloud.x + 900,
            y: cloud.y + 900,
        },
    ));
    let stationary = crate::game::MovementTrace::new(game.actors[1].position);
    let mut contacts = vec![false; 2];
    game.include_storm_movement_contacts(&mut contacts, &[around, stationary], &[]);
    assert_eq!(contacts, [false, false]);
}

#[test]
fn teleport_checks_endpoints_without_connecting_sweep() {
    let mut game = state_with(
        BoardSize::FifteenByThirteen,
        PlayerCount::Two,
        EnabledItems::TELEPORTER,
        [0; 32],
    );
    let cloud = game.storm_cloud().unwrap().position;
    game.actors[0].position = Position {
        x: cloud.x - 2_000,
        y: cloud.y,
    };
    game.actors[1].position = Position {
        x: cloud.x + 2_000,
        y: cloud.y,
    };
    game.commit_test_teleport([true, false], Tick(1));
    let mut contacts = vec![false; 2];
    game.include_storm_endpoint_contacts(&mut contacts);
    assert_eq!(contacts, [false, false]);
}

#[test]
fn death_and_terminal_transition_clear_slow_without_removing_cloud_pose() {
    let mut game = state([0; 32]);
    game.actors[0].storm_slow_until = Some(Tick(180));
    game.actors[0].kill();
    assert_eq!(game.actors[0].storm_slow_until(), None);

    game.actors[1].storm_slow_until = Some(Tick(180));
    let cloud = game.storm_cloud();
    assert!(game.decide_result().is_some());
    assert_eq!(game.actors[1].storm_slow_until(), None);
    assert_eq!(game.storm_cloud(), cloud);
}
