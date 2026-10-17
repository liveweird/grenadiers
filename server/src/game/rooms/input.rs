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
    place_bomb: bool,
    last_bomb_action: Option<u32>,
    admitted: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputError {
    RateLimited,
    DuplicateAction,
    TickOverflow,
}

impl InputMailbox {
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
        if self.seats[seat.index()]
            .last_bomb_action
            .is_some_and(|previous| action_id <= previous)
        {
            return Err(InputError::DuplicateAction);
        }
        let mailbox = self.admit(seat)?;
        mailbox.last_bomb_action = Some(action_id);
        mailbox.place_bomb = true;
        Ok(())
    }

    pub(super) fn take(&mut self, tick: Tick) -> Vec<SeatInput> {
        let mut inputs = Vec::new();
        let mut placements = [false; 2];
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
            placements[seat.index()] = mailbox.place_bomb;
            mailbox.place_bomb = false;
            mailbox.admitted = 0;
        }
        for seat in SeatId::ALL {
            if placements[seat.index()] {
                inputs.push(SeatInput {
                    seat,
                    action: InputAction::PlaceBomb,
                });
            }
        }
        inputs
    }

    pub(super) fn clear_pending(&mut self) {
        for mailbox in &mut self.seats {
            mailbox.direction = None;
            mailbox.expires_at = None;
            mailbox.inputs.clear();
            mailbox.place_bomb = false;
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
        mailbox.place_bomb = false;
        mailbox.admitted = 0;
    }

    pub(super) fn reset_round(&mut self) {
        self.clear_pending();
        for mailbox in &mut self.seats {
            mailbox.last_bomb_action = None;
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
