use grenadiers_core::game::{EffectKind, EnabledItems, SeatId};

use super::replay;
use crate::game::rooms::match_runtime::{collect_generated_pickup, plant, start_optional_runtime};

#[test]
fn blindness_deadline_bombs_and_quiet_resume_replay_deterministically() {
    let mut runtime = start_optional_runtime(EnabledItems::INVISIBLE_BOMBS, 31);
    collect_generated_pickup(&mut runtime);
    let collected_tick = runtime.sim().unwrap().state.tick();
    let expires_at = collected_tick.0 + EffectKind::InvisibleBombs.duration_ticks();
    plant(&mut runtime, 2);
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    assert!(
        runtime.sim().unwrap().state.actors()[0]
            .effects()
            .active(EffectKind::InvisibleBombs, collected_tick)
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    let before_resume = runtime.sim().unwrap().state.tick();
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    assert_eq!(runtime.sim().unwrap().state.tick(), before_resume);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    while runtime.sim().unwrap().state.tick().0 < expires_at {
        runtime.tick().unwrap();
    }
    let state = &runtime.sim().unwrap().state;
    assert_eq!(state.tick().0, expires_at);
    assert!(
        !state.actors()[0]
            .effects()
            .active(EffectKind::InvisibleBombs, state.tick())
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}
