use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, AtomicUsize, Ordering},
};
use std::time::Duration;

use grenadiers_core::{
    game::Direction,
    protocol::{RoomCode, RoomId, v9::GameServerMessage},
};
use serde_json::json;
use tokio::{
    sync::watch,
    task::JoinHandle,
    time::{Instant, MissedTickBehavior, interval_at, sleep_until, timeout},
};

use super::{
    MatchAdmission, MatchOwnerHandle, MatchOwnerNotice, PreparedMatchOwner, ShutdownState,
    runner::tick_deadline, test_support::binding,
};
use crate::{
    game::rooms::{
        RandomFill,
        match_runtime::MatchAction,
        measurement,
        profile::RoomProfile,
        types::{RoomError, RoomIdentity},
    },
    round_seed::RoundSeedSource,
};

const PROFILE_DURATION: Duration = Duration::from_secs(30);
const COMMAND_CAPACITY: usize = 32;

struct MeasuredRoom {
    handle: MatchOwnerHandle,
    admissions: [MatchAdmission; 2],
    bindings: [crate::game::rooms::BindingKey; 2],
    shutdown: watch::Sender<ShutdownState>,
    task: JoinHandle<()>,
    cleaned: Arc<AtomicUsize>,
    created_at: Instant,
}

#[derive(Default)]
struct OwnerObservations {
    deadline_lag: Vec<Duration>,
    intervals: Vec<Duration>,
    frame_bytes: Vec<usize>,
    ticks: Vec<u64>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "30-second optimized reference measurement; run through scripts/measure-game.sh"]
async fn owner_one_room_30s_measurement() {
    run_owner_profile(1).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "30-second optimized reference measurement; run through scripts/measure-game.sh"]
async fn owner_eight_rooms_30s_measurement() {
    run_owner_profile(8).await;
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "optimized queue-pressure reference measurement; run through scripts/measure-game.sh"]
async fn owner_queue_pressure_measurement() {
    measurement::require_release();
    let observed = queue_pressure().await;
    measurement::emit(
        "owner-queue-pressure",
        json!({
            "workload": "fill an unstarted owner's bounded command queue, reject the next command, then shut down",
            "commandCapacity": COMMAND_CAPACITY,
            "observedCommandOccupancy": observed,
            "firstExcessRejected": true,
            "cleanupCompleted": true,
        }),
    );
}

async fn run_owner_profile(room_count: usize) {
    measurement::require_release();
    let mut rooms = Vec::with_capacity(room_count);
    for index in 0..room_count {
        rooms.push(start_room(index).await);
    }
    let start_ticks = rooms.iter().map(current_tick).collect::<Vec<_>>();
    let observations = (0..room_count)
        .map(|_| Arc::new(Mutex::new(OwnerObservations::default())))
        .collect::<Vec<_>>();
    let observers = rooms
        .iter()
        .zip(&observations)
        .map(|(room, data)| {
            observe(
                room.admissions[0].notices.clone(),
                room.created_at,
                Arc::clone(data),
            )
        })
        .collect::<Vec<_>>();

    let mut action_latency = Vec::with_capacity(room_count * 2 * 300);
    let mut sampled_queue_max = 0;
    let started = Instant::now();
    let deadline = started + PROFILE_DURATION;
    let mut refresh = interval_at(
        started + Duration::from_millis(100),
        Duration::from_millis(100),
    );
    refresh.set_missed_tick_behavior(MissedTickBehavior::Skip);
    while refresh.tick().await < deadline {
        for room in &rooms {
            for (index, direction) in [Direction::North, Direction::South].into_iter().enumerate() {
                sampled_queue_max = sampled_queue_max.max(queue_occupancy(&room.handle));
                let began = Instant::now();
                let reply = room.handle.action(
                    room.bindings[index],
                    room.admissions[index].room_id.clone(),
                    room.admissions[index].round_id,
                    MatchAction::Movement(Some(direction)),
                );
                sampled_queue_max = sampled_queue_max.max(queue_occupancy(&room.handle));
                let result = reply.await.expect("owner dropped normal-load reply");
                action_latency.push(began.elapsed());
                assert_eq!(result, Ok(()), "normal load rejected a legal refresh");
                sampled_queue_max = sampled_queue_max.max(queue_occupancy(&room.handle));
            }
        }
    }
    sleep_until(deadline).await;
    tokio::task::yield_now().await;
    let measured_duration = started.elapsed();
    let end_ticks = rooms.iter().map(current_tick).collect::<Vec<_>>();
    for observer in observers {
        observer.abort();
        let _ = observer.await;
    }

    let mut lags = Vec::new();
    let mut intervals = Vec::new();
    let mut frames = Vec::new();
    let mut observed_ticks = 0;
    for data in &observations {
        let data = data.lock().unwrap();
        lags.extend_from_slice(&data.deadline_lag);
        intervals.extend_from_slice(&data.intervals);
        frames.extend_from_slice(&data.frame_bytes);
        observed_ticks += data.ticks.len();
    }
    let expected_ticks = u64::try_from(measured_duration.as_millis() * 60 / 1_000).unwrap();
    for (start, end) in start_ticks.iter().zip(&end_ticks) {
        assert!(end >= &(start + expected_ticks.saturating_sub(6)));
    }
    let max_frame = frames.iter().copied().max().unwrap_or_default();
    let lag_p99 = measurement::percentile_duration(&lags, 99);
    assert!(max_frame <= 65_536);
    assert!(
        lag_p99 <= measurement::OWNER_LAG_P99_BUDGET,
        "observed publication lag p99 {lag_p99:?} exceeds {:?}",
        measurement::OWNER_LAG_P99_BUDGET
    );
    for room in &rooms {
        assert!(matches!(
            current_notice(room).room.phase,
            grenadiers_core::protocol::v9::RoomPhase::Running
        ));
    }

    shutdown_all(rooms).await;
    measurement::emit(
        if room_count == 1 {
            "owner-1"
        } else {
            "owner-8"
        },
        json!({
            "workload": "generated occupied rooms; two safe 10Hz movement refreshes per room",
            "rooms": room_count,
            "durationMs": measured_duration.as_millis(),
            "tokioWorkers": 2,
            "startTicks": start_ticks,
            "endTicks": end_ticks,
            "actionRoundTrip": measurement::summary(&action_latency),
            "publicationDeadlineLag": measurement::summary(&lags),
            "observedPublicationIntervals": measurement::summary(&intervals),
            "observedWatchChanges": observed_ticks,
            "maxObservedFrameBytes": max_frame,
            "sampledCommandOccupancyMax": sampled_queue_max,
            "sampledOccupancyIsNotTruePeak": true,
            "watchChangesMayCoalesce": true,
            "unreadSubscriberPerRoom": true,
            "publicationLagBudgetUs": measurement::OWNER_LAG_P99_BUDGET.as_micros(),
        }),
    );
}

async fn start_room(index: usize) -> MeasuredRoom {
    let created_at = Instant::now();
    let (shutdown, shutdown_rx) = watch::channel(false);
    let fill_counter = Arc::new(AtomicU8::new(1));
    let fill: Arc<RandomFill> = Arc::new(move |bytes| {
        bytes.fill(fill_counter.fetch_add(1, Ordering::Relaxed));
        Ok(())
    });
    let room_id = format!("{:032x}", index + 1);
    let identity = RoomIdentity {
        id: RoomId::new(&room_id).unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        profile: RoomProfile::GeneratedClassicV2,
    };
    let seed_byte = u8::try_from(index + 1).unwrap();
    let seeds = RoundSeedSource::with_entropy(move |bytes| {
        bytes.fill(seed_byte);
        Ok(())
    });
    let (handle, owner) =
        PreparedMatchOwner::new(identity, created_at, seeds, fill, shutdown_rx).unwrap();
    let cleaned = Arc::new(AtomicUsize::new(0));
    let cleaned_callback = Arc::clone(&cleaned);
    let task = owner.start(move || {
        cleaned_callback.fetch_add(1, Ordering::SeqCst);
    });
    let bindings = [
        binding((index * 2 + 1) as u64, 1),
        binding((index * 2 + 2) as u64, 1),
    ];
    let first = handle.join(bindings[0]).await.unwrap().unwrap();
    let second = handle.join(bindings[1]).await.unwrap().unwrap();
    for (binding, admission) in bindings.into_iter().zip([&first, &second]) {
        handle
            .action(
                binding,
                admission.room_id.clone(),
                admission.round_id,
                MatchAction::Ready(true),
            )
            .await
            .unwrap()
            .unwrap();
    }
    let mut running = first.notices.clone();
    timeout(Duration::from_secs(5), async {
        loop {
            if matches!(
                notice(&running).room.phase,
                grenadiers_core::protocol::v9::RoomPhase::Running
            ) {
                break;
            }
            running.changed().await.unwrap();
        }
    })
    .await
    .expect("real-time countdown exceeded five seconds");
    MeasuredRoom {
        handle,
        admissions: [first, second],
        bindings,
        shutdown,
        task,
        cleaned,
        created_at,
    }
}

fn observe(
    mut notices: watch::Receiver<MatchOwnerNotice>,
    created_at: Instant,
    data: Arc<Mutex<OwnerObservations>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut previous = None;
        while notices.changed().await.is_ok() {
            let received = Instant::now();
            let current = notice(&notices);
            let tick = current.room.tick.get();
            let expected = tick_deadline(created_at, tick);
            let lag = received.saturating_duration_since(expected);
            let bytes = current
                .frame(grenadiers_core::game::SeatId::One)
                .as_ref()
                .map_or_else(
                    || {
                        grenadiers_core::protocol::v9::encode_server_message(
                            &GameServerMessage::RoomState {
                                room: current.room.clone(),
                            },
                        )
                        .unwrap()
                        .len()
                    },
                    |snapshot| snapshot.len(),
                );
            let mut data = data.lock().unwrap();
            data.deadline_lag.push(lag);
            if let Some(previous) = previous.replace(received) {
                data.intervals.push(received.duration_since(previous));
            }
            data.frame_bytes.push(bytes);
            data.ticks.push(tick);
        }
    })
}

async fn queue_pressure() -> usize {
    let created_at = Instant::now();
    let (shutdown, shutdown_rx) = watch::channel(false);
    let (handle, owner) = PreparedMatchOwner::new(
        RoomIdentity {
            id: RoomId::new("ffffffffffffffffffffffffffffffff").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            profile: RoomProfile::GeneratedClassicV2,
        },
        created_at,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(0xee);
            Ok(())
        }),
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(1);
            Ok(())
        }),
        shutdown_rx,
    )
    .unwrap();
    let mut queued = Vec::with_capacity(COMMAND_CAPACITY);
    for index in 0..COMMAND_CAPACITY {
        queued.push(handle.join(binding(10_000 + index as u64, 1)));
    }
    let observed = queue_occupancy(&handle);
    assert_eq!(observed, COMMAND_CAPACITY);
    assert!(matches!(
        handle.join(binding(20_000, 1)).await.unwrap(),
        Err(RoomError::RateLimited)
    ));
    shutdown.send(true).unwrap();
    let cleaned = Arc::new(AtomicUsize::new(0));
    let cleaned_callback = Arc::clone(&cleaned);
    let task = owner.start(move || {
        cleaned_callback.fetch_add(1, Ordering::SeqCst);
    });
    timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    for reply in queued {
        assert!(matches!(reply.await.unwrap(), Err(RoomError::RoomNotFound)));
    }
    observed
}

async fn shutdown_all(rooms: Vec<MeasuredRoom>) {
    for room in &rooms {
        room.shutdown.send(true).unwrap();
    }
    for room in rooms {
        timeout(Duration::from_secs(5), room.task)
            .await
            .expect("owner shutdown exceeded five seconds")
            .unwrap();
        assert_eq!(room.cleaned.load(Ordering::SeqCst), 1);
    }
}

fn queue_occupancy(handle: &MatchOwnerHandle) -> usize {
    COMMAND_CAPACITY - handle.commands.capacity()
}

fn notice(receiver: &watch::Receiver<MatchOwnerNotice>) -> crate::game::rooms::MatchNotice {
    match &*receiver.borrow() {
        MatchOwnerNotice::Publication(notice) | MatchOwnerNotice::Closing { last: notice, .. } => {
            notice.clone()
        }
    }
}

fn current_notice(room: &MeasuredRoom) -> crate::game::rooms::MatchNotice {
    notice(&room.admissions[0].notices)
}

fn current_tick(room: &MeasuredRoom) -> u64 {
    current_notice(room).room.tick.get()
}
