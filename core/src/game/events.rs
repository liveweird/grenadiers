use super::simulation::RoundResult;
use super::types::{EntityId, SeatId, Tick};
use super::view::BombView;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepEvent {
    pub tick: Tick,
    pub kind: StepEventKind,
}

impl StepEvent {
    pub(crate) const fn new(tick: Tick, kind: StepEventKind) -> Self {
        Self { tick, kind }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepEventKind {
    BombPlaced(BombView),
    BombDetonated(BombView),
    ActorEliminated { id: EntityId, seat: SeatId },
    RoundEnded(RoundResult),
}
