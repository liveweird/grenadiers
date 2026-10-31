use super::Seat;
use super::mine_contract_tests::dense_mine_case;
use super::server_conformance_tests::{FrameCase, snapshot_cases};

pub(super) fn cases() -> Vec<FrameCase> {
    let dense = dense_mine_case(false);
    let mut cases = snapshot_cases(&dense, "denseMinesMixedVisibility", &[Seat::Two, Seat::One]);
    let cloaked = dense_mine_case(true);
    cases.extend(snapshot_cases(
        &cloaked,
        "denseMinesCloakedOwner",
        &[Seat::One],
    ));
    cases
}
