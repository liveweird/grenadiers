use std::{collections::VecDeque, future::pending, time::Duration};

use grenadiers_core::room::MatchPhase;
use tokio::time::{Instant, sleep_until};

use super::{
    driver::Driver,
    types::{COMMAND_CAPACITY, OwnerCommand, PreparedRosterOwner},
};
use crate::game::rooms::{match_owner::ClosingReason, types::RoomError};

pub(super) const LOBBY_LIFETIME: Duration = Duration::from_secs(300);
const TERMINAL_LIFETIME: Duration = Duration::from_secs(30);
const OVERLOAD_LIMIT: Duration = Duration::from_millis(500);
const MAX_CATCH_UP: usize = 4;

pub(super) async fn run(mut owner: PreparedRosterOwner) {
    let mut driver = Driver::new(
        owner.runtime,
        owner.initial,
        owner.notices,
        owner.created_at + LOBBY_LIFETIME,
        owner.random_fill,
        owner.cpus,
    );
    let mut admissions = VecDeque::with_capacity(COMMAND_CAPACITY);
    let mut catching_up = false;
    loop {
        if shutdown_requested(&mut owner.shutdown) {
            stop_for_shutdown(&mut driver, &mut owner.receiver, &mut admissions);
            return;
        }
        if close_at_boundaries(&mut driver) {
            return;
        }
        let tick = tick_timer(&driver, owner.created_at);
        let retirement = driver.terminal_at.map(|at| at + TERMINAL_LIFETIME);
        enum Wake {
            Command(Option<OwnerCommand>),
            Clock,
            Shutdown,
            Deadline,
        }
        let wake = tokio::select! {
            command = receive_commands(&mut owner.receiver, !catching_up) => Wake::Command(command),
            changed = owner.shutdown.changed() => { let _ = changed; Wake::Shutdown },
            () = sleep_optional(tick) => Wake::Clock,
            () = sleep_optional(driver.lobby_deadline) => Wake::Deadline,
            () = sleep_optional(retirement) => Wake::Deadline,
        };
        if matches!(wake, Wake::Shutdown) || shutdown_requested(&mut owner.shutdown) {
            stop_for_shutdown(&mut driver, &mut owner.receiver, &mut admissions);
            return;
        }
        if close_at_boundaries(&mut driver) {
            return;
        }
        match wake {
            Wake::Command(None) => {
                reject_admissions(&mut admissions);
                driver.fail();
                return;
            }
            Wake::Command(Some(command)) if command.is_admission() => {
                stage(&mut admissions, command)
            }
            Wake::Command(Some(command)) => {
                if driver.command(command, Instant::now()).is_err() {
                    driver.fail();
                    return;
                }
            }
            Wake::Clock | Wake::Deadline | Wake::Shutdown => {}
        }
        for _ in 1..COMMAND_CAPACITY {
            if catching_up {
                break;
            }
            match owner.receiver.try_recv() {
                Ok(command) if command.is_admission() => stage(&mut admissions, command),
                Ok(command) => {
                    if driver.command(command, Instant::now()).is_err() {
                        driver.fail();
                        return;
                    }
                }
                Err(_) => break,
            }
        }
        driver.inspect_departures();
        if close_at_boundaries(&mut driver) {
            return;
        }
        if matches!(
            driver.runtime.lifecycle().phase(),
            MatchPhase::Interrupted { .. }
        ) {
            reject_admissions(&mut admissions);
            continue;
        }
        let next = tick_deadline(owner.created_at, driver.runtime.lifecycle().tick().0 + 1);
        if !terminal(&driver) && Instant::now() >= next + OVERLOAD_LIMIT {
            if driver
                .interrupt_and_commit(
                    grenadiers_core::room::InterruptReason::ServerOverloaded,
                    Instant::now(),
                )
                .is_err()
            {
                reject_admissions(&mut admissions);
                driver.fail();
                return;
            }
            reject_admissions(&mut admissions);
            if driver.is_empty() {
                return;
            }
            continue;
        }
        let mut completed = 0;
        while completed < MAX_CATCH_UP
            && Instant::now()
                >= tick_deadline(owner.created_at, driver.runtime.lifecycle().tick().0 + 1)
        {
            if driver.tick(Instant::now()).is_err() {
                driver.fail();
                return;
            }
            if driver.is_empty() {
                return;
            }
            completed += 1;
            if matches!(
                driver.runtime.lifecycle().phase(),
                MatchPhase::Interrupted { .. }
            ) || close_at_boundaries(&mut driver)
            {
                break;
            }
        }
        if completed == MAX_CATCH_UP
            && Instant::now()
                >= tick_deadline(owner.created_at, driver.runtime.lifecycle().tick().0 + 1)
        {
            catching_up = true;
            tokio::task::yield_now().await;
            continue;
        }
        catching_up = false;
        if !driver.has_pending() {
            while let Some(command) = admissions.pop_front() {
                if driver.command(command, Instant::now()).is_err() {
                    reject_admissions(&mut admissions);
                    driver.fail();
                    return;
                }
            }
        }
    }
}

fn stage(admissions: &mut VecDeque<OwnerCommand>, command: OwnerCommand) {
    if admissions.len() == COMMAND_CAPACITY {
        command.reject(RoomError::RateLimited);
    } else {
        admissions.push_back(command);
    }
}

async fn receive_commands(
    receiver: &mut tokio::sync::mpsc::Receiver<OwnerCommand>,
    enabled: bool,
) -> Option<OwnerCommand> {
    if enabled {
        receiver.recv().await
    } else {
        pending().await
    }
}

fn reject_admissions(admissions: &mut VecDeque<OwnerCommand>) {
    while let Some(command) = admissions.pop_front() {
        command.reject_unavailable();
    }
}

fn close_at_boundaries(driver: &mut Driver) -> bool {
    let now = Instant::now();
    if driver
        .lobby_deadline
        .is_some_and(|deadline| now >= deadline)
    {
        driver.closing(ClosingReason::Expired);
        return true;
    }
    if driver
        .terminal_at
        .is_some_and(|at| now >= at + TERMINAL_LIFETIME)
    {
        driver.closing(ClosingReason::Retired);
        return true;
    }
    false
}

fn stop_for_shutdown(
    driver: &mut Driver,
    receiver: &mut tokio::sync::mpsc::Receiver<OwnerCommand>,
    admissions: &mut VecDeque<OwnerCommand>,
) {
    driver.settle_shutdown(Instant::now());
    while let Some(command) = admissions.pop_front() {
        command.reject(RoomError::RoomNotFound);
    }
    while let Ok(command) = receiver.try_recv() {
        command.reject(RoomError::RoomNotFound);
    }
}

fn terminal(driver: &Driver) -> bool {
    matches!(
        driver.runtime.lifecycle().phase(),
        MatchPhase::MatchResult { .. } | MatchPhase::Interrupted { .. }
    )
}

fn tick_timer(driver: &Driver, created_at: Instant) -> Option<Instant> {
    (!matches!(
        driver.runtime.lifecycle().phase(),
        MatchPhase::Interrupted { .. }
    ))
    .then(|| tick_deadline(created_at, driver.runtime.lifecycle().tick().0 + 1))
}

fn tick_deadline(epoch: Instant, tick: u64) -> Instant {
    let nanos = u128::from(tick) * 1_000_000_000 / 60;
    epoch
        + Duration::new(
            (nanos / 1_000_000_000) as u64,
            (nanos % 1_000_000_000) as u32,
        )
}

fn shutdown_requested(shutdown: &mut tokio::sync::watch::Receiver<bool>) -> bool {
    *shutdown.borrow_and_update() || shutdown.has_changed().is_err()
}

async fn sleep_optional(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => sleep_until(deadline).await,
        None => pending().await,
    }
}
