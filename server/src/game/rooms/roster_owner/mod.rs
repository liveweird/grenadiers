mod admissions;
mod cpu_debug;
#[cfg(test)]
mod cpu_debug_tests;
mod driver;
mod handle;
mod runner;
mod types;

use std::sync::Arc;

use grenadiers_core::room::{CpuDifficulty, CpuProfile};
use tokio::{sync::watch, task::JoinHandle, time::Instant};

use super::{RandomFill, cpu::CpuController, roster_runtime::RosterRuntime};
pub(in crate::game::rooms) use types::{PreparedRosterOwner, RosterOwnerHandle};
pub(crate) use types::{RosterAdmission, RosterOwnerNotice};

impl PreparedRosterOwner {
    pub(in crate::game::rooms) fn new(
        mut runtime: RosterRuntime,
        cpus: Vec<CpuController>,
        created_at: Instant,
        random_fill: Arc<RandomFill>,
        shutdown: watch::Receiver<bool>,
    ) -> Result<(RosterOwnerHandle, Self), super::roster_runtime::RosterRuntimeError> {
        validate_cpus(&runtime, &cpus)?;
        let initial = runtime.current_notice()?;
        let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(initial.clone()));
        drop(receiver);
        let (commands, command_receiver) = tokio::sync::mpsc::channel(types::COMMAND_CAPACITY);
        let (cpu_debug_commands, cpu_debug_receiver) =
            tokio::sync::mpsc::channel(types::CPU_DEBUG_COMMAND_CAPACITY);
        Ok((
            RosterOwnerHandle {
                commands,
                cpu_debug_commands,
            },
            Self {
                created_at,
                receiver: command_receiver,
                cpu_debug_receiver,
                notices,
                initial,
                runtime,
                random_fill,
                cpus,
                shutdown,
            },
        ))
    }

    pub(in crate::game::rooms) fn start(
        self,
        on_exit: impl FnOnce() + Send + 'static,
    ) -> JoinHandle<()> {
        let cleanup = CleanupGuard(Some(Box::new(on_exit)));
        tokio::spawn(async move {
            let _cleanup = cleanup;
            runner::run(self).await;
        })
    }
}

fn validate_cpus(
    runtime: &RosterRuntime,
    cpus: &[CpuController],
) -> Result<(), super::roster_runtime::RosterRuntimeError> {
    let identity = runtime.identity();
    let configured: Vec<_> = identity.configuration.cpu_seats().collect();
    if configured.len() != cpus.len() {
        return Err(super::roster_runtime::RosterRuntimeError::UnsupportedConfiguration);
    }
    let expected_profile = match (
        runtime.spike_tuning(),
        runtime.respawn_tuning(),
        runtime.lair_tuning(),
        runtime.environment(),
        identity.configuration.cpu_difficulty(),
    ) {
        (Some(_), _, _, _, Some(CpuDifficulty::Easy)) => Some(CpuProfile::RELIABLE_V27_EASY),
        (Some(_), _, _, _, Some(CpuDifficulty::Standard)) => {
            Some(CpuProfile::RELIABLE_V27_STANDARD)
        }
        (None, Some(_), _, _, Some(CpuDifficulty::Easy)) => Some(CpuProfile::RESPAWN_LAIRS_EASY),
        (None, Some(_), _, _, Some(CpuDifficulty::Standard)) => {
            Some(CpuProfile::RESPAWN_LAIRS_STANDARD)
        }
        (None, None, Some(_), _, Some(CpuDifficulty::Easy)) => Some(CpuProfile::LAIRS_EASY),
        (None, None, Some(_), _, Some(CpuDifficulty::Standard)) => Some(CpuProfile::LAIRS_STANDARD),
        (None, None, None, Some(_), Some(CpuDifficulty::Easy)) => {
            Some(CpuProfile::ENVIRONMENT_EASY)
        }
        (None, None, None, Some(_), Some(CpuDifficulty::Standard)) => {
            Some(CpuProfile::ENVIRONMENT_STANDARD)
        }
        (None, None, None, None, Some(CpuDifficulty::Easy)) => Some(CpuProfile::MULTIPLAYER_EASY),
        (None, None, None, None, Some(CpuDifficulty::Standard)) => {
            Some(CpuProfile::MULTIPLAYER_STANDARD)
        }
        (_, _, _, _, None) => None,
        (Some(_), _, _, _, Some(CpuDifficulty::Hard)) if runtime.uses_gameplay19() => {
            Some(CpuProfile::HARD_V28)
        }
        (_, _, _, _, Some(CpuDifficulty::Hard)) => {
            return Err(super::roster_runtime::RosterRuntimeError::UnsupportedConfiguration);
        }
    };
    for (index, cpu) in cpus.iter().enumerate() {
        if cpu.seat() != configured[index]
            || cpus[..index].iter().any(|other| other.seat() == cpu.seat())
            || cpu.room_instance() != identity.registry_serial
            || Some(cpu.profile()) != expected_profile
        {
            return Err(super::roster_runtime::RosterRuntimeError::UnsupportedConfiguration);
        }
    }
    Ok(())
}

struct CleanupGuard(Option<Box<dyn FnOnce() + Send>>);

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Some(cleanup) = self.0.take() {
            cleanup();
        }
    }
}

#[cfg(test)]
mod environment_acceptance_tests;
#[cfg(test)]
mod environment_cpu_tests;
#[cfg(test)]
mod forming_replacement_tests;
#[cfg(test)]
mod lair_cpu_acceptance_tests;
#[cfg(test)]
mod lair_cpu_opening_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod cpu_pickup_progress_tests;

#[cfg(test)]
mod cpu_strategy_progress_tests;

#[cfg(test)]
mod spike_cpu_acceptance_tests;

#[cfg(test)]
mod cpu_intent_progress_tests;

#[cfg(test)]
mod cpu_farm_progress_tests;

#[cfg(test)]
mod reliable_profile_tests;

#[cfg(test)]
#[path = "../cpu/hard_episode_tests.rs"]
mod hard_episode_tests;
