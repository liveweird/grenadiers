use super::{CpuPlanner, PlannedResultWitness, PumpReport, QUEUE_CAPACITY, execute_job, record};

impl CpuPlanner {
    pub(in crate::game::rooms) fn pump_for_test_witnessing(
        &self,
        limit: usize,
        mut inspect: impl FnMut(PlannedResultWitness<'_>),
    ) -> PumpReport {
        assert!(
            self.inner
                .workers
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .is_empty(),
            "only a zero-worker planner may be pumped"
        );
        let mut report = PumpReport::default();
        for _ in 0..limit.min(QUEUE_CAPACITY) {
            let job = self
                .inner
                ._receiver
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .try_recv();
            let job = match job {
                Ok(job) => job,
                Err(
                    std::sync::mpsc::TryRecvError::Empty
                    | std::sync::mpsc::TryRecvError::Disconnected,
                ) => break,
            };
            record(&mut report, execute_job(job, true, Some(&mut inspect)));
        }
        report
    }
}
