use grenadiers_core::{
    game::{
        BombTrigger, BombView, CellId, EntityId, RoundAssignment, SeatId, StepEvent, StepEventKind,
    },
    protocol::v6::RoomPhase,
    room::{InterruptReason, MatchLifecycle, MatchPhase},
};

use super::publication;
use super::test_support::*;
use super::{MatchRuntimeError, publication::phase_has_frozen_body};
use crate::round_seed::RoundSeedSource;

#[test]
fn result_hold_advances_room_clock_while_freezing_the_real_world() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let completed = finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    let frozen_state = snapshot_state(&completed);
    let frozen_tick = runtime.sim().unwrap().state.tick();
    let room_tick = completed.room.tick.get();

    let mut later = None;
    for _ in 0..3 {
        later = runtime.tick().unwrap().or(later);
    }
    let later = later.expect("twenty hertz hold snapshot");
    let value = snapshot_json(&later);

    assert!(phase_has_frozen_body(&later.room.phase));
    assert!(later.room.tick.get() > room_tick);
    assert!(later.room.tick.get() <= room_tick + 3);
    assert_eq!(runtime.lifecycle().tick().0, room_tick + 3);
    assert_eq!(runtime.sim().unwrap().state.tick(), frozen_tick);
    assert_eq!(value["state"], frozen_state);
    assert_eq!(value["events"], serde_json::json!([]));
}

#[test]
fn final_report_is_flushed_before_inter_round_world_replacement() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let completed = finish_round_with_bombs(&mut runtime, &[SeatId::Two]);
    let completed_json = snapshot_json(&completed);
    assert!(
        completed_json["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["kind"]["type"] == "roundEnded")
    );

    for _ in 0..3 {
        if let Some(hold) = runtime.tick().unwrap() {
            assert_eq!(snapshot_json(&hold)["events"], serde_json::json!([]));
        }
    }
    let countdown = advance_to_inter_round(&mut runtime);
    assert_view_only(&countdown);
    let boundary = tick_until_running(&mut runtime);
    let boundary_json = snapshot_json(&boundary);
    assert_eq!(boundary_json["events"], serde_json::json!([]));
    assert_eq!(boundary_json["state"]["runningUpdates"], 0);
    assert_eq!(boundary.room.round_number.get(), 2);
}

#[test]
fn generated_snapshot_redacts_seed_generation_record_and_assignment() {
    let mut runtime = generated_runtime(RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(0x13);
        Ok(())
    }));
    let notice = start_first_round(&mut runtime);
    let encoded = notice.snapshot.as_ref().unwrap();

    assert!(runtime.sim().unwrap().generation_record.is_some());
    for private in [
        "seed",
        "generationRecord",
        "concealed",
        "pending",
        "assignment",
    ] {
        assert!(!encoded.contains(&format!("\"{private}\"")));
    }
    assert!(encoded.len() <= grenadiers_core::protocol::MAX_SERVER_MESSAGE_BYTES);
}

#[test]
fn entropy_failure_interrupts_with_view_only_and_no_simulation_record() {
    let mut runtime = generated_runtime(RoundSeedSource::with_entropy(|_| Err(())));
    occupy_both(&mut runtime);
    runtime
        .action(SeatId::One, 1, super::MatchAction::Ready(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, super::MatchAction::Ready(true))
        .unwrap();
    let mut terminal = None;
    for _ in 0..180 {
        terminal = runtime.tick().unwrap().or(terminal);
    }
    let terminal = terminal.expect("failure notice");

    assert_view_only(&terminal);
    assert!(runtime.sim().is_none());
    assert!(matches!(
        terminal.room.phase,
        RoomPhase::MatchResult {
            result: grenadiers_core::protocol::v6::MatchResult::Interrupted {
                reason: grenadiers_core::protocol::v6::MatchInterruptReason::InternalError
            },
            ..
        }
    ));
}

#[test]
fn publication_rejects_profile_rule_mismatch_and_oversized_report_window() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    let generated_identity = identity(super::super::profile::RoomProfile::GeneratedClassicV2);
    assert_eq!(
        publication::snapshot(
            &generated_identity,
            runtime.lifecycle(),
            99,
            1,
            None,
            runtime.sim().unwrap(),
            &[],
        ),
        Err(MatchRuntimeError::Publication)
    );

    let identity = identity(super::super::profile::RoomProfile::FixedClassicV1);
    let mut lifecycle = MatchLifecycle::new();
    for seat in SeatId::ALL {
        lifecycle.occupy(seat).unwrap();
        lifecycle.set_ready(seat, true).unwrap();
    }
    for _ in 0..180 {
        lifecycle.advance_idle_tick().unwrap();
    }
    let mut state = identity
        .profile
        .initial_round(
            lifecycle.tick(),
            RoundAssignment::first(),
            &RoundSeedSource::with_entropy(|_| panic!("fixed profile requested entropy")),
        )
        .unwrap()
        .state;
    let mut reports = Vec::new();
    for _ in 0..4 {
        reports.push(state.step(&[]).unwrap());
        lifecycle.complete_running_tick(None).unwrap();
    }
    let sim = super::runtime::SimRound {
        epoch: lifecycle.round_attempt(),
        state,
        generation_record: None,
    };
    assert_eq!(
        publication::snapshot(&identity, &lifecycle, 100, 1, None, &sim, &reports),
        Err(MatchRuntimeError::Publication)
    );
}

#[test]
fn failed_unsupported_event_publication_keeps_revision_and_pending_reports() {
    let mut runtime = fixed_runtime();
    start_first_round(&mut runtime);
    assert_eq!(runtime.tick(), Ok(None));

    let mut reports = runtime.replay_inspection().reports;
    assert_eq!(reports.len(), 1);
    let tick = reports[0].tick;
    reports[0].events.push(StepEvent {
        tick,
        kind: StepEventKind::BombPlaced(BombView {
            id: EntityId(999),
            cell: CellId::new(2, 2),
            owner: EntityId(10),
            range: 2,
            placed_at: tick,
            trigger: BombTrigger::Remote,
        }),
    });
    runtime.replace_reports_for_test(reports.clone());
    let revision = runtime.publication_revision_for_test();

    assert_eq!(
        runtime.replay_report_flush(),
        Err(MatchRuntimeError::Publication)
    );
    assert_eq!(runtime.publication_revision_for_test(), revision);
    assert_eq!(runtime.replay_inspection().reports, reports);
}

#[test]
fn prematch_interrupt_is_view_only_and_legacy_participant_left_is_rejected() {
    let mut runtime = fixed_runtime();
    runtime.occupy(SeatId::One).unwrap();
    assert_eq!(
        runtime.interrupt(InterruptReason::ParticipantLeft),
        Err(MatchRuntimeError::InvalidInterrupt)
    );
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Waiting);

    let notice = runtime
        .interrupt(InterruptReason::ServerStopped)
        .unwrap()
        .unwrap();
    assert_view_only(&notice);
    assert!(matches!(notice.room.phase, RoomPhase::Interrupted { .. }));
}
