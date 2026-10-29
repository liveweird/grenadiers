use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, v14},
    room::{CpuDifficulty, MatchPhase, SeatControl},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::{
    super::{roster_runtime::RosterAction, types::RoomError},
    RosterOwnerNotice,
    driver::Driver,
    tests::{advance_tick, binding, owner},
};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn seat_three_and_four_restore_then_replace_with_exact_binding_fences() {
    let (handle, prepared) = owner(&[SeatControl::Human; 4], None);
    let task = prepared.start(|| {});
    let _creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let _second = handle
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
    let room_id = third.room_id.clone();
    let round_id = third.round_id;
    let third_token = third.resume_token.clone();
    let fourth_token = fourth.resume_token.clone();
    drop(third.guard);
    drop(fourth.guard);
    advance_tick().await;

    let restored_three = handle
        .resume(
            binding(30),
            Counter::new(2).unwrap(),
            room_id.clone(),
            third_token.clone(),
        )
        .await
        .unwrap()
        .unwrap();
    let restored_four = handle
        .resume(
            binding(40),
            Counter::new(2).unwrap(),
            room_id.clone(),
            fourth_token.clone(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored_three.seat, SeatId::Three);
    assert_eq!(restored_four.seat, SeatId::Four);

    let replacement_three = handle
        .resume(
            binding(31),
            Counter::new(3).unwrap(),
            room_id.clone(),
            third_token,
        )
        .await
        .unwrap()
        .unwrap();
    let replacement_four = handle
        .resume(
            binding(41),
            Counter::new(3).unwrap(),
            room_id.clone(),
            fourth_token,
        )
        .await
        .unwrap()
        .unwrap();
    assert!(restored_three.replaced.await.is_ok());
    assert!(restored_four.replaced.await.is_ok());
    for stale in [binding(30), binding(40)] {
        assert_eq!(
            handle
                .action(
                    stale,
                    room_id.clone(),
                    round_id,
                    RosterAction::Ready {
                        ready: false,
                        settings_revision: 1,
                    },
                )
                .await
                .unwrap(),
            Err(RoomError::StaleRoom)
        );
    }
    assert_eq!(replacement_three.seat, SeatId::Three);
    assert_eq!(replacement_four.seat, SeatId::Four);
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn overload_interrupts_with_a_frozen_publication_after_large_clock_jump() {
    let (handle, prepared) = owner(&[SeatControl::Human; 3], None);
    let task = prepared.start(|| {});
    let creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let notices = creator.notices.clone();
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    for _ in 0..4 {
        tokio::task::yield_now().await;
    }
    assert!(matches!(
        &*notices.borrow(),
        super::RosterOwnerNotice::Publication(notice)
            if matches!(
                notice.legacy_room().unwrap().phase,
                v14::RoomPhase::Interrupted {
                    reason: v14::MatchInterruptReason::ServerOverloaded,
                    ..
                }
            )
    ));
    task.abort();
}

#[test]
fn three_cpu_observations_apply_as_a_batch_and_service_rotates_first_seat() {
    let (_handle, prepared) = owner(
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Cpu,
            SeatControl::Cpu,
        ],
        Some(CpuDifficulty::Standard),
    );
    let mut runtime = prepared.runtime;
    let creator = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(creator);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Running);
    let notice = runtime.current_notice().unwrap();
    let (notices, receiver) =
        tokio::sync::watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let mut driver = Driver::new(
        runtime,
        notice.clone(),
        notices,
        tokio::time::Instant::now() + std::time::Duration::from_secs(300),
        prepared.random_fill,
        prepared.cpus,
    );
    driver.publish(notice);
    assert_eq!(driver.cpu_stop_at[SeatId::One.index()], None);
    for seat in [SeatId::Two, SeatId::Three, SeatId::Four] {
        assert_eq!(
            driver.cpu_stop_at[seat.index()],
            Some(driver.runtime.lifecycle().tick())
        );
    }
    assert_eq!(driver.next_cpu, 0);
    driver.service_cpu().unwrap();
    assert_eq!(driver.next_cpu, 1);
    driver.service_cpu().unwrap();
    assert_eq!(driver.next_cpu, 2);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn exact_transport_expiry_fences_resume_and_cleans_the_empty_room() {
    let (handle, prepared) = owner(&[SeatControl::Human; 3], None);
    let cleaned = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&cleaned);
    let task = prepared.start(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    let admission = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let room_id = admission.room_id.clone();
    let token = admission.resume_token.clone();
    drop(admission.guard);
    for _ in 0..grenadiers_core::room::reconnect::RECONNECT_GRACE_TICKS {
        advance_tick().await;
    }
    task.await.unwrap();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    assert!(matches!(
        handle
            .resume(binding(2), Counter::new(2).unwrap(), room_id, token,)
            .await
            .unwrap(),
        Err(RoomError::ResumeRejected)
    ));
}
