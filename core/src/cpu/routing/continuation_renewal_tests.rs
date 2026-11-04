use crate::game::Tick;

use super::continuation_renewal::stationary_renew_at;

#[test]
fn stationary_hold_clips_last_one_two_and_three_updates_at_exclusive_horizon() {
    for (publication, horizon) in [(40, 41), (40, 42), (40, 43)] {
        assert_eq!(
            stationary_renew_at(Tick(publication), Tick(horizon)),
            Ok(Tick(horizon))
        );
    }
    assert!(stationary_renew_at(Tick(43), Tick(43)).is_err());
    assert!(stationary_renew_at(Tick(44), Tick(43)).is_err());
}
