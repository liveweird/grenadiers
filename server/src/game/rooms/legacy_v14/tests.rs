use grenadiers_core::{
    game::{SeatId, State},
    protocol::{Counter, Tick, v13, v14},
    room::{CpuProfile, RoomMode, RoomSettings},
};

use super::{admission, room_state, room_view, snapshot};
use crate::{
    game::rooms::{
        match_runtime::{MatchAction, MatchRuntime},
        profile::RoomProfile,
        types::RoomIdentity,
    },
    round_seed::RoundSeedSource,
};

fn identity(profile: RoomProfile, mode: RoomMode) -> RoomIdentity {
    RoomIdentity {
        id: grenadiers_core::protocol::RoomId::new("00000000000000000000000000000001").unwrap(),
        code: grenadiers_core::protocol::RoomCode::new("ABCDEFGH").unwrap(),
        profile,
        mode,
        registry_serial: 1,
        private_cpu_seed: matches!(mode, RoomMode::Solo(_)).then_some([17; 32]),
    }
}

fn runtime(profile: RoomProfile, mode: RoomMode) -> MatchRuntime {
    MatchRuntime::new(
        identity(profile, mode),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(23);
            Ok(())
        }),
    )
}

fn occupied_waiting(mode: RoomMode) -> (MatchRuntime, v13::RoomView) {
    let mut runtime = runtime(RoomProfile::ExpandedV5, mode);
    runtime.occupy(SeatId::One).unwrap();
    let notice = if matches!(mode, RoomMode::HumanDuel) {
        runtime.occupy(SeatId::Two).unwrap()
    } else {
        runtime.cpu_ready(SeatId::Two).unwrap().unwrap()
    };
    (runtime, notice.room)
}

fn running_fixture() -> (
    v13::RoomView,
    State,
    Option<grenadiers_core::game::GenerationRecord>,
) {
    let mut runtime = runtime(RoomProfile::FixedClassicV1, RoomMode::HumanDuel);
    runtime.occupy(SeatId::One).unwrap();
    runtime.occupy(SeatId::Two).unwrap();
    runtime
        .action(SeatId::One, 1, MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, MatchAction::Ready(true))
        .unwrap();
    let notice = loop {
        if let Some(notice) = runtime.tick().unwrap()
            && matches!(notice.room.phase, v13::RoomPhase::Running)
        {
            break notice;
        }
    };
    let sim = runtime.sim().unwrap();
    (notice.room, sim.state.clone(), sim.generation_record)
}

#[test]
fn real_legacy_duel_and_both_solo_difficulties_keep_metadata() {
    let (_, duel) = occupied_waiting(RoomMode::HumanDuel);
    let projected = room_view(&duel).unwrap();
    assert_eq!(
        projected.configuration.domain(),
        &grenadiers_core::room::RoomConfiguration::human_duel()
    );
    assert_eq!(projected.settings, duel.settings);
    assert_eq!(projected.settings_revision, duel.settings_revision);
    assert_eq!(projected.settings_editor, Some(v14::Seat::One));
    assert_eq!(projected.revision, duel.revision);
    assert_eq!(
        projected.rules_version.get(),
        duel.rules_version.get() as u8
    );
    assert_eq!(projected.scores.len(), 2);
    assert_eq!(projected.scores[0].wins, duel.scores.one);
    assert_eq!(projected.scores[1].wins, duel.scores.two);

    for (profile, expected) in [
        (CpuProfile::CURRENT_EASY, v14::Difficulty::Easy),
        (CpuProfile::CURRENT, v14::Difficulty::Standard),
    ] {
        let (_, legacy) = occupied_waiting(RoomMode::Solo(profile));
        let projected = room_view(&legacy).unwrap();
        assert_eq!(projected.configuration.domain().controllers().len(), 2);
        assert_eq!(
            projected.configuration.domain().cpu_difficulty(),
            Some(profile.difficulty)
        );
        assert_eq!(
            projected.seats[1].controller,
            v14::Controller::Cpu {
                difficulty: expected
            }
        );
        assert_eq!(
            projected.seats[1].connection,
            v14::ConnectionStatus::ServerControlled
        );
    }
}

#[test]
fn vacant_legacy_editor_and_unready_solo_cpu_normalize_only_for_v14() {
    let (_, mut legacy) = occupied_waiting(RoomMode::Solo(CpuProfile::CURRENT));
    legacy.seats[0].occupied = false;
    legacy.seats[0].connection = v13::ConnectionStatus::Vacant;
    legacy.settings_editor = Some(grenadiers_core::protocol::Seat::One);
    legacy.seats[1].ready = false;
    let before = legacy.clone();

    let projected = room_view(&legacy).unwrap();
    assert_eq!(projected.settings_editor, None);
    assert!(projected.seats[1].ready);
    assert_eq!(legacy, before);
}

#[test]
fn settings_terminal_result_and_disconnect_grace_are_copied() {
    let (mut runtime, _) = occupied_waiting(RoomMode::HumanDuel);
    let settings = RoomSettings::new(grenadiers_core::game::SpeedLevel::Two, 192, 3, 150)
        .unwrap()
        .with_enabled_items(grenadiers_core::game::EnabledItems::ALL);
    let notice = runtime
        .action(
            SeatId::One,
            1,
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .unwrap()
        .unwrap();
    let mut legacy = notice.room;
    let settings_projection = room_view(&legacy).unwrap();
    assert_eq!(settings_projection.settings, legacy.settings);
    assert_eq!(settings_projection.settings_revision.get(), 2);

    legacy.tick = Tick::new(1_000).unwrap();
    legacy.round_id = Counter::new(3).unwrap();
    legacy.round_number = v13::RoundCount::new(3).unwrap();
    legacy.scores.one = v13::Wins::new(3).unwrap();
    legacy.last_round = Some(v13::CompletedRound {
        round_id: Counter::new(3).unwrap(),
        round_number: Counter::new(3).unwrap(),
        ended_at: Tick::new(900).unwrap(),
        result: grenadiers_core::protocol::WireRoundResult::Winner {
            seat: grenadiers_core::protocol::Seat::One,
        },
    });
    legacy.seats[1].connection = v13::ConnectionStatus::Disconnected {
        expires_at: Tick::new(1_400).unwrap(),
    };
    legacy.phase = v13::RoomPhase::MatchResult {
        ended_at: Tick::new(900).unwrap(),
        result: v13::MatchResult::Winner {
            seat: grenadiers_core::protocol::Seat::One,
            reason: v13::MatchWinnerReason::RoundWins,
        },
        rematch_at: Tick::new(1_080).unwrap(),
    };
    let projected = room_view(&legacy).unwrap();
    assert_eq!(projected.last_round.unwrap().ended_at.get(), 900);
    assert_eq!(
        projected.seats[1].connection,
        v14::ConnectionStatus::Disconnected {
            expires_at: Tick::new(1_400).unwrap()
        }
    );
    assert!(matches!(
        projected.phase,
        v14::RoomPhase::MatchResult {
            result: v14::MatchResult::Winner {
                seat: v14::Seat::One,
                reason: v14::MatchWinnerReason::RoundWins
            },
            ..
        }
    ));
}

#[test]
fn snapshot_captures_actual_state_and_pending_reports_as_two_v14_frames() {
    let (mut legacy, mut state, generation) = running_fixture();
    let report = state.step(&[]).unwrap();
    legacy.tick = Tick::new(state.tick().0).unwrap();
    let frames = snapshot(
        &legacy,
        legacy.round_id,
        &state,
        std::slice::from_ref(&report),
        generation,
    )
    .unwrap();
    for frame in frames {
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(value["type"], "snapshot");
        assert_eq!(value["room"]["rulesVersion"], 1);
        assert_eq!(value["state"]["physicalTick"], state.tick().0);
        assert!(value["events"].is_array());
    }
}

#[test]
fn admission_is_personal_to_connected_humans_and_copies_last_action() {
    let (legacy, state, generation) = running_fixture();
    let capture = v14::capture_expanded(&state, &[]).unwrap();
    let token = v14::ResumeToken::new("a".repeat(64)).unwrap();
    let encoded = admission(
        &legacy,
        Some(legacy.round_id),
        Some(&capture),
        generation,
        SeatId::One,
        Some(Counter::new(7).unwrap()),
        Counter::new(11).unwrap(),
        token.clone(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["type"], "admitted");
    assert_eq!(value["youSeat"], "one");
    assert_eq!(value["lastActionId"], 7);
    assert_eq!(value["sequence"], 11);
    assert!(value.get("events").is_none());

    for unsupported in [SeatId::Three, SeatId::Four] {
        assert!(
            admission(
                &legacy,
                Some(legacy.round_id),
                Some(&capture),
                generation,
                unsupported,
                None,
                Counter::new(1).unwrap(),
                token.clone(),
            )
            .is_err()
        );
    }

    let (_, solo) = occupied_waiting(RoomMode::Solo(CpuProfile::CURRENT));
    assert!(
        admission(
            &solo,
            None,
            None,
            None,
            SeatId::Two,
            None,
            Counter::new(1).unwrap(),
            token,
        )
        .is_err()
    );
}

#[test]
fn invalid_legacy_metadata_rejects_without_partial_encoding() {
    let (_, mut legacy) = occupied_waiting(RoomMode::HumanDuel);
    legacy.seats.swap(0, 1);
    assert!(room_view(&legacy).is_err());
    assert!(room_state(&legacy).is_err());

    let (_, valid) = occupied_waiting(RoomMode::HumanDuel);
    let before = valid.clone();
    assert!(
        admission(
            &valid,
            None,
            None,
            None,
            SeatId::Three,
            None,
            Counter::new(1).unwrap(),
            v14::ResumeToken::new("b".repeat(64)).unwrap(),
        )
        .is_err()
    );
    assert_eq!(valid, before);
}
