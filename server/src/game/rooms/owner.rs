use std::{future::pending, sync::Weak, time::Duration};

use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, MapId, RoomPhase, RoomView, RulesVersion, Seat, SeatStatus, Tick},
    room::{DepartureReason, Phase, RoomLifecycle},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    time::{Instant, MissedTickBehavior, interval_at, sleep_until},
};

use super::{GameRooms, types::*};

pub(super) const COMMAND_CAPACITY: usize = 8;
pub(super) const ROOM_LIFETIME: Duration = Duration::from_secs(300);
const TICK_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

pub(super) enum RoomCommand {
    Join {
        binding: BindingKey,
        reply: oneshot::Sender<Result<Admission, RoomError>>,
    },
    Leave {
        binding: BindingKey,
        room_id: grenadiers_core::protocol::RoomId,
        round_id: u64,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
    Validate {
        binding: BindingKey,
        room_id: grenadiers_core::protocol::RoomId,
        round_id: u64,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
}

pub(super) fn spawn(
    registry: Weak<GameRooms>,
    identity: RoomIdentity,
    created_at: Instant,
) -> mpsc::Sender<RoomCommand> {
    let (commands, receiver) = mpsc::channel(COMMAND_CAPACITY);
    let initial =
        RoomNotice::View(view(&identity, &RoomLifecycle::new(), 1).expect("initial view"));
    let (notices, notice_receiver) = watch::channel(initial);
    tokio::spawn(run(registry, identity, created_at, receiver, notices));
    drop(notice_receiver);
    commands
}

async fn run(
    registry: Weak<GameRooms>,
    identity: RoomIdentity,
    created_at: Instant,
    mut commands: mpsc::Receiver<RoomCommand>,
    notices: watch::Sender<RoomNotice>,
) {
    let expires_at = created_at + ROOM_LIFETIME;
    let mut ticks = interval_at(created_at + TICK_INTERVAL, TICK_INTERVAL);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Burst);
    let mut lifecycle = RoomLifecycle::new();
    let mut revision = 0_u32;
    let mut bindings: [Option<BindingKey>; 2] = [None, None];
    let mut departures: [Option<oneshot::Receiver<DepartureReason>>; 2] = [None, None];

    loop {
        enum Event {
            Command(Option<RoomCommand>),
            Tick,
            Expired,
            Departure(usize, DepartureReason),
        }
        let [first_departure, second_departure] = &mut departures;
        let event = tokio::select! {
            biased;
            _ = sleep_until(expires_at) => Event::Expired,
            departure = receive_departure(first_departure) => {
                Event::Departure(0, departure)
            }
            departure = receive_departure(second_departure) => {
                Event::Departure(1, departure)
            }
            command = commands.recv() => Event::Command(command),
            _ = ticks.tick() => Event::Tick,
        };
        if Instant::now() >= expires_at || matches!(event, Event::Expired) {
            let _ = notices.send(RoomNotice::Expired);
            cleanup(&registry, &identity).await;
            return;
        }
        match event {
            Event::Command(Some(command)) => {
                handle_command(
                    command,
                    &identity,
                    &mut lifecycle,
                    &mut revision,
                    &mut bindings,
                    &mut departures,
                    &notices,
                );
                if bindings.iter().all(Option::is_none) {
                    cleanup(&registry, &identity).await;
                    return;
                }
            }
            Event::Command(None) | Event::Expired => {
                cleanup(&registry, &identity).await;
                return;
            }
            Event::Tick => {
                if lifecycle.advance_idle_tick().is_err() {
                    let _ = notices.send(RoomNotice::Expired);
                    cleanup(&registry, &identity).await;
                    return;
                }
            }
            Event::Departure(index, reason) => {
                departures[index] = None;
                bindings[index] = None;
                let seat = SeatId::ALL[index];
                let _ = lifecycle.depart(seat, reason);
                publish(&identity, &lifecycle, &mut revision, &notices);
                if bindings.iter().all(Option::is_none) {
                    cleanup(&registry, &identity).await;
                    return;
                }
            }
        }
    }
}

fn handle_command(
    command: RoomCommand,
    identity: &RoomIdentity,
    lifecycle: &mut RoomLifecycle,
    revision: &mut u32,
    bindings: &mut [Option<BindingKey>; 2],
    departures: &mut [Option<oneshot::Receiver<DepartureReason>>; 2],
    notices: &watch::Sender<RoomNotice>,
) {
    match command {
        RoomCommand::Join { binding, reply } => {
            if reply.is_closed() {
                return;
            }
            let Some(index) = bindings.iter().position(Option::is_none) else {
                let _ = reply.send(Err(RoomError::RoomFull));
                return;
            };
            let seat_id = SeatId::ALL[index];
            if lifecycle.occupy(seat_id).is_err() {
                let _ = reply.send(Err(RoomError::RoomFull));
                return;
            }
            bindings[index] = Some(binding);
            let (departure, receiver) = oneshot::channel();
            departures[index] = Some(receiver);
            let Some(room_view) = publish(identity, lifecycle, revision, notices) else {
                rollback(index, lifecycle, bindings, departures);
                let _ = reply.send(Err(RoomError::CapacityReached));
                return;
            };
            let admission = Admission {
                room_id: identity.id.clone(),
                round_id: lifecycle.round_attempt(),
                seat: wire_seat(seat_id),
                view: room_view,
                notices: notices.subscribe(),
                guard: DepartureGuard::new(departure),
            };
            if let Err(Ok(admission)) = reply.send(Ok(admission)) {
                drop(admission);
                rollback(index, lifecycle, bindings, departures);
                publish(identity, lifecycle, revision, notices);
            }
        }
        RoomCommand::Leave {
            binding,
            room_id,
            round_id,
            reply,
        } => {
            if reply.is_closed() {
                return;
            }
            if room_id != identity.id || round_id != lifecycle.round_attempt() {
                let _ = reply.send(Err(RoomError::StaleRoom));
                return;
            }
            let Some(index) = bindings
                .iter()
                .position(|candidate| *candidate == Some(binding))
            else {
                let _ = reply.send(Err(RoomError::StaleRoom));
                return;
            };
            departures[index] = None;
            bindings[index] = None;
            let _ = lifecycle.depart(SeatId::ALL[index], DepartureReason::Explicit);
            publish(identity, lifecycle, revision, notices);
            let _ = reply.send(Ok(()));
        }
        RoomCommand::Validate {
            binding,
            room_id,
            round_id,
            reply,
        } => {
            if reply.is_closed() {
                return;
            }
            let valid = room_id == identity.id
                && round_id == lifecycle.round_attempt()
                && bindings.contains(&Some(binding));
            let _ = reply.send(if valid {
                Ok(())
            } else {
                Err(RoomError::StaleRoom)
            });
        }
    }
}

fn rollback(
    index: usize,
    lifecycle: &mut RoomLifecycle,
    bindings: &mut [Option<BindingKey>; 2],
    departures: &mut [Option<oneshot::Receiver<DepartureReason>>; 2],
) {
    departures[index] = None;
    bindings[index] = None;
    let _ = lifecycle.depart(SeatId::ALL[index], DepartureReason::TransportLoss);
}

fn publish(
    identity: &RoomIdentity,
    lifecycle: &RoomLifecycle,
    revision: &mut u32,
    notices: &watch::Sender<RoomNotice>,
) -> Option<RoomView> {
    let next_revision = revision.checked_add(1)?;
    let room_view = view(identity, lifecycle, next_revision)?;
    *revision = next_revision;
    notices.send_replace(RoomNotice::View(room_view.clone()));
    Some(room_view)
}

fn view(identity: &RoomIdentity, lifecycle: &RoomLifecycle, revision: u32) -> Option<RoomView> {
    let seats = SeatId::ALL.map(|seat| {
        let state = lifecycle.seats()[seat.index()];
        SeatStatus {
            seat: wire_seat(seat),
            occupied: state.occupied(),
            ready: state.ready(),
        }
    });
    let phase = match lifecycle.phase() {
        Phase::Waiting => RoomPhase::Waiting,
        _ => return None,
    };
    Some(RoomView {
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: Counter::new(u32::try_from(lifecycle.round_attempt()).ok()?).ok()?,
        revision: Counter::new(revision).ok()?,
        tick: Tick::new(lifecycle.tick().0).ok()?,
        rules_version: RulesVersion,
        map_id: MapId,
        seats,
        phase,
    })
}

const fn wire_seat(seat: SeatId) -> Seat {
    match seat {
        SeatId::One => Seat::One,
        SeatId::Two => Seat::Two,
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

async fn cleanup(registry: &Weak<GameRooms>, identity: &RoomIdentity) {
    if let Some(registry) = registry.upgrade() {
        registry.cleanup(&identity.id, &identity.code);
    }
}
