#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClosingReason {
    Expired,
    Retired,
    ServerStopped,
    InternalFailure,
}
