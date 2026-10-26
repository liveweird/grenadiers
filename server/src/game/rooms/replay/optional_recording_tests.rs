use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::{
    game::SeatId,
    room::{InterruptReason, MatchPhase},
};

use super::{ReplayError, ReplayStatus, replay, runner::identity_for_profile};
use crate::{
    game::rooms::{
        match_runtime::{MatchAction, MatchRuntime},
        profile::RoomProfile,
    },
    round_seed::RoundSeedSource,
};

#[test]
fn disabled_and_recorded_runtimes_match_through_cancellation_and_real_start() {
    let disabled_requests = Arc::new(AtomicUsize::new(0));
    let recorded_requests = Arc::new(AtomicUsize::new(0));
    let mut disabled = MatchRuntime::new(
        identity_for_profile(RoomProfile::ExpandedV5),
        counting_source(Arc::clone(&disabled_requests), true),
    );
    let mut recorded = MatchRuntime::new_recording(
        identity_for_profile(RoomProfile::ExpandedV5),
        counting_source(Arc::clone(&recorded_requests), true),
    );

    assert!(!disabled.recording_enabled_for_test());
    assert!(recorded.recording_enabled_for_test());
    assert_eq!(disabled.initial_notice(), recorded.initial_notice());
    for seat in SeatId::ALL {
        assert_eq!(disabled.occupy(seat), recorded.occupy(seat));
        assert_eq!(
            disabled.action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                }
            ),
            recorded.action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                }
            )
        );
    }
    for _ in 0..12 {
        assert_eq!(disabled.tick(), recorded.tick());
    }
    assert_eq!(disabled_requests.load(Ordering::Relaxed), 0);
    assert_eq!(recorded_requests.load(Ordering::Relaxed), 0);
    assert_eq!(
        disabled.action(
            SeatId::One,
            1,
            MatchAction::ReadyChecked {
                ready: false,
                settings_revision: 1,
            }
        ),
        recorded.action(
            SeatId::One,
            1,
            MatchAction::ReadyChecked {
                ready: false,
                settings_revision: 1,
            }
        )
    );
    assert_eq!(disabled.replay_inspection(), recorded.replay_inspection());
    assert_eq!(
        disabled.publication_revision_for_test(),
        recorded.publication_revision_for_test()
    );

    let epoch = disabled.lifecycle().round_attempt();
    assert_eq!(recorded.lifecycle().round_attempt(), epoch);
    for seat in SeatId::ALL {
        assert_eq!(
            disabled.action(
                seat,
                epoch,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                }
            ),
            recorded.action(
                seat,
                epoch,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                }
            )
        );
    }
    for _ in 0..500 {
        if disabled.lifecycle().phase() == MatchPhase::Running {
            break;
        }
        assert_eq!(disabled.tick(), recorded.tick());
    }
    assert_eq!(disabled.lifecycle().phase(), MatchPhase::Running);
    assert_eq!(disabled_requests.load(Ordering::Relaxed), 1);
    assert_eq!(recorded_requests.load(Ordering::Relaxed), 1);
    assert_eq!(disabled.replay_inspection(), recorded.replay_inspection());
    assert_eq!(
        disabled.publication_revision_for_test(),
        recorded.publication_revision_for_test()
    );
    assert_eq!(
        replay(recorded.replay_journal()).unwrap(),
        recorded.replay_inspection()
    );
}

#[test]
fn entropy_failure_matches_without_recording_and_recorded_evidence_is_incomplete() {
    let disabled_requests = Arc::new(AtomicUsize::new(0));
    let recorded_requests = Arc::new(AtomicUsize::new(0));
    let mut disabled = MatchRuntime::new(
        identity_for_profile(RoomProfile::ExpandedV5),
        counting_source(Arc::clone(&disabled_requests), false),
    );
    let mut recorded = MatchRuntime::new_recording(
        identity_for_profile(RoomProfile::ExpandedV5),
        counting_source(Arc::clone(&recorded_requests), false),
    );

    assert_eq!(disabled.initial_notice(), recorded.initial_notice());
    for seat in SeatId::ALL {
        assert_eq!(disabled.occupy(seat), recorded.occupy(seat));
        assert_eq!(
            disabled.action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                }
            ),
            recorded.action(
                seat,
                1,
                MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                }
            )
        );
    }
    for _ in 0..500 {
        if is_internal_error(disabled.lifecycle().phase()) {
            break;
        }
        assert_eq!(disabled.tick(), recorded.tick());
    }

    assert!(is_internal_error(disabled.lifecycle().phase()));
    assert_eq!(disabled_requests.load(Ordering::Relaxed), 1);
    assert_eq!(recorded_requests.load(Ordering::Relaxed), 1);
    let disabled_inspection = disabled.replay_inspection();
    let recorded_inspection = recorded.replay_inspection();
    assert_eq!(disabled_inspection, recorded_inspection);
    assert!(disabled_inspection.simulation.is_none());
    assert!(disabled_inspection.inputs.contains("direction: None"));
    assert!(disabled_inspection.inputs.contains("manual_actions: []"));
    assert_eq!(
        disabled.publication_revision_for_test(),
        recorded.publication_revision_for_test()
    );
    assert!(matches!(
        recorded.replay_journal().status(),
        ReplayStatus::Incomplete { .. }
    ));
    assert_eq!(
        replay(recorded.replay_journal()),
        Err(ReplayError::Incomplete)
    );
    assert!(!disabled.recording_enabled_for_test());
}

fn counting_source(requests: Arc<AtomicUsize>, succeeds: bool) -> RoundSeedSource {
    RoundSeedSource::with_entropy(move |bytes| {
        requests.fetch_add(1, Ordering::Relaxed);
        if succeeds {
            bytes.fill(0x5a);
            Ok(())
        } else {
            Err(())
        }
    })
}

fn is_internal_error(phase: MatchPhase) -> bool {
    matches!(
        phase,
        MatchPhase::Interrupted {
            reason: InterruptReason::InternalError,
            ..
        }
    ) || matches!(phase, MatchPhase::MatchResult { outcome, .. }
    if matches!(outcome.result(), grenadiers_core::room::series::MatchResult::Interrupted(
        InterruptReason::InternalError
    )))
}
