use std::{future::pending, sync::Weak, time::Duration};

use grenadiers_core::{
    protocol::RoomId,
    room::{DepartureReason, InterruptReason, Phase, RoomLifecycle},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
    time::{Instant, sleep_until},
};

use super::{GameRooms, input::RoomAction, publication, runtime::Runtime, types::*};
use crate::round_seed::RoundSeedSource;

pub(super) const COMMAND_CAPACITY: usize = 32;
pub(super) const ROOM_LIFETIME: Duration = Duration::from_secs(300);
const TERMINAL_LIFETIME: Duration = Duration::from_secs(30);
const OVERLOAD_LIMIT: Duration = Duration::from_millis(500);
const MAX_CATCH_UP: usize = 4;

pub(super) enum RoomCommand {
    Join {
        binding: BindingKey,
        reply: oneshot::Sender<Result<Admission, RoomError>>,
    },
    Leave {
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
    Action {
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: RoomAction,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
}

pub(super) struct PreparedOwner {
    identity: RoomIdentity,
    created_at: Instant,
    receiver: mpsc::Receiver<RoomCommand>,
    notices: watch::Sender<RoomNotice>,
    round_seed_source: RoundSeedSource,
}

impl PreparedOwner {
    pub(super) fn new(
        identity: RoomIdentity,
        created_at: Instant,
        round_seed_source: RoundSeedSource,
    ) -> (mpsc::Sender<RoomCommand>, Self) {
        let (commands, receiver) = mpsc::channel(COMMAND_CAPACITY);
        let initial = RoomNotice::View(
            publication::room_view(&identity, &RoomLifecycle::new(), 1).expect("initial view"),
        );
        let (notices, initial_receiver) = watch::channel(initial);
        drop(initial_receiver);
        (
            commands,
            Self {
                identity,
                created_at,
                receiver,
                notices,
                round_seed_source,
            },
        )
    }

    pub(super) fn start(self, registry: Weak<GameRooms>) -> JoinHandle<()> {
        let guard = RegistrationGuard {
            registry,
            identity: self.identity.clone(),
        };
        tokio::spawn(run(guard, self))
    }
}

struct RegistrationGuard {
    registry: Weak<GameRooms>,
    identity: RoomIdentity,
}

impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.cleanup(&self.identity.id, &self.identity.code);
        }
    }
}

async fn run(_guard: RegistrationGuard, mut owner: PreparedOwner) {
    let mut runtime = Runtime::new(owner.identity, owner.notices, owner.round_seed_source);
    loop {
        let deadline = tick_deadline(owner.created_at, runtime.lifecycle.tick().0 + 1);
        let retirement = runtime.terminal_at.map(|at| at + TERMINAL_LIFETIME);
        let waiting_expiry = matches!(
            runtime.lifecycle.phase(),
            Phase::Waiting | Phase::Countdown { .. }
        )
        .then_some(owner.created_at + ROOM_LIFETIME);
        enum Wake {
            Command(Option<RoomCommand>),
            Departure(usize, DepartureReason),
            Clock,
            Expired,
            Retired,
        }
        let [first, second] = &mut runtime.departures;
        let wake = tokio::select! {
            departure = receive_departure(first) => Wake::Departure(0, departure),
            departure = receive_departure(second) => Wake::Departure(1, departure),
            command = owner.receiver.recv() => Wake::Command(command),
            () = sleep_until(deadline), if !matches!(runtime.lifecycle.phase(), Phase::Interrupted { .. }) => Wake::Clock,
            () = sleep_optional(waiting_expiry) => Wake::Expired,
            () = sleep_optional(retirement) => Wake::Retired,
        };
        if waiting_expiry.is_some_and(|expires_at| Instant::now() >= expires_at) {
            let _ = runtime.notices.send(RoomNotice::Expired);
            return;
        }
        if retirement.is_some_and(|retires_at| Instant::now() >= retires_at) {
            let _ = runtime.notices.send(RoomNotice::Retired);
            return;
        }
        match wake {
            Wake::Command(None) => return,
            Wake::Command(Some(command)) => runtime.command(command),
            Wake::Departure(index, reason) => runtime.depart(index, reason),
            Wake::Clock => {}
            Wake::Expired => {
                let _ = runtime.notices.send(RoomNotice::Expired);
                return;
            }
            Wake::Retired => {
                let _ = runtime.notices.send(RoomNotice::Retired);
                return;
            }
        }
        for _ in 1..COMMAND_CAPACITY {
            if waiting_expiry.is_some_and(|expires_at| Instant::now() >= expires_at) {
                let _ = runtime.notices.send(RoomNotice::Expired);
                return;
            }
            if retirement.is_some_and(|retires_at| Instant::now() >= retires_at) {
                let _ = runtime.notices.send(RoomNotice::Retired);
                return;
            }
            match owner.receiver.try_recv() {
                Ok(command) => runtime.command(command),
                Err(_) => break,
            }
        }
        runtime.drain_departures();
        if runtime.failed {
            return;
        }
        if runtime.bindings.iter().all(Option::is_none) {
            return;
        }
        if matches!(runtime.lifecycle.phase(), Phase::Interrupted { .. }) {
            continue;
        }
        let next = tick_deadline(owner.created_at, runtime.lifecycle.tick().0 + 1);
        if !matches!(runtime.lifecycle.phase(), Phase::Result { .. })
            && Instant::now() >= next + OVERLOAD_LIMIT
        {
            runtime.interrupt(InterruptReason::ServerOverloaded);
            if runtime.failed {
                return;
            }
            continue;
        }
        let mut completed = 0;
        while completed < MAX_CATCH_UP
            && Instant::now() >= tick_deadline(owner.created_at, runtime.lifecycle.tick().0 + 1)
        {
            if !runtime.tick() {
                return;
            }
            completed += 1;
            if matches!(runtime.lifecycle.phase(), Phase::Interrupted { .. }) {
                break;
            }
        }
        if completed == MAX_CATCH_UP {
            tokio::task::yield_now().await;
        }
        if runtime.failed {
            return;
        }
    }
}
fn tick_deadline(epoch: Instant, tick: u64) -> Instant {
    let nanos = u128::from(tick) * 1_000_000_000 / 60;
    let seconds = (nanos / 1_000_000_000) as u64;
    let subsecond = (nanos % 1_000_000_000) as u32;
    epoch + Duration::new(seconds, subsecond)
}

async fn sleep_optional(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => sleep_until(deadline).await,
        None => pending().await,
    }
}

async fn receive_departure(
    receiver: &mut Option<oneshot::Receiver<DepartureReason>>,
) -> DepartureReason {
    match receiver {
        Some(receiver) => receiver.await.unwrap_or(DepartureReason::TransportLoss),
        None => pending().await,
    }
}

#[cfg(test)]
#[path = "owner_tests.rs"]
mod tests;
