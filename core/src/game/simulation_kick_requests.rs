use super::State;
use std::collections::BTreeSet;

impl State {
    pub(super) fn resolve_kick_requests(
        &mut self,
        requests: Vec<Option<(crate::game::EntityId, crate::game::Direction)>>,
    ) {
        let mut accepted = BTreeSet::new();
        for seat in self.assignment.priority_order(self.player_count) {
            let Some((id, direction)) = requests[seat.index()] else {
                continue;
            };
            if !accepted.insert(id) {
                continue;
            }
            let Some(bomb) = self.world.bombs.get_mut(&id) else {
                continue;
            };
            if bomb.motion().is_none()
                && !matches!(bomb.trigger(), crate::game::BombTrigger::Contact)
            {
                bomb.arm_kick(direction);
            }
        }
    }
}
