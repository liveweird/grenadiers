use super::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Terrain};
use super::bombs::{BlastKind, Bomb, BombTrigger};
use super::input::Direction;
use super::oil::{OilBarrelState, OilState};
use super::pickups::PickupKind;
use super::simulation::{RoundResult, State};
use super::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicState {
    pub rules_version: u16,
    pub tick: Tick,
    pub running_updates: u64,
    pub remaining_ticks: u64,
    pub arena: ArenaView,
    pub actors: Vec<ActorView>,
    pub bombs: Vec<BombView>,
    pub flames: Vec<FlameView>,
    pub oil_barrels: Vec<OilBarrelView>,
    pub pickups: Vec<PickupView>,
    pub result: Option<RoundResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArenaView {
    pub width: u16,
    pub height: u16,
    pub terrain: Vec<Terrain>,
    pub blocks: Vec<CellId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorView {
    pub id: EntityId,
    pub seat: SeatId,
    pub position: Position,
    pub heading: Direction,
    pub speed: SpeedLevel,
    pub alive: bool,
    pub bomb_capacity: u16,
    pub bomb_range: u16,
    pub live_bomb_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BombView {
    pub id: EntityId,
    pub cell: CellId,
    pub owner: EntityId,
    pub range: u16,
    pub placed_at: Tick,
    pub trigger: BombTrigger,
    pub blast: BlastKind,
}

impl From<Bomb> for BombView {
    fn from(bomb: Bomb) -> Self {
        Self {
            id: bomb.id(),
            cell: bomb.cell(),
            owner: bomb.owner(),
            range: bomb.range(),
            placed_at: bomb.placed_at(),
            trigger: bomb.trigger(),
            blast: bomb.blast(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlameView {
    pub cell: CellId,
    pub expires_at: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickupView {
    pub cell: CellId,
    pub kind: PickupKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OilBarrelView {
    pub cell: CellId,
    pub state: OilBarrelState,
}

impl State {
    #[must_use]
    pub fn public_view(&self) -> PublicState {
        let arena = self.world().arena();
        let terrain = (0..ARENA_HEIGHT)
            .flat_map(|row| {
                (0..ARENA_WIDTH).map(move |column| {
                    arena
                        .terrain(CellId::new(column, row))
                        .expect("arena dimensions contain every projected cell")
                })
            })
            .collect();
        debug_assert_eq!(Vec::<Terrain>::len(&terrain), ARENA_CELL_COUNT);

        PublicState {
            rules_version: self.rules().version(),
            tick: self.tick(),
            running_updates: self.running_updates(),
            remaining_ticks: self
                .rules()
                .round_ticks()
                .saturating_sub(self.running_updates()),
            arena: ArenaView {
                width: ARENA_WIDTH,
                height: ARENA_HEIGHT,
                terrain,
                blocks: arena.blocks().iter().copied().collect(),
            },
            actors: self
                .player_count()
                .seats()
                .iter()
                .map(|&seat| {
                    let actor = &self.actors()[seat.index()];
                    ActorView {
                        id: actor.id(),
                        seat: actor.seat(),
                        position: actor.position(),
                        heading: actor.heading(),
                        speed: actor.speed(),
                        alive: actor.alive(),
                        bomb_capacity: actor.bomb_capacity(),
                        bomb_range: actor.bomb_range(),
                        live_bomb_count: self.live_bomb_count(seat),
                    }
                })
                .collect(),
            bombs: self
                .world()
                .bombs()
                .values()
                .copied()
                .map(BombView::from)
                .collect(),
            flames: self
                .world()
                .hazards()
                .iter()
                .map(|(&cell, &expires_at)| FlameView { cell, expires_at })
                .collect(),
            oil_barrels: self
                .world()
                .oil_barrels()
                .filter_map(|(cell, state)| {
                    let state = match state {
                        OilState::Unlit => OilBarrelState::Unlit,
                        OilState::Burning => OilBarrelState::Burning,
                        OilState::Concealed | OilState::Pending => return None,
                    };
                    Some(OilBarrelView { cell, state })
                })
                .collect(),
            pickups: self
                .world()
                .exposed_pickups()
                .iter()
                .map(|(&cell, &pickup)| PickupView {
                    cell,
                    kind: pickup.kind(),
                })
                .collect(),
            result: self.result(),
        }
    }
}
