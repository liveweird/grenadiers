use grenadiers_core::game::SeatId;

use super::{MatchAction, MatchNotice, MatchRuntime};
use crate::game::rooms::match_runtime::{MatchRuntimeError, error::map_input_error};

impl MatchRuntime {
    pub(in crate::game::rooms) fn action(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        action: MatchAction,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        self.validate_actor(seat, expected_epoch)?;
        if matches!(action, MatchAction::Bomb(id) | MatchAction::Equipment(id)
            if grenadiers_core::protocol::Counter::new(id).is_err())
        {
            return Err(MatchRuntimeError::DuplicateAction);
        }
        let tick = self.lifecycle.tick();
        let result = match action {
            MatchAction::Ready(ready) => self.ready(seat, ready),
            MatchAction::ReadyChecked {
                ready,
                settings_revision,
            } => self.ready_checked(seat, ready, settings_revision),
            MatchAction::UpdateSettings {
                settings_revision,
                settings,
            } => self.update_settings(seat, settings_revision, settings),
            MatchAction::Movement(direction) => {
                self.require_running()?;
                self.inputs
                    .movement(seat, direction, self.lifecycle.tick())
                    .map_err(map_input_error)?;
                Ok(None)
            }
            MatchAction::Bomb(action_id) => {
                self.require_running()?;
                self.inputs.bomb(seat, action_id).map_err(map_input_error)?;
                Ok(None)
            }
            MatchAction::Equipment(action_id) => {
                self.require_running()?;
                self.require_punch_action()?;
                self.inputs
                    .equipment(seat, action_id)
                    .map_err(map_input_error)?;
                Ok(None)
            }
            MatchAction::Rematch(ready) => self.rematch(seat, ready),
        };
        if result.is_ok() {
            self.capture_action(tick, seat, expected_epoch, action);
        }
        result
    }

    fn require_punch_action(&self) -> Result<(), MatchRuntimeError> {
        let expanded = matches!(
            self.identity.profile,
            crate::game::rooms::profile::RoomProfile::ExpandedV5
        );
        if expanded && self.lifecycle.settings().enabled_items().contains_punch() {
            Ok(())
        } else {
            Err(MatchRuntimeError::InvalidPhase)
        }
    }
}
