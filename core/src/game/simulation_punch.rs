use super::{Direction, State};
use crate::game::bombs::BombTrigger;
use crate::game::movement;
use crate::game::types::{CellId, EntityId, SeatId, Tick};
use std::collections::BTreeSet;

const PUNCH_COOLDOWN_TICKS: u64 = 12;

#[derive(Clone, Copy)]
struct PunchClaim {
    seat: SeatId,
    bomb: EntityId,
    landing: CellId,
}

impl State {
    pub(super) fn resolve_punch_requests(
        &mut self,
        requests: [Option<SeatId>; 2],
        tick: Tick,
        remote_reservations: &BTreeSet<EntityId>,
    ) {
        let claims = requests.map(|request| {
            request.and_then(|seat| self.punch_claim(seat, tick, remote_reservations))
        });
        let accepted = match claims {
            [Some(left), Some(right)] if left.bomb == right.bomb => {
                if left.seat == self.assignment.priority_seat() {
                    [Some(left), None]
                } else {
                    [None, Some(right)]
                }
            }
            [Some(left), Some(right)] if left.landing == right.landing => [None, None],
            _ => claims,
        };
        for claim in accepted.into_iter().flatten() {
            let bomb = self
                .world
                .bombs
                .get_mut(&claim.bomb)
                .expect("punch claims reference live bombs");
            bomb.apply_punch(claim.landing);
            self.actors[claim.seat.index()].last_successful_punch_tick = Some(tick);
            for actor in &mut self.actors {
                actor.bomb_grace.remove(&claim.bomb);
            }
        }
    }

    fn punch_claim(
        &self,
        seat: SeatId,
        tick: Tick,
        remote_reservations: &BTreeSet<EntityId>,
    ) -> Option<PunchClaim> {
        let actor = &self.actors[seat.index()];
        let ready = actor.last_successful_punch_tick.is_none_or(|last| {
            tick.0
                .checked_sub(last.0)
                .is_some_and(|elapsed| elapsed >= PUNCH_COOLDOWN_TICKS)
        });
        if !actor.alive
            || !self.rules.enabled_items().contains_punch()
            || !actor.inventory.can_punch()
            || !ready
        {
            return None;
        }
        let actor_cell = actor.position.cell()?;
        let target = crate::game::punch_adjacent_cell(actor_cell, actor.heading)?;
        let bomb = self
            .world
            .bombs
            .values()
            .find(|bomb| bomb.cell() == target)?;
        let source = bomb.cell();
        let source_burning = self.world.has_fire(source, tick);
        let due =
            matches!(bomb.trigger(), BombTrigger::Timed { detonates_at } if detonates_at <= tick);
        if matches!(bomb.trigger(), BombTrigger::Contact)
            || source_burning
            || due
            || remote_reservations.contains(&bomb.id())
        {
            return None;
        }
        let landing = self.punch_landing(bomb.id(), source, actor.heading)?;
        Some(PunchClaim {
            seat,
            bomb: bomb.id(),
            landing,
        })
    }

    fn punch_landing(&self, id: EntityId, source: CellId, direction: Direction) -> Option<CellId> {
        for candidate in crate::game::punch_landing_candidates(source, direction) {
            let open = crate::game::punch_landing_is_open(
                self.world.arena.terrain(candidate),
                self.world.arena.blocks().contains(&candidate),
                self.world.oil.unlit(candidate),
                self.world
                    .bombs
                    .values()
                    .any(|bomb| bomb.id() != id && bomb.cell() == candidate),
                self.actors.iter().any(|actor| {
                    actor.alive && movement::position_overlaps_cell(actor.position, candidate)
                }),
            );
            if open {
                return Some(candidate);
            }
        }
        None
    }
}
