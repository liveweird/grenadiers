use grenadiers_core::game::{Direction, SeatId};

use super::super::super::super::super::match_runtime::{MatchAction, MatchRuntimeError};
use super::{super::super::Driver, HumanScript};

pub(super) fn apply(
    driver: &mut Driver,
    script: HumanScript,
    running_tick: u64,
    manual_id: &mut u32,
) -> Result<(), MatchRuntimeError> {
    if script == HumanScript::Idle {
        return Ok(());
    }
    let action = match running_tick % 360 {
        0 | 10 => Some(MatchAction::Movement(Some(Direction::East))),
        20 => Some(MatchAction::Movement(None)),
        21 => {
            let id = *manual_id;
            *manual_id = manual_id
                .checked_add(1)
                .ok_or(MatchRuntimeError::CapacityReached)?;
            Some(MatchAction::Bomb(id))
        }
        22 | 32 => Some(MatchAction::Movement(Some(Direction::West))),
        42 => Some(MatchAction::Movement(None)),
        43 | 53 => Some(MatchAction::Movement(Some(Direction::South))),
        63 => Some(MatchAction::Movement(None)),
        270 | 280 => Some(MatchAction::Movement(Some(Direction::North))),
        290 => Some(MatchAction::Movement(None)),
        _ => None,
    };
    if let Some(action) = action {
        driver.runtime.action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            action,
        )?;
    }
    Ok(())
}
