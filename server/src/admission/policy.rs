use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{peer_identity::PeerKey, rate_limit::TokenBucket};

use super::{
    BudgetSpecs, DenialOperation, DenialReason, DiagnosticLimits, GameplayLimits,
    GameplayOperation, IDLE_EVICTION_AGE, PolicyDenial, SWEEP_INTERVAL,
};

#[derive(Clone)]
pub(crate) struct GameplayAdmission {
    inner: Arc<Mutex<AdmissionTable>>,
}

impl GameplayAdmission {
    #[must_use]
    pub(crate) fn new(now: Instant) -> Self {
        Self::with_limits(now, GameplayLimits::production())
    }

    fn with_limits(now: Instant, limits: GameplayLimits) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AdmissionTable::new(
                now,
                limits.max_keys,
                limits.socket_limit,
                limits.room_admission_limit,
                BudgetSpecs::Gameplay(limits.buckets),
            ))),
        }
    }

    pub(crate) fn check(
        &self,
        key: PeerKey,
        operation: GameplayOperation,
        now: Instant,
    ) -> Result<(), PolicyDenial> {
        self.inner
            .lock()
            .expect("gameplay admission mutex is not poisoned")
            .check(key, operation.denial_operation(), operation as usize, now)
    }

    pub(crate) fn acquire_socket(
        &self,
        key: PeerKey,
        now: Instant,
    ) -> Result<GameplaySocketPermit, PolicyDenial> {
        self.acquire(
            key,
            PermitKind::Socket,
            DenialOperation::GameplayUpgrade,
            now,
        )
        .map(|permit| GameplaySocketPermit { _permit: permit })
    }

    pub(crate) fn acquire_room_admission(
        &self,
        key: PeerKey,
        now: Instant,
    ) -> Result<RoomAdmissionPermit, PolicyDenial> {
        self.acquire(
            key,
            PermitKind::Admission,
            DenialOperation::RoomAdmission,
            now,
        )
        .map(|permit| RoomAdmissionPermit { _permit: permit })
    }

    fn acquire(
        &self,
        key: PeerKey,
        kind: PermitKind,
        operation: DenialOperation,
        now: Instant,
    ) -> Result<Permit, PolicyDenial> {
        self.inner
            .lock()
            .expect("gameplay admission mutex is not poisoned")
            .acquire(key, kind, operation, now)?;
        Ok(Permit::new(Arc::clone(&self.inner), key, kind))
    }

    #[cfg(test)]
    fn counts(&self, key: PeerKey) -> Option<(usize, usize)> {
        self.inner
            .lock()
            .ok()?
            .entries
            .get(&key)
            .map(|entry| (entry.sockets, entry.admissions))
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.inner
            .lock()
            .expect("mutex is not poisoned")
            .entries
            .len()
    }
}

#[derive(Clone)]
pub(crate) struct DiagnosticAdmission {
    inner: Arc<Mutex<AdmissionTable>>,
}

impl DiagnosticAdmission {
    #[must_use]
    pub(crate) fn new(now: Instant) -> Self {
        Self::with_limits(now, DiagnosticLimits::production())
    }

    fn with_limits(now: Instant, limits: DiagnosticLimits) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AdmissionTable::new(
                now,
                limits.max_keys,
                limits.socket_limit,
                0,
                BudgetSpecs::Diagnostic(limits.bucket),
            ))),
        }
    }

    pub(crate) fn check_upgrade(&self, key: PeerKey, now: Instant) -> Result<(), PolicyDenial> {
        self.inner
            .lock()
            .expect("diagnostic admission mutex is not poisoned")
            .check(key, DenialOperation::DiagnosticUpgrade, 0, now)
    }

    pub(crate) fn acquire_socket(
        &self,
        key: PeerKey,
        now: Instant,
    ) -> Result<DiagnosticSocketPermit, PolicyDenial> {
        self.inner
            .lock()
            .expect("diagnostic admission mutex is not poisoned")
            .acquire(
                key,
                PermitKind::Socket,
                DenialOperation::DiagnosticUpgrade,
                now,
            )?;
        Ok(DiagnosticSocketPermit {
            _permit: Permit::new(Arc::clone(&self.inner), key, PermitKind::Socket),
        })
    }
}

enum Budgets {
    Gameplay([TokenBucket; GameplayOperation::COUNT]),
    Diagnostic(TokenBucket),
}

impl Budgets {
    fn new(now: Instant, specs: BudgetSpecs) -> Self {
        match specs {
            BudgetSpecs::Gameplay(specs) => Self::Gameplay(
                specs.map(|spec| TokenBucket::new(now, spec.burst, spec.refill_per_second)),
            ),
            BudgetSpecs::Diagnostic(spec) => {
                Self::Diagnostic(TokenBucket::new(now, spec.burst, spec.refill_per_second))
            }
        }
    }

    fn take(&mut self, index: usize, now: Instant) -> bool {
        match self {
            Self::Gameplay(buckets) => buckets[index].take(now),
            Self::Diagnostic(bucket) => bucket.take(now),
        }
    }
}

struct AdmissionTable {
    entries: HashMap<PeerKey, Entry>,
    max_keys: usize,
    socket_limit: usize,
    admission_limit: usize,
    specs: BudgetSpecs,
    last_sweep: Instant,
}

impl AdmissionTable {
    fn new(
        now: Instant,
        max_keys: usize,
        socket_limit: usize,
        admission_limit: usize,
        specs: BudgetSpecs,
    ) -> Self {
        Self {
            entries: HashMap::new(),
            max_keys,
            socket_limit,
            admission_limit,
            specs,
            last_sweep: now,
        }
    }

    fn check(
        &mut self,
        key: PeerKey,
        operation: DenialOperation,
        bucket: usize,
        now: Instant,
    ) -> Result<(), PolicyDenial> {
        let entry = self.entry(key, operation, now)?;
        entry.last_activity = entry.last_activity.max(now);
        if entry.budgets.take(bucket, now) {
            Ok(())
        } else {
            Err(PolicyDenial {
                operation,
                reason: DenialReason::PeerRate,
            })
        }
    }

    fn acquire(
        &mut self,
        key: PeerKey,
        kind: PermitKind,
        operation: DenialOperation,
        now: Instant,
    ) -> Result<(), PolicyDenial> {
        let limit = match kind {
            PermitKind::Socket => self.socket_limit,
            PermitKind::Admission => self.admission_limit,
        };
        let entry = self.entry(key, operation, now)?;
        entry.last_activity = entry.last_activity.max(now);
        let count = match kind {
            PermitKind::Socket => &mut entry.sockets,
            PermitKind::Admission => &mut entry.admissions,
        };
        if *count >= limit {
            return Err(PolicyDenial {
                operation,
                reason: DenialReason::PeerCapacity,
            });
        }
        *count += 1;
        Ok(())
    }

    fn entry(
        &mut self,
        key: PeerKey,
        operation: DenialOperation,
        now: Instant,
    ) -> Result<&mut Entry, PolicyDenial> {
        if !self.entries.contains_key(&key) {
            self.sweep(now);
            if self.entries.len() >= self.max_keys {
                return Err(PolicyDenial {
                    operation,
                    reason: DenialReason::TrackingCapacity,
                });
            }
            self.entries.insert(key, Entry::new(now, self.specs));
        }
        Ok(self.entries.get_mut(&key).expect("entry exists"))
    }

    fn sweep(&mut self, now: Instant) {
        if now.saturating_duration_since(self.last_sweep) < SWEEP_INTERVAL {
            return;
        }
        self.last_sweep = self.last_sweep.max(now);
        self.entries.retain(|_, entry| {
            entry.sockets != 0
                || entry.admissions != 0
                || now.saturating_duration_since(entry.last_activity) < IDLE_EVICTION_AGE
        });
    }

    fn release(&mut self, key: PeerKey, kind: PermitKind, now: Instant) {
        if let Some(entry) = self.entries.get_mut(&key) {
            let count = match kind {
                PermitKind::Socket => &mut entry.sockets,
                PermitKind::Admission => &mut entry.admissions,
            };
            *count = count.checked_sub(1).expect("permit count cannot underflow");
            entry.last_activity = entry.last_activity.max(now);
        }
    }
}

struct Entry {
    budgets: Budgets,
    sockets: usize,
    admissions: usize,
    last_activity: Instant,
}

impl Entry {
    fn new(now: Instant, specs: BudgetSpecs) -> Self {
        Self {
            budgets: Budgets::new(now, specs),
            sockets: 0,
            admissions: 0,
            last_activity: now,
        }
    }
}

#[derive(Clone, Copy)]
enum PermitKind {
    Socket,
    Admission,
}

struct Permit {
    state: Option<(Arc<Mutex<AdmissionTable>>, PeerKey, PermitKind)>,
}

impl Permit {
    fn new(table: Arc<Mutex<AdmissionTable>>, key: PeerKey, kind: PermitKind) -> Self {
        Self {
            state: Some((table, key, kind)),
        }
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        if let Some((table, key, kind)) = self.state.take() {
            table
                .lock()
                .expect("admission mutex is not poisoned")
                .release(key, kind, Instant::now());
        }
    }
}

// Each typed wrapper owns the permit so scope exit and cancellation release the quota.
pub(crate) struct GameplaySocketPermit {
    _permit: Permit,
}
pub(crate) struct RoomAdmissionPermit {
    _permit: Permit,
}
pub(crate) struct DiagnosticSocketPermit {
    _permit: Permit,
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
