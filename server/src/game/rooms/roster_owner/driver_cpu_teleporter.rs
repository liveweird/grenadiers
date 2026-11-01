use grenadiers_core::cpu::MovementIntent;

use super::{Driver, fatal};
use crate::game::rooms::{
    cpu::TeleporterService,
    roster_runtime::{RosterAction, RosterRuntimeError},
};

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TeleporterOwnerTrace {
    Movement {
        runtime_tick: grenadiers_core::game::Tick,
        sealed_tick: grenadiers_core::game::Tick,
        accepted: bool,
    },
    Stop {
        runtime_tick: grenadiers_core::game::Tick,
        sealed_tick: grenadiers_core::game::Tick,
        mailbox_accepted: bool,
    },
    Fence {
        runtime_tick: grenadiers_core::game::Tick,
    },
}

#[cfg(test)]
thread_local! {
    static TELEPORTER_TRACE: std::cell::RefCell<Vec<TeleporterOwnerTrace>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

impl Driver {
    pub(super) fn apply_teleporter(&mut self, index: usize) -> Result<bool, RosterRuntimeError> {
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        match self.cpus[index].service_teleporter(tick, epoch) {
            TeleporterService::Idle => Ok(false),
            TeleporterService::Controlled | TeleporterService::Completed => Ok(true),
            TeleporterService::Invalidated => {
                self.fail_teleporter(index)?;
                Ok(true)
            }
            TeleporterService::Submit(submission) => {
                let seat = self.cpus[index].seat();
                if submission.admission_tick() != tick {
                    self.fail_teleporter(index)?;
                    return Ok(true);
                }
                let is_stop = submission.is_stop();
                let direction = movement(submission.movement());
                #[cfg(test)]
                let rejected =
                    std::mem::take(&mut self.continuation_rejections[usize::from(is_stop)]);
                #[cfg(not(test))]
                let rejected = false;
                let result = if rejected {
                    Err(RosterRuntimeError::InvalidPhase)
                } else {
                    self.runtime
                        .submit_cpu_intent(seat, epoch, RosterAction::Movement(direction))
                };
                if is_stop {
                    // This real clear is the event sealed by the Stop token,
                    // including when the redundant mailbox Stop is rejected.
                    self.runtime.stop_seat_inputs(seat);
                    #[cfg(test)]
                    trace(TeleporterOwnerTrace::Stop {
                        runtime_tick: tick,
                        sealed_tick: submission.admission_tick(),
                        mailbox_accepted: result.is_ok(),
                    });
                    if !self.cpus[index].commit_teleporter(submission) {
                        self.fail_teleporter(index)?;
                        return Ok(true);
                    }
                    self.cpu_stationary[seat.index()] = true;
                    self.cpu_stop_at[seat.index()] = None;
                    self.cpus[index].on_stop_admitted(tick);
                    return match result {
                        Err(error) if fatal(error) => Err(error),
                        Ok(_) | Err(_) => Ok(true),
                    };
                }
                #[cfg(test)]
                trace(TeleporterOwnerTrace::Movement {
                    runtime_tick: tick,
                    sealed_tick: submission.admission_tick(),
                    accepted: result.is_ok(),
                });
                match result {
                    Ok(_) => {}
                    Err(error) if fatal(error) => {
                        self.fail_teleporter(index)?;
                        return Err(error);
                    }
                    Err(_) => {
                        self.fail_teleporter(index)?;
                        return Ok(true);
                    }
                }
                if !self.cpus[index].commit_teleporter(submission) {
                    self.fail_teleporter(index)?;
                    return Ok(true);
                }
                self.cpu_stationary[seat.index()] = direction.is_none();
                self.cpu_stop_at[seat.index()] = None;
                self.cpus[index].on_movement_admitted();
                Ok(true)
            }
        }
    }

    fn fail_teleporter(&mut self, index: usize) -> Result<(), RosterRuntimeError> {
        let seat = self.cpus[index].seat();
        let epoch = self.runtime.lifecycle().round_attempt();
        let result = self
            .runtime
            .submit_cpu_intent(seat, epoch, RosterAction::Movement(None));
        self.runtime.stop_seat_inputs(seat);
        #[cfg(test)]
        trace(TeleporterOwnerTrace::Fence {
            runtime_tick: self.runtime.lifecycle().tick(),
        });
        self.cpus[index].invalidate_teleporter();
        self.cpu_stationary[seat.index()] = true;
        self.cpu_stop_at[seat.index()] = None;
        match result {
            Err(error) if fatal(error) => Err(error),
            Ok(_) | Err(_) => Ok(()),
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn reset_teleporter_trace_for_test(&self) {
        TELEPORTER_TRACE.with(|trace| trace.borrow_mut().clear());
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn teleporter_trace_for_test(
        &self,
    ) -> Vec<(
        u8,
        grenadiers_core::game::Tick,
        grenadiers_core::game::Tick,
        bool,
    )> {
        TELEPORTER_TRACE.with(|trace| {
            trace
                .borrow()
                .iter()
                .map(|event| match *event {
                    TeleporterOwnerTrace::Movement {
                        runtime_tick,
                        sealed_tick,
                        accepted,
                    } => (0, runtime_tick, sealed_tick, accepted),
                    TeleporterOwnerTrace::Stop {
                        runtime_tick,
                        sealed_tick,
                        mailbox_accepted,
                    } => (1, runtime_tick, sealed_tick, mailbox_accepted),
                    TeleporterOwnerTrace::Fence { runtime_tick } => {
                        (2, runtime_tick, runtime_tick, true)
                    }
                })
                .collect()
        })
    }
}

#[cfg(test)]
fn trace(event: TeleporterOwnerTrace) {
    TELEPORTER_TRACE.with(|trace| trace.borrow_mut().push(event));
}

const fn movement(intent: MovementIntent) -> Option<grenadiers_core::game::Direction> {
    match intent {
        MovementIntent::Direction(direction) => Some(direction),
        MovementIntent::Stop => None,
    }
}
