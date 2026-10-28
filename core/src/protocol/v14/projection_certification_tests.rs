use super::*;
use crate::game::{EventAudience, InputAction, SeatId, SeatInput};
use crate::protocol::{Cell, EntityId, Tick};

pub(super) fn bind_result(
    capture: &projection_facts::ExpandedCapture,
) -> Result<BoundPublication, ProjectionError> {
    let count = crate::game::PlayerCount::try_from(capture.world.actors.len()).unwrap();
    let mut room = tests::running_room(count, capture.world.physical_tick);
    room.settings = GameSettings::new(0, 256, 3, 180, capture.world.enabled_items).unwrap();
    let generation = projection_validation_tests::generation(count, capture.rules.enabled_items());
    bind_expanded(&room, room.round_id, capture, Some(generation))
}

fn timed_event_capture(blast: BlastKind) -> projection_facts::ExpandedCapture {
    let mut capture = projection_validation_tests::valid_capture(crate::game::PlayerCount::Four);
    let tick = capture.world.physical_tick;
    let placed_at = Tick::new(tick.get() - 1).unwrap();
    let bomb = BombView {
        id: EntityId::new(200),
        cell: Cell::new(2, 1).unwrap(),
        owner: capture.world.actors[0].view.id,
        range: 2,
        placed_at,
        trigger: BombTrigger::timed(179, false).unwrap(),
        blast,
    };
    capture.reports = vec![projection_facts::ReportFacts {
        tick,
        trace_count: 4,
        forced_movements: Vec::new(),
        events: vec![projection_facts::EventFacts {
            tick,
            audience: EventAudience::all(crate::game::PlayerCount::Four),
            kind: EventKind::BombDetonated { bomb },
            bomb_history: Some(projection_facts::BombHistory {
                id: bomb.id,
                trigger: projection_facts::TriggerHistory::Timed {
                    captured_fuse_ticks: 180,
                    paused_ticks: 0,
                    detonates_at: Tick::new(placed_at.get() + 180).unwrap(),
                    paused: false,
                },
                motion: None,
                has_been_kicked: false,
                has_been_punched: false,
            }),
        }],
    }];
    capture
}

#[test]
fn removed_timed_event_bombs_still_require_their_blast_item() {
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        let capture = timed_event_capture(blast);
        assert_eq!(
            projection_validation::validate(&capture.world, &capture.reports, &capture.rules),
            Err(ProjectionError::InvalidEvent)
        );
        assert_eq!(
            bind_result(&capture),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

fn stopwatch_capture(hidden_bombs: bool) -> projection_facts::ExpandedCapture {
    let bits = crate::game::EnabledItems::STOPWATCH.bits()
        | if hidden_bombs {
            crate::game::EnabledItems::INVISIBLE_BOMBS.bits()
        } else {
            0
        };
    let enabled = crate::game::EnabledItems::from_bits(bits).unwrap();
    let mut state = tests::multiplayer_state_with_items(crate::game::PlayerCount::Four, enabled);
    state.apply_test_effect(
        SeatId::One,
        crate::game::EffectKind::Stopwatch,
        crate::game::Tick(0),
    );
    if hidden_bombs {
        state.apply_test_effect(
            SeatId::Two,
            crate::game::EffectKind::InvisibleBombs,
            crate::game::Tick(0),
        );
    }
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    capture_expanded(&state, &[report]).unwrap()
}

#[test]
fn world_timed_pause_equals_a_living_stopwatch_holder_even_when_bombs_are_hidden() {
    let capture = stopwatch_capture(true);
    projection_validation_tests::assert_valid(&capture);
    let publication = bind_result(&capture).unwrap();
    assert!(
        project_recipient(&publication, Seat::Two)
            .unwrap()
            .state
            .bombs
            .is_empty()
    );

    let mut forged = capture.clone();
    forged.world.actors[0].view.effects = EffectViews::default();
    assert_eq!(
        projection_validation::validate(&forged.world, &forged.reports, &forged.rules),
        Err(ProjectionError::InvalidBombs)
    );

    let mut inverse = capture.clone();
    let crate::protocol::v7::BombTriggerObservation::Timed {
        remaining_ticks, ..
    } = inverse.world.bombs[0].trigger.observation()
    else {
        unreachable!()
    };
    inverse.world.bombs[0].trigger = BombTrigger::timed(remaining_ticks, false).unwrap();
    let projection_facts::TriggerHistory::Timed { ref mut paused, .. } =
        inverse.world.bomb_history[0].trigger
    else {
        unreachable!()
    };
    *paused = false;
    assert_eq!(
        projection_validation::validate(&inverse.world, &inverse.reports, &inverse.rules),
        Err(ProjectionError::InvalidBombs)
    );
}

#[test]
fn consistent_public_and_private_pause_flags_without_a_holder_are_rejected() {
    let enabled = crate::game::EnabledItems::STOPWATCH;
    let mut state = tests::multiplayer_state_with_items(crate::game::PlayerCount::Four, enabled);
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let mut capture = capture_expanded(&state, &[report]).unwrap();
    let crate::protocol::v7::BombTriggerObservation::Timed {
        remaining_ticks, ..
    } = capture.world.bombs[0].trigger.observation()
    else {
        unreachable!()
    };
    capture.world.bombs[0].trigger = BombTrigger::timed(remaining_ticks, true).unwrap();
    let projection_facts::TriggerHistory::Timed { ref mut paused, .. } =
        capture.world.bomb_history[0].trigger
    else {
        unreachable!()
    };
    *paused = true;
    assert_eq!(
        projection_validation::validate(&capture.world, &capture.reports, &capture.rules),
        Err(ProjectionError::InvalidBombs)
    );
}

#[test]
fn public_elimination_and_round_end_audiences_are_canonical_for_all_players() {
    let mut state = tests::multiplayer_state(crate::game::PlayerCount::Three);
    let mut report = state.step(&[]).unwrap();
    report.events.extend(
        state
            .eliminate_departed([true, false, false, false])
            .unwrap(),
    );
    let capture = capture_expanded(&state, &[report]).unwrap();
    projection_validation_tests::assert_valid(&capture);
    let mut private = capture.clone();
    let event = private.reports[0]
        .events
        .iter_mut()
        .find(|event| matches!(event.kind, EventKind::ActorEliminated { .. }))
        .unwrap();
    event.audience = EventAudience::only(SeatId::One);
    assert_eq!(
        projection_validation::validate(&private.world, &private.reports, &private.rules),
        Err(ProjectionError::InvalidEvent)
    );
    assert_eq!(
        bind_result(&private),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut state = tests::multiplayer_state(crate::game::PlayerCount::Three);
    let mut report = state.step(&[]).unwrap();
    report.events.extend(
        state
            .eliminate_departed([true, true, false, false])
            .unwrap(),
    );
    let mut completed = capture_expanded(&state, &[report]).unwrap();
    completed.reports[0]
        .events
        .iter_mut()
        .find(|event| matches!(event.kind, EventKind::RoundEnded { .. }))
        .unwrap()
        .audience = EventAudience::only(SeatId::Three);
    assert_eq!(
        projection_validation::validate(&completed.world, &completed.reports, &completed.rules),
        Err(ProjectionError::InvalidEvent)
    );
}

#[test]
fn two_real_disjoint_shoves_pass_and_overlapping_participants_fail() {
    let enabled = crate::game::EnabledItems::from_bits(
        crate::game::EnabledItems::BRACELET.bits() | crate::game::EnabledItems::TELEPORTER.bits(),
    )
    .unwrap();
    let mut state = tests::multiplayer_state_with_items(crate::game::PlayerCount::Four, enabled);
    let placements = [
        (
            crate::game::Position { x: 4_096, y: 3_072 },
            crate::game::Direction::East,
        ),
        (
            crate::game::Position { x: 4_736, y: 3_072 },
            crate::game::Direction::West,
        ),
        (
            crate::game::Position { x: 4_096, y: 7_168 },
            crate::game::Direction::East,
        ),
        (
            crate::game::Position { x: 4_736, y: 7_168 },
            crate::game::Direction::West,
        ),
    ];
    for (seat, (position, heading)) in SeatId::ALL.into_iter().zip(placements) {
        state.set_test_actor_pose(seat, position, heading);
    }
    state.replace_test_actor_equipment(SeatId::One, crate::game::Equipment::Bracelet);
    state.replace_test_actor_equipment(SeatId::Three, crate::game::Equipment::Bracelet);
    let report = state
        .step(&[
            SeatInput {
                seat: SeatId::Three,
                action: InputAction::Press(crate::game::Direction::East),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(crate::game::Direction::East),
            },
        ])
        .unwrap();
    assert_eq!(report.forced_movements.len(), 2);
    let capture = capture_expanded(&state, &[report]).unwrap();
    projection_validation_tests::assert_valid(&capture);

    let mut shared_pusher = capture.clone();
    shared_pusher.reports[0].forced_movements[1].pusher = Seat::One;
    shared_pusher.reports[0].forced_movements[1].victim = Seat::Three;
    shared_pusher.world.actors[2].view.position = shared_pusher.reports[0].forced_movements[1].end;
    assert!(
        projection_validation::validate(
            &shared_pusher.world,
            &shared_pusher.reports,
            &shared_pusher.rules
        )
        .is_err()
    );

    let mut victim_as_pusher = capture.clone();
    victim_as_pusher.reports[0].forced_movements[1].pusher = Seat::Two;
    victim_as_pusher.world.actors[1].last_successful_shove_tick =
        Some(victim_as_pusher.reports[0].tick);
    assert!(
        projection_validation::validate(
            &victim_as_pusher.world,
            &victim_as_pusher.reports,
            &victim_as_pusher.rules
        )
        .is_err()
    );

    let mut unrelated_teleport = capture.clone();
    let tick = unrelated_teleport.reports[0].tick;
    unrelated_teleport.world.last_teleport_tick = Some(tick);
    for index in [2, 3] {
        unrelated_teleport.world.actors[index].last_teleport_tick = Some(tick);
        unrelated_teleport.world.actors[index]
            .view
            .last_teleport_tick = Some(tick);
    }
    unrelated_teleport.world.actors[1].view.position =
        unrelated_teleport.reports[0].forced_movements[0].start;
    assert_eq!(
        projection_validation::validate(
            &unrelated_teleport.world,
            &unrelated_teleport.reports,
            &unrelated_teleport.rules
        ),
        Err(ProjectionError::InvalidReports)
    );
}

#[test]
fn per_actor_teleport_markers_prove_latest_pair_and_preserve_older_pairs() {
    let mut state = tests::multiplayer_state_with_items(
        crate::game::PlayerCount::Four,
        crate::game::EnabledItems::TELEPORTER,
    );
    state.step(&[]).unwrap();
    state.commit_test_teleport([true, false, false, false], crate::game::Tick(1));
    let first = capture_expanded(&state, &[]).unwrap();
    projection_validation_tests::assert_valid(&first);
    assert_eq!(
        first
            .world
            .actors
            .iter()
            .filter(|actor| actor.last_teleport_tick == first.world.last_teleport_tick)
            .count(),
        2
    );

    state.step(&[]).unwrap();
    state.commit_test_teleport([false, false, true, false], crate::game::Tick(2));
    let sequential = capture_expanded(&state, &[]).unwrap();
    projection_validation_tests::assert_valid(&sequential);
    assert_eq!(
        sequential.world.last_teleport_tick,
        Some(Tick::new(2).unwrap())
    );
    assert_eq!(
        sequential
            .world
            .actors
            .iter()
            .filter(|actor| actor.last_teleport_tick == sequential.world.last_teleport_tick)
            .count(),
        2
    );

    let mut wrong_global = sequential.clone();
    wrong_global.world.last_teleport_tick = Some(Tick::new(1).unwrap());
    assert_eq!(
        projection_validation::validate(
            &wrong_global.world,
            &wrong_global.reports,
            &wrong_global.rules
        ),
        Err(ProjectionError::InvalidClock)
    );

    let mut third_latest = sequential.clone();
    third_latest.world.actors[0].last_teleport_tick = third_latest.world.last_teleport_tick;
    third_latest.world.actors[0].view.last_teleport_tick = third_latest.world.last_teleport_tick;
    assert_eq!(
        projection_validation::validate(
            &third_latest.world,
            &third_latest.reports,
            &third_latest.rules
        ),
        Err(ProjectionError::InvalidClock)
    );
}
