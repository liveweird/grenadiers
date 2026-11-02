use grenadiers_core::{
    game::SeatId,
    protocol::{v14, v15, v16, v17, v18, v19, v20},
};

macro_rules! seat_conversion {
    ($domain:ident, $wire:ident, $version:ident) => {
        pub(super) const fn $domain(seat: $version::Seat) -> Option<SeatId> {
            Some(match seat {
                $version::Seat::One => SeatId::One,
                $version::Seat::Two => SeatId::Two,
                $version::Seat::Three => SeatId::Three,
                $version::Seat::Four => SeatId::Four,
            })
        }

        pub(super) const fn $wire(seat: SeatId) -> $version::Seat {
            match seat {
                SeatId::One => $version::Seat::One,
                SeatId::Two => $version::Seat::Two,
                SeatId::Three => $version::Seat::Three,
                SeatId::Four => $version::Seat::Four,
            }
        }
    };
}

seat_conversion!(domain_seat, wire_seat, v14);
pub(super) const fn domain_seat_v15(seat: v15::Seat) -> Option<SeatId> {
    Some(match seat {
        v15::Seat::One => SeatId::One,
        v15::Seat::Two => SeatId::Two,
        v15::Seat::Three => SeatId::Three,
        v15::Seat::Four => SeatId::Four,
    })
}
seat_conversion!(domain_seat_v16, wire_seat_v16, v16);
seat_conversion!(domain_seat_v17, wire_seat_v17, v17);
seat_conversion!(domain_seat_v18, wire_seat_v18, v18);
seat_conversion!(domain_seat_v19, wire_seat_v19, v19);
seat_conversion!(domain_seat_v20, wire_seat_v20, v20);
