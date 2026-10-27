use grenadiers_core::game::{EnabledItems, SeatId};

use super::replay;
use crate::game::rooms::match_runtime::{collect_generated_pickup, start_optional_runtime};

#[test]
fn generated_teleport_marker_survives_both_admissions_and_replays_exactly() {
    let mut runtime = start_optional_runtime(EnabledItems::TELEPORTER, 31);
    collect_generated_pickup(&mut runtime);
    let mut marker = None;

    for seat in SeatId::DUEL {
        let admission = runtime.prepare_resume(seat).unwrap();
        let frame = admission
            .notice
            .frame(seat)
            .expect("running admission snapshot");
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        for actor in value["state"]["actors"].as_array().unwrap() {
            let actual = actor["lastTeleportTick"]
                .as_u64()
                .expect("seed 31 route teleports");
            assert_eq!(*marker.get_or_insert(actual), actual);
        }
        runtime.commit_admission(admission);
    }

    let replayed = replay(runtime.replay_journal()).unwrap();
    assert_eq!(replayed, runtime.replay_inspection());
    assert!(marker.is_some());
}
