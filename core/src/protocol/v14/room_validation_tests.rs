use super::*;

#[test]
fn phase_scores_and_creator_editor_are_checked() {
    let tick = crate::protocol::Tick::new(1).unwrap();
    let room = super::tests::running_room(crate::game::PlayerCount::Four, tick);
    assert!(validate_room_view(&room).is_ok());

    let mut zero_round = room.clone();
    zero_round.round_number = RoundCount::new(0).unwrap();
    assert_eq!(
        validate_room_view(&zero_round),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut waiting_score = room.clone();
    waiting_score.phase = RoomPhase::Waiting;
    waiting_score.round_number = RoundCount::new(0).unwrap();
    waiting_score.scores[0].wins = Wins::new(1).unwrap();
    assert_eq!(
        validate_room_view(&waiting_score),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut expired_creator = room;
    expired_creator.seats[0].connection = ConnectionStatus::Expired;
    assert_eq!(
        validate_room_view(&expired_creator),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut wrong_map = super::tests::running_room(crate::game::PlayerCount::Four, tick);
    wrong_map.map_id = MapId::GeneratedV1;
    assert_eq!(
        validate_room_view(&wrong_map),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut wrong_availability = super::tests::running_room(crate::game::PlayerCount::Four, tick);
    wrong_availability.available_items = ItemSet::empty();
    assert_eq!(
        validate_room_view(&wrong_availability),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut noncanonical_classic = super::tests::running_room(crate::game::PlayerCount::Two, tick);
    noncanonical_classic.rules_version = RulesVersion::CLASSIC_V1;
    noncanonical_classic.map_id = MapId::FixedV1;
    noncanonical_classic.available_items = ItemSet::empty();
    assert_eq!(
        validate_room_view(&noncanonical_classic),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn four_seat_forfeit_preserves_a_distinct_physical_round_winner() {
    use crate::game::{ArenaGenerationConfig, RoundAssignment};

    let mut state = super::tests::multiplayer_state(crate::game::PlayerCount::Four);
    let report = state.step(&[]).unwrap();
    let mut capture = capture_expanded(&state, &[report]).unwrap();
    for actor in &mut capture.world.actors {
        actor.view.alive = actor.view.seat == Seat::Two;
    }
    capture.world.result = Some(RoundResult::Winner { seat: Seat::Two });

    let ended_at = capture.world.physical_tick;
    let mut room = super::tests::running_room(crate::game::PlayerCount::Four, ended_at);
    room.scores[1].wins = Wins::new(1).unwrap();
    room.last_round = Some(CompletedRound {
        round_id: room.round_id,
        round_number: crate::protocol::Counter::new(1).unwrap(),
        ended_at,
        result: RoundResult::Winner { seat: Seat::Two },
    });
    room.seats[0].connection = ConnectionStatus::Expired;
    room.seats[1].connection = ConnectionStatus::Expired;
    room.seats[2].connection = ConnectionStatus::Expired;
    room.settings_editor = None;
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result: MatchResult::Winner {
            seat: Seat::Four,
            reason: MatchWinnerReason::Forfeit,
        },
        rematch_at: crate::protocol::Tick::new(ended_at.get() + 180).unwrap(),
    };

    let generation = crate::game::generate_multiplayer_arena(
        [9; 32],
        crate::game::EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: crate::game::PlayerCount::Four,
            board: crate::room::BoardSize::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap()
    .record();
    assert!(bind_expanded(&room, room.round_id, &capture, Some(generation)).is_ok());
}

#[test]
fn internal_error_match_result_allows_null_admission_body() {
    let ended_at = crate::protocol::Tick::new(5).unwrap();
    let mut room = super::tests::running_room(crate::game::PlayerCount::Three, ended_at);
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result: MatchResult::Interrupted {
            reason: MatchInterruptReason::InternalError,
        },
        rematch_at: crate::protocol::Tick::new(ended_at.get() + 180).unwrap(),
    };
    assert!(
        prepare_admission(
            AdmissionProjection {
                room: &room,
                state_epoch: None,
                capture: None,
                generation: None,
            },
            PersonalAdmission {
                recipient: Seat::Three,
                last_action_id: None,
                sequence: crate::protocol::Counter::new(1).unwrap(),
                resume_token: ResumeToken::new("b".repeat(64)).unwrap(),
            },
        )
        .is_ok()
    );
}

fn initial_countdown_with_inactive_third() -> RoomView {
    let tick = crate::protocol::Tick::new(10).unwrap();
    let mut room = super::tests::running_room(crate::game::PlayerCount::Three, tick);
    room.round_number = RoundCount::new(0).unwrap();
    room.seats[0].ready = true;
    room.seats[1].ready = true;
    room.seats[2].connection = ConnectionStatus::Expired;
    room.phase = RoomPhase::Countdown {
        kind: CountdownKind::Initial,
        ends_at: crate::protocol::Tick::new(tick.get() + 180).unwrap(),
    };
    room
}

#[test]
fn countdown_consensus_ignores_expired_but_not_vacant_slots() {
    let room = initial_countdown_with_inactive_third();
    assert!(validate_room_view(&room).is_ok());

    let mut vacant = room;
    vacant.seats[2].occupied = false;
    vacant.seats[2].connection = ConnectionStatus::Vacant;
    assert_eq!(
        validate_room_view(&vacant),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn cpu_ready_is_derived_for_eligible_countdown_pair() {
    let mut room = initial_countdown_with_inactive_third();
    let configuration = crate::room::RoomConfiguration::new(
        &[
            crate::room::SeatControl::Human,
            crate::room::SeatControl::Cpu,
            crate::room::SeatControl::Human,
        ],
        Some(crate::room::CpuDifficulty::Standard),
        crate::room::BoardSize::FifteenByThirteen,
    )
    .unwrap();
    room.configuration = RoomConfiguration::from_domain(configuration).unwrap();
    room.seats[1].controller = Controller::Cpu {
        difficulty: Difficulty::Standard,
    };
    room.seats[1].connection = ConnectionStatus::ServerControlled;
    assert!(validate_room_view(&room).is_ok());

    room.seats[1].ready = false;
    assert_eq!(
        validate_room_view(&room),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    room.round_number = RoundCount::new(1).unwrap();
    room.seats[0].ready = false;
    room.phase = RoomPhase::Running;
    assert!(validate_room_view(&room).is_ok());
    room.seats[1].ready = true;
    assert_eq!(
        validate_room_view(&room),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let ended_at = room.tick;
    let rematch_at = crate::protocol::Tick::new(ended_at.get() + 180).unwrap();
    room.tick = rematch_at;
    room.seats[1].ready = false;
    room.seats[1].rematch_ready = true;
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result: MatchResult::Interrupted {
            reason: MatchInterruptReason::InternalError,
        },
        rematch_at,
    };
    assert!(validate_room_view(&room).is_ok());
    room.seats[1].rematch_ready = false;
    assert_eq!(
        validate_room_view(&room),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

fn terminal_room() -> RoomView {
    let ended_at = crate::protocol::Tick::new(10).unwrap();
    let rematch_at = crate::protocol::Tick::new(190).unwrap();
    let mut room = super::tests::running_room(crate::game::PlayerCount::Three, rematch_at);
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result: MatchResult::Interrupted {
            reason: MatchInterruptReason::InternalError,
        },
        rematch_at,
    };
    room
}

#[test]
fn terminal_rematch_consensus_requires_two_eligible_and_a_connected_human() {
    let mut abandoned = terminal_room();
    abandoned.settings_editor = None;
    for seat in &mut abandoned.seats {
        seat.connection = ConnectionStatus::Expired;
    }
    abandoned.phase = RoomPhase::MatchResult {
        ended_at: crate::protocol::Tick::new(10).unwrap(),
        result: MatchResult::Abandoned,
        rematch_at: crate::protocol::Tick::new(190).unwrap(),
    };
    assert!(validate_room_view(&abandoned).is_ok());

    let mut sole_human = terminal_room();
    sole_human.seats[0].rematch_ready = true;
    sole_human.seats[1].connection = ConnectionStatus::Expired;
    sole_human.seats[2].connection = ConnectionStatus::Expired;
    sole_human.phase = RoomPhase::MatchResult {
        ended_at: crate::protocol::Tick::new(10).unwrap(),
        result: MatchResult::Winner {
            seat: Seat::One,
            reason: MatchWinnerReason::Forfeit,
        },
        rematch_at: crate::protocol::Tick::new(190).unwrap(),
    };
    assert!(validate_room_view(&sole_human).is_ok());

    let mut sole_cpu = cpu_terminal_room(false);
    sole_cpu.seats[1].rematch_ready = true;
    sole_cpu.phase = RoomPhase::MatchResult {
        ended_at: crate::protocol::Tick::new(10).unwrap(),
        result: MatchResult::Winner {
            seat: Seat::Two,
            reason: MatchWinnerReason::Forfeit,
        },
        rematch_at: crate::protocol::Tick::new(190).unwrap(),
    };
    assert!(validate_room_view(&sole_cpu).is_ok());

    let two_cpus = cpu_terminal_room(true);
    assert!(validate_room_view(&two_cpus).is_ok());

    let mut cpu_only_countdown = cpu_terminal_room(true);
    cpu_only_countdown.round_number = RoundCount::new(0).unwrap();
    for seat in &mut cpu_only_countdown.seats[1..] {
        seat.ready = true;
        seat.rematch_ready = false;
    }
    cpu_only_countdown.phase = RoomPhase::Countdown {
        kind: CountdownKind::Initial,
        ends_at: crate::protocol::Tick::new(cpu_only_countdown.tick.get() + 180).unwrap(),
    };
    assert_eq!(
        validate_room_view(&cpu_only_countdown),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

fn cpu_terminal_room(two_cpus: bool) -> RoomView {
    let mut room = terminal_room();
    let controls = if two_cpus {
        vec![
            crate::room::SeatControl::Human,
            crate::room::SeatControl::Cpu,
            crate::room::SeatControl::Cpu,
        ]
    } else {
        vec![
            crate::room::SeatControl::Human,
            crate::room::SeatControl::Cpu,
            crate::room::SeatControl::Human,
        ]
    };
    let configuration = crate::room::RoomConfiguration::new(
        &controls,
        Some(crate::room::CpuDifficulty::Standard),
        crate::room::BoardSize::FifteenByThirteen,
    )
    .unwrap();
    room.configuration = RoomConfiguration::from_domain(configuration).unwrap();
    room.settings_editor = None;
    room.seats[0].connection = ConnectionStatus::Expired;
    for index in 1..=(if two_cpus { 2 } else { 1 }) {
        room.seats[index].controller = Controller::Cpu {
            difficulty: Difficulty::Standard,
        };
        room.seats[index].connection = ConnectionStatus::ServerControlled;
        room.seats[index].rematch_ready = true;
    }
    if !two_cpus {
        room.seats[2].connection = ConnectionStatus::Expired;
    }
    room
}

#[test]
fn waiting_creator_reservation_keeps_truthful_connection_and_editor_metadata() {
    let tick = crate::protocol::Tick::new(10).unwrap();
    let mut room = super::tests::running_room(crate::game::PlayerCount::Three, tick);
    room.phase = RoomPhase::Waiting;
    room.round_number = RoundCount::new(0).unwrap();
    room.seats[0].connection = ConnectionStatus::Disconnected {
        expires_at: crate::protocol::Tick::new(70).unwrap(),
    };
    assert_eq!(room.settings_editor, Some(Seat::One));
    assert!(room.seats[0].occupied);
    assert!(validate_room_view(&room).is_ok());
    room.seats[0].connection = ConnectionStatus::Disconnected { expires_at: tick };
    assert_eq!(
        validate_room_view(&room),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}
