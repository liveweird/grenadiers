use grenadiers_core::game::{Direction, InputAction, SeatId, SeatInput, Tick};

pub(super) const COMMANDS_PER_UPDATE: u8 = 8;
pub(super) const MOVEMENT_TTL_TICKS: u64 = 15;

#[derive(Clone, Debug, Default)]
pub(super) struct InputMailbox {
    seats: [SeatMailbox; 2],
}

#[derive(Clone, Debug, Default)]
struct SeatMailbox {
    direction: Option<Direction>,
    expires_at: Option<Tick>,
    inputs: Vec<SeatInput>,
    manual_actions: Vec<ManualAction>,
    manual_action_high_water: Option<u32>,
    admitted: u8,
}

#[derive(Clone, Copy, Debug)]
enum ManualAction {
    PlaceBomb,
    UseEquipment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputError {
    RateLimited,
    DuplicateAction,
    TickOverflow,
}

impl InputMailbox {
    pub(super) fn last_manual_action(&self, seat: SeatId) -> Option<u32> {
        self.seats[seat.index()].manual_action_high_water
    }

    pub(super) fn movement(
        &mut self,
        seat: SeatId,
        direction: Option<Direction>,
        tick: Tick,
    ) -> Result<(), InputError> {
        let mailbox = self.admit(seat)?;
        mailbox.expires_at = Some(Tick(
            tick.0
                .checked_add(MOVEMENT_TTL_TICKS + 1)
                .ok_or(InputError::TickOverflow)?,
        ));
        if mailbox.direction == direction {
            return Ok(());
        }
        if let Some(new_direction) = direction {
            mailbox.inputs.push(SeatInput {
                seat,
                action: InputAction::Press(new_direction),
            });
        }
        match (mailbox.direction, direction) {
            (Some(old), Some(_)) => mailbox.inputs.push(SeatInput {
                seat,
                action: InputAction::Release(old),
            }),
            (Some(_), None) => mailbox.inputs.push(SeatInput {
                seat,
                action: InputAction::Clear,
            }),
            _ => {}
        }
        mailbox.direction = direction;
        Ok(())
    }

    pub(super) fn bomb(&mut self, seat: SeatId, action_id: u32) -> Result<(), InputError> {
        self.manual_action(seat, action_id, ManualAction::PlaceBomb)
    }

    pub(super) fn equipment(&mut self, seat: SeatId, action_id: u32) -> Result<(), InputError> {
        self.manual_action(seat, action_id, ManualAction::UseEquipment)
    }

    fn manual_action(
        &mut self,
        seat: SeatId,
        action_id: u32,
        action: ManualAction,
    ) -> Result<(), InputError> {
        if self.seats[seat.index()]
            .manual_action_high_water
            .is_some_and(|previous| action_id <= previous)
        {
            return Err(InputError::DuplicateAction);
        }
        let mailbox = self.admit(seat)?;
        mailbox.manual_action_high_water = Some(action_id);
        mailbox.manual_actions.push(action);
        Ok(())
    }

    pub(super) fn take(&mut self, tick: Tick) -> Vec<SeatInput> {
        let mut inputs = Vec::new();
        for seat in SeatId::ALL {
            let mailbox = &mut self.seats[seat.index()];
            if mailbox
                .direction
                .is_some_and(|_| mailbox.expires_at == Some(tick))
            {
                mailbox.direction = None;
                mailbox.expires_at = None;
                mailbox.inputs.push(SeatInput {
                    seat,
                    action: InputAction::Clear,
                });
            }
            inputs.append(&mut mailbox.inputs);
            mailbox.admitted = 0;
        }
        for seat in SeatId::ALL {
            for action in self.seats[seat.index()].manual_actions.drain(..) {
                let action = match action {
                    ManualAction::PlaceBomb => InputAction::PlaceBomb,
                    ManualAction::UseEquipment => InputAction::UseEquipment,
                };
                inputs.push(SeatInput { seat, action });
            }
        }
        inputs
    }

    pub(super) fn clear_pending(&mut self) {
        for mailbox in &mut self.seats {
            mailbox.direction = None;
            mailbox.expires_at = None;
            mailbox.inputs.clear();
            mailbox.manual_actions.clear();
            mailbox.admitted = 0;
        }
    }

    pub(super) fn stop(&mut self, seat: SeatId) {
        let mailbox = &mut self.seats[seat.index()];
        mailbox.direction = None;
        mailbox.expires_at = None;
        mailbox.inputs.clear();
        mailbox.inputs.push(SeatInput {
            seat,
            action: InputAction::Clear,
        });
        mailbox.manual_actions.clear();
        mailbox.admitted = 0;
    }

    pub(super) fn reset_round(&mut self) {
        self.clear_pending();
        for mailbox in &mut self.seats {
            mailbox.manual_action_high_water = None;
        }
    }

    fn admit(&mut self, seat: SeatId) -> Result<&mut SeatMailbox, InputError> {
        let mailbox = &mut self.seats[seat.index()];
        if mailbox.admitted >= COMMANDS_PER_UPDATE {
            return Err(InputError::RateLimited);
        }
        mailbox.admitted += 1;
        Ok(mailbox)
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
