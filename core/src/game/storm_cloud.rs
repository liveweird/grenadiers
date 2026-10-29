use std::fmt;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

use super::{Direction, MovementSegment, Position, SimulationSeed};

pub const STORM_CLOUD_RANDOMNESS_VERSION: u16 = 1;
pub const STORM_CLOUD_HALF_EXTENT: i32 = 512;
pub const STORM_CLOUD_SPEED_PER_TICK: i32 = 8;
pub const STORM_CLOUD_LEG_TICKS: u16 = 128;
pub const STORM_SLOW_TICKS: u64 = 180;
const STORM_STREAM: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StormCloudView {
    pub position: Position,
    pub direction: Direction,
    pub leg_remaining_ticks: u16,
}

#[derive(Clone)]
pub(crate) struct StormCloudState {
    view: StormCloudView,
    rng: ChaCha20Rng,
    seed: SimulationSeed,
    words_consumed: u64,
    min: Position,
    max: Position,
}

impl StormCloudState {
    pub(crate) fn new(seed: SimulationSeed, width: u16, height: u16) -> Self {
        let mut rng = ChaCha20Rng::from_seed(seed.bytes());
        rng.set_stream(STORM_STREAM);
        let direction = direction_from_word(rng.next_u32());
        Self {
            view: StormCloudView {
                position: Position::center(super::CellId::new(width / 2, height / 2)),
                direction,
                leg_remaining_ticks: STORM_CLOUD_LEG_TICKS,
            },
            rng,
            seed,
            words_consumed: 1,
            min: Position::center(super::CellId::new(1, 1)),
            max: Position::center(super::CellId::new(width - 2, height - 2)),
        }
    }

    pub(crate) const fn view(&self) -> StormCloudView {
        self.view
    }

    pub(crate) fn advance(&mut self) -> MovementSegment {
        if self.view.leg_remaining_ticks == 0 {
            let chosen = direction_from_word(self.rng.next_u32());
            self.words_consumed = self
                .words_consumed
                .checked_add(1)
                .expect("a storm cloud cannot consume u64::MAX random words");
            self.view.direction = self.reflect_outward(chosen);
            self.view.leg_remaining_ticks = STORM_CLOUD_LEG_TICKS;
        }
        let start = self.view.position;
        match self.view.direction {
            Direction::North => self.view.position.y -= STORM_CLOUD_SPEED_PER_TICK,
            Direction::East => self.view.position.x += STORM_CLOUD_SPEED_PER_TICK,
            Direction::South => self.view.position.y += STORM_CLOUD_SPEED_PER_TICK,
            Direction::West => self.view.position.x -= STORM_CLOUD_SPEED_PER_TICK,
        }
        self.view.leg_remaining_ticks -= 1;
        MovementSegment::new(start, self.view.position)
    }

    fn reflect_outward(&self, chosen: Direction) -> Direction {
        match chosen {
            Direction::North if self.view.position.y <= self.min.y => Direction::South,
            Direction::East if self.view.position.x >= self.max.x => Direction::West,
            Direction::South if self.view.position.y >= self.max.y => Direction::North,
            Direction::West if self.view.position.x <= self.min.x => Direction::East,
            direction => direction,
        }
    }
}

const fn direction_from_word(word: u32) -> Direction {
    match word & 3 {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        _ => Direction::West,
    }
}

impl PartialEq for StormCloudState {
    fn eq(&self, other: &Self) -> bool {
        self.view == other.view
            && self.seed == other.seed
            && self.words_consumed == other.words_consumed
            && self.min == other.min
            && self.max == other.max
    }
}
impl Eq for StormCloudState {}

impl fmt::Debug for StormCloudState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StormCloudState")
            .field("view", &self.view)
            .field("seed", &"<redacted>")
            .field("state", &"<redacted>")
            .field("words_consumed", &self.words_consumed)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_outward_edge_choice_reflects_inward() {
        let mut cloud = StormCloudState::new(SimulationSeed::new([0; 32]), 11, 9);
        cloud.view.position = Position {
            x: cloud.min.x,
            y: cloud.min.y,
        };
        assert_eq!(cloud.reflect_outward(Direction::North), Direction::South);
        assert_eq!(cloud.reflect_outward(Direction::West), Direction::East);
        cloud.view.position = Position {
            x: cloud.max.x,
            y: cloud.max.y,
        };
        assert_eq!(cloud.reflect_outward(Direction::South), Direction::North);
        assert_eq!(cloud.reflect_outward(Direction::East), Direction::West);
    }

    #[test]
    fn equality_tracks_seed_and_word_position_while_debug_redacts_both() {
        let cloud = StormCloudState::new(SimulationSeed::new([3; 32]), 15, 13);
        assert_eq!(cloud, cloud.clone());
        assert_ne!(
            cloud,
            StormCloudState::new(SimulationSeed::new([4; 32]), 15, 13)
        );
        let debug = format!("{cloud:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("[3, 3"));
    }
}
