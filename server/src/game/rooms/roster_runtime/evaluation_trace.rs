use grenadiers_core::{
    game::{SeatId, Tick},
    room::DepartureReason,
};

use super::{runtime::RosterRuntime, types::RosterAction};

const MAX_EVALUATION_OPERATIONS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum EvaluationOperation {
    HumanAction {
        seat: SeatId,
        epoch: u64,
        action: RosterAction,
    },
    CpuAction {
        seat: SeatId,
        epoch: u64,
        action: RosterAction,
    },
    InputFence(SeatId),
    Settle {
        tick: Tick,
        departures: [Option<DepartureReason>; 4],
    },
    PublicationFlush,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum EvaluationTraceError {
    Disabled,
    Overflow,
    UnsupportedOperation,
}

#[derive(Clone, Debug, Default)]
pub(super) struct EvaluationTrace {
    operations: Vec<EvaluationOperation>,
    failure: Option<EvaluationTraceError>,
}

impl EvaluationTrace {
    fn push(&mut self, operation: EvaluationOperation) {
        if self.failure.is_some() {
            return;
        }
        if self.operations.len() == MAX_EVALUATION_OPERATIONS {
            self.failure = Some(EvaluationTraceError::Overflow);
            return;
        }
        self.operations.push(operation);
    }

    fn drain(&mut self) -> Result<Vec<EvaluationOperation>, EvaluationTraceError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        Ok(std::mem::take(&mut self.operations))
    }

    fn latch_unsupported_operation(&mut self) {
        self.failure
            .get_or_insert(EvaluationTraceError::UnsupportedOperation);
    }
}

impl RosterRuntime {
    pub(in crate::game::rooms) fn enable_evaluation_trace_for_test(&mut self) {
        self.evaluation_trace.get_or_insert_default();
    }

    pub(in crate::game::rooms) fn drain_evaluation_trace_for_test(
        &mut self,
    ) -> Result<Vec<EvaluationOperation>, EvaluationTraceError> {
        self.evaluation_trace
            .as_mut()
            .ok_or(EvaluationTraceError::Disabled)?
            .drain()
    }

    pub(super) fn record_evaluation_operation(&mut self, operation: EvaluationOperation) {
        if let Some(trace) = &mut self.evaluation_trace {
            trace.push(operation);
        }
    }

    pub(super) fn latch_unsupported_evaluation_operation(&mut self) {
        if let Some(trace) = &mut self.evaluation_trace {
            trace.latch_unsupported_operation();
        }
    }
}
