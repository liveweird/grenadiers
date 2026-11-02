use super::{Direction, State};
use crate::game::bombs::BombTrigger;
use crate::game::movement;
use crate::game::movement::MovementOutcome;
use crate::game::rules::PunchActivationPolicy;
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
        requests: &[Option<SeatId>],
        tick: Tick,
        remote_reservations: &BTreeSet<EntityId>,
    ) {
        let claims: Vec<_> = requests
            .iter()
            .filter_map(|request| {
                request.and_then(|seat| self.manual_punch_claim(seat, tick, remote_reservations))
            })
            .collect();
        self.apply_punch_claims(&claims, tick);
    }

    pub(super) fn latch_contact_punches(
        &mut self,
        outcomes: &[MovementOutcome],
    ) -> Vec<Option<(EntityId, Direction)>> {
        if self.rules.punch_activation_policy() != PunchActivationPolicy::ContactV2 {
            return vec![None; self.actors.len()];
        }
        self.actors
            .iter_mut()
            .zip(outcomes)
            .map(|(actor, outcome)| {
                let request = outcome.punch_contact.filter(|(id, _)| {
                    actor.alive
                        && !actor.contact_punch_latches.contains(id)
                        && actor.inventory.can_punch()
                });
                if let Some((id, _)) = request {
                    actor.contact_punch_latches.insert(id);
                }
                request
            })
            .collect()
    }

    pub(super) fn resolve_contact_punches(
        &mut self,
        requests: &[Option<(EntityId, Direction)>],
        tick: Tick,
        remote_targets: &[Option<EntityId>],
    ) {
        let remote_reservations = remote_targets.iter().flatten().copied().collect();
        let claims: Vec<_> = requests
            .iter()
            .enumerate()
            .filter_map(|(index, request)| {
                let seat = self.actors[index].seat;
                request.and_then(|(id, direction)| {
                    self.contact_punch_claim(seat, id, direction, tick, &remote_reservations)
                })
            })
            .collect();
        self.apply_punch_claims(&claims, tick);
    }

    pub(super) fn reconcile_contact_punch_latches(&mut self) {
        let world = &self.world;
        for actor in &mut self.actors {
            if !actor.alive {
                actor.contact_punch_latches.clear();
                continue;
            }
            let position = actor.position;
            actor.contact_punch_latches.retain(|id| {
                world
                    .bombs
                    .get(id)
                    .is_some_and(|bomb| actor_touches_cell(position, bomb.cell()))
            });
        }
    }

    fn apply_punch_claims(&mut self, claims: &[PunchClaim], tick: Tick) {
        let priority: Vec<_> = self.assignment.priority_order(self.player_count).collect();
        let mut retained = Vec::new();
        for seat in priority {
            let Some(claim) = claims.iter().find(|claim| claim.seat == seat) else {
                continue;
            };
            if retained
                .iter()
                .any(|winner: &PunchClaim| winner.bomb == claim.bomb)
            {
                continue;
            }
            retained.push(*claim);
        }
        let accepted: Vec<_> = retained
            .iter()
            .copied()
            .filter(|claim| {
                !retained
                    .iter()
                    .any(|other| other.bomb != claim.bomb && other.landing == claim.landing)
            })
            .collect();
        for claim in accepted {
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

    fn manual_punch_claim(
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
        self.punch_claim_for_bomb(seat, bomb.id(), actor.heading, tick, remote_reservations)
    }

    fn contact_punch_claim(
        &self,
        seat: SeatId,
        id: EntityId,
        direction: Direction,
        tick: Tick,
        remote_reservations: &BTreeSet<EntityId>,
    ) -> Option<PunchClaim> {
        let actor = &self.actors[seat.index()];
        let bomb = self.world.bombs.get(&id)?;
        (crate::game::geometry::distance_to_blocking_cell(actor.position, direction, bomb.cell())
            == Some(0))
        .then_some(())?;
        self.punch_claim_for_bomb(seat, id, direction, tick, remote_reservations)
    }

    fn punch_claim_for_bomb(
        &self,
        seat: SeatId,
        id: EntityId,
        direction: Direction,
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
        let bomb = self.world.bombs.get(&id)?;
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
        let landing = self.punch_landing(bomb.id(), source, direction)?;
        Some(PunchClaim {
            seat,
            bomb: bomb.id(),
            landing,
        })
    }

    fn punch_landing(&self, id: EntityId, source: CellId, direction: Direction) -> Option<CellId> {
        for candidate in crate::game::punch_landing_candidates_for_board(
            source,
            direction,
            self.world.arena.board(),
        ) {
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

fn actor_touches_cell(position: crate::game::Position, cell: CellId) -> bool {
    movement::position_overlaps_cell(position, cell)
        || [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
        .into_iter()
        .any(|direction| {
            crate::game::geometry::distance_to_blocking_cell(position, direction, cell) == Some(0)
        })
}
