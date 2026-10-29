use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::{
    game::{SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, RoomCode, RoomId},
    room::{BoardSize, RoomConfiguration, RoomSettings, SeatControl},
};
use tokio::{sync::watch, time::Instant};

use super::{
    PreparedRosterOwner, RosterOwnerNotice,
    tests::{advance_tick, binding},
};
use crate::{
    game::rooms::{
        RandomFill,
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
        types::RoomError,
    },
    round_seed::RoundSeedSource,
};

fn owner_with_entropy(
    random_fill: Arc<RandomFill>,
) -> (super::RosterOwnerHandle, PreparedRosterOwner) {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Human],
        None,
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let runtime = RosterRuntime::new_environment(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: 15,
            configuration,
        },
        RoomSettings::DEFAULT,
        StormCloudConfig::Single,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(3);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap();
    let (shutdown, receiver) = watch::channel(false);
    std::mem::forget(shutdown);
    PreparedRosterOwner::new(runtime, vec![], Instant::now(), random_fill, receiver).unwrap()
}

fn entropy_counter() -> Arc<RandomFill> {
    let draw = AtomicUsize::new(1);
    Arc::new(move |bytes| {
        bytes.fill(draw.fetch_add(1, Ordering::Relaxed) as u8);
        Ok(())
    })
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn explicit_leave_publishes_vacancy_and_replacement_fences_old_authority() {
    let draws = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&draws);
    let entropy: Arc<RandomFill> = Arc::new(move |bytes| {
        let index = count.fetch_add(1, Ordering::Relaxed);
        bytes.fill([11, 22, 22, 33][index]);
        Ok(())
    });
    let (handle, prepared) = owner_with_entropy(entropy);
    let task = prepared.start(|| {});
    let _creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let second = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let old_token = second.resume_token.clone();
    let notices = second.notices.clone();
    let leave = handle.leave(binding(2), second.room_id.clone(), second.round_id);
    advance_tick().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    assert!(matches!(
        &*notices.borrow(),
        RosterOwnerNotice::Publication(notice)
            if !notice.environment_room().unwrap().seats[1].occupied
    ));

    let replacement = handle
        .join_sequenced(binding(3), Counter::new(2).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replacement.seat, SeatId::Two);
    assert_ne!(replacement.resume_token, old_token);
    assert_eq!(draws.load(Ordering::Relaxed), 4);
    assert!(matches!(
        handle
            .resume(
                binding(20),
                Counter::new(3).unwrap(),
                replacement.room_id.clone(),
                old_token,
            )
            .await
            .unwrap(),
        Err(RoomError::ResumeRejected)
    ));
    assert_eq!(
        handle
            .action(
                binding(2),
                replacement.room_id.clone(),
                replacement.round_id,
                RosterAction::Ready {
                    ready: false,
                    settings_revision: 1
                },
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    drop(second.guard);
    advance_tick().await;
    assert_eq!(
        handle
            .action(
                binding(3),
                replacement.room_id,
                replacement.round_id,
                RosterAction::Ready {
                    ready: false,
                    settings_revision: 1
                },
            )
            .await
            .unwrap(),
        Ok(())
    );
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn transport_loss_reserves_until_grace_expiry_then_allows_replacement() {
    let (handle, prepared) = owner_with_entropy(entropy_counter());
    let task = prepared.start(|| {});
    let _creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let second = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    drop(second.guard);
    advance_tick().await;
    assert!(matches!(
        handle
            .join_sequenced(binding(3), Counter::new(2).unwrap())
            .await
            .unwrap(),
        Err(RoomError::RoomFull)
    ));
    for _ in 1..grenadiers_core::room::reconnect::RECONNECT_GRACE_TICKS {
        advance_tick().await;
    }
    let replacement = handle
        .join_sequenced(binding(4), Counter::new(3).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replacement.seat, SeatId::Two);
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn failed_replacement_admission_leaves_released_seat_and_budget_atomic() {
    let draws = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&draws);
    let entropy: Arc<RandomFill> = Arc::new(move |bytes| {
        let draw = count.fetch_add(1, Ordering::Relaxed);
        if draw == 2 {
            return Err(());
        }
        bytes.fill(draw as u8 + 1);
        Ok(())
    });
    let (handle, prepared) = owner_with_entropy(entropy);
    let task = prepared.start(|| {});
    let _creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let second = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let leave = handle.leave(binding(2), second.room_id, second.round_id);
    advance_tick().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    assert!(matches!(
        handle
            .join_sequenced(binding(3), Counter::new(2).unwrap())
            .await
            .unwrap(),
        Err(RoomError::CapacityReached)
    ));
    let replacement = handle
        .join_sequenced(binding(4), Counter::new(3).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replacement.seat, SeatId::Two);
    assert_eq!(draws.load(Ordering::Relaxed), 4);
    task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn environment_forming_admission_budget_is_bounded() {
    let (handle, prepared) = owner_with_entropy(entropy_counter());
    let task = prepared.start(|| {});
    let _creator = handle
        .admit_creator(binding(1), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    let mut current = handle
        .join_sequenced(binding(2), Counter::new(1).unwrap())
        .await
        .unwrap()
        .unwrap();
    for generation in 0..254_u64 {
        let leave = handle.leave(
            binding(generation + 2),
            current.room_id.clone(),
            current.round_id,
        );
        advance_tick().await;
        assert_eq!(leave.await.unwrap(), Ok(()));
        current = handle
            .join_sequenced(binding(generation + 3), Counter::new(1).unwrap())
            .await
            .unwrap()
            .unwrap();
    }
    let leave = handle.leave(binding(256), current.room_id, current.round_id);
    advance_tick().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    assert!(matches!(
        handle
            .join_sequenced(binding(257), Counter::new(1).unwrap())
            .await
            .unwrap(),
        Err(RoomError::CapacityReached)
    ));
    task.abort();
}
