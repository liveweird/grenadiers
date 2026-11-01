use grenadiers_core::cpu::survival::{
    CommittedContinuation, PreparedContinuation, PreparedRenewal,
};
use grenadiers_core::cpu::{DecisionId, MemoryKey, Observation};
use grenadiers_core::game::Tick;

pub(super) struct ContinuationClock {
    pub(super) decision: DecisionId,
    pub(super) basis_sequence: u64,
    pub(super) observed: Vec<(u64, Observation)>,
    pub(super) revoked: bool,
}

impl ContinuationClock {
    pub(super) const fn new(decision: DecisionId, basis_sequence: u64) -> Self {
        Self {
            decision,
            basis_sequence,
            observed: Vec::new(),
            revoked: false,
        }
    }

    pub(super) fn observe(&mut self, sequence: u64, observation: Option<&Observation>) -> bool {
        let stage = match u64::try_from(self.observed.len()) {
            Ok(value) => value + 1,
            Err(_) => {
                self.revoked = true;
                return false;
            }
        };
        let expected_sequence = self.basis_sequence.checked_add(stage);
        let expected_tick = stage
            .checked_mul(3)
            .and_then(|delta| self.decision.observation_tick.0.checked_add(delta))
            .map(Tick);
        let valid = !self.revoked
            && stage <= 2
            && expected_sequence == Some(sequence)
            && observation.is_some_and(|value| {
                Some(value.parts().tick) == expected_tick
                    && value.parts().world == self.decision.world
            });
        if !valid {
            self.revoked = true;
            return false;
        }
        self.observed.push((
            sequence,
            observation.expect("validated observation").clone(),
        ));
        true
    }
}

pub(super) struct PreparedAdmission {
    pub(super) prepared: PreparedContinuation,
    pub(super) decision: DecisionId,
    pub(super) key: MemoryKey,
}

pub(super) struct ContinuationAuthority {
    pub(super) committed: CommittedContinuation,
    pub(super) decision: DecisionId,
    pub(super) key: MemoryKey,
    pub(super) renewal: Option<PreparedRenewal>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::game::rooms) struct ContinuationTrace {
    pub(in crate::game::rooms) admission_tick: Option<Tick>,
    pub(in crate::game::rooms) first_movement_command_tick: Option<Tick>,
    pub(in crate::game::rooms) renewal_commits: u32,
    pub(in crate::game::rooms) command_commits: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use grenadiers_core::cpu::DecisionId;

    fn observation(tick: u64) -> Observation {
        let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
        parts.tick = Tick(tick);
        Observation::new(parts).unwrap()
    }

    fn clock() -> ContinuationClock {
        let basis = observation(20);
        ContinuationClock::new(
            DecisionId {
                world: basis.parts().world,
                command_epoch: 4,
                observation_tick: Tick(20),
                checked_request_serial: 1,
            },
            9,
        )
    }

    #[test]
    fn continuation_clock_accepts_only_actual_b_plus_three_and_six() {
        let mut clock = clock();
        assert!(clock.observe(10, Some(&observation(23))));
        assert!(clock.observe(11, Some(&observation(26))));
        assert_eq!(clock.observed.len(), 2);
        assert!(!clock.observe(12, Some(&observation(29))));
    }

    #[test]
    fn early_duplicate_missing_or_changed_publication_revokes_clock() {
        for (sequence, value) in [
            (10, Some(observation(21))),
            (9, Some(observation(23))),
            (10, None),
        ] {
            let mut clock = clock();
            assert!(!clock.observe(sequence, value.as_ref()));
            assert!(clock.revoked);
            assert!(!clock.observe(10, Some(&observation(23))));
        }
    }
}
