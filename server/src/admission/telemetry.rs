use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

const DEFAULT_WINDOW: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum DenialOperation {
    Mint,
    GameplayUpgrade,
    DiagnosticUpgrade,
    Create,
    Join,
    Resume,
    RoomAdmission,
}

impl DenialOperation {
    const COUNT: usize = 7;
    const ALL: [Self; Self::COUNT] = [
        Self::Mint,
        Self::GameplayUpgrade,
        Self::DiagnosticUpgrade,
        Self::Create,
        Self::Join,
        Self::Resume,
        Self::RoomAdmission,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum DenialReason {
    PeerRate,
    GlobalRate,
    PeerCapacity,
    GlobalCapacity,
    TrackingCapacity,
    InvalidProxyIdentity,
    OriginRejected,
    AuthenticationFailed,
    ResumeRejected,
}

impl DenialReason {
    const COUNT: usize = 9;
    const ALL: [Self; Self::COUNT] = [
        Self::PeerRate,
        Self::GlobalRate,
        Self::PeerCapacity,
        Self::GlobalCapacity,
        Self::TrackingCapacity,
        Self::InvalidProxyIdentity,
        Self::OriginRejected,
        Self::AuthenticationFailed,
        Self::ResumeRejected,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DenialSummary {
    counts: [[u64; DenialReason::COUNT]; DenialOperation::COUNT],
}

#[derive(Debug, PartialEq, Eq)]
struct DenialCount {
    operation: DenialOperation,
    reason: DenialReason,
    count: u64,
}

impl DenialSummary {
    fn nonzero(&self) -> Vec<DenialCount> {
        DenialOperation::ALL
            .into_iter()
            .enumerate()
            .flat_map(|(operation_index, operation)| {
                DenialReason::ALL.into_iter().enumerate().filter_map(
                    move |(reason_index, reason)| {
                        let count = self.counts[operation_index][reason_index];
                        (count != 0).then_some(DenialCount {
                            operation,
                            reason,
                            count,
                        })
                    },
                )
            })
            .collect()
    }
}

pub(crate) struct DenialTelemetry {
    window: Duration,
    state: Mutex<TelemetryWindow>,
}

impl DenialTelemetry {
    #[must_use]
    pub(crate) fn new(now: Instant) -> Self {
        Self::with_window(now, DEFAULT_WINDOW)
    }

    fn with_window(now: Instant, window: Duration) -> Self {
        Self {
            window,
            state: Mutex::new(TelemetryWindow {
                started_at: now,
                counts: [[0; DenialReason::COUNT]; DenialOperation::COUNT],
            }),
        }
    }

    pub(crate) fn record(&self, operation: DenialOperation, reason: DenialReason, now: Instant) {
        let summary = {
            let mut state = self
                .state
                .lock()
                .expect("denial telemetry mutex is not poisoned");
            let summary = if now.saturating_duration_since(state.started_at) >= self.window {
                let summary = DenialSummary {
                    counts: state.counts,
                };
                state.counts = [[0; DenialReason::COUNT]; DenialOperation::COUNT];
                state.started_at = state.started_at.max(now);
                Some(summary)
            } else {
                None
            };
            state.counts[operation as usize][reason as usize] =
                state.counts[operation as usize][reason as usize].saturating_add(1);
            summary
        };
        if let Some(summary) = summary {
            tracing::warn!(target: "grenadiers::admission", denials = ?summary.nonzero(), "admission denials");
        }
    }

    #[cfg(test)]
    fn snapshot(&self) -> DenialSummary {
        DenialSummary {
            counts: self.state.lock().expect("mutex is not poisoned").counts,
        }
    }
}

struct TelemetryWindow {
    started_at: Instant,
    counts: [[u64; DenialReason::COUNT]; DenialOperation::COUNT],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_are_a_fixed_operation_by_reason_matrix() {
        let now = Instant::now();
        let telemetry = DenialTelemetry::new(now);
        telemetry.record(DenialOperation::Join, DenialReason::PeerRate, now);
        telemetry.record(DenialOperation::Join, DenialReason::PeerRate, now);
        telemetry.record(DenialOperation::Resume, DenialReason::ResumeRejected, now);
        let summary = telemetry.snapshot();
        assert_eq!(
            summary.counts[DenialOperation::Join as usize][DenialReason::PeerRate as usize],
            2
        );
        assert_eq!(
            summary.counts[DenialOperation::Resume as usize][DenialReason::ResumeRejected as usize],
            1
        );
        assert_eq!(summary.counts.len(), DenialOperation::COUNT);
        assert_eq!(summary.counts[0].len(), DenialReason::COUNT);
    }

    #[test]
    fn backwards_time_does_not_roll_or_reset_the_window() {
        let now = Instant::now();
        let later = now + Duration::from_secs(1);
        let telemetry = DenialTelemetry::with_window(now, Duration::from_secs(2));
        telemetry.record(DenialOperation::Mint, DenialReason::PeerRate, later);
        telemetry.record(DenialOperation::Mint, DenialReason::PeerRate, now);
        assert_eq!(telemetry.snapshot().counts[0][0], 2);
    }

    #[test]
    fn elapsed_window_is_aggregated_before_the_new_denial() {
        let now = Instant::now();
        let telemetry = DenialTelemetry::with_window(now, Duration::from_secs(1));
        telemetry.record(DenialOperation::Mint, DenialReason::PeerRate, now);
        telemetry.record(
            DenialOperation::Join,
            DenialReason::GlobalRate,
            now + Duration::from_secs(1),
        );
        let current = telemetry.snapshot();
        assert_eq!(
            current.counts[DenialOperation::Mint as usize][DenialReason::PeerRate as usize],
            0
        );
        assert_eq!(
            current.counts[DenialOperation::Join as usize][DenialReason::GlobalRate as usize],
            1
        );
    }

    #[test]
    fn readable_summary_contains_only_nonzero_typed_pairs() {
        let mut counts = [[0; DenialReason::COUNT]; DenialOperation::COUNT];
        counts[DenialOperation::Mint as usize][DenialReason::PeerRate as usize] = 3;
        let entries = DenialSummary { counts }.nonzero();
        assert_eq!(
            entries,
            vec![DenialCount {
                operation: DenialOperation::Mint,
                reason: DenialReason::PeerRate,
                count: 3,
            }]
        );
    }
}
