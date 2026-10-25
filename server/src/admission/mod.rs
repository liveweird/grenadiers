use std::time::Duration;

mod policy;
mod telemetry;

const MAX_TRACKED_PEERS: usize = 4_096;
const IDLE_EVICTION_AGE: Duration = Duration::from_secs(60);
const SWEEP_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
struct BucketSpec {
    burst: f64,
    refill_per_second: f64,
}

#[derive(Clone, Copy)]
enum BudgetSpecs {
    Gameplay([BucketSpec; GameplayOperation::COUNT]),
    Diagnostic(BucketSpec),
}

#[derive(Clone, Copy)]
struct GameplayLimits {
    max_keys: usize,
    buckets: [BucketSpec; GameplayOperation::COUNT],
    socket_limit: usize,
    room_admission_limit: usize,
}

impl GameplayLimits {
    fn production() -> Self {
        Self {
            max_keys: MAX_TRACKED_PEERS,
            buckets: [BucketSpec {
                burst: 8.0,
                refill_per_second: 2.0,
            }; GameplayOperation::COUNT],
            socket_limit: 8,
            room_admission_limit: 4,
        }
    }
}

#[derive(Clone, Copy)]
struct DiagnosticLimits {
    max_keys: usize,
    bucket: BucketSpec,
    socket_limit: usize,
}

impl DiagnosticLimits {
    fn production() -> Self {
        Self {
            max_keys: MAX_TRACKED_PEERS,
            bucket: BucketSpec {
                burst: 4.0,
                refill_per_second: 1.0,
            },
            socket_limit: 4,
        }
    }
}

pub(crate) use policy::{
    DiagnosticAdmission, DiagnosticSocketPermit, GameplayAdmission, GameplaySocketPermit,
    RoomAdmissionPermit,
};
pub(crate) use telemetry::{DenialOperation, DenialReason, DenialTelemetry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum GameplayOperation {
    Mint,
    Upgrade,
    Create,
    Join,
    Resume,
}

impl GameplayOperation {
    const COUNT: usize = 5;

    fn denial_operation(self) -> DenialOperation {
        match self {
            Self::Mint => DenialOperation::Mint,
            Self::Upgrade => DenialOperation::GameplayUpgrade,
            Self::Create => DenialOperation::Create,
            Self::Join => DenialOperation::Join,
            Self::Resume => DenialOperation::Resume,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PolicyDenial {
    pub(crate) operation: DenialOperation,
    pub(crate) reason: DenialReason,
}
