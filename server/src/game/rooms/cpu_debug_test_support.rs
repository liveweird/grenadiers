use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use tokio::sync::{Notify, watch};

use super::{GameRooms, Registry, cpu::CpuPlanner, rate_limit::OperationBudgets};
use crate::round_seed::RoundSeedSource;

impl GameRooms {
    pub(crate) fn for_cpu_debug_test(now: Instant) -> Arc<Self> {
        let draws = AtomicU64::new(0);
        let (shutdown, _) = watch::channel(false);
        Arc::new(Self {
            registry: std::sync::Mutex::new(Registry {
                rooms: HashMap::new(),
                codes: HashMap::new(),
                prefix: None,
                next_serial: 1,
                operations: OperationBudgets::new(now),
            }),
            random_fill: Arc::new(move |bytes| {
                let draw = draws.fetch_add(1, Ordering::Relaxed);
                if bytes.len() == 8 {
                    bytes.copy_from_slice(&0x1234_5678_9abc_def0_u64.to_be_bytes());
                } else if bytes.len() == 5 {
                    bytes.copy_from_slice(&draw.to_be_bytes()[3..]);
                } else {
                    bytes.fill(draw as u8 + 1);
                }
                Ok(())
            }),
            round_seed_source: RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(23);
                Ok(())
            }),
            cpu_planner: CpuPlanner::without_workers(),
            profile: super::profile::RoomProfile::ExpandedV5,
            shutdown,
            empty: Notify::new(),
        })
    }

    pub(crate) fn pump_cpu_debug_test_jobs(&self) {
        self.cpu_planner.pump_for_test(32);
    }
}
