use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::protocol::{RoomCode, RoomId};
use tokio::{
    sync::watch,
    task::JoinHandle,
    time::{Duration, Instant, advance},
};

use super::super::{
    match_runtime::{MatchAction, MatchNotice},
    profile::RoomProfile,
    types::{BindingKey, RoomIdentity},
};
use super::{
    MatchAdmission, MatchOwnerHandle, MatchOwnerNotice, PreparedMatchOwner, ShutdownState,
};
use crate::{game::sessions::ConnectionId, round_seed::RoundSeedSource};

pub(super) struct Harness {
    pub(super) handle: MatchOwnerHandle,
    pub(super) shutdown: watch::Sender<ShutdownState>,
    pub(super) notices: watch::Receiver<MatchOwnerNotice>,
    pub(super) task: JoinHandle<()>,
    pub(super) cleaned: Arc<AtomicUsize>,
}

pub(super) fn binding(connection: u64, generation: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection),
        generation,
    }
}

pub(super) fn identity(profile: RoomProfile) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        profile,
        mode: grenadiers_core::room::RoomMode::HumanDuel,
        registry_serial: 1,
        private_cpu_seed: None,
    }
}

pub(super) fn prepared() -> (
    MatchOwnerHandle,
    PreparedMatchOwner,
    watch::Sender<ShutdownState>,
) {
    prepared_with(
        RoomProfile::FixedClassicV1,
        RoundSeedSource::with_entropy(|_| panic!("fixed profile requested entropy")),
    )
}

pub(super) fn prepared_with(
    profile: RoomProfile,
    source: RoundSeedSource,
) -> (
    MatchOwnerHandle,
    PreparedMatchOwner,
    watch::Sender<ShutdownState>,
) {
    let (shutdown, shutdown_rx) = watch::channel(false);
    let (handle, owner) = PreparedMatchOwner::new(
        identity(profile),
        Instant::now(),
        source,
        std::sync::Arc::new({
            let next = std::sync::atomic::AtomicU8::new(42);
            move |bytes: &mut [u8]| {
                bytes.fill(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                Ok(())
            }
        }),
        shutdown_rx,
    )
    .unwrap();
    (handle, owner, shutdown)
}

pub(super) fn started() -> Harness {
    let (handle, owner, shutdown) = prepared();
    start_prepared(handle, owner, shutdown)
}

pub(super) fn start_prepared(
    handle: MatchOwnerHandle,
    owner: PreparedMatchOwner,
    shutdown: watch::Sender<ShutdownState>,
) -> Harness {
    let notices = owner.notices.subscribe();
    let cleaned = Arc::new(AtomicUsize::new(0));
    let cleanup = cleaned.clone();
    let task = owner.start(move || {
        cleanup.fetch_add(1, Ordering::SeqCst);
    });
    Harness {
        handle,
        shutdown,
        notices,
        task,
        cleaned,
    }
}

pub(super) async fn join(handle: &MatchOwnerHandle, binding: BindingKey) -> MatchAdmission {
    handle.join(binding).await.unwrap().unwrap()
}

pub(super) async fn advance_ticks(count: usize) {
    for _ in 0..count {
        advance(Duration::from_nanos(16_666_667)).await;
        tokio::task::yield_now().await;
    }
}

pub(super) async fn start_round(
    handle: &MatchOwnerHandle,
) -> (MatchAdmission, MatchAdmission, BindingKey, BindingKey) {
    let one = binding(1, 1);
    let two = binding(2, 1);
    let first = join(handle, one).await;
    let second = join(handle, two).await;
    handle
        .action(
            one,
            first.room_id.clone(),
            first.round_id,
            MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();
    handle
        .action(
            two,
            second.room_id.clone(),
            second.round_id,
            MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();
    advance_ticks(180).await;
    (first, second, one, two)
}

pub(super) async fn finish_round(
    handle: &MatchOwnerHandle,
    notices: &watch::Receiver<MatchOwnerNotice>,
    bomber: BindingKey,
    admission: &MatchAdmission,
) {
    let round_id = publication(notices).room.round_id.get() as u64;
    handle
        .action(
            bomber,
            admission.room_id.clone(),
            round_id,
            MatchAction::Bomb(1),
        )
        .await
        .unwrap()
        .unwrap();
    for _ in 0..400 {
        advance_ticks(1).await;
        if !matches!(
            publication(notices).room.phase,
            grenadiers_core::protocol::v12::RoomPhase::Running
        ) {
            return;
        }
    }
    panic!("round did not finish");
}

pub(super) async fn advance_to_running(notices: &watch::Receiver<MatchOwnerNotice>) {
    for _ in 0..400 {
        advance_ticks(1).await;
        if matches!(
            publication(notices).room.phase,
            grenadiers_core::protocol::v12::RoomPhase::Running
        ) {
            return;
        }
    }
    panic!("next round did not start");
}

pub(super) fn publication(receiver: &watch::Receiver<MatchOwnerNotice>) -> MatchNotice {
    match &*receiver.borrow() {
        MatchOwnerNotice::Publication(notice) | MatchOwnerNotice::Closing { last: notice, .. } => {
            notice.clone()
        }
    }
}

pub(super) fn assert_running(receiver: &watch::Receiver<MatchOwnerNotice>) {
    assert_eq!(
        publication(receiver).room.phase,
        grenadiers_core::protocol::v12::RoomPhase::Running
    );
}
