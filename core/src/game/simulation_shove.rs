use super::contract::ForcedMovement;
use super::{Direction, SeatId, State, Tick};
use crate::game::geometry::MovementSegment;
use crate::game::movement::{self, MovementOutcome};
use crate::game::types::{HALF_FOOTPRINT, Position, UNITS_PER_CELL};

const SHOVE_COOLDOWN_TICKS: u64 = 30;

#[derive(Clone, Copy)]
struct ShoveClaim {
    seat: SeatId,
    direction: Direction,
}

impl State {
    pub(super) fn resolve_shove(
        &mut self,
        outcomes: &[MovementOutcome; 2],
        tick: Tick,
    ) -> Option<ForcedMovement> {
        if !self.rules.enabled_items().contains_bracelet() {
            return None;
        }
        let claims = SeatId::ALL.map(|seat| self.shove_claim(seat, outcomes, tick));
        let claim = match claims {
            [Some(one), Some(two)] => {
                if self.assignment.priority_seat() == SeatId::One {
                    one
                } else {
                    two
                }
            }
            [Some(claim), None] | [None, Some(claim)] => claim,
            [None, None] => return None,
        };
        self.apply_shove(claim, tick)
    }

    fn shove_claim(
        &self,
        seat: SeatId,
        outcomes: &[MovementOutcome; 2],
        tick: Tick,
    ) -> Option<ShoveClaim> {
        let actor = &self.actors[seat.index()];
        let ready = actor.last_successful_shove_tick.is_none_or(|last| {
            tick.0
                .checked_sub(last.0)
                .is_some_and(|elapsed| elapsed >= SHOVE_COOLDOWN_TICKS)
        });
        if !actor.alive || !actor.inventory().can_shove() || !ready {
            return None;
        }
        let direction = super::shove_geometry::contact_entry_direction(seat, outcomes)?;
        Some(ShoveClaim { seat, direction })
    }

    fn apply_shove(&mut self, claim: ShoveClaim, tick: Tick) -> Option<ForcedMovement> {
        let victim_index = 1 - claim.seat.index();
        let pusher_index = claim.seat.index();
        if !self.actors[victim_index].alive {
            return None;
        }
        let victim = &self.actors[victim_index];
        let mut distance = crate::game::movement_collision::allowed_distance(
            &self.world,
            victim.position,
            claim.direction,
            UNITS_PER_CELL,
            &victim.bomb_grace,
            self.rules.enabled_items().contains_wall_pass()
                && victim.inventory().can_pass_soft_walls(),
            &victim.soft_wall_grace,
        );
        let pusher = self.actors[pusher_index].position;
        if !footprints_overlap(victim.position, pusher) {
            distance = distance.min(distance_to_actor(
                victim.position,
                pusher,
                claim.direction,
                distance,
            ));
        }
        if distance <= 0 {
            return None;
        }
        let start = self.actors[victim_index].position;
        let end = movement::translated(start, claim.direction, distance);
        let victim = &mut self.actors[victim_index];
        victim.position = end;
        victim.residual = 0;
        victim.pending_turn = None;
        movement::prune_bomb_grace(&self.world, victim);
        movement::prune_soft_wall_grace(&self.world, victim);
        self.actors[pusher_index].last_successful_shove_tick = Some(tick);
        Some(ForcedMovement {
            seat: self.actors[victim_index].seat,
            segment: MovementSegment::new(start, end),
        })
    }
}

fn footprints_overlap(left: Position, right: Position) -> bool {
    (left.x - right.x).abs() < 2 * HALF_FOOTPRINT && (left.y - right.y).abs() < 2 * HALF_FOOTPRINT
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
