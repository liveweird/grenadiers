use crate::game::Tick;
use crate::room::SpikePitSettings;

use super::{SpikePitClock, SpikePitClockError, SpikePitClockState, SpikePitPhase};

impl SpikePitClock {
    /// Rebuilds a checked clock from one public observation without inferring
    /// private activation history.
    pub fn from_observation(
        phase: SpikePitPhase,
        remaining_ticks: u64,
        observed_at: Tick,
        settings: SpikePitSettings,
    ) -> Result<Self, SpikePitClockError> {
        let invalid = || SpikePitClockError::InvalidRemainingTicks {
            phase,
            remaining_ticks,
        };
        let state = match phase {
            SpikePitPhase::Idle => {
                if remaining_ticks != 0 {
                    return Err(invalid());
                }
                return Ok(Self {
                    state: SpikePitClockState::Idle,
                    last_retraction_update: Some(observed_at),
                    last_updated_at: Some(observed_at),
                });
            }
            SpikePitPhase::Warning => {
                if remaining_ticks == 0 || remaining_ticks > settings.warning_ticks() {
                    return Err(invalid());
                }
                let rises_at = checked_deadline(observed_at, remaining_ticks)?;
                let retracts_at = checked_deadline(rises_at, settings.raised_ticks())?;
                SpikePitClockState::Warning {
                    rises_at,
                    retracts_at,
                }
            }
            SpikePitPhase::Raised => {
                if remaining_ticks == 0 || remaining_ticks > settings.raised_ticks() {
                    return Err(invalid());
                }
                SpikePitClockState::Raised {
                    rises_at: observed_at,
                    retracts_at: checked_deadline(observed_at, remaining_ticks)?,
                }
            }
        };
        Ok(Self {
            state,
            last_retraction_update: None,
            last_updated_at: Some(observed_at),
        })
    }
}

fn checked_deadline(at: Tick, duration: u64) -> Result<Tick, SpikePitClockError> {
    at.0.checked_add(duration)
        .map(Tick)
        .ok_or(SpikePitClockError::DeadlineOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_bounds_are_phase_specific() {
        let settings = SpikePitSettings::DEFAULT;
        assert!(
            SpikePitClock::from_observation(SpikePitPhase::Idle, 0, Tick(10), settings).is_ok()
        );
        for (phase, remaining) in [
            (SpikePitPhase::Idle, 1),
            (SpikePitPhase::Warning, 0),
            (SpikePitPhase::Warning, 37),
            (SpikePitPhase::Raised, 0),
            (SpikePitPhase::Raised, 181),
        ] {
            assert_eq!(
                SpikePitClock::from_observation(phase, remaining, Tick(10), settings),
                Err(SpikePitClockError::InvalidRemainingTicks {
                    phase,
                    remaining_ticks: remaining,
                })
            );
        }
        assert!(
            SpikePitClock::from_observation(SpikePitPhase::Warning, 36, Tick(10), settings).is_ok()
        );
        assert!(
            SpikePitClock::from_observation(SpikePitPhase::Raised, 180, Tick(10), settings).is_ok()
        );
    }

    #[test]
    fn future_deadlines_are_checked_before_construction() {
        let settings = SpikePitSettings::DEFAULT;
        assert_eq!(
            SpikePitClock::from_observation(
                SpikePitPhase::Warning,
                1,
                Tick(u64::MAX - 100),
                settings,
            ),
            Err(SpikePitClockError::DeadlineOverflow)
        );
        assert_eq!(
            SpikePitClock::from_observation(SpikePitPhase::Raised, 2, Tick(u64::MAX - 1), settings,),
            Err(SpikePitClockError::DeadlineOverflow)
        );
    }

    #[test]
    fn reconstructed_phases_round_trip_across_future_boundaries() {
        let settings = SpikePitSettings::DEFAULT;
        let warning =
            SpikePitClock::from_observation(SpikePitPhase::Warning, 6, Tick(100), settings)
                .unwrap();
        assert_eq!(warning.phase_at(Tick(100)), Ok(SpikePitPhase::Warning));
        assert_eq!(warning.remaining_ticks(Tick(100)), Ok(6));
        assert_eq!(warning.phase_at(Tick(105)), Ok(SpikePitPhase::Warning));
        assert_eq!(warning.phase_at(Tick(106)), Ok(SpikePitPhase::Raised));
        assert_eq!(warning.phase_at(Tick(285)), Ok(SpikePitPhase::Raised));
        assert_eq!(warning.phase_at(Tick(286)), Ok(SpikePitPhase::Idle));

        let raised =
            SpikePitClock::from_observation(SpikePitPhase::Raised, 3, Tick(200), settings).unwrap();
        assert!(raised.raised_interval().unwrap().contains(Tick(200)));
        assert_eq!(raised.phase_at(Tick(202)), Ok(SpikePitPhase::Raised));
        assert_eq!(raised.phase_at(Tick(203)), Ok(SpikePitPhase::Idle));
    }

    #[test]
    fn observed_idle_uses_a_same_tick_retraction_fence() {
        let settings = SpikePitSettings::DEFAULT;
        let mut idle =
            SpikePitClock::from_observation(SpikePitPhase::Idle, 0, Tick(50), settings).unwrap();
        assert_eq!(
            idle.update(Tick(50), true, settings),
            Ok(SpikePitPhase::Idle)
        );
        assert_eq!(
            idle.update(Tick(51), true, settings),
            Ok(SpikePitPhase::Warning)
        );
    }
}
