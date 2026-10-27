use super::actions::ManualAction;
use super::{MAX_INPUTS_PER_TICK, SeatId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManualActionQueue {
    actions: [ManualAction; MAX_INPUTS_PER_TICK],
    len: usize,
}

impl Default for ManualActionQueue {
    fn default() -> Self {
        Self {
            actions: [ManualAction::PlaceBomb; MAX_INPUTS_PER_TICK],
            len: 0,
        }
    }
}

impl ManualActionQueue {
    pub(super) fn push(&mut self, action: ManualAction) {
        let slot = self
            .actions
            .get_mut(self.len)
            .expect("the validated logical-input bound guarantees queue capacity");
        *slot = action;
        self.len += 1;
    }

    pub(super) fn clear(&mut self) {
        self.len = 0;
    }

    pub(super) fn as_slice(&self) -> &[ManualAction] {
        &self.actions[..self.len]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManualActionQueues {
    pub(super) seats: Vec<ManualActionQueue>,
}

impl ManualActionQueues {
    pub(super) fn new(count: usize) -> Self {
        Self {
            seats: vec![ManualActionQueue::default(); count],
        }
    }

    pub(super) fn for_seat(&self, seat: SeatId) -> &ManualActionQueue {
        &self.seats[seat.index()]
    }
}
