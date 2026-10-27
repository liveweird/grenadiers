use super::contract::ForcedMovement;
use super::shove_geometry::ContactEntry;
use super::{Direction, SeatId, State, Tick};
use crate::game::geometry::MovementSegment;
use crate::game::movement::{self, MovementOutcome};
use crate::game::types::{HALF_FOOTPRINT, Position, UNITS_PER_CELL};
use std::cmp::Ordering;

const SHOVE_COOLDOWN_TICKS: u64 = 30;

#[derive(Clone, Copy)]
struct ShoveClaim {
    pusher: SeatId,
    victim: SeatId,
    direction: Direction,
}

impl State {
    pub(super) fn resolve_shove(
        &mut self,
        outcomes: &[MovementOutcome],
        tick: Tick,
    ) -> Vec<ForcedMovement> {
        if !self.rules.enabled_items().contains_bracelet() {
            return Vec::new();
        }
        let priority: Vec<_> = self.assignment.priority_order(self.player_count).collect();
        let claims: Vec<_> = priority
            .iter()
            .filter_map(|&pusher| self.shove_claim(pusher, outcomes, tick, &priority))
            .collect();
        let paired = disjoint_claims(claims);
        let mut accepted: Vec<(ShoveClaim, ForcedMovement)> = Vec::new();
        for claim in paired {
            let Some(forced) = self.shove_proposal(claim) else {
                continue;
            };
            if accepted
                .iter()
                .any(|(_, prior)| sweeps_conflict(prior.segment, forced.segment))
            {
                continue;
            }
            accepted.push((claim, forced));
        }
        for (claim, forced) in &accepted {
            let victim = &mut self.actors[claim.victim.index()];
            victim.position = forced.segment.end();
            victim.residual = 0;
            victim.pending_turn = None;
            movement::prune_bomb_grace(&self.world, victim);
            movement::prune_soft_wall_grace(&self.world, victim);
            movement::prune_unlit_oil_grace(&self.world, victim);
            self.actors[claim.pusher.index()].last_successful_shove_tick = Some(tick);
        }
        accepted.into_iter().map(|(_, forced)| forced).collect()
    }

    fn shove_claim(
        &self,
        pusher: SeatId,
        outcomes: &[MovementOutcome],
        tick: Tick,
        priority: &[SeatId],
    ) -> Option<ShoveClaim> {
        let actor = &self.actors[pusher.index()];
        let ready = actor.last_successful_shove_tick.is_none_or(|last| {
            tick.0
                .checked_sub(last.0)
                .is_some_and(|elapsed| elapsed >= SHOVE_COOLDOWN_TICKS)
        });
        if !actor.alive || !actor.inventory().can_shove() || !ready {
            return None;
        }
        let mut best: Option<(SeatId, ContactEntry)> = None;
        for &victim in priority {
            if victim == pusher || !self.actors[victim.index()].alive {
                continue;
            }
            let Some(entry) = super::shove_geometry::contact_entry(
                &outcomes[pusher.index()],
                &outcomes[victim.index()],
            ) else {
                continue;
            };
            if best.is_none_or(|(_, current)| entry.compare_time(current) == Ordering::Less) {
                best = Some((victim, entry));
            }
        }
        let (victim, entry) = best?;
        Some(ShoveClaim {
            pusher,
            victim,
            direction: entry.direction,
        })
    }

    fn shove_proposal(&self, claim: ShoveClaim) -> Option<ForcedMovement> {
        let victim = &self.actors[claim.victim.index()];
        let mut distance = crate::game::movement_collision::allowed_distance(
            &self.world,
            victim.position,
            claim.direction,
            UNITS_PER_CELL,
            &victim.bomb_grace,
            crate::game::movement_collision::ObstacleAccess {
                can_pass: self.rules.enabled_items().contains_wall_pass()
                    && victim.inventory().can_pass_soft_walls(),
                soft_grace: &victim.soft_wall_grace,
                oil_grace: &victim.unlit_oil_grace,
            },
        );
        for blocker in self
            .actors
            .iter()
            .filter(|actor| actor.alive && actor.seat != claim.victim)
        {
            if blocker.seat == claim.pusher && footprints_overlap(victim.position, blocker.position)
            {
                continue;
            }
            distance = distance.min(distance_to_actor(
                victim.position,
                blocker.position,
                claim.direction,
                distance,
            ));
        }
        if distance <= 0 {
            return None;
        }
        let end = movement::translated(victim.position, claim.direction, distance);
        Some(ForcedMovement {
            pusher: claim.pusher,
            seat: claim.victim,
            segment: MovementSegment::new(victim.position, end),
        })
    }
}

fn disjoint_claims(claims: Vec<ShoveClaim>) -> Vec<ShoveClaim> {
    let mut participants = Vec::new();
    let mut accepted = Vec::new();
    for claim in claims {
        if participants.contains(&claim.pusher) || participants.contains(&claim.victim) {
            continue;
        }
        participants.extend([claim.pusher, claim.victim]);
        accepted.push(claim);
    }
    accepted
}

fn footprints_overlap(left: Position, right: Position) -> bool {
    (left.x - right.x).abs() < 2 * HALF_FOOTPRINT && (left.y - right.y).abs() < 2 * HALF_FOOTPRINT
}

fn sweeps_conflict(left: MovementSegment, right: MovementSegment) -> bool {
    let bounds = |segment: MovementSegment| {
        (
            segment.start().x.min(segment.end().x) - HALF_FOOTPRINT,
            segment.start().x.max(segment.end().x) + HALF_FOOTPRINT,
            segment.start().y.min(segment.end().y) - HALF_FOOTPRINT,
            segment.start().y.max(segment.end().y) + HALF_FOOTPRINT,
        )
    };
    let left = bounds(left);
    let right = bounds(right);
    left.0 < right.1 && right.0 < left.1 && left.2 < right.3 && right.2 < left.3
}

fn distance_to_actor(
    position: Position,
    blocker: Position,
    direction: Direction,
    requested: i32,
) -> i32 {
    let transverse = match direction {
        Direction::East | Direction::West => (position.y - blocker.y).abs() < 2 * HALF_FOOTPRINT,
        Direction::North | Direction::South => (position.x - blocker.x).abs() < 2 * HALF_FOOTPRINT,
    };
    if !transverse {
        return requested;
    }
    let gap = match direction {
        Direction::East if position.x <= blocker.x => blocker.x - position.x - 2 * HALF_FOOTPRINT,
        Direction::West if position.x >= blocker.x => position.x - blocker.x - 2 * HALF_FOOTPRINT,
        Direction::South if position.y <= blocker.y => blocker.y - position.y - 2 * HALF_FOOTPRINT,
        Direction::North if position.y >= blocker.y => position.y - blocker.y - 2 * HALF_FOOTPRINT,
        _ => requested,
    };
    requested.min(gap.max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_actor_claim_cycle_accepts_only_the_first_priority_pair() {
        let claims = vec![
            ShoveClaim {
                pusher: SeatId::One,
                victim: SeatId::Two,
                direction: Direction::East,
            },
            ShoveClaim {
                pusher: SeatId::Two,
                victim: SeatId::Three,
                direction: Direction::South,
            },
            ShoveClaim {
                pusher: SeatId::Three,
                victim: SeatId::One,
                direction: Direction::West,
            },
        ];

        let accepted = disjoint_claims(claims);

        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].pusher, SeatId::One);
        assert_eq!(accepted[0].victim, SeatId::Two);
    }
}
