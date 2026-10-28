use serde::{Deserialize, Deserializer, Serialize};

use crate::room::{
    BoardSize as DomainBoardSize, CpuDifficulty as DomainDifficulty,
    RoomConfiguration as DomainConfiguration, SeatControl as DomainControl,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigurationController {
    Human,
    Cpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Difficulty {
    Easy,
    Standard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoardSize {
    width: u16,
    height: u16,
}

impl BoardSize {
    fn domain(self) -> Result<DomainBoardSize, ()> {
        DomainBoardSize::try_from((self.width, self.height))
    }

    fn from_domain(value: DomainBoardSize) -> Self {
        let (width, height) = value.dimensions();
        Self { width, height }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    controllers: Vec<ConfigurationController>,
    #[serde(deserialize_with = "required_optional")]
    cpu_difficulty: Option<Difficulty>,
    board: BoardSize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomConfiguration {
    domain: DomainConfiguration,
}

impl RoomConfiguration {
    #[must_use]
    pub const fn domain(&self) -> &DomainConfiguration {
        &self.domain
    }

    #[must_use]
    pub fn from_domain(domain: DomainConfiguration) -> Self {
        Self { domain }
    }
}

impl<'de> Deserialize<'de> for RoomConfiguration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = Fields::deserialize(deserializer)?;
        let controls: Vec<_> = fields
            .controllers
            .into_iter()
            .map(|control| match control {
                ConfigurationController::Human => DomainControl::Human,
                ConfigurationController::Cpu => DomainControl::Cpu,
            })
            .collect();
        let difficulty = fields.cpu_difficulty.map(|difficulty| match difficulty {
            Difficulty::Easy => DomainDifficulty::Easy,
            Difficulty::Standard => DomainDifficulty::Standard,
        });
        let board = fields
            .board
            .domain()
            .map_err(|()| serde::de::Error::custom("board dimensions are not supported"))?;
        let domain = DomainConfiguration::new(&controls, difficulty, board)
            .map_err(serde::de::Error::custom)?;
        Ok(Self { domain })
    }
}

impl Serialize for RoomConfiguration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct BorrowedFields<'a> {
            controllers: Vec<ConfigurationController>,
            cpu_difficulty: Option<Difficulty>,
            board: BoardSize,
            #[serde(skip)]
            _marker: std::marker::PhantomData<&'a ()>,
        }

        let controllers = self
            .domain
            .controllers()
            .iter()
            .map(|control| match control {
                DomainControl::Human => ConfigurationController::Human,
                DomainControl::Cpu => ConfigurationController::Cpu,
            })
            .collect();
        let cpu_difficulty = self
            .domain
            .cpu_difficulty()
            .map(|difficulty| match difficulty {
                DomainDifficulty::Easy => Difficulty::Easy,
                DomainDifficulty::Standard => Difficulty::Standard,
            });
        BorrowedFields {
            controllers,
            cpu_difficulty,
            board: BoardSize::from_domain(self.domain.board()),
            _marker: std::marker::PhantomData,
        }
        .serialize(serializer)
    }
}

fn required_optional<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Difficulty>, D::Error> {
    Option::deserialize(deserializer)
}
