use super::EventAudience;
use crate::game::{PlayerCount, SeatId};

#[test]
fn legacy_audiences_keep_duel_meaning_while_multiplayer_masks_cover_all_seats() {
    assert_eq!(EventAudience::Both, EventAudience::all(PlayerCount::Two));
    assert!(EventAudience::Both.includes(SeatId::One));
    assert!(EventAudience::Both.includes(SeatId::Two));
    assert!(!EventAudience::Both.includes(SeatId::Three));

    let all = EventAudience::all(PlayerCount::Four);
    assert!(SeatId::ALL.into_iter().all(|seat| all.includes(seat)));
    assert!(EventAudience::Both.is_subset_of(all));
    assert!(!all.is_subset_of(EventAudience::Both));
    assert_eq!(
        all.without(SeatId::Three),
        EventAudience::all(PlayerCount::Four).without(SeatId::Three)
    );
    assert!(!all.without(SeatId::Three).includes(SeatId::Three));
}
