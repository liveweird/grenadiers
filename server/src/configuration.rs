use std::{
    collections::HashSet,
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use sqlx::postgres::PgPoolOptions;
use tokio::sync::Semaphore;

use crate::{
    AppState, DEFAULT_ORIGINS, MAX_CONNECTIONS, MAX_DIAGNOSTIC_CONNECTIONS, READINESS_TIMEOUT,
    admission::{DenialTelemetry, DiagnosticAdmission, GameplayAdmission},
    game::{GameRooms, GameSessions, UpgradeLimit},
    peer_identity::TrustedProxyIps,
    round_seed::RoundSeedSource,
};

impl AppState {
    #[must_use]
    pub fn without_database(allowed_origins: impl IntoIterator<Item = String>) -> Self {
        Self::build(
            allowed_origins.into_iter().collect(),
            None,
            None,
            TrustedProxyIps::default(),
        )
        .expect("default state construction cannot fail")
    }

    pub fn from_environment() -> Result<Self, StartupError> {
        Self::from_configuration(
            std::env::var("ALLOWED_ORIGINS").ok(),
            std::env::var("DATABASE_URL").ok(),
            std::env::var_os("GAME_ROUND_SEED_FILE").map(PathBuf::from),
            std::env::var_os("TRUSTED_PROXY_IPS"),
        )
    }

    pub(crate) fn from_configuration(
        allowed_origins: Option<String>,
        database_url: Option<String>,
        round_seed_file: Option<PathBuf>,
        trusted_proxy_ips: Option<OsString>,
    ) -> Result<Self, StartupError> {
        let allowed_origins = allowed_origins.map_or_else(
            || DEFAULT_ORIGINS.iter().map(ToString::to_string).collect(),
            |value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|origin| !origin.is_empty())
                    .map(ToString::to_string)
                    .collect::<HashSet<_>>()
            },
        );
        let trusted_proxy_ips = match trusted_proxy_ips.as_deref() {
            Some(value) => Some(value.to_str().ok_or(StartupError::TrustedProxy)?),
            None => None,
        };
        let trusted_proxies =
            TrustedProxyIps::parse(trusted_proxy_ips).map_err(|_| StartupError::TrustedProxy)?;
        Self::build(
            allowed_origins,
            database_url,
            round_seed_file.as_deref(),
            trusted_proxies,
        )
    }

    fn build(
        allowed_origins: HashSet<String>,
        database_url: Option<String>,
        round_seed_file: Option<&Path>,
        trusted_proxies: TrustedProxyIps,
    ) -> Result<Self, StartupError> {
        let now = Instant::now();
        let round_seed_source = RoundSeedSource::from_optional_path(round_seed_file)
            .map_err(|_| StartupError::RoundSeed)?;
        let database = database_pool(database_url).map_err(|_| StartupError::Database)?;
        Ok(Self {
            database,
            allowed_origins: Arc::new(allowed_origins),
            connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            diagnostic_connections: Arc::new(Semaphore::new(MAX_DIAGNOSTIC_CONNECTIONS)),
            diagnostic_upgrade_limit: Arc::new(UpgradeLimit::new(now)),
            game_sessions: Arc::new(GameSessions::new(now)),
            game_rooms: GameRooms::new(now, round_seed_source),
            trusted_proxies: Arc::new(trusted_proxies),
            gameplay_admission: GameplayAdmission::new(now),
            diagnostic_admission: DiagnosticAdmission::new(now),
            denial_telemetry: Arc::new(DenialTelemetry::new(now)),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StartupError {
    Database,
    RoundSeed,
    TrustedProxy,
}

impl fmt::Debug for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Database => "StartupError::Database",
            Self::RoundSeed => "StartupError::RoundSeed",
            Self::TrustedProxy => "StartupError::TrustedProxy",
        })
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Database => "database configuration failed",
            Self::RoundSeed => "round seed configuration failed",
            Self::TrustedProxy => "trusted proxy configuration failed",
        })
    }
}

impl std::error::Error for StartupError {}

pub(crate) fn database_pool(
    database_url: Option<String>,
) -> Result<Option<sqlx::PgPool>, sqlx::Error> {
    let Some(database_url) = database_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    else {
        return Ok(None);
    };
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(READINESS_TIMEOUT)
        .connect_lazy(database_url)
        .map(Some)
}
