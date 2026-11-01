use serde::{Deserialize, Serialize};

use crate::game::StormCloudConfig;
use crate::room::{
    CpuDifficulty as DomainDifficulty, LairRoomConfiguration, LairSettingsError,
    RespawnLairSettings, RespawnLairTuning, RoomConfiguration as BaseConfiguration, SeatControl,
    SpikePitSettings, SpikePitSettingsError, SpikePitTuning,
    SpikeRoomConfiguration as DomainConfiguration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Difficulty {
    Easy,
    Standard,
    Hard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Lairs {
    pub enabled: bool,
    pub spawn_seconds: u8,
    pub max_living_mobs: u8,
}

impl Lairs {
    fn domain(self) -> Result<RespawnLairTuning, LairSettingsError> {
        if self.max_living_mobs != 1 {
            return Err(LairSettingsError::InvalidMobsPerLair(self.max_living_mobs));
        }
        Ok(RespawnLairTuning::new(
            self.enabled,
            RespawnLairSettings::new(self.spawn_seconds)?,
        ))
    }

    const fn from_domain(value: RespawnLairTuning) -> Self {
        Self {
            enabled: value.enabled(),
            spawn_seconds: value.settings().spawn_seconds(),
            max_living_mobs: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Environment {
    pub storm_cloud: bool,
    pub lairs: Lairs,
    pub spike_pits: SpikePits,
}

impl Environment {
    pub(crate) fn domain(
        self,
    ) -> Result<(StormCloudConfig, RespawnLairTuning, SpikePitTuning), EnvironmentError> {
        Ok((
            if self.storm_cloud {
                StormCloudConfig::Single
            } else {
                StormCloudConfig::Disabled
            },
            self.lairs.domain()?,
            self.spike_pits.domain()?,
        ))
    }

    const fn from_domain(
        cloud: StormCloudConfig,
        lairs: RespawnLairTuning,
        spike_pits: SpikePitTuning,
    ) -> Self {
        Self {
            storm_cloud: matches!(cloud, StormCloudConfig::Single),
            lairs: Lairs::from_domain(lairs),
            spike_pits: SpikePits::from_domain(spike_pits),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpikePits {
    pub enabled: bool,
    pub warning_ticks: u16,
    pub raised_ticks: u16,
}

impl SpikePits {
    fn domain(self) -> Result<SpikePitTuning, SpikePitSettingsError> {
        Ok(SpikePitTuning::new(
            self.enabled,
            SpikePitSettings::new(self.warning_ticks.into(), self.raised_ticks.into())?,
        ))
    }

    const fn from_domain(value: SpikePitTuning) -> Self {
        Self {
            enabled: value.enabled(),
            warning_ticks: value.settings().warning_ticks() as u16,
            raised_ticks: value.settings().raised_ticks() as u16,
        }
    }
}

#[derive(Debug)]
pub(crate) enum EnvironmentError {
    Lairs,
    SpikePits,
}

impl From<LairSettingsError> for EnvironmentError {
    fn from(_: LairSettingsError) -> Self {
        Self::Lairs
    }
}
impl From<SpikePitSettingsError> for EnvironmentError {
    fn from(_: SpikePitSettingsError) -> Self {
        Self::SpikePits
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    controllers: Vec<super::ConfigurationController>,
    #[serde(deserialize_with = "required_optional")]
    cpu_difficulty: Option<super::Difficulty>,
    board: super::BoardSize,
    environment: Environment,
}

fn required_optional<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<super::Difficulty>, D::Error> {
    Option::deserialize(deserializer)
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
    pub fn from_domain(domain: DomainConfiguration) -> Result<Self, LairSettingsError> {
        RespawnLairTuning::try_from(domain.base().lairs())?;
        Ok(Self { domain })
    }
}

impl<'de> Deserialize<'de> for RoomConfiguration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = Fields::deserialize(deserializer)?;
        let controls = fields
            .controllers
            .into_iter()
            .map(|value| match value {
                super::ConfigurationController::Human => SeatControl::Human,
                super::ConfigurationController::Cpu => SeatControl::Cpu,
            })
            .collect::<Vec<_>>();
        let difficulty = fields.cpu_difficulty.map(domain_difficulty);
        let base = BaseConfiguration::new(
            &controls,
            difficulty,
            fields
                .board
                .domain()
                .map_err(|()| serde::de::Error::custom("board dimensions are not supported"))?,
        )
        .map_err(serde::de::Error::custom)?;
        let (cloud, lairs, spike_pits) = fields
            .environment
            .domain()
            .map_err(|_| serde::de::Error::custom("invalid lair settings"))?;
        Ok(Self {
            domain: DomainConfiguration::new(
                LairRoomConfiguration::new(base, cloud, lairs.as_legacy()),
                spike_pits,
            ),
        })
    }
}

impl Serialize for RoomConfiguration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Borrowed {
            controllers: Vec<super::ConfigurationController>,
            cpu_difficulty: Option<Difficulty>,
            board: super::BoardSize,
            environment: Environment,
        }
        let base = self.domain.base().base();
        Borrowed {
            controllers: base
                .controllers()
                .iter()
                .map(|value| match value {
                    SeatControl::Human => super::ConfigurationController::Human,
                    SeatControl::Cpu => super::ConfigurationController::Cpu,
                })
                .collect(),
            cpu_difficulty: base.cpu_difficulty().map(wire_difficulty),
            board: super::BoardSize::from_domain(base.board()),
            environment: Environment::from_domain(
                self.domain.base().storm_cloud(),
                RespawnLairTuning::try_from(self.domain.base().lairs())
                    .expect("validated respawn configuration"),
                self.domain.spike_pits(),
            ),
        }
        .serialize(serializer)
    }
}

const fn domain_difficulty(value: Difficulty) -> DomainDifficulty {
    match value {
        Difficulty::Easy => DomainDifficulty::Easy,
        Difficulty::Standard => DomainDifficulty::Standard,
        Difficulty::Hard => DomainDifficulty::Hard,
    }
}

const fn wire_difficulty(value: DomainDifficulty) -> Difficulty {
    match value {
        DomainDifficulty::Easy => Difficulty::Easy,
        DomainDifficulty::Standard => Difficulty::Standard,
        DomainDifficulty::Hard => Difficulty::Hard,
    }
}
