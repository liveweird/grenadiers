use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicUsize, Ordering},
};
use std::time::Duration;

use grenadiers_core::{
    game::{SeatId, SimulationSeed},
    protocol::{Counter, RoomCode, RoomId},
    room::{BoardSize, CpuDifficulty, CpuProfile, RoomConfiguration, RoomSettings, SeatControl},
};
use tokio::{sync::watch, time::Instant};

use super::super::{
    RandomFill,
    cpu::{CpuController, CpuPlanner},
    roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    types::{BindingKey, RoomError},
};
use super::PreparedRosterOwner;
use crate::{game::sessions::ConnectionId, round_seed::RoundSeedSource};

pub(super) fn binding(value: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(value),
        generation: 1,
    }
}

pub(super) fn owner(
    controls: &[SeatControl],
    difficulty: Option<CpuDifficulty>,
) -> (super::RosterOwnerHandle, PreparedRosterOwner) {
    let entropy = AtomicU8::new(1);
    let random_fill = Arc::new(move |bytes: &mut [u8]| {
        bytes.fill(entropy.fetch_add(1, Ordering::Relaxed));
        Ok(())
    });
    let (handle, owner, shutdown) = owner_with_entropy(controls, difficulty, random_fill);
    std::mem::forget(shutdown);
    (handle, owner)
}

fn owner_with_entropy(
    controls: &[SeatControl],
    difficulty: Option<CpuDifficulty>,
    random_fill: Arc<RandomFill>,
) -> (
    super::RosterOwnerHandle,
    PreparedRosterOwner,
    watch::Sender<bool>,
) {
    let configuration =
        RoomConfiguration::new(controls, difficulty, BoardSize::FifteenByThirteen).unwrap();
    let identity = RosterIdentity {
        id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        registry_serial: 9,
        configuration,
    };
    let runtime = RosterRuntime::new(
        identity,
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(3);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap();
    let profile = match difficulty {
        Some(CpuDifficulty::Easy) => CpuProfile::MULTIPLAYER_EASY,
        Some(CpuDifficulty::Standard) => CpuProfile::MULTIPLAYER_STANDARD,
        None => CpuProfile::MULTIPLAYER_STANDARD,
    };
    let cpus = configuration
        .cpu_seats()
        .enumerate()
        .map(|(index, seat)| {
            CpuController::new(
                CpuPlanner::without_workers(),
                seat,
                9,
                [index as u8 + 1; 32],
                profile,
            )
        })
        .collect();
    let (shutdown, receiver) = watch::channel(false);
    let (handle, owner) =
        PreparedRosterOwner::new(runtime, cpus, Instant::now(), random_fill, receiver).unwrap();
    (handle, owner, shutdown)
}

pub(super) async fn advance_tick() {
    tokio::time::advance(Duration::from_millis(17)).await;
    tokio::task::yield_now().await;
}

#[tokio::test]
async fn four_humans_admit_domain_seats_and_connected_resume_replaces_binding() {
    let (handle, prepared) = owner(&[SeatControl::Human; 4], None);
    let task = prepared.start(|| {});
    let first = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.seat, SeatId::One);
    let second = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let third = handle
        .join_sequenced(binding(3), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let fourth = handle
        .join_sequenced(binding(4), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        [second.seat, third.seat, fourth.seat],
        [SeatId::Two, SeatId::Three, SeatId::Four]
    );

    let replacement = handle
        .resume(
            binding(30),
            Counter::new(2).unwrap(),
            third.room_id.clone(),
            third.resume_token.clone(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replacement.seat, SeatId::Three);
    assert!(third.replaced.await.is_ok());
    task.abort();
}

#[tokio::test]
async fn mixed_human_cpu_human_skips_cpu_seat_for_join() {
    let (handle, prepared) = owner(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Standard),
    );
    let task = prepared.start(|| {});
    let creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let joiner = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(creator.seat, SeatId::One);
    assert_eq!(joiner.seat, SeatId::Three);
    task.abort();
}

#[test]
fn three_cpu_constructor_accepts_both_multiplayer_difficulties() {
    for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
        let (_handle, _prepared) = owner(
            &[
                SeatControl::Human,
                SeatControl::Cpu,
                SeatControl::Cpu,
                SeatControl::Cpu,
            ],
            Some(difficulty),
        );
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn departed_token_is_a_tombstone_and_cannot_alias_a_later_human() {
    let draws = Arc::new(AtomicUsize::new(0));
    let entropy = Arc::clone(&draws);
    let fill = Arc::new(move |bytes: &mut [u8]| {
        let draw = entropy.fetch_add(1, Ordering::Relaxed);
        bytes.fill([11, 22, 11, 33][draw]);
        Ok(())
    });
    let (handle, prepared, _shutdown) = owner_with_entropy(&[SeatControl::Human; 3], None, fill);
    let task = prepared.start(|| {});
    let first_binding = binding(1);
    let first = handle
        .admit_creator(first_binding, Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let _second = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let leave = handle.leave(first_binding, first.room_id.clone(), first.round_id);
    advance_tick().await;
    assert_eq!(leave.await.unwrap(), Ok(()));

    let third = handle
        .join_sequenced(binding(3), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(third.seat, SeatId::Three);
    assert_ne!(third.resume_token, first.resume_token);
    assert_eq!(draws.load(Ordering::Relaxed), 4);
    assert!(matches!(
        handle
            .resume(
                binding(30),
                Counter::new(2).unwrap(),
                third.room_id.clone(),
                first.resume_token,
            )
            .await
            .unwrap(),
        Err(RoomError::ResumeRejected)
    ));
    assert_eq!(
        handle
            .action(
                binding(3),
                third.room_id,
                third.round_id,
                RosterAction::Ready {
                    ready: false,
                    settings_revision: 1,
                },
            )
            .await
            .unwrap(),
        Ok(())
    );
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn canceled_admission_delivery_does_not_commit_or_fence_the_next_creator() {
    let (handle, prepared) = owner(&[SeatControl::Human; 3], None);
    let canceled = handle.admit_creator(binding(1), Counter::new(1).unwrap());
    drop(canceled);
    let accepted = handle.admit_creator(binding(2), Counter::new(1).unwrap());
    let task = prepared.start(|| {});
    let admission = accepted.await.unwrap().unwrap();
    assert_eq!(admission.seat, SeatId::One);
    assert_eq!(admission.room.revision.get(), 3);
    assert!(matches!(
        handle
            .action(
                binding(1),
                admission.room_id.clone(),
                admission.round_id,
                RosterAction::Ready {
                    ready: false,
                    settings_revision: 1,
                },
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    ));
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn abort_before_first_poll_and_normal_shutdown_each_run_cleanup_once() {
    let (handle, prepared, shutdown) = owner_with_entropy(
        &[SeatControl::Human; 3],
        None,
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(1);
            Ok(())
        }),
    );
    drop(handle);
    let aborted = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&aborted);
    let task = prepared.start(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    task.abort();
    let _ = task.await;
    assert_eq!(aborted.load(Ordering::SeqCst), 1);
    drop(shutdown);

    let (handle, prepared, shutdown) = owner_with_entropy(
        &[SeatControl::Human; 3],
        None,
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(2);
            Ok(())
        }),
    );
    let cleaned = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&cleaned);
    let task = prepared.start(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    let admission = handle
        .admit_creator(binding(9), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let notices = admission.notices.clone();
    shutdown.send(true).unwrap();
    task.await.unwrap();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    assert!(matches!(
        &*notices.borrow(),
        super::RosterOwnerNotice::Closing {
            last,
            reason: super::super::match_owner::ClosingReason::ServerStopped,
        } if matches!(
            last.legacy_room().unwrap().phase,
            grenadiers_core::protocol::v14::RoomPhase::Interrupted {
                reason: grenadiers_core::protocol::v14::MatchInterruptReason::ServerStopped,
                ..
            }
        )
    ));
    drop(handle);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn last_human_explicit_departure_cleans_up_even_with_three_cpu_slots() {
    let (handle, prepared) = owner(
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Cpu,
            SeatControl::Cpu,
        ],
        Some(CpuDifficulty::Easy),
    );
    let cleaned = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&cleaned);
    let task = prepared.start(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    let creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let leave = handle.leave(binding(1), creator.room_id, creator.round_id);
    advance_tick().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    task.await.unwrap();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn command_queue_pressure_rejects_the_thirty_third_admission() {
    let (handle, _prepared) = owner(&[SeatControl::Human; 4], None);
    let mut queued = Vec::new();
    for index in 0..32 {
        queued.push(handle.join_sequenced(binding(index), Counter::new(1).unwrap()));
    }
    assert!(matches!(
        handle
            .join_sequenced(binding(33), Counter::new(1).unwrap())
            .await
            .unwrap(),
        Err(RoomError::RateLimited)
    ));
    drop(queued);
}

#[path = "variable_board_tests.rs"]
mod variable_board_tests;
