use std::{hint::black_box, sync::Arc, time::Instant};

use grenadiers_core::protocol::{RoomCode, RoomId};
use serde_json::json;
use tokio::sync::watch;

use super::{metadata, require_release, summary_ns};
use crate::{
    game::rooms::{
        RandomFill, match_owner::PreparedMatchOwner, profile::RoomProfile, types::RoomIdentity,
    },
    round_seed::RoundSeedSource,
};

const SAMPLES: usize = 512;

#[test]
#[ignore = "isolated release construction measurement; coordinate before running"]
fn expanded_v5_prepared_owner_construction_measurement() {
    require_release();
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| getrandom::fill(bytes).map_err(|_| ()));
    let mut timings = Vec::with_capacity(SAMPLES);
    let mut drop_timings = Vec::with_capacity(SAMPLES);

    for serial in 1..=SAMPLES {
        let identity = identity(serial);
        let seeds = RoundSeedSource::from_optional_path(None)
            .expect("OS entropy source construction cannot fail");
        let (_, shutdown) = watch::channel(false);

        let started = Instant::now();
        let prepared = PreparedMatchOwner::new(
            identity,
            tokio::time::Instant::now(),
            seeds,
            Arc::clone(&random_fill),
            shutdown,
        )
        .expect("ExpandedV5 owner construction succeeds");
        black_box(&prepared);
        let elapsed = started.elapsed();
        timings.push(elapsed);

        let drop_started = Instant::now();
        drop(prepared);
        drop_timings.push(drop_started.elapsed());
    }

    super::emit(
        "expanded-v5-prepared-owner-construction",
        json!({
            "metadata": metadata(),
            "workload": "PreparedMatchOwner::new with actual ExpandedV5 runtime and initial publication",
            "sampleCount": SAMPLES,
            "identitySeedAndDropOutsideTiming": true,
            "roundEntropyRequestedDuringConstruction": false,
            "timings": summary_ns(&timings),
            "dropTimings": summary_ns(&drop_timings),
            "spawnedOwnerCount": 0,
            "preparedOwnersDropped": SAMPLES,
        }),
    );
}

fn identity(serial: usize) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new(&format!("{serial:032x}")).expect("measurement serial is a valid room id"),
        code: RoomCode::new("ABCDEFGH").expect("static room code"),
        profile: RoomProfile::ExpandedV5,
        mode: grenadiers_core::room::RoomMode::HumanDuel,
        registry_serial: serial as u64,
        private_cpu_seed: None,
    }
}
