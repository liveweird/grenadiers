use std::{ffi::OsString, path::PathBuf};

use grenadiers_core::protocol::v11::{MapId, RulesVersion};

use crate::{
    AppState, Config, StartupError, StartupWarning,
    configuration::Environment,
    game::{BindingKey, ConnectionId},
};

fn environment() -> Environment {
    Environment {
        server_address: None,
        web_dist_dir: None,
        log_filter: None,
        allowed_origins: None,
        database_url: None,
        round_seed_file: None,
        trusted_proxy_ips: None,
        cpu_debug_enabled: None,
    }
}

#[test]
fn startup_configuration_uses_documented_standalone_defaults() {
    let configuration = Config::from_configuration(environment()).unwrap();

    assert_eq!(configuration.log_filter(), "info");
    assert_eq!(configuration.warnings().count(), 0);
    let (state, address, web_dist_dir) = configuration.into_parts();
    assert_eq!(address, "127.0.0.1:8085".parse().unwrap());
    assert!(web_dist_dir.is_none());
    assert_eq!(state.allowed_origins.len(), 4);
    assert!(state.allowed_origins.contains("http://localhost:5177"));
    assert!(state.allowed_origins.contains("http://127.0.0.1:8085"));
}

#[tokio::test]
async fn startup_configuration_applies_process_overrides() {
    let mut values = environment();
    values.server_address = Some("0.0.0.0:9090".into());
    values.web_dist_dir = Some("custom-dist".into());
    values.log_filter = Some("grenadiers_server=debug".into());
    values.allowed_origins = Some(" https://game.example,https://game.example ".into());
    values.database_url = Some("postgres://user:secret@db.example/game".into());
    values.trusted_proxy_ips = Some("192.0.2.10".into());

    let configuration = Config::from_configuration(values).unwrap();

    assert_eq!(configuration.log_filter(), "grenadiers_server=debug");
    let (state, address, web_dist_dir) = configuration.into_parts();
    assert_eq!(address, "0.0.0.0:9090".parse().unwrap());
    assert_eq!(web_dist_dir, Some(PathBuf::from("custom-dist")));
    assert_eq!(state.allowed_origins.len(), 1);
    assert!(state.allowed_origins.contains("https://game.example"));
    assert!(state.database.is_some());
}

#[test]
fn invalid_log_filter_retains_info_fallback() {
    let mut values = environment();
    values.log_filter = Some("grenadiers_server==debug".into());

    let configuration = Config::from_configuration(values).unwrap();

    assert_eq!(configuration.log_filter(), "info");
}

#[test]
fn cpu_debug_operator_flag_is_strict_and_defaults_off() {
    let defaults = Config::from_configuration(environment())
        .unwrap()
        .into_parts()
        .0;
    assert!(!defaults.cpu_debug_enabled);

    let mut enabled = environment();
    enabled.cpu_debug_enabled = Some("true".into());
    assert!(
        Config::from_configuration(enabled)
            .unwrap()
            .into_parts()
            .0
            .cpu_debug_enabled
    );

    for invalid in ["TRUE", "1", " true", ""] {
        let mut values = environment();
        values.cpu_debug_enabled = Some(invalid.into());
        assert_eq!(
            Config::from_configuration(values).err().unwrap(),
            StartupError::CpuDebugEnabled
        );
    }
}

#[test]
fn public_app_state_environment_constructor_honors_strict_cpu_debug_flag() {
    const CHILD: &str = "GRENADIERS_CPU_DEBUG_ENV_TEST_CHILD";
    match std::env::var(CHILD).as_deref() {
        Ok("enabled") => {
            assert!(AppState::from_environment().unwrap().cpu_debug_enabled);
            return;
        }
        Ok("invalid") => {
            assert!(matches!(
                AppState::from_environment(),
                Err(StartupError::CpuDebugEnabled)
            ));
            return;
        }
        _ => {}
    }

    let executable = std::env::current_exe().unwrap();
    for (child, flag) in [("enabled", "true"), ("invalid", "TRUE")] {
        let status = std::process::Command::new(&executable)
            .arg("--exact")
            .arg("startup_tests::public_app_state_environment_constructor_honors_strict_cpu_debug_flag")
            .env(CHILD, child)
            .env("CPU_DEBUG_ENABLED", flag)
            .env_remove("DATABASE_URL")
            .env_remove("GAME_ROUND_SEED_FILE")
            .env_remove("TRUSTED_PROXY_IPS")
            .status()
            .unwrap();
        assert!(status.success(), "environment child {child} failed");
    }
}

#[test]
fn explicitly_empty_origins_are_fail_closed_and_emit_typed_warning() {
    let mut values = environment();
    values.allowed_origins = Some(" , , ".into());

    let configuration = Config::from_configuration(values).unwrap();

    assert_eq!(
        configuration.warnings().collect::<Vec<_>>(),
        vec![StartupWarning::EmptyAllowedOrigins]
    );
    assert!(
        StartupWarning::EmptyAllowedOrigins
            .to_string()
            .contains("browser WebSocket access is disabled")
    );
    let (state, _, _) = configuration.into_parts();
    assert!(state.allowed_origins.is_empty());
}

#[test]
fn invalid_address_error_is_typed_and_redacted() {
    let secret_address = "secret.internal:not-a-port";
    let mut values = environment();
    values.server_address = Some(secret_address.into());

    let error = Config::from_configuration(values).err().unwrap();

    assert_eq!(error, StartupError::Address);
    let diagnostics = format!("{error:?} {error}");
    assert_eq!(
        diagnostics,
        "StartupError::Address server address configuration failed"
    );
    assert!(!diagnostics.contains(secret_address));
}

#[tokio::test]
async fn default_configuration_creates_expanded_v5_rooms() {
    let state = AppState::from_configuration(None, None, None, None).unwrap();

    let admission = state
        .game_rooms
        .create(
            BindingKey {
                connection_id: ConnectionId(1),
                generation: 1,
            },
            std::time::Instant::now(),
        )
        .await
        .unwrap();

    assert_eq!(admission.room.rules_version, RulesVersion::ExpandedV5);
    assert_eq!(admission.room.map_id, MapId::GeneratedV2);
}

#[test]
fn seed_configuration_failure_is_redacted_and_distinct_from_database_failure() {
    let secret_path = PathBuf::from(format!(
        "/tmp/grenadiers-secret-missing-seed-{}",
        std::process::id()
    ));
    let result = AppState::from_configuration(None, None, Some(secret_path.clone()), None);
    let Err(error) = result else {
        panic!("missing seed configuration should fail");
    };

    assert_eq!(error, StartupError::RoundSeed);
    let diagnostics = format!("{error:?} {error}");
    assert!(!diagnostics.contains(secret_path.to_string_lossy().as_ref()));

    let result =
        AppState::from_configuration(None, Some("not a postgres URL".to_owned()), None, None);
    assert!(matches!(result, Err(StartupError::Database)));
}

#[test]
fn state_validation_keeps_proxy_seed_database_failure_order() {
    let missing_seed = PathBuf::from(format!(
        "/tmp/grenadiers-secret-validation-order-{}",
        std::process::id()
    ));
    let result = AppState::from_configuration(
        None,
        Some("not a postgres URL".to_owned()),
        Some(missing_seed.clone()),
        Some("invalid proxy".into()),
    );
    assert!(matches!(result, Err(StartupError::TrustedProxy)));

    let result = AppState::from_configuration(
        None,
        Some("not a postgres URL".to_owned()),
        Some(missing_seed),
        None,
    );
    assert!(matches!(result, Err(StartupError::RoundSeed)));
}

#[test]
fn trusted_proxy_configuration_is_strict_and_redacted() {
    let result = AppState::from_configuration(None, None, None, Some("127.0.0.1, ::1".into()));
    let Err(error) = result else {
        panic!("spaced literals are invalid");
    };
    assert_eq!(error, StartupError::TrustedProxy);
    assert_eq!(
        format!("{error:?} {error}"),
        "StartupError::TrustedProxy trusted proxy configuration failed"
    );
}

#[cfg(unix)]
#[test]
fn non_unicode_trusted_proxy_configuration_fails_closed() {
    use std::os::unix::ffi::OsStringExt;

    let value = std::ffi::OsString::from_vec(vec![0xff, b'1']);
    assert!(matches!(
        AppState::from_configuration(None, None, None, Some(value)),
        Err(StartupError::TrustedProxy)
    ));
}

#[cfg(unix)]
#[test]
fn non_unicode_text_state_configuration_fails_closed_without_leaking_values() {
    use std::os::unix::ffi::OsStringExt;

    for (field, expected) in [
        ("origins", StartupError::AllowedOrigins),
        ("database", StartupError::Database),
    ] {
        let secret = OsString::from_vec(vec![0xff, b's', b'e', b'c', b'r', b'e', b't']);
        let mut values = environment();
        match field {
            "origins" => values.allowed_origins = Some(secret),
            "database" => values.database_url = Some(secret),
            _ => unreachable!(),
        }

        let error = Config::from_configuration(values).err().unwrap();
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
}

#[cfg(unix)]
#[test]
fn non_unicode_paths_remain_paths() {
    use std::os::unix::ffi::OsStringExt;

    let path = OsString::from_vec(vec![b'w', b'e', b'b', b'-', 0xff]);
    let mut values = environment();
    values.web_dist_dir = Some(path.clone());

    let configuration = Config::from_configuration(values).unwrap();
    let (_, _, web_dist_dir) = configuration.into_parts();

    assert_eq!(web_dist_dir, Some(PathBuf::from(path)));
}
