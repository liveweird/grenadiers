use std::{fmt, num::NonZeroU64};

use grenadiers_core::game::SimulationSeed;

/// Private room root, independently generated from arena fixtures and CPU seeds.
#[derive(Clone)]
pub(in crate::game::rooms) struct SimulationRoot([u8; 32]);

impl SimulationRoot {
    pub(in crate::game::rooms) fn generate() -> Result<Self, ()> {
        Self::with_entropy(|bytes| getrandom::fill(bytes).map_err(|_| ()))
    }

    fn with_entropy(fill: impl FnOnce(&mut [u8]) -> Result<(), ()>) -> Result<Self, ()> {
        let mut bytes = [0; 32];
        fill(&mut bytes)?;
        Ok(Self(bytes))
    }

    pub(in crate::game::rooms) fn for_round(&self, epoch: u64) -> Result<SimulationSeed, ()> {
        let epoch = NonZeroU64::new(epoch).ok_or(())?;
        Ok(SimulationSeed::new(self.0).for_round(epoch))
    }
}

impl fmt::Debug for SimulationRoot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SimulationRoot(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_entropy_is_requested_once_and_round_derivation_never_requests_more() {
        let mut calls = 0;
        let root = SimulationRoot::with_entropy(|bytes| {
            calls += 1;
            assert_eq!(bytes.len(), 32);
            bytes.fill(7);
            Ok(())
        })
        .unwrap();
        assert_eq!(calls, 1);
        let first = root.for_round(1).unwrap();
        assert_ne!(first, root.for_round(2).unwrap());
        assert_eq!(first, root.clone().for_round(1).unwrap());
        assert_eq!(root.for_round(0), Err(()));
        assert_eq!(first, root.for_round(1).unwrap());
        assert_eq!(calls, 1);
    }

    #[test]
    fn private_entropy_failure_never_returns_a_partial_root() {
        assert!(
            SimulationRoot::with_entropy(|bytes| {
                bytes[..8].fill(7);
                Err(())
            })
            .is_err()
        );
    }

    #[test]
    fn root_and_derived_seed_debug_output_are_redacted() {
        let root = SimulationRoot::with_entropy(|bytes| {
            bytes.fill(7);
            Ok(())
        })
        .unwrap();
        assert_eq!(format!("{root:?}"), "SimulationRoot(<redacted>)");
        assert_eq!(
            format!("{:?}", root.for_round(1).unwrap()),
            "SimulationSeed(<redacted>)"
        );
    }
}
