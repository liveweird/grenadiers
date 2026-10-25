use std::{
    collections::HashSet,
    ffi::OsString,
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use sqlx::postgres::PgPoolOptions;
use tokio::sync::Semaphore;
use tracing_subscriber::EnvFilter;

use crate::{
    AppState, DEFAULT_ORIGINS, MAX_CONNECTIONS, MAX_DIAGNOSTIC_CONNECTIONS, READINESS_TIMEOUT,
    admission::{DenialTelemetry, DiagnosticAdmission, GameplayAdmission},
    game::{GameRooms, GameSessions, UpgradeLimit},
    peer_identity::TrustedProxyIps,
    round_seed::RoundSeedSource,
};

const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:8085";
const DEFAULT_LOG_FILTER: &str = "info";

pub struct Config {
    state: AppState,
    server_address: SocketAddr,
    web_dist_dir: Option<PathBuf>,
    log_filter: String,
    explicitly_empty_origins: bool,
}

impl Config {
    pub fn from_environment() -> Result<Self, StartupError> {
        Self::from_configuration(Environment::read())
    }

    pub fn log_filter(&self) -> &str {
        &self.log_filter
    }

    pub fn warnings(&self) -> impl Iterator<Item = StartupWarning> {
        self.explicitly_empty_origins
            .then_some(StartupWarning::EmptyAllowedOrigins)
            .into_iter()
    }

    pub fn into_parts(self) -> (AppState, SocketAddr, Option<PathBuf>) {
        (self.state, self.server_address, self.web_dist_dir)
    }

    pub(crate) fn from_configuration(environment: Environment) -> Result<Self, StartupError> {
        let explicitly_empty_origins =
            environment.allowed_origins.as_deref().is_some_and(|value| {
                value
                    .to_str()
                    .is_some_and(|value| parse_origins(value).is_empty())
            });
        let state = AppState::from_os_configuration(
            environment.allowed_origins,
            environment.database_url,
            environment.round_seed_file.map(PathBuf::from),
            environment.trusted_proxy_ips,
        )?;
        let server_address = environment
            .server_address
            .unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.into())
            .to_str()
            .ok_or(StartupError::Address)?
            .parse()
            .map_err(|_| StartupError::Address)?;
        let log_filter = environment
            .log_filter
            .and_then(|value| value.into_string().ok())
            .filter(|value| EnvFilter::try_new(value).is_ok())
            .unwrap_or_else(|| DEFAULT_LOG_FILTER.to_owned());

        Ok(Self {
            state,
            server_address,
            web_dist_dir: environment.web_dist_dir.map(PathBuf::from),
            log_filter,
            explicitly_empty_origins,
        })
    }
}

pub(crate) struct Environment {
    pub(crate) server_address: Option<OsString>,
    pub(crate) web_dist_dir: Option<OsString>,
    pub(crate) log_filter: Option<OsString>,
    pub(crate) allowed_origins: Option<OsString>,
    pub(crate) database_url: Option<OsString>,
    pub(crate) round_seed_file: Option<OsString>,
    pub(crate) trusted_proxy_ips: Option<OsString>,
}

impl Environment {
    fn read() -> Self {
        Self {
            server_address: std::env::var_os("SERVER_ADDR"),
            web_dist_dir: std::env::var_os("WEB_DIST_DIR"),
            log_filter: std::env::var_os("RUST_LOG"),
            allowed_origins: std::env::var_os("ALLOWED_ORIGINS"),
            database_url: std::env::var_os("DATABASE_URL"),
            round_seed_file: std::env::var_os("GAME_ROUND_SEED_FILE"),
            trusted_proxy_ips: std::env::var_os("TRUSTED_PROXY_IPS"),
        }
    }
}

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
        Self::from_os_configuration(
            allowed_origins.map(OsString::from),
            database_url.map(OsString::from),
            round_seed_file,
            trusted_proxy_ips,
        )
    }

    fn from_os_configuration(
        allowed_origins: Option<OsString>,
        database_url: Option<OsString>,
        round_seed_file: Option<PathBuf>,
        trusted_proxy_ips: Option<OsString>,
    ) -> Result<Self, StartupError> {
        let trusted_proxy_ips = match trusted_proxy_ips.as_deref() {
            Some(value) => Some(value.to_str().ok_or(StartupError::TrustedProxy)?),
            None => None,
        };
        let trusted_proxies =
            TrustedProxyIps::parse(trusted_proxy_ips).map_err(|_| StartupError::TrustedProxy)?;
        let allowed_origins = match allowed_origins.as_deref() {
            Some(value) => parse_origins(value.to_str().ok_or(StartupError::AllowedOrigins)?),
            None => DEFAULT_ORIGINS.iter().map(ToString::to_string).collect(),
        };
        let now = Instant::now();
        let round_seed_source = RoundSeedSource::from_optional_path(round_seed_file.as_deref())
            .map_err(|_| StartupError::RoundSeed)?;
        let database_url = match database_url {
            Some(value) => Some(value.into_string().map_err(|_| StartupError::Database)?),
            None => None,
        };
        let database = database_pool(database_url).map_err(|_| StartupError::Database)?;
        Ok(Self::build_with_dependencies(
            allowed_origins,
            database,
            round_seed_source,
            trusted_proxies,
            now,
        ))
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
        Ok(Self::build_with_dependencies(
            allowed_origins,
            database,
            round_seed_source,
            trusted_proxies,
            now,
        ))
    }

    fn build_with_dependencies(
        allowed_origins: HashSet<String>,
        database: Option<sqlx::PgPool>,
        round_seed_source: RoundSeedSource,
        trusted_proxies: TrustedProxyIps,
        now: Instant,
    ) -> Self {
        Self {
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
        }
    }
}

fn parse_origins(value: &str) -> HashSet<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupWarning {
    EmptyAllowedOrigins,
}

impl fmt::Display for StartupWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAllowedOrigins => formatter.write_str(
                "ALLOWED_ORIGINS was explicitly configured but contains no origins; browser WebSocket access is disabled",
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StartupError {
    Address,
    AllowedOrigins,
    Database,
    RoundSeed,
    TrustedProxy,
}

impl fmt::Debug for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Address => "StartupError::Address",
            Self::AllowedOrigins => "StartupError::AllowedOrigins",
            Self::Database => "StartupError::Database",
            Self::RoundSeed => "StartupError::RoundSeed",
            Self::TrustedProxy => "StartupError::TrustedProxy",
        })
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Address => "server address configuration failed",
            Self::AllowedOrigins => "allowed origins configuration failed",
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
